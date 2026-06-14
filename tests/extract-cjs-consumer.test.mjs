import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { extractDefinitionsAndUses } from "../_lib/extract-ts.mjs";

function extractInfo(source) {
  const dir = mkdtempSync(path.join(os.tmpdir(), "lrl-vitest-cjs-extract-"));
  const file = path.join(dir, "consumer.js");
  writeFileSync(file, source);
  try {
    return extractDefinitionsAndUses(file);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function extractUses(source) {
  return extractInfo(source).uses;
}

function hasUse(uses, kind, name) {
  return uses.some(
    (use) => use.kind === kind && (name === undefined || use.name === name),
  );
}

describe("CJS consumer extraction", () => {
  it("CJS1. bare require statement emits side-effect-only use", () => {
    const uses = extractUses('require("./exporter");\n');

    expect(hasUse(uses, "cjs-side-effect-only", "*")).toBe(true);
  });

  it("CJS1b. bare require statement does not emit exact consumer", () => {
    const uses = extractUses('require("./exporter");\n');

    expect(
      uses.some(
        (use) =>
          use.kind === "cjs-require-exact" ||
          use.kind === "cjs-namespace-member",
      ),
    ).toBe(false);
  });

  it("CJS2. require destructuring emits exact foo consumer", () => {
    const uses = extractUses(
      'const { foo, bar: renamed } = require("./exporter");\n',
    );

    expect(hasUse(uses, "cjs-require-exact", "foo")).toBe(true);
  });

  it("CJS2b. require destructuring emits exact aliased property consumer", () => {
    const uses = extractUses(
      'const { foo, bar: renamed } = require("./exporter");\n',
    );

    expect(hasUse(uses, "cjs-require-exact", "bar")).toBe(true);
  });

  it("CJS3. const namespace require emits exact member call consumer", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nmod.foo();\nmod.bar;\n',
    );

    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(true);
  });

  it("CJS3b. const namespace require emits exact member read consumer", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nmod.foo();\nmod.bar;\n',
    );

    expect(hasUse(uses, "cjs-namespace-member", "bar")).toBe(true);
  });

  it("CJS3c. const namespace require alias destructuring emits exact foo consumer", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nconst { foo, bar: renamed } = mod;\n',
    );

    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(true);
  });

  it("CJS3d. const namespace require alias destructuring emits exact aliased member consumer", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nconst { foo, bar: renamed } = mod;\n',
    );

    expect(hasUse(uses, "cjs-namespace-member", "bar")).toBe(true);
  });

  it("CJS3e. require alias rest destructuring stays broad", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nconst { foo, ...rest } = mod;\n',
    );

    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(false);
  });

  it("CJS4. direct require member call emits exact member consumer", () => {
    const uses = extractUses('require("./exporter").foo();\n');

    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(true);
  });

  it("CJS5. escaping require namespace emits broad escape", () => {
    const uses = extractUses('const mod = require("./exporter");\nuse(mod);\n');

    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(true);
  });

  it("CJS6. non-const require namespace is broad escape, not exact", () => {
    const uses = extractUses('let mod = require("./exporter");\nmod.foo();\n');

    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(false);
  });

  it("CJS7. module.exports require emits broad re-export", () => {
    const uses = extractUses('module.exports = require("./exporter");\n');

    expect(hasUse(uses, "cjs-reexport-broad", "*")).toBe(true);
  });

  it("CJS8. dynamic require records CJS opacity", () => {
    const info = extractInfo(
      'const target = "./exporter";\nrequire(target);\n',
    );

    expect(
      info.cjsRequireOpacity?.some(
        (entry) => entry.kind === "dynamic-require" && entry.line === 2,
      ),
    ).toBe(true);
  });

  it("CJS8b. dynamic require does not pretend to be exact CJS consumer", () => {
    const info = extractInfo(
      'const target = "./exporter";\nrequire(target);\n',
    );

    expect(info.uses.some((use) => use.kind?.startsWith("cjs-"))).toBe(false);
  });

  it("CJS9. static package.json metadata require does not create CJS opacity", () => {
    const info = extractInfo(
      [
        'import path from "node:path";',
        'import { createRequire } from "node:module";',
        "const require = createRequire(import.meta.url);",
        "export function getCurrentVersion() {",
        '  return require(path.resolve(import.meta.dirname, "../../package.json")).version;',
        "}",
        "",
      ].join("\n"),
    );

    expect(
      info.cjsRequireOpacity?.some((entry) => entry.kind === "dynamic-require"),
    ).not.toBe(true);
  });

  it("CJS9b. static package.json metadata require does not pretend to be a CJS consumer", () => {
    const info = extractInfo(
      [
        'import path from "node:path";',
        'import { createRequire } from "node:module";',
        "const require = createRequire(import.meta.url);",
        "export function getCurrentVersion() {",
        '  return require(path.resolve(import.meta.dirname, "../../package.json")).version;',
        "}",
        "",
      ].join("\n"),
    );

    expect(info.uses.some((use) => use.kind?.startsWith("cjs-"))).toBe(false);
  });

  it("CJS10. static computed CJS members are exact consumers", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nmod["foo"]();\nrequire("./exporter")["bar"];\n',
    );

    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "bar")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(false);
  });

  it("CJS11. simple guard reads do not degrade exact CJS member consumers", () => {
    const uses = extractUses(
      [
        'const mod = require("./exporter");',
        "if (mod) mod.foo();",
        "mod && mod.bar();",
        'typeof mod !== "undefined" && mod.baz;',
        "",
      ].join("\n"),
    );

    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "bar")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "baz")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(false);
  });

  it("CJS12. key introspection remains broad CJS evidence", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nif ("foo" in mod) mod.foo();\n',
    );

    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(false);
  });

  it("CJS13. shadowed function parameter does not exact-protect outer require", () => {
    const uses = extractUses(
      'const mod = require("./exporter");\nfunction f(mod) { mod.foo(); }\n',
    );

    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(false);
    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(false);
  });

  it("CJS14. namespace member writes degrade to broad escape, not exact consumers", () => {
    const uses = extractUses(
      [
        'const mod = require("./exporter");',
        "mod.foo = 1;",
        "mod.bar++;",
        "delete mod.baz;",
        "",
      ].join("\n"),
    );

    expect(hasUse(uses, "cjs-namespace-escape", "*")).toBe(true);
    expect(hasUse(uses, "cjs-namespace-member", "foo")).toBe(false);
    expect(hasUse(uses, "cjs-namespace-member", "bar")).toBe(false);
    expect(hasUse(uses, "cjs-namespace-member", "baz")).toBe(false);
  });
});
