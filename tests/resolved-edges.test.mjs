import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function buildResolvedEdgesFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-resolved-edges-",
  });

  const files = {
    "src/named.ts": "export const named = 1;\n",
    "src/defaulted.ts": "export default function defaulted() {}\n",
    "src/namespace.ts": "export const member = 1;\nexport const other = 2;\n",
    "src/types.ts": "export interface TypeOnly { value: string }\n",
    "src/side-effect.ts":
      "globalThis.sideEffectRan = true;\nexport const hidden = 1;\n",
    "src/reexport-source.ts": "export const reexported = 1;\n",
    "src/star-source.ts": "export const star = 1;\n",
    "src/dynamic.ts": "export const dyn = 1;\n",
    "src/cjs.js":
      "export const cjsNamed = 1;\nexport const cjsEscaped = 2;\nexport const cjsSide = 3;\n",
    "src/style.css": ".root { color: red; }\n",
    "src/consumer.ts": [
      'import { named } from "./named";',
      'import defaulted from "./defaulted";',
      'import * as ns from "./namespace";',
      'import type { TypeOnly } from "./types";',
      'import styles from "./style.css?inline";',
      'import "./side-effect";',
      'export { reexported } from "./reexport-source";',
      'export * from "./star-source";',
      'const mod = await import("./dynamic");',
      "mod.dyn;",
      'const { cjsNamed } = require("./cjs.js");',
      'const cjsNs = require("./cjs.js");',
      "use(cjsNs);",
      'require("./cjs.js");',
      "named; defaulted; ns.member; styles; let t: TypeOnly;",
    ].join("\n"),
  };

  for (const [relPath, content] of Object.entries(files)) {
    fixture.write(relPath, content);
  }

  execFileSync(
    process.execPath,
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      "--production",
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );

  return {
    fixture,
    symbols: fixture.readJson("symbols.json", { from: "output" }),
  };
}

function hasEdge(symbols, expected) {
  return (symbols.resolvedInternalEdges ?? []).some((edge) =>
    Object.entries(expected).every(([key, value]) => edge[key] === value),
  );
}

function edgeKey(edge) {
  return `${edge.from} -> ${edge.to} :: ${edge.kind} :: typeOnly=${edge.typeOnly}`;
}

function expectEdge(symbols, expected) {
  expect(
    hasEdge(symbols, expected),
    (symbols.resolvedInternalEdges ?? []).map(edgeKey).join("\n"),
  ).toBe(true);
}

describe("resolved internal edge artifact contract", () => {
  let fixture;
  let symbols;

  beforeAll(() => {
    ({ fixture, symbols } = buildResolvedEdgesFixture());
  });

  afterAll(() => {
    fixture?.cleanup();
  });

  it("advertises resolvedInternalEdges support and emits an array", () => {
    expect(symbols.meta?.supports?.resolvedInternalEdges).toBe(true);
    expect(symbols.resolvedInternalEdges).toEqual(expect.any(Array));
  });

  it("records ESM import and re-export edge kinds", () => {
    for (const edge of [
      {
        from: "src/consumer.ts",
        to: "src/named.ts",
        kind: "import-named",
        typeOnly: false,
      },
      {
        from: "src/consumer.ts",
        to: "src/defaulted.ts",
        kind: "import-default",
        typeOnly: false,
      },
      {
        from: "src/consumer.ts",
        to: "src/namespace.ts",
        kind: "import-namespace",
        typeOnly: false,
      },
      {
        from: "src/consumer.ts",
        to: "src/side-effect.ts",
        kind: "import-side-effect",
        typeOnly: false,
      },
      {
        from: "src/consumer.ts",
        to: "src/reexport-source.ts",
        kind: "reexport-named",
        typeOnly: false,
      },
      {
        from: "src/consumer.ts",
        to: "src/star-source.ts",
        kind: "reexport-broad",
        typeOnly: false,
      },
    ]) {
      expectEdge(symbols, edge);
    }
  });

  it("keeps type-only and literal dynamic edges distinct", () => {
    expectEdge(symbols, {
      from: "src/consumer.ts",
      to: "src/types.ts",
      kind: "import-named",
      typeOnly: true,
    });
    expectEdge(symbols, {
      from: "src/consumer.ts",
      to: "src/dynamic.ts",
      kind: "dynamic-literal",
      typeOnly: false,
    });
  });

  it("records CommonJS exact, namespace escape, and side-effect edge kinds", () => {
    for (const kind of [
      "cjs-require-exact",
      "cjs-namespace-escape",
      "cjs-side-effect",
    ]) {
      expectEdge(symbols, {
        from: "src/consumer.ts",
        to: "src/cjs.js",
        kind,
        typeOnly: false,
      });
    }
  });

  it("does not turn side-effect reachability into named fan-in", () => {
    expect(symbols.fanInByIdentity["src/side-effect.ts::hidden"]).toBe(0);
    expect(symbols.fanInByIdentity["src/cjs.js::cjsSide"]).toBe(0);
  });

  it("does not report non-source assets as resolver blindness or JS reachability", () => {
    expect(
      (symbols.unresolvedInternalSpecifierRecords ?? []).some(
        (record) => record.specifier === "./style.css?inline",
      ),
    ).toBe(false);
    expect(
      (symbols.resolvedInternalEdges ?? []).some(
        (edge) =>
          edge.source === "./style.css?inline" || edge.to === "src/style.css",
      ),
    ).toBe(false);
  });
});
