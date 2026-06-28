import { execFileSync } from "node:child_process";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function runSymbolGraph(fixture) {
  execFileSync(
    process.execPath,
    [
      "build-symbol-graph.mjs",
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

describe("CJS export surface artifact", () => {
  function readArtifact() {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-cjs-export-artifact-",
      packageJson: { private: true },
    });

    try {
      fixture.write(
        "src/exporter.cjs",
        [
          "exports.foo = 1;",
          "module.exports.bar = 2;",
          "module.exports = { baz: 3 };",
          "exports[dynamicName] = 4;",
          "",
        ].join("\n"),
      );

      runSymbolGraph(fixture);

      return fixture.readJson("symbols.json", { from: "output" });
    } finally {
      fixture.cleanup();
    }
  }

  it("CJSXA1. symbols.json advertises CJS export surface support", () => {
    const symbols = readArtifact();

    expect(symbols.meta?.supports?.cjsExportSurface).toBe(true);
  });

  it("CJSXA2. symbols.json keeps exact CJS export names by file", () => {
    const symbols = readArtifact();
    const surface = symbols.cjsExportSurfaceByFile?.["src/exporter.cjs"];

    expect(surface?.exact).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          name: "foo",
          kind: "exports-member",
        }),
        expect.objectContaining({
          name: "bar",
          kind: "module-exports-member",
        }),
        expect.objectContaining({
          name: "baz",
          kind: "module-exports-object",
        }),
      ]),
    );
  });

  it("CJSXA3. symbols.json keeps opaque CJS export forms by file", () => {
    const symbols = readArtifact();
    const surface = symbols.cjsExportSurfaceByFile?.["src/exporter.cjs"];

    expect(surface?.opaque).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "computed-export-name" }),
      ]),
    );
  });
});
