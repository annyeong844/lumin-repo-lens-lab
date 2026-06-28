import { execFileSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = path.resolve(import.meta.dirname, "..");
const MODULE_REACHABILITY_TEST_TIMEOUT_MS = 60_000;

function runScript(script, fixture, args = []) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, script),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      ...args,
    ],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  );
}

function writeModuleReachabilityFixture(fixture) {
  fixture.writeJson("package.json", {
    name: "module-reachability-fixture",
    private: true,
    exports: {
      ".": "./src/index.ts",
    },
  });

  fixture.write(
    "src/index.ts",
    [
      'import type { TypeOnly } from "./types";',
      'import { run } from "./runtime";',
      "export { run };",
      "export type IndexType = TypeOnly;",
    ].join("\n"),
  );
  fixture.write(
    "src/runtime.ts",
    'import { deep } from "./deep";\nexport const run = () => deep;\n',
  );
  fixture.write("src/deep.ts", "export const deep = 1;\n");
  fixture.write(
    "src/types.ts",
    "export interface TypeOnly { value: string }\n",
  );
  fixture.write("src/isolated.ts", "export const isolated = true;\n");
  fixture.write(
    "src/components/App.ts",
    'import { Modal } from "./Modal";\nexport function App() { return Modal; }\n',
  );
  fixture.write(
    "src/components/Modal.ts",
    'import { App } from "./App";\nexport function Modal() { return App; }\n',
  );
}

function createModuleReachabilityFixture(prefix) {
  const fixture = createTempRepoFixture({
    prefix,
    packageJson: {
      name: "module-reachability-fixture",
      private: true,
    },
  });
  writeModuleReachabilityFixture(fixture);
  return fixture;
}

function runModuleReachabilityProducer(extraArgs = []) {
  const fixture = createModuleReachabilityFixture(
    "vitest-module-reachability-",
  );
  try {
    runScript("build-symbol-graph.mjs", fixture, ["--production"]);
    runScript("build-entry-surface.mjs", fixture, ["--production"]);
    runScript("build-module-reachability.mjs", fixture, [
      "--production",
      ...extraArgs,
    ]);
    return fixture.readJson("module-reachability.json", { from: "output" });
  } finally {
    fixture.cleanup();
  }
}

function runAuditRepo(profile = "quick") {
  const fixture = createModuleReachabilityFixture(
    `vitest-module-reachability-${profile}-audit-`,
  );
  try {
    runScript("audit-repo.mjs", fixture, [
      "--profile",
      profile,
      "--production",
    ]);
    return {
      manifest: fixture.readJson("manifest.json", { from: "output" }),
      reachability: fixture.readJson("module-reachability.json", {
        from: "output",
      }),
      summaryMd: fixture.read("audit-summary.latest.md", { from: "output" }),
      reviewPackMd:
        profile === "quick"
          ? ""
          : fixture.read("audit-review-pack.latest.md", { from: "output" }),
    };
  } finally {
    fixture.cleanup();
  }
}

function asSet(value) {
  return new Set(value ?? []);
}

describe("module reachability artifact", () => {
  it(
    "E1-E12. separates runtime/type reachability and records unreachable SCC review evidence",
    () => {
      const artifact = runModuleReachabilityProducer();
      const runtimeReachable = asSet(artifact.runtimeReachableFiles);
      const typeReachable = asSet(artifact.typeReachableFiles);
      const reachable = asSet(artifact.reachableFiles);
      const unreachable = asSet(artifact.unreachableFiles);
      const boundedOut = asSet(artifact.boundedOutFiles);

      expect(artifact.meta?.tool).toBe("build-module-reachability.mjs");
      expect(artifact.meta?.entrySurfaceFile).toBe("entry-surface.json");
      expect(runtimeReachable.has("src/index.ts")).toBe(true);
      expect(runtimeReachable.has("src/runtime.ts")).toBe(true);
      expect(runtimeReachable.has("src/deep.ts")).toBe(true);
      expect(runtimeReachable.has("src/types.ts")).toBe(false);
      expect(typeReachable.has("src/types.ts")).toBe(true);
      expect(reachable.has("src/index.ts")).toBe(true);
      expect(reachable.has("src/runtime.ts")).toBe(true);
      expect(reachable.has("src/deep.ts")).toBe(true);
      expect(reachable.has("src/types.ts")).toBe(true);
      expect(unreachable.has("src/isolated.ts")).toBe(true);
      expect(boundedOut.has("src/isolated.ts")).toBe(false);
      expect(artifact.meta?.boundedOutReason).toBe(null);
      expect(artifact.summary?.boundedOut).toBe(0);
      expect(artifact.meta?.completenessBySubmodule?.src).toBe("high");
      expect(
        artifact.meta?.supports?.unreachableStronglyConnectedComponents,
      ).toBe(true);
      expect(artifact.unreachableStronglyConnectedComponents).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            files: ["src/components/App.ts", "src/components/Modal.ts"],
            kind: "entry-unreachable-scc",
            graph: "runtime",
          }),
        ]),
      );
      expect(artifact.summary?.unreachableStronglyConnectedComponents).toBe(1);
      expect(artifact.summary?.unreachableStronglyConnectedFiles).toBe(2);
    },
    MODULE_REACHABILITY_TEST_TIMEOUT_MS,
  );

  it(
    "E13-E14. treats capped traversal as bounded-out uncertainty",
    () => {
      const artifact = runModuleReachabilityProducer([
        "--max-files-visited",
        "1",
      ]);

      expect(artifact.meta?.boundedOutReason).toBe("max-files-visited");
      expect(artifact.boundedOutFiles).toContain("src/isolated.ts");
      expect(artifact.unreachableFiles).not.toContain("src/isolated.ts");
    },
    MODULE_REACHABILITY_TEST_TIMEOUT_MS,
  );

  it(
    "E15-E18. quick audit wires module reachability and summary evidence",
    () => {
      const audit = runAuditRepo();

      expect(
        audit.manifest.commandsRun?.some(
          (step) =>
            step.step === "build-module-reachability.mjs" &&
            step.status === "ok",
        ),
      ).toBe(true);
      expect(audit.manifest.artifactsProduced).toContain(
        "module-reachability.json",
      );
      expect(audit.reachability.unreachableFiles).toContain("src/isolated.ts");
      expect(audit.summaryMd).toContain("Unreachable SCCs: 1 group, 2 files");
      expect(audit.summaryMd).toContain(
        "module-reachability.json.unreachableStronglyConnectedComponents",
      );
      expect(audit.summaryMd).toContain(
        "before treating intra-cycle imports as liveness",
      );
    },
    MODULE_REACHABILITY_TEST_TIMEOUT_MS,
  );

  it(
    "E19. full audit review pack mirrors unreachable SCC review evidence",
    () => {
      const audit = runAuditRepo("full");

      expect(audit.reviewPackMd).toContain(
        "Unreachable SCCs: 1 group, 2 files",
      );
      expect(audit.reviewPackMd).toContain(
        "module-reachability.json.unreachableStronglyConnectedComponents",
      );
      expect(audit.reviewPackMd).toContain(
        "dead-file-group review evidence, not export SAFE_FIX",
      );
    },
    MODULE_REACHABILITY_TEST_TIMEOUT_MS,
  );
});
