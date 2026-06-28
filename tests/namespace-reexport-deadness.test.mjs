import { execFileSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = path.resolve(import.meta.dirname, "..");
const TEST_TIMEOUT = 60_000;

function runSymbolGraph(files) {
  const fixture = createTempRepoFixture({
    prefix: "vitest-namespace-reexport-",
    packageJson: {
      name: "namespace-reexport-fixture",
      private: true,
      type: "module",
    },
  });

  try {
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
        "--no-incremental",
      ],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );

    return fixture.readJson("symbols.json", { from: "output" });
  } finally {
    fixture.cleanup();
  }
}

function deadIdentitySet(symbols) {
  return new Set(
    (symbols.deadProdList ?? []).map((item) => `${item.file}::${item.symbol}`),
  );
}

describe("namespace re-export deadness precision", () => {
  it(
    "NR1-NR5. direct namespace member use protects only observed exports",
    () => {
      const symbols = runSymbolGraph({
        "src/source.ts": [
          "export function nsUsedFunc() { return 1; }",
          "export function nsUnusedFunc() { return 2; }",
          "export const nsUsedConst = 3;",
          "export const nsUnusedConst = 4;",
        ].join("\n"),
        "src/barrel.ts": 'export * as ns from "./source";\n',
        "src/consumer.ts": [
          'import { ns } from "./barrel";',
          "ns.nsUsedFunc();",
          "console.log(ns.nsUsedConst);",
        ].join("\n"),
      });

      const fanIn = symbols.fanInByIdentity ?? {};
      const fanInSpace = symbols.fanInByIdentitySpace ?? {};
      const dead = deadIdentitySet(symbols);

      expect(fanIn["src/source.ts::nsUsedFunc"]).toBe(1);
      expect(fanIn["src/source.ts::nsUsedConst"]).toBe(1);
      expect(dead.has("src/source.ts::nsUnusedFunc")).toBe(true);
      expect(dead.has("src/source.ts::nsUnusedConst")).toBe(true);
      expect(fanInSpace["src/source.ts::nsUnusedFunc"]?.broad).toBe(0);
      expect(fanInSpace["src/source.ts::nsUnusedConst"]?.broad).toBe(0);
    },
    TEST_TIMEOUT,
  );

  it(
    "NR6-NR10. chained namespace re-export preserves exact member fan-in",
    () => {
      const symbols = runSymbolGraph({
        "src/source.ts": [
          "export function chainedUsedFunc() { return 1; }",
          "export function chainedUnusedFunc() { return 2; }",
          "export const chainedUsedConst = 3;",
          "export const chainedUnusedConst = 4;",
        ].join("\n"),
        "src/barrel.ts": 'export * as ns from "./source";\n',
        "src/outer.ts": 'export { ns } from "./barrel";\n',
        "src/consumer.ts": [
          'import { ns } from "./outer";',
          "ns.chainedUsedFunc();",
          "console.log(ns.chainedUsedConst);",
        ].join("\n"),
      });

      const fanIn = symbols.fanInByIdentity ?? {};
      const fanInSpace = symbols.fanInByIdentitySpace ?? {};
      const dead = deadIdentitySet(symbols);

      expect(fanIn["src/source.ts::chainedUsedFunc"]).toBe(1);
      expect(fanIn["src/source.ts::chainedUsedConst"]).toBe(1);
      expect(dead.has("src/source.ts::chainedUnusedFunc")).toBe(true);
      expect(dead.has("src/source.ts::chainedUnusedConst")).toBe(true);
      expect(fanInSpace["src/source.ts::chainedUnusedFunc"]?.broad).toBe(0);
      expect(fanInSpace["src/source.ts::chainedUnusedConst"]?.broad).toBe(0);
    },
    TEST_TIMEOUT,
  );

  it(
    "NR11-NR12. opaque namespace escape broad-shadows members and emits diagnostics",
    () => {
      const symbols = runSymbolGraph({
        "src/source.ts": [
          "export function escapeFunc() { return 1; }",
          "export const escapeConst = 2;",
        ].join("\n"),
        "src/barrel.ts": 'export * as ns from "./source";\n',
        "src/consumer.ts": [
          'import { ns } from "./barrel";',
          "function observe(value: unknown) { return value; }",
          "observe(ns);",
        ].join("\n"),
      });

      const fanInSpace = symbols.fanInByIdentitySpace ?? {};
      const diagnostics = symbols.namespaceReExportDiagnostics ?? [];

      expect(fanInSpace["src/source.ts::escapeFunc"]?.broad).toBe(1);
      expect(fanInSpace["src/source.ts::escapeConst"]?.broad).toBe(1);
      expect(diagnostics).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            kind: "opaque-namespace-escape",
            consumerFile: "src/consumer.ts",
            exportedName: "ns",
            targetFile: "src/source.ts",
            reason: "namespace-object-escaped",
          }),
        ]),
      );
    },
    TEST_TIMEOUT,
  );
});
