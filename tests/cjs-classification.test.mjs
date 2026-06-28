import { execFileSync } from "node:child_process";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function runSymbolGraph(files) {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-cjs-graph-",
    packageJson: { private: true },
  });

  try {
    for (const [name, content] of Object.entries(files)) {
      fixture.write(`src/${name}`, content);
    }

    execFileSync(
      process.execPath,
      [
        "build-symbol-graph.mjs",
        "--root",
        fixture.root,
        "--output",
        fixture.output,
        "--production",
      ],
      {
        cwd: process.cwd(),
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    return fixture.readJson("symbols.json", { from: "output" });
  } finally {
    fixture.cleanup();
  }
}

describe("CJS consumer classification", () => {
  it("G1. cjs destructuring increases exact fan-in", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js": 'const { foo } = require("./exporter.js");\n',
    });

    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(1);
  });

  it("G1b. unrelated sibling export remains dead", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js": 'const { foo } = require("./exporter.js");\n',
    });

    expect(symbols.fanInByIdentity["src/exporter.js::bar"]).toBe(0);
    expect(
      symbols.deadProdList.some(
        (entry) => entry.file === "src/exporter.js" && entry.symbol === "bar",
      ),
    ).toBe(true);
  });

  it("does not let side-effect-only require protect named exports", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js": 'require("./exporter.js");\n',
    });

    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(0);
    expect(symbols.fanInByIdentity["src/exporter.js::bar"]).toBe(0);
  });

  it("uses namespace member access as exact fan-in only for the accessed member", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js": 'const mod = require("./exporter.js");\nmod.foo();\n',
    });

    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(1);
    expect(symbols.fanInByIdentity["src/exporter.js::bar"]).toBe(0);
  });

  it("uses namespace alias destructuring as exact fan-in only for the destructured member", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js":
        'const mod = require("./exporter.js");\nconst { foo } = mod;\n',
    });

    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(1);
    expect(symbols.fanInByIdentity["src/exporter.js::bar"]).toBe(0);
  });

  it("keeps namespace escapes as broad evidence that blocks truly-dead confidence", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js": 'const mod = require("./exporter.js");\nuse(mod);\n',
    });

    expect(symbols.deadTotal).toBe(2);
    expect(symbols.trulyDead).toBe(0);
  });

  it("preserves dynamic require as CJS opacity evidence", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\n",
      "consumer.js": 'const target = "./exporter.js";\nrequire(target);\n',
    });

    expect(symbols.cjsRequireOpacity).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          consumerFile: "src/consumer.js",
          kind: "dynamic-require",
          line: 2,
        }),
      ]),
    );
  });

  it("does not treat static package metadata require as dynamic CJS opacity", () => {
    const symbols = runSymbolGraph({
      "consumer.js": [
        'import path from "node:path";',
        'import { createRequire } from "node:module";',
        "const require = createRequire(import.meta.url);",
        "export function getCurrentVersion() {",
        '  return require(path.resolve(import.meta.dirname, "../package.json")).version;',
        "}",
        "",
      ].join("\n"),
    });

    expect(symbols.cjsRequireOpacity ?? []).toHaveLength(0);
  });

  it("uses guarded and static computed CJS members as exact fan-in", () => {
    const symbols = runSymbolGraph({
      "exporter.js":
        "export const foo = 1;\nexport const bar = 2;\nexport const baz = 3;\nexport const unused = 4;\n",
      "consumer.js": [
        'const mod = require("./exporter.js");',
        "if (mod) mod.foo();",
        'mod && mod["bar"];',
        'require("./exporter.js")["baz"];',
        "",
      ].join("\n"),
    });

    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(1);
    expect(symbols.fanInByIdentity["src/exporter.js::bar"]).toBe(1);
    expect(symbols.fanInByIdentity["src/exporter.js::baz"]).toBe(1);
    expect(symbols.fanInByIdentity["src/exporter.js::unused"]).toBe(0);
  });

  it("keeps key introspection broad and prevents truly-dead confidence", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js":
        'const mod = require("./exporter.js");\nif ("foo" in mod) mod.foo();\n',
    });

    expect(symbols.deadTotal).toBe(2);
    expect(symbols.trulyDead).toBe(0);
    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(0);
  });

  it("keeps namespace member writes broad instead of exact fan-in", () => {
    const symbols = runSymbolGraph({
      "exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
      "consumer.js": [
        'const mod = require("./exporter.js");',
        "mod.foo = 10;",
        "",
      ].join("\n"),
    });

    expect(symbols.deadTotal).toBe(2);
    expect(symbols.trulyDead).toBe(0);
    expect(symbols.fanInByIdentity["src/exporter.js::foo"]).toBe(0);
    expect(symbols.fanInByIdentity["src/exporter.js::bar"]).toBe(0);
  });
});
