import { execFileSync } from "node:child_process";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function runSymbolGraph(files) {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-cjs-integration-",
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

describe("CJS integration", () => {
  function integratedSymbols() {
    return runSymbolGraph({
      "exporter.cjs": [
        "exports.foo = 1;",
        "module.exports.bar = 2;",
        "exports[dynamicName] = 3;",
        "",
      ].join("\n"),
      "consumer.js": [
        'const mod = require("./typed-exporter.js");',
        "const { foo } = mod;",
        'const target = "./typed-exporter.js";',
        "require(target);",
        "",
      ].join("\n"),
      "typed-exporter.js": "export const foo = 1;\nexport const bar = 2;\n",
    });
  }

  it("CJSI1. exact and opaque CJS export surface facts coexist", () => {
    const symbols = integratedSymbols();

    const surface = symbols.cjsExportSurfaceByFile?.["src/exporter.cjs"];

    expect(symbols.meta?.supports?.cjsExportSurface).toBe(true);
    expect(surface?.exact).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ name: "foo" }),
        expect.objectContaining({ name: "bar" }),
      ]),
    );
    expect(surface?.opaque).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "computed-export-name" }),
      ]),
    );
  });

  it("CJSI2. namespace alias destructuring protects exact ESM export", () => {
    const symbols = integratedSymbols();

    expect(symbols.fanInByIdentity?.["src/typed-exporter.js::foo"]).toBe(1);
    expect(symbols.fanInByIdentity?.["src/typed-exporter.js::bar"]).toBe(0);
  });

  it("CJSI3. dynamic require is reported as CJS opacity evidence", () => {
    const symbols = integratedSymbols();

    expect(symbols.cjsRequireOpacity).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          consumerFile: "src/consumer.js",
          kind: "dynamic-require",
          line: 4,
        }),
      ]),
    );
  });
});
