import { describe, expect, it } from "vitest";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const NODE = process.execPath;
const AUDIT_REPO = path.join(ROOT, "audit-repo.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildPerformanceFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify({ name: "manifest-performance-fixture", type: "module" }),
  );
  write(
    root,
    "docs/current/audit/lumin-structural-audit.md",
    "# Living Structural Audit\n\n## Tracked Items\n\n",
  );
  const bigBody = Array.from(
    { length: 155 },
    (_, i) => `  const x${i} = ${i};`,
  ).join("\n");

  write(
    root,
    "src/a.ts",
    [
      "import { b } from './b';",
      "export function a() { return b(); }",
      "export function parseMaybe(raw: string) {",
      "  try { return JSON.parse(raw); } catch { return null; }",
      "}",
      `export function hugeProd() {\n${bigBody}\n  return 0;\n}`,
      "",
    ].join("\n"),
  );
  write(
    root,
    "src/b.ts",
    "export function b() { return 1; }\nexport function unused() { return 2; }\n",
  );
  write(
    root,
    "scripts/huge-smoke.mjs",
    `export function hugeScript() {\n${bigBody}\n  return 0;\n}\n`,
  );
}

function runAudit(args, options = {}) {
  return spawnSync(NODE, [AUDIT_REPO, ...args], {
    cwd: ROOT,
    encoding: "utf8",
    ...options,
  });
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

describe("audit-repo manifest and producer-performance split track", () => {
  it("O0. distinguishes default output privacy notes from explicit outside-root output notes", () => {
    const repo = mkdtempSync(
      path.join(tmpdir(), "lumin-manifest-output-notes-"),
    );
    const outsideOutput = mkdtempSync(
      path.join(tmpdir(), "lumin-manifest-output-outside-"),
    );
    try {
      write(
        repo,
        "package.json",
        JSON.stringify({ name: "output-note-fixture", type: "module" }),
      );
      write(repo, "src/a.ts", "export const a = 1;\n");

      const defaultOut = runAudit([
        "--root",
        repo,
        "--profile",
        "quick",
        "--production",
      ]);
      expect(
        defaultOut.status,
        `${defaultOut.stdout}\n${defaultOut.stderr}`,
      ).toBe(0);
      expect(existsSync(path.join(repo, ".audit", "manifest.json"))).toBe(true);
      expect(defaultOut.stderr).toMatch(
        /privacy note: default artifacts are written/,
      );
      expect(defaultOut.stderr).toContain(".audit/");

      const explicitOut = runAudit([
        "--root",
        repo,
        "--output",
        outsideOutput,
        "--profile",
        "quick",
        "--production",
      ]);
      expect(
        explicitOut.status,
        `${explicitOut.stdout}\n${explicitOut.stderr}`,
      ).toBe(0);
      expect(explicitOut.stderr).toMatch(/note: --output is outside --root/);
      expect(explicitOut.stderr).not.toMatch(/privacy note: default artifacts/);
    } finally {
      rmSync(repo, { recursive: true, force: true });
      rmSync(outsideOutput, { recursive: true, force: true });
    }
  }, 60_000);

  it("O1-O3. mirrors manifest and producer-performance evidence for quick production runs", () => {
    const repo = mkdtempSync(
      path.join(tmpdir(), "lumin-manifest-performance-"),
    );
    const output = path.join(repo, "audit-out");

    try {
      buildPerformanceFixture(repo);

      const result = runAudit([
        "--root",
        repo,
        "--output",
        output,
        "--profile",
        "quick",
        "--production",
      ]);
      expect(result.status, result.stderr.slice(0, 1200)).toBe(0);

      const manifest = readJson(path.join(output, "manifest.json"));
      const producerPerformance = readJson(
        path.join(output, "producer-performance.json"),
      );

      expect(manifest.profile).toBe("quick");
      expect(Array.isArray(manifest.commandsRun)).toBe(true);
      expect(manifest.scanRange).toBeDefined();
      expect(manifest.confidence).toBeDefined();
      expect(manifest.resolverDiagnostics).toMatchObject({
        resolverCapabilityArtifact: "resolver-capabilities.json",
        resolverDiagnosticsArtifact: "resolver-diagnostics.json",
      });
      expect(
        Array.isArray(manifest.resolverDiagnostics.topSpecifierRoots),
      ).toBe(true);
      expect(
        Array.isArray(manifest.resolverDiagnostics.topUnresolvedReasons),
      ).toBe(true);
      expect(Array.isArray(manifest.blindZones)).toBe(true);
      expect(manifest.generatedArtifacts).toMatchObject({
        mode: "default",
        generatedArtifactPolicyVersion: "generated-artifact-policy-v1",
        executedGenerators: false,
      });
      expect(
        Array.isArray(manifest.generatedArtifacts.topGeneratedMisses),
      ).toBe(true);

      expect(manifest.performance).toMatchObject({
        artifact: "producer-performance.json",
        producerCount: manifest.commandsRun.length,
      });
      expect(typeof manifest.performance.totalWallMs).toBe("number");
      expect(manifest.artifactsProduced).toContain("producer-performance.json");

      expect(producerPerformance).toMatchObject({
        schemaVersion: "producer-performance.v1",
        root: repo,
        output,
        profile: "quick",
      });
      expect(producerPerformance.scanRange).toMatchObject({
        includeTests: false,
        production: true,
      });
      expect(producerPerformance.producers).toHaveLength(
        manifest.commandsRun.length,
      );
      expect(
        producerPerformance.producers.every(
          (entry) =>
            typeof entry.name === "string" &&
            typeof entry.wallMs === "number" &&
            typeof entry.status === "string",
        ),
      ).toBe(true);
      expect(producerPerformance.summary.producerCount).toBe(
        producerPerformance.producers.length,
      );

      expect(producerPerformance.artifacts.totalBytes).toBeGreaterThan(0);
      expect(producerPerformance.artifacts.producedCount).toBeGreaterThan(0);
      expect(producerPerformance.artifacts.largest.length).toBeGreaterThan(0);
      expect(
        producerPerformance.artifacts.byName["symbols.json"].bytes,
      ).toBeGreaterThan(0);
      expect(manifest.performance.totalArtifactBytes).toBe(
        producerPerformance.artifacts.totalBytes,
      );
      expect(manifest.performance.largestArtifacts.length).toBeGreaterThan(0);

      expect(producerPerformance.artifactReads).toMatchObject({
        schemaVersion: "artifact-read-metrics.v1",
      });
      expect(producerPerformance.artifactReads.totalReadCount).toBeGreaterThan(
        0,
      );
      expect(producerPerformance.artifactReads.totalReadBytes).toBeGreaterThan(
        0,
      );
      expect(typeof producerPerformance.artifactReads.totalReadMs).toBe(
        "number",
      );
      expect(typeof producerPerformance.artifactReads.totalJsonParseMs).toBe(
        "number",
      );
      expect(
        producerPerformance.artifactReads.byName["symbols.json"].readCount,
      ).toBeGreaterThan(0);
      expect(
        producerPerformance.artifactReads.largestReads.length,
      ).toBeGreaterThan(0);
      expect(manifest.performance.artifactReadCount).toBe(
        producerPerformance.artifactReads.totalReadCount,
      );
      expect(manifest.performance.totalArtifactReadBytes).toBe(
        producerPerformance.artifactReads.totalReadBytes,
      );
      expect(manifest.performance.totalJsonParseMs).toBe(
        producerPerformance.artifactReads.totalJsonParseMs,
      );

      expect(producerPerformance.memory).toMatchObject({
        measurement: "orchestrator-process-snapshots",
        childPeakRssAvailable: false,
      });
      expect(
        producerPerformance.producers.every(
          (entry) =>
            typeof entry.memory?.before?.rssBytes === "number" &&
            typeof entry.memory?.after?.rssBytes === "number" &&
            typeof entry.memory?.delta?.rssBytes === "number",
        ),
      ).toBe(true);
      expect(
        typeof producerPerformance.summary.maxObservedOrchestratorRssBytes,
      ).toBe("number");

      const symbolProducer = producerPerformance.producers.find(
        (entry) => entry.name === "build-symbol-graph.mjs",
      );
      const topologyProducer = producerPerformance.producers.find(
        (entry) => entry.name === "measure-topology.mjs",
      );
      expect(
        producerPerformance.summary.phaseSupportCount,
      ).toBeGreaterThanOrEqual(2);
      expect(manifest.performance.phaseSupportCount).toBe(
        producerPerformance.summary.phaseSupportCount,
      );
      expect(symbolProducer?.phases).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            name: "snapshot",
            wallMs: expect.any(Number),
          }),
          expect.objectContaining({
            name: "extract-changed-files",
            wallMs: expect.any(Number),
          }),
        ]),
      );
      expect(topologyProducer?.phases).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            name: "process-changed-files",
            wallMs: expect.any(Number),
          }),
        ]),
      );
      expect(topologyProducer?.counters).toMatchObject({
        jsFilesProcessed: expect.any(Number),
        scannerFilesAttempted: expect.any(Number),
        scannerAcceptedFiles: expect.any(Number),
        scannerFallbackFiles: expect.any(Number),
        oxcParseCalls: expect.any(Number),
        resolverMemoHits: expect.any(Number),
        resolverMemoMisses: expect.any(Number),
      });
      expect(topologyProducer.counters.jsFilesProcessed).toBeGreaterThan(0);
      expect(topologyProducer.counters.scannerFilesAttempted).toBeGreaterThan(
        0,
      );
      expect(topologyProducer.counters.scannerAcceptedFiles).toBeGreaterThan(0);

      const symbolCounters = symbolProducer?.counters ?? {};
      expect(symbolCounters.snapshotFiles).toBeGreaterThan(0);
      expect(symbolCounters.changedFiles).toBeGreaterThan(0);
      expect(symbolCounters.changedJsFiles).toBeGreaterThan(0);
      expect(symbolCounters.extractedFiles).toBeGreaterThan(0);
      expect(symbolCounters.fileDataFiles).toBeGreaterThan(0);
      expect(symbolCounters.definitionCount).toBeGreaterThan(0);
      expect(symbolCounters.useCount).toBeGreaterThan(0);
      expect(typeof symbolCounters.reExportCount).toBe("number");
      expect(typeof symbolCounters.parseErrorCount).toBe("number");
      expect(typeof symbolCounters.totalUses).toBe("number");
      expect(typeof symbolCounters.resolvedInternalUses).toBe("number");
      expect(typeof symbolCounters.unresolvedInternalUses).toBe("number");
      expect(symbolCounters.symbolsJsonBytes).toBeGreaterThan(0);

      const symbolPhaseNames = new Set(
        (symbolProducer?.phases ?? []).map((phase) => phase.name),
      );
      expect(
        [
          "assemble-file-data",
          "assemble-def-index",
          "assemble-namespace-reexports",
          "assemble-source-uses",
          "assemble-mdx-uses",
          "assemble-generated-blind-zones",
          "assemble-dead-candidates",
          "assemble-fan-in",
          "assemble-any-contamination",
          "assemble-source-use-resolve",
          "assemble-source-use-external",
          "assemble-source-use-asset",
          "assemble-source-use-unresolved",
          "assemble-source-use-generated-virtual",
          "assemble-source-use-namespace-reexport",
          "assemble-source-use-resolved-internal",
        ].every((name) => symbolPhaseNames.has(name)),
      ).toBe(true);
      expect(typeof symbolCounters.sourceUseResolveMs).toBe("number");
      expect(typeof symbolCounters.sourceUseResolverMemoHits).toBe("number");
      expect(typeof symbolCounters.sourceUseResolverMemoMisses).toBe("number");
      expect(typeof symbolCounters.sourceUseResolverStageRelativeAttempts).toBe(
        "number",
      );
      expect(typeof symbolCounters.sourceUseResolverStageRelativeMs).toBe(
        "number",
      );
      expect(
        typeof symbolCounters.sourceUseResolverStageRelativeCacheMisses,
      ).toBe("number");
      expect(
        typeof symbolCounters.sourceUseResolverStageScopedTsconfigProbeHits,
      ).toBe("number");
      expect(
        typeof symbolCounters.sourceUseResolverStageScopedTsconfigProbeMisses,
      ).toBe("number");
      expect(typeof symbolCounters.sourceUseResolverStageExternalResults).toBe(
        "number",
      );
      expect(typeof symbolCounters.sourceUseResolvedInternalBranchCount).toBe(
        "number",
      );
      expect(typeof symbolCounters.sourceUseExternalBranchCount).toBe("number");

      const steps = manifest.commandsRun.map((command) => command.step);
      expect(steps).toEqual(
        expect.arrayContaining([
          "triage-repo.mjs",
          "build-symbol-graph.mjs",
          "build-resolver-diagnostics.mjs",
          "classify-dead-exports.mjs",
          "rank-fixes.mjs",
        ]),
      );
      expect(steps).not.toEqual(
        expect.arrayContaining([
          "measure-staleness.mjs",
          "merge-runtime-evidence.mjs",
          "build-call-graph.mjs",
          "check-barrel-discipline.mjs",
          "build-shape-index.mjs",
          "build-function-clone-index.mjs",
        ]),
      );
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 90_000);
});
