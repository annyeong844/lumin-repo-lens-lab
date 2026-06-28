import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const SMOKE_TEST_TIMEOUT_MS = 30_000;

function runScript(script, args = [], options = {}) {
  try {
    return {
      ok: true,
      stdout: execFileSync(process.execPath, [script, ...args], {
        cwd: options.cwd ?? REPO_ROOT,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      }),
      stderr: "",
    };
  } catch (error) {
    return {
      ok: false,
      stdout: error.stdout ?? "",
      stderr: error.stderr ?? error.message,
    };
  }
}

function expectRunOk(result) {
  expect(result.ok, `${result.stderr}`.slice(0, 600)).toBe(true);
}

function writeSmokeSourceFixture(fixture) {
  fixture.write(
    "src/lib.ts",
    [
      "export function helper(x: number): number {",
      "  return x + 1;",
      "}",
      "export function unused(): void {}",
      "",
    ].join("\n"),
  );
  fixture.write(
    "src/app.ts",
    [
      "import { helper } from './lib';",
      "export function main(): number {",
      "  return helper(1);",
      "}",
      "",
    ].join("\n"),
  );
  fixture.write(
    "src/index.ts",
    [
      "export { helper } from './lib';",
      "export { main } from './app';",
      "",
    ].join("\n"),
  );
}

function withSmokeSourceFixture(fn) {
  const fixture = createTempRepoFixture({
    prefix: "vitest-smoke-uncovered-",
    packageJson: { name: "fx-smoke", type: "module" },
  });
  try {
    writeSmokeSourceFixture(fixture);
    return fn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function readOutputJson(fixture, artifactName) {
  return fixture.readJson(artifactName, { from: "output" });
}

function hasAnyKey(value, keys) {
  return (
    value &&
    typeof value === "object" &&
    keys.some((key) => Object.hasOwn(value, key))
  );
}

describe("smoke coverage for previously uncovered scripts", () => {
  it("A1-A3. build-call-graph completes and writes a recognizable artifact", () => {
    withSmokeSourceFixture((fixture) => {
      const result = runScript("build-call-graph.mjs", [
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ]);
      expectRunOk(result);

      const artifact = readOutputJson(fixture, "call-graph.json");
      expect(hasAnyKey(artifact, ["edges", "summary", "meta", "nodes"])).toBe(
        true,
      );
    });
  });

  it("B1-B3. check-barrel-discipline completes and writes a recognizable artifact", () => {
    withSmokeSourceFixture((fixture) => {
      const result = runScript("check-barrel-discipline.mjs", [
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ]);
      expectRunOk(result);

      const artifact = readOutputJson(fixture, "barrels.json");
      expect(hasAnyKey(artifact, ["barrels", "summary", "meta"])).toBe(true);
    });
  });

  it("C1-C3. measure-discipline completes and writes a recognizable artifact", () => {
    withSmokeSourceFixture((fixture) => {
      const result = runScript("measure-discipline.mjs", [
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ]);
      expectRunOk(result);

      const artifact = readOutputJson(fixture, "discipline.json");
      expect(hasAnyKey(artifact, ["summary", "files", "metrics", "meta"])).toBe(
        true,
      );
    });
  });

  it("D1-D6. emit-sarif accepts zero upstream artifacts and writes SARIF 2.1.0", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-smoke-sarif-empty-",
      packageJson: { name: "fx-smoke", type: "module" },
    });
    try {
      const result = runScript("emit-sarif.mjs", [
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ]);
      expectRunOk(result);

      const sarifPath = fixture.outputPath("lumin-repo-lens-lab.sarif");
      expect(existsSync(sarifPath)).toBe(true);
      const sarif = JSON.parse(readFileSync(sarifPath, "utf8"));

      expect(sarif.version).toBe("2.1.0");
      expect(Array.isArray(sarif.runs)).toBe(true);
      expect(sarif.runs.length).toBeGreaterThan(0);
      expect(sarif.runs[0].tool?.driver?.name).toBeTruthy();
      expect(sarif.runs[0].tool?.driver?.version).toBeTruthy();
    } finally {
      fixture.cleanup();
    }
  });

  it(
    "D'1-D'4. emit-sarif respects classifier policy filters",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-smoke-sarif-policy-",
        packageJson: { name: "sarif-policy", type: "module" },
      });
      try {
        fixture.write("eslint.config.mjs", "export default [{ rules: {} }];\n");
        fixture.write(
          "src/realDead.ts",
          "export const genuinelyUnused = 42;\n",
        );
        fixture.write("src/consumer.ts", "export const c = 1;\n");

        expectRunOk(
          runScript("build-symbol-graph.mjs", [
            "--root",
            fixture.root,
            "--output",
            fixture.output,
          ]),
        );
        expectRunOk(
          runScript("classify-dead-exports.mjs", [
            "--root",
            fixture.root,
            "--output",
            fixture.output,
          ]),
        );
        expectRunOk(
          runScript("emit-sarif.mjs", [
            "--root",
            fixture.root,
            "--output",
            fixture.output,
          ]),
        );

        const sarif = readOutputJson(fixture, "lumin-repo-lens-lab.sarif");
        const ga001 = sarif.runs[0].results.filter(
          (result) => result.ruleId === "GA001",
        );
        const symbols = ga001
          .map((result) => result.properties?.symbol)
          .filter(Boolean);

        expect(symbols).toContain("genuinelyUnused");
        expect(symbols).not.toContain("default");
        expect(
          JSON.stringify(sarif).includes("dead-classify.json") ||
            ga001.some((result) => result.properties?.proposalBucket),
        ).toBe(true);
      } finally {
        fixture.cleanup();
      }
    },
    SMOKE_TEST_TIMEOUT_MS,
  );

  it(
    'D"1-D"4. symbol parse warnings are structured and forwarded to SARIF',
    () => {
      const broken = createTempRepoFixture({
        prefix: "vitest-smoke-warnings-",
        packageJson: { name: "warnings-fx", type: "module" },
      });
      const clean = createTempRepoFixture({
        prefix: "vitest-smoke-clean-",
        packageJson: { name: "clean-fx", type: "module" },
      });
      try {
        broken.write("src/ok.ts", "export const ok = 1;\n");
        broken.write("src/broken.ts", "export const = ;\n");

        expectRunOk(
          runScript("build-symbol-graph.mjs", [
            "--root",
            broken.root,
            "--output",
            broken.output,
          ]),
        );
        const symbols = readOutputJson(broken, "symbols.json");

        expect(Array.isArray(symbols.meta?.warnings)).toBe(true);
        expect(
          symbols.meta.warnings.some(
            (warning) => warning.code === "parse-errors" && warning.count >= 1,
          ),
        ).toBe(true);

        clean.write("src/ok.ts", "export const ok = 1;\n");
        expectRunOk(
          runScript("build-symbol-graph.mjs", [
            "--root",
            clean.root,
            "--output",
            clean.output,
          ]),
        );
        const cleanSymbols = readOutputJson(clean, "symbols.json");
        expect(cleanSymbols.meta?.warnings).toEqual([]);

        expectRunOk(
          runScript("classify-dead-exports.mjs", [
            "--root",
            broken.root,
            "--output",
            broken.output,
          ]),
        );
        expectRunOk(
          runScript("emit-sarif.mjs", [
            "--root",
            broken.root,
            "--output",
            broken.output,
          ]),
        );
        const sarif = readOutputJson(broken, "lumin-repo-lens-lab.sarif");
        const upstreamWarnings =
          sarif.runs[0].properties?.upstreamWarnings ?? [];

        expect(
          upstreamWarnings.some(
            (warning) =>
              warning.source === "symbols.json" &&
              warning.code === "parse-errors",
          ),
        ).toBe(true);
      } finally {
        broken.cleanup();
        clean.cleanup();
      }
    },
    SMOKE_TEST_TIMEOUT_MS,
  );

  it("F1-F3. check-drift detects package-lock version drift", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-smoke-drift-",
      packageJson: { name: "x", version: "9.9.9", type: "module" },
    });
    try {
      const driftScript = readFileSync(
        path.join(REPO_ROOT, "scripts/check-drift.mjs"),
        "utf8",
      );
      fixture.write("scripts/check-drift.mjs", driftScript);

      const writeDriftFixture = (lockVersion, version = "9.9.9") => {
        fixture.writeJson("package.json", {
          name: "x",
          version,
          type: "module",
        });
        fixture.write("emit-sarif.mjs", `const TOOL_VERSION = '${version}';\n`);
        fixture.write(
          "CHANGELOG.md",
          [
            "# Changelog",
            "",
            `## ${version} - 2026-04-19`,
            "",
            "Synthetic.",
            "",
          ].join("\n"),
        );
        fixture.writeJson("package-lock.json", {
          name: "x",
          version: lockVersion,
          lockfileVersion: 3,
          packages: { "": { name: "x", version: lockVersion } },
        });
      };

      writeDriftFixture("9.9.9");
      expectRunOk(
        runScript("scripts/check-drift.mjs", [], { cwd: fixture.root }),
      );

      writeDriftFixture("0.9.0-beta.1", "0.9.0-beta.1");
      expectRunOk(
        runScript("scripts/check-drift.mjs", [], { cwd: fixture.root }),
      );

      writeDriftFixture("1.0.0");
      const mismatch = runScript("scripts/check-drift.mjs", [], {
        cwd: fixture.root,
      });
      expect(mismatch.ok).toBe(false);
      expect(mismatch.stderr).toContain("package-lock.json");
    } finally {
      fixture.cleanup();
    }
  });

  it("E1-E3. merge-runtime-evidence writes a recognizable runtime artifact", () => {
    withSmokeSourceFixture((fixture) => {
      const helperFile = fixture.path("src/lib.ts");
      const appFile = fixture.path("src/app.ts");
      fixture.writeJson(
        "symbols.json",
        {
          meta: { generated: new Date().toISOString(), root: fixture.root },
          symbolsByFile: {
            [helperFile]: [
              {
                name: "helper",
                kind: "FunctionDeclaration",
                line: 1,
                exported: true,
              },
              {
                name: "unused",
                kind: "FunctionDeclaration",
                line: 4,
                exported: true,
              },
            ],
            [appFile]: [
              {
                name: "main",
                kind: "FunctionDeclaration",
                line: 2,
                exported: true,
              },
            ],
          },
          deadProdList: [
            {
              file: helperFile,
              symbol: "unused",
              kind: "FunctionDeclaration",
              line: 4,
            },
          ],
        },
        { to: "output" },
      );
      fixture.writeJson(
        "coverage-final.json",
        {
          [helperFile]: {
            path: helperFile,
            fnMap: {
              0: {
                name: "helper",
                decl: { start: { line: 1 }, end: { line: 1 } },
                loc: { start: { line: 1 }, end: { line: 3 } },
                line: 1,
              },
              1: {
                name: "unused",
                decl: { start: { line: 4 }, end: { line: 4 } },
                loc: { start: { line: 4 }, end: { line: 4 } },
                line: 4,
              },
            },
            f: { 0: 1, 1: 0 },
            statementMap: {},
            s: {},
            branchMap: {},
            b: {},
          },
        },
        { to: "output" },
      );

      const result = runScript("merge-runtime-evidence.mjs", [
        "--root",
        fixture.root,
        "--output",
        fixture.output,
        "--coverage",
        fixture.outputPath("coverage-final.json"),
      ]);
      expectRunOk(result);

      const evidence = readOutputJson(fixture, "runtime-evidence.json");
      expect(
        hasAnyKey(evidence, ["meta", "touched", "summary", "perSymbol"]),
      ).toBe(true);
    });
  });
});
