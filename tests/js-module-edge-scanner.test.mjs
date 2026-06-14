import { describe, expect, it } from "vitest";

import { parseOxcOrThrow } from "../_lib/parse-oxc.mjs";
import {
  MODULE_EDGE_SCANNER_POLICY_VERSION,
  scanJsModuleEdgesFast,
} from "../_lib/js-module-edge-scanner.mjs";

function normalizeEdges(edges) {
  return [...edges]
    .map((edge) => ({
      source: edge.source,
      typeOnly: !!edge.typeOnly,
      reExport: !!edge.reExport,
      dynamic: !!edge.dynamic,
    }))
    .sort(
      (a, b) =>
        a.source.localeCompare(b.source) ||
        Number(a.typeOnly) - Number(b.typeOnly) ||
        Number(a.reExport) - Number(b.reExport) ||
        Number(a.dynamic) - Number(b.dynamic),
    );
}

function oxcTopologyEdges(filename, source) {
  const parsed = parseOxcOrThrow(filename, source);
  const edges = [];

  function pushImportExpression(node) {
    const sourceNode = node.source;
    if (
      sourceNode &&
      (sourceNode.type === "Literal" || sourceNode.type === "StringLiteral") &&
      typeof sourceNode.value === "string"
    ) {
      edges.push({
        source: sourceNode.value,
        typeOnly: false,
        reExport: false,
        dynamic: true,
      });
    }
  }

  function walk(node) {
    if (!node || typeof node !== "object") return;
    if (node.type === "ImportExpression") pushImportExpression(node);
    for (const key of Object.keys(node)) {
      if (key === "type" || key === "start" || key === "end") continue;
      const value = node[key];
      if (Array.isArray(value)) {
        for (const child of value) walk(child);
      } else if (
        value &&
        typeof value === "object" &&
        typeof value.type === "string"
      ) {
        walk(value);
      }
    }
  }

  for (const node of parsed.program.body) {
    if (node.type === "ImportDeclaration") {
      edges.push({
        source: node.source.value,
        typeOnly: node.importKind === "type",
        reExport: false,
        dynamic: false,
      });
    } else if (
      (node.type === "ExportNamedDeclaration" ||
        node.type === "ExportAllDeclaration") &&
      node.source
    ) {
      const specs = node.specifiers ?? [];
      const allSpecsTypeOnly =
        specs.length > 0 && specs.every((spec) => spec.exportKind === "type");
      edges.push({
        source: node.source.value,
        typeOnly: node.exportKind === "type" || allSpecsTypeOnly,
        reExport: true,
        dynamic: false,
      });
    }
  }
  walk(parsed.program);
  return normalizeEdges(edges);
}

function expectAcceptedEquivalent(filename, source) {
  const scan = scanJsModuleEdgesFast(source, { filename });

  expect(scan.ok).toBe(true);
  expect(scan.policyVersion).toBe(MODULE_EDGE_SCANNER_POLICY_VERSION);
  expect(scan.mode).toBe("fast-module-edge");
  expect(normalizeEdges(scan.edges ?? [])).toEqual(
    oxcTopologyEdges(filename, source),
  );

  return scan;
}

function expectFallback(source, reason) {
  const scan = scanJsModuleEdgesFast(source, { filename: "fixture.ts" });

  expect(scan.ok).toBe(false);
  expect(scan.mode).toBe("fallback-required");
  expect(scan.risk).toContain(reason);
}

describe("JS module edge scanner fast-path contract", () => {
  it("accepts static imports, re-exports, type edges, and literal dynamic imports when equivalent to Oxc topology edges", () => {
    expectAcceptedEquivalent(
      "fixture.ts",
      [
        "import def, { named } from './dep';",
        "import type { T } from './types';",
        "import './side-effect';",
        "export { named as renamed } from './dep';",
        "export { type T2 } from './more-types';",
        "export type { T3 } from './even-more-types';",
        "export * from './barrel';",
        "export type * from './type-barrel';",
        "export async function lazy() { return import('./lazy'); }",
      ].join("\n"),
    );
  });

  it("ignores fake module syntax inside comments, strings, regex literals, and template literals", () => {
    expectAcceptedEquivalent(
      "fixture.ts",
      [
        "// import fake from './comment';",
        "const s = 'export * from \"./string\"';",
        'const d = "import(\\"./double\\")";',
        "const r = /import\\s+fake\\s+from\\s+[\"\\']\\.\\/regex[\"\\']/;",
        'const t = `export * from "./template"`;',
        "import real from './real';",
      ].join("\n"),
    );
  });

  it("accepts unrelated interpolated template literals while preserving module edges", () => {
    expectAcceptedEquivalent(
      "fixture.ts",
      [
        "const name = 'world';",
        "const message = `hello ${name}`;",
        "import real from './real';",
        "export async function lazy() { return import('./lazy'); }",
      ].join("\n"),
    );
  });

  it("accepts import and export attributes when string specifiers remain safely represented", () => {
    expectAcceptedEquivalent(
      "fixture.ts",
      [
        "import data from './data.json' with { type: 'json' };",
        "export * from './other.json' assert { type: 'json' };",
      ].join("\n"),
    );
  });

  it("preserves source line numbers for accepted edges", () => {
    const scan = scanJsModuleEdgesFast("import value from './dep';\n", {
      filename: "fixture.ts",
    });

    expect(scan.ok).toBe(true);
    expect(scan.edges?.[0]?.line).toBe(1);
  });

  it("preserves dynamic import line numbers after multiline template literals", () => {
    const source = [
      "const help = `",
      "line one",
      "line two",
      "`;",
      "async function load() {",
      "  return import('node:child_process');",
      "}",
    ].join("\n");
    const scan = scanJsModuleEdgesFast(source, { filename: "fixture.mjs" });

    expect(scan.ok).toBe(true);
    expect(scan.edges).toEqual([
      {
        source: "node:child_process",
        typeOnly: false,
        reExport: false,
        dynamic: true,
        line: 6,
      },
    ]);
  });

  it("falls back for unsupported dynamic and CommonJS module forms with stable reason codes", () => {
    expectFallback(
      "export function load(name) { return import(name); }",
      "non-literal-dynamic-import",
    );
    expectFallback(
      "export function load(name) { return import(`./${name}.ts`); }",
      "template-dynamic-import",
    );
    expectFallback("const value = require('./cjs');", "require-call");
    expectFallback(
      "const routes = import.meta.glob('./routes/*.ts');",
      "import-meta-glob",
    );
  });

  it("falls back for TypeScript module forms that the scanner does not model", () => {
    expectFallback("import foo = require('./foo');", "ts-import-equals");
    expectFallback("export = foo;", "ts-export-assignment");
    expectFallback(
      "declare module './virtual' { export const x: number; }",
      "ts-ambient-module",
    );
  });

  it("falls back for JSX text instead of risking fake edge extraction", () => {
    expectFallback(
      "export const View = () => <span>import fake from './jsx-text'</span>;",
      "unsupported-syntax",
    );
  });

  it("handles many string literals without quadratic line scans", () => {
    const source = Array.from(
      { length: 6000 },
      (_, i) => `const s${i} = "value-${i}";`,
    ).join("\n");

    const started = Date.now();
    const scan = scanJsModuleEdgesFast(source, { filename: "many-strings.ts" });
    const elapsedMs = Date.now() - started;

    expect(scan.ok).toBe(true);
    expect(scan.edges).toHaveLength(0);
    expect(elapsedMs).toBeLessThan(1500);
  });
});
