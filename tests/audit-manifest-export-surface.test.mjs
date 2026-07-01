import { describe, expect, it } from "vitest";

import * as auditManifest from "../_lib/audit-manifest.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function withManifestFixture(fn, options = {}) {
  const fixture = createTempRepoFixture({
    prefix: options.prefix ?? "audit-manifest-export-surface-",
  });
  try {
    return fn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function buildManifestEvidence(fixture, options = {}) {
  return auditManifest.buildManifestEvidence({
    root: fixture.root,
    outDir: fixture.output,
    includeTests: true,
    production: false,
    ...options,
  });
}

describe("audit-manifest public surface", () => {
  it("AMES1. exposes manifest builders, not living-audit internals", () => {
    expect(typeof auditManifest.buildManifestEvidence).toBe("function");
    expect(typeof auditManifest.refreshManifestEvidence).toBe("function");
    expect(typeof auditManifest.collectProducedArtifacts).toBe("function");
    expect(typeof auditManifest.buildManifestCompanionUpdate).toBe("function");
    expect(typeof auditManifest.buildProducerPerformanceArtifactForAuditRun).toBe(
      "function",
    );

    for (const symbol of [
      "LIVING_AUDIT_DOC_CANDIDATES",
      "detectLivingAuditDocs",
      "mergeRustAnalysisRun",
      "buildArtifactSizeSummary",
      "buildArtifactReadMetricsSummary",
      "buildProducerPerformanceArtifactFromRuntime",
      "buildManifestMeta",
      "buildManifestEvidenceUpdate",
      "ARTIFACT_READ_EVENTS_SCHEMA_VERSION",
    ]) {
      expect(Object.hasOwn(auditManifest, symbol)).toBe(false);
    }
  });

  it("AMES1d. companion manifest wrapper leaves companion block shapes in audit-core", () => {
    const update = auditManifest.buildManifestCompanionUpdate({
      topologyMermaidPath: "C:/repo/.audit/topology.mermaid.md",
      auditSummaryPath: "C:/repo/.audit/audit-summary.latest.md",
      reviewPackPath: "C:/repo/.audit/audit-review-pack.latest.md",
    });

    expect(update).toMatchObject({
      topologyMermaid: {
        path: "C:/repo/.audit/topology.mermaid.md",
        format: "markdown",
        source: "topology.json",
        use: "human visual companion; topology.json remains authoritative for exact citations",
      },
      auditSummary: {
        path: "C:/repo/.audit/audit-summary.latest.md",
        format: "markdown",
      },
      reviewPack: {
        path: "C:/repo/.audit/audit-review-pack.latest.md",
        format: "markdown",
      },
    });
    expect(update.reviewPack.use).toContain("the engine never calls external APIs");
  });

  it("AMES1e. refreshManifestEvidence applies the Rust-owned evidence patch", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "triage.json",
        {
          shape: {
            totalFiles: 2,
            tsFiles: 1,
            rsFiles: 1,
          },
        },
        { to: "output" },
      );
      fixture.writeJson(
        "symbols.json",
        {
          uses: {
            external: 0,
            resolvedInternal: 0,
            unresolvedInternal: 0,
            unresolvedInternalRatio: 0,
          },
        },
        { to: "output" },
      );
      fixture.write("framework-resource-surfaces.json", "{not-json", {
        to: "output",
      });
      fixture.write(
        "rust-analyzer-health.latest.json",
        JSON.stringify({
          schemaVersion: "lumin-rust-analyzer-health.v1",
        }),
        { to: "output" },
      );

      const reads = [];
      const manifest = {};
      auditManifest.refreshManifestEvidence(manifest, {
        root: fixture.root,
        outDir: fixture.output,
        includeTests: false,
        production: true,
        onArtifactRead: (read) => reads.push(read),
      });

      expect(manifest.scanRange.files).toBe(2);
      expect(manifest.scanRange.includeTests).toBe(false);
      expect(manifest.scanRange.production).toBe(true);
      expect(manifest.blindZones).toEqual(expect.any(Array));
      expect(manifest.frameworkResourceSurfaces?.status).toBe("unavailable");
      expect(manifest.frameworkResourceSurfaces?.reason?.kind).toBe(
        "malformed-json",
      );
      expect(reads.some((read) => read.filePath.endsWith("triage.json"))).toBe(
        true,
      );
      expect(reads.some((read) => read.filePath.endsWith("symbols.json"))).toBe(
        true,
      );
      expect(
        reads.some((read) =>
          read.filePath.endsWith("rust-analyzer-health.latest.json"),
        ),
      ).toBe(true);
    }));

  it("AMES1c. producer performance audit-run wrapper leaves audit context projection in audit-core", () =>
    withManifestFixture((fixture) => {
      fixture.write("triage.json", "{}", { to: "output" });
      const artifact = auditManifest.buildProducerPerformanceArtifactForAuditRun({
        generated: "2026-07-01T00:00:00.000Z",
        root: fixture.root,
        outDir: fixture.output,
        profile: "quick",
        includeTests: true,
        production: false,
        excludes: ["dist"],
        autoExcludes: [".audit"],
        noIncremental: true,
        cacheRoot: fixture.path(".audit/.cache"),
        clearIncrementalCache: true,
        generatedArtifactsMode: "prepared",
        artifactReads: {
          schemaVersion: "artifact-read-metrics.v1",
          measurement: "audit-repo-orchestrator-json-reads",
          totalReadCount: 0,
          totalReadBytes: 0,
          totalReadMs: 0,
          totalJsonParseMs: 0,
          parseFailureCount: 0,
          byName: {},
        },
        artifactsProduced: ["triage.json"],
        commandsRun: [{ step: "triage-repo.mjs", status: "ok", ms: 3 }],
        skipped: [{ step: "emit-sarif.mjs", reason: "not in --sarif mode" }],
      });

      expect(artifact).toMatchObject({
        schemaVersion: "producer-performance.v1",
        profile: "quick",
        scanRange: {
          includeTests: true,
          production: false,
          excludes: ["dist"],
          autoExcludes: [".audit"],
        },
        cache: {
          noIncremental: true,
          clearIncrementalCache: true,
        },
        generatedArtifacts: { mode: "prepared" },
        summary: {
          producerCount: 1,
          okCount: 1,
          skippedCount: 1,
          artifactCount: 1,
        },
      });
    }));
});

describe("audit-manifest evidence summaries", () => {
  it("AMES1b. buildManifestEvidence can merge rustAnalysis run state in audit-core", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "rust-analyzer-health.latest.json",
        {
          schemaVersion: "lumin-rust-analyzer.v1",
          policyVersion: "lumin-rust-analyzer-policy.v1",
          meta: {
            producer: "lumin-rust-analyzer",
            mode: "rust-main",
            input: { root: fixture.root },
          },
          summary: { files: 1, syntaxReviewSignals: 0 },
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture, {
        rustAnalysisRun: {
          requested: true,
          ran: true,
          status: "complete",
          rustFiles: 1,
          sourceCommit: "abc123",
        },
        mergeRustAnalysisRun: true,
      });

      expect(evidence.rustAnalysis).toMatchObject({
        requested: true,
        ran: true,
        status: "complete",
        available: true,
        files: 1,
        sourceCommit: "abc123",
      });
    }));

  it("AMES2. buildManifestEvidence summarizes generated artifact misses", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "symbols.json",
        {
          uses: {
            unresolvedInternalRatio: 0.2,
            unresolvedInternal: 3,
          },
          unresolvedInternalSpecifierRecords: [
            {
              specifier: "@scope/prisma/enums",
              consumerFile: "apps/web/src/a.ts",
              reason: "workspace-generated-artifact-missing",
              hint: "generated-artifact-missing",
              generatedArtifact: {
                policyVersion: "generated-artifact-policy-v1",
                generatorFamily: "prisma",
                confidence: "strong",
                matchedPackage: "@scope/prisma",
                targetSubpath: "enums",
              },
            },
            {
              specifier: "@scope/prisma/enums",
              consumerFile: "apps/web/src/b.ts",
              reason: "workspace-generated-artifact-missing",
              hint: "generated-artifact-missing",
              generatedArtifact: {
                policyVersion: "generated-artifact-policy-v1",
                generatorFamily: "prisma",
                confidence: "strong",
                matchedPackage: "@scope/prisma",
                targetSubpath: "enums",
              },
            },
            {
              specifier: "@scope/types/missing",
              consumerFile: "apps/web/src/c.ts",
              reason: "workspace-package-subpath-target-missing",
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.generatedArtifacts?.reasonSummary).toEqual({
        "workspace-generated-artifact-missing": 2,
      });
      expect(evidence.generatedArtifacts?.mode).toBe("default");
      expect(evidence.generatedArtifacts?.executedGenerators).toBe(false);
      expect(evidence.generatedArtifacts?.generatedArtifactPolicyVersion).toBe(
        "generated-artifact-policy-v1",
      );
      expect(evidence.generatedArtifacts?.supportedGenerators).toEqual([]);
      expect(evidence.generatedArtifacts?.topGeneratedMisses).toEqual([
        {
          specifier: "@scope/prisma/enums",
          matchedPackage: "@scope/prisma",
          targetSubpath: "enums",
          count: 2,
          generatorFamily: "prisma",
          confidence: "strong",
        },
      ]);
    }));

  it("AMES2c. buildManifestEvidence summarizes framework/resource surfaces", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "framework-resource-surfaces.json",
        {
          schemaVersion: "framework-resource-surfaces.v1",
          policyVersion: "framework-resource-surface-policy-v1",
          files: [
            {
              file: "src/Button.stories.tsx",
              surfaceLanes: [
                {
                  lane: "framework-dispatch-entry",
                  capabilityPack: "framework.storybook",
                  confidence: "grounded",
                  framework: "storybook",
                  reason: "storybook-story-file",
                },
              ],
            },
            {
              file: "templates/controller.ts.hbs",
              surfaceLanes: [
                {
                  lane: "scaffold-template-resource",
                  capabilityPack: "surface.scaffold-template",
                  confidence: "resource-only",
                  reason: "handlebars-template-resource",
                },
              ],
            },
          ],
          summary: {
            totalFilesWithSurfaces: 2,
            totalSurfaceLanes: 2,
            byLane: {
              "framework-dispatch-entry": 1,
              "scaffold-template-resource": 1,
            },
            byCapabilityPack: {
              "framework.storybook": 1,
              "surface.scaffold-template": 1,
            },
            byConfidence: {
              grounded: 1,
              "resource-only": 1,
            },
          },
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.frameworkResourceSurfaces?.artifact).toBe(
        "framework-resource-surfaces.json",
      );
      expect(evidence.frameworkResourceSurfaces?.policyVersion).toBe(
        "framework-resource-surface-policy-v1",
      );
      expect(evidence.frameworkResourceSurfaces?.totalFilesWithSurfaces).toBe(
        2,
      );
      expect(evidence.frameworkResourceSurfaces?.byLane).toEqual({
        "framework-dispatch-entry": 1,
        "scaffold-template-resource": 1,
      });
      expect(evidence.frameworkResourceSurfaces?.byCapabilityPack).toEqual({
        "framework.storybook": 1,
        "surface.scaffold-template": 1,
      });
      expect(evidence.frameworkResourceSurfaces?.topExamples).toEqual([
        {
          file: "src/Button.stories.tsx",
          lanes: ["framework-dispatch-entry"],
          capabilityPacks: ["framework.storybook"],
          reasons: ["storybook-story-file"],
        },
        {
          file: "templates/controller.ts.hbs",
          lanes: ["scaffold-template-resource"],
          capabilityPacks: ["surface.scaffold-template"],
          reasons: ["handlebars-template-resource"],
        },
      ]);
    }));

  it("AMES2d. buildManifestEvidence summarizes unused dependency evidence", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "unused-deps.json",
        {
          schemaVersion: "unused-deps.v1",
          policyVersion: "unused-deps-review-policy-v1",
          status: "complete",
          summary: {
            packageCount: 2,
            declaredDependencyCount: 5,
            usedCount: 1,
            reviewUnusedCount: 2,
            mutedCount: 2,
            confidenceLimitedCount: 0,
            unavailableCount: 0,
            byReason: {
              "external-import-consumer": 1,
              "no-observed-consumer": 2,
              "package-script-tool": 1,
              "ambient-types": 1,
            },
          },
          packages: [
            {
              packageDir: "packages/app",
              manifestPath: "packages/app/package.json",
              dependencies: [
                {
                  name: "left-pad",
                  field: "dependencies",
                  status: "review-unused",
                  reason: "no-observed-consumer",
                  confidence: "review",
                },
              ],
            },
            {
              packageDir: ".",
              manifestPath: "package.json",
              dependencies: [
                {
                  name: "unused-lib",
                  field: "devDependencies",
                  status: "review-unused",
                  reason: "no-observed-consumer",
                  confidence: "review",
                },
                {
                  name: "tsx",
                  field: "devDependencies",
                  status: "muted",
                  reason: "package-script-tool",
                  confidence: "grounded",
                },
              ],
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.unusedDependencies).toMatchObject({
        artifact: "unused-deps.json",
        schemaVersion: "unused-deps.v1",
        policyVersion: "unused-deps-review-policy-v1",
        status: "complete",
        declaredDependencyCount: 5,
        reviewUnusedCount: 2,
        mutedCount: 2,
        byReason: {
          "external-import-consumer": 1,
          "no-observed-consumer": 2,
          "package-script-tool": 1,
          "ambient-types": 1,
        },
      });
      expect(evidence.unusedDependencies?.topReviewUnused).toEqual([
        {
          packageDir: ".",
          manifestPath: "package.json",
          name: "unused-lib",
          field: "devDependencies",
          reason: "no-observed-consumer",
          confidence: "review",
        },
        {
          packageDir: "packages/app",
          manifestPath: "packages/app/package.json",
          name: "left-pad",
          field: "dependencies",
          reason: "no-observed-consumer",
          confidence: "review",
        },
      ]);
    }));

  it("AMES2g. buildManifestEvidence mirrors block clone summary without source fragments", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "block-clones.json",
        {
          schemaVersion: "block-clones.v1",
          policyVersion: "block-clone-review-policy-v1",
          status: "complete",
          normalization: {
            policyId: "block-clone-normalization-v1",
            mode: "alpha-identifier",
          },
          thresholds: {
            policyId: "block-clone-threshold-policy-v2",
            minTokens: 50,
            minLines: 5,
            minOccurrences: 2,
            maxInstancesPerGroup: 20,
            maxCandidateGroups: 1000,
            maxReviewGroups: 100,
            maxMutedGroups: 100,
            maxGroups: 40,
            maxTokensPerFile: 200000,
          },
          summary: {
            fileCount: 12,
            tokenCount: 3400,
            groupCount: 2,
            instanceCount: 5,
            reviewGroupCount: 1,
            mutedGroupCount: 1,
            skippedFileCount: 1,
            unavailableFileCount: 0,
          },
          noisePolicy: {
            policyId: "block-clone-noise-policy-v1",
            reviewGroupCount: 1,
            mutedGroupCount: 1,
            mutedByReason: {
              "node-vitest-mirror-pair": 1,
            },
            candidateCapSaturated: false,
            reviewCapSaturated: false,
            mutedCapSaturated: false,
          },
          groups: [
            {
              id: "block-clone:sha256:abc",
              claim: "repeated normalized token region",
              instances: [
                { file: "src/a.ts", startLine: 1, endLine: 8 },
                { file: "src/b.ts", startLine: 2, endLine: 9 },
              ],
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.blockClones).toEqual({
        artifact: "block-clones.json",
        schemaVersion: "block-clones.v1",
        policyVersion: "block-clone-review-policy-v1",
        status: "complete",
        reviewOnly: true,
        normalizationPolicyId: "block-clone-normalization-v1",
        normalizationMode: "alpha-identifier",
        thresholdPolicyId: "block-clone-threshold-policy-v2",
        noisePolicyId: "block-clone-noise-policy-v1",
        thresholds: {
          minTokens: 50,
          minLines: 5,
          minOccurrences: 2,
          maxInstancesPerGroup: 20,
          maxTokensPerFile: 200000,
          maxCandidateGroups: 1000,
          maxReviewGroups: 100,
          maxMutedGroups: 100,
          maxGroups: 40,
        },
        fileCount: 12,
        tokenCount: 3400,
        groupCount: 2,
        instanceCount: 5,
        reviewGroupCount: 1,
        mutedGroupCount: 1,
        mutedByReason: {
          "node-vitest-mirror-pair": 1,
        },
        candidateCapSaturated: false,
        reviewCapSaturated: false,
        mutedCapSaturated: false,
        skippedFileCount: 1,
        unavailableFileCount: 0,
      });
      expect(Object.hasOwn(evidence.blockClones, "groups")).toBe(false);
      expect(Object.hasOwn(evidence.blockClones, "instances")).toBe(false);
    }));

  it("AMES2e. buildManifestEvidence preserves unavailable unused dependency status", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "unused-deps.json",
        {
          schemaVersion: "unused-deps.v1",
          policyVersion: "unused-deps-review-policy-v1",
          status: "unavailable",
          reason: "input-artifact-missing",
          summary: {
            packageCount: 0,
            declaredDependencyCount: 0,
            usedCount: 0,
            reviewUnusedCount: 0,
            mutedCount: 0,
            confidenceLimitedCount: 0,
            unavailableCount: 0,
            byReason: {},
          },
          packages: [],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.unusedDependencies).toMatchObject({
        artifact: "unused-deps.json",
        status: "unavailable",
        reason: "input-artifact-missing",
        reviewUnusedCount: 0,
        topReviewUnused: [],
      });
    }));

  it("AMES2f. buildManifestEvidence tolerates malformed unused dependency package lists", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "unused-deps.json",
        {
          schemaVersion: "unused-deps.v1",
          policyVersion: "unused-deps-review-policy-v1",
          status: "complete",
          summary: {
            packageCount: 1,
            declaredDependencyCount: 1,
            usedCount: 0,
            reviewUnusedCount: 1,
            mutedCount: 0,
            confidenceLimitedCount: 0,
            unavailableCount: 0,
            byReason: { "no-observed-consumer": 1 },
          },
          packages: [
            {
              packageDir: ".",
              manifestPath: "package.json",
              dependencies: {},
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.unusedDependencies).toMatchObject({
        status: "complete",
        reviewUnusedCount: 1,
        topReviewUnused: [],
      });
    }));

  it("AMES2b. buildManifestEvidence summarizes generated consumer blind zones by scope", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "symbols.json",
        {
          generatedConsumerBlindZones: [
            {
              reason: "generated-consumer-blind-zone",
              sourceReason: "workspace-generated-artifact-missing",
              specifier: "@scope/prisma/enums",
              consumerFile: "apps/web/src/a.ts",
              matchedPackage: "@scope/prisma",
              targetSubpath: "enums",
              candidatePath: "packages/prisma/generated/enums.ts",
              status: "missing",
              scopePackageRoot: "packages/prisma",
              mode: "default",
            },
            {
              reason: "generated-consumer-blind-zone",
              sourceReason: "workspace-generated-artifact-missing",
              specifier: "@scope/prisma/enums",
              consumerFile: "apps/api/src/b.ts",
              matchedPackage: "@scope/prisma",
              targetSubpath: "enums",
              candidatePath: "packages/prisma/generated/enums.ts",
              status: "present-but-out-of-scope",
              scanScopeReason: "excluded",
              scopePackageRoot: "packages/prisma",
              mode: "prepared",
              staleStatus: "unknown",
              staleReason: "generator-input-hash-not-recorded",
            },
            {
              reason: "generated-consumer-blind-zone",
              sourceReason: "workspace-generated-artifact-missing",
              specifier: "@scope/kysely/types",
              consumerFile: "apps/api/src/c.ts",
              matchedPackage: "@scope/kysely",
              targetSubpath: "types",
              candidatePath: "packages/kysely/generated/types.ts",
              status: "missing",
              scopePackageRoot: "packages/kysely",
              mode: "default",
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.generatedArtifacts?.generatedConsumerBlindZoneCount).toBe(
        3,
      );
      expect(
        evidence.generatedArtifacts?.topGeneratedConsumerBlindZones,
      ).toEqual([
        {
          scopePackageRoot: "packages/prisma",
          count: 2,
          statuses: {
            missing: 1,
            "present-but-out-of-scope": 1,
          },
          topSpecifiers: [{ specifier: "@scope/prisma/enums", count: 2 }],
          examples: [
            {
              specifier: "@scope/prisma/enums",
              consumerFile: "apps/api/src/b.ts",
              candidatePath: "packages/prisma/generated/enums.ts",
              status: "present-but-out-of-scope",
              scanScopeReason: "excluded",
              mode: "prepared",
            },
            {
              specifier: "@scope/prisma/enums",
              consumerFile: "apps/web/src/a.ts",
              candidatePath: "packages/prisma/generated/enums.ts",
              status: "missing",
              mode: "default",
            },
          ],
        },
        {
          scopePackageRoot: "packages/kysely",
          count: 1,
          statuses: {
            missing: 1,
          },
          topSpecifiers: [{ specifier: "@scope/kysely/types", count: 1 }],
          examples: [
            {
              specifier: "@scope/kysely/types",
              consumerFile: "apps/api/src/c.ts",
              candidatePath: "packages/kysely/generated/types.ts",
              status: "missing",
              mode: "default",
            },
          ],
        },
      ]);
    }));

  it("AMES3. generated present mode reports existing targets excluded by scan policy", () =>
    withManifestFixture((fixture) => {
      fixture.write(
        "packages/prisma/generated/enums.ts",
        'export enum Kind { A = "A" }\n',
      );
      fixture.writeJson(
        "symbols.json",
        {
          unresolvedInternalSpecifierRecords: [
            {
              specifier: "@scope/prisma/generated/enums",
              consumerFile: "apps/web/src/a.ts",
              reason: "workspace-generated-artifact-missing",
              hint: "generated-artifact-missing",
              targetCandidates: ["packages/prisma/generated/enums.ts"],
              generatedArtifact: {
                policyVersion: "generated-artifact-policy-v1",
                generatorFamily: "prisma",
                confidence: "strong",
                matchedPackage: "@scope/prisma",
                targetSubpath: "generated/enums",
              },
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture, {
        excludes: ["packages/prisma/generated"],
        generatedArtifactsMode: "present",
      });

      expect(evidence.generatedArtifacts?.mode).toBe("present");
      expect(evidence.generatedArtifacts?.presentButOutOfScopeCount).toBe(1);
      expect(evidence.generatedArtifacts?.presentButOutOfScope).toEqual([
        {
          specifier: "@scope/prisma/generated/enums",
          consumerFile: "apps/web/src/a.ts",
          matchedPackage: "@scope/prisma",
          targetSubpath: "generated/enums",
          candidatePath: "packages/prisma/generated/enums.ts",
          reason: "present-but-out-of-scope",
          mode: "present",
        },
      ]);
    }));

  it("AMES4. generated prepared mode marks existing excluded targets as stale-unknown", () =>
    withManifestFixture((fixture) => {
      fixture.write(
        "packages/prisma/generated/enums.ts",
        'export enum Kind { A = "A" }\n',
      );
      fixture.writeJson(
        "symbols.json",
        {
          unresolvedInternalSpecifierRecords: [
            {
              specifier: "@scope/prisma/generated/enums",
              consumerFile: "apps/web/src/a.ts",
              reason: "workspace-generated-artifact-missing",
              hint: "generated-artifact-missing",
              targetCandidates: [
                fixture.path("packages/prisma/generated/enums.ts"),
              ],
              generatedArtifact: {
                policyVersion: "generated-artifact-policy-v1",
                generatorFamily: "prisma",
                confidence: "strong",
                matchedPackage: "@scope/prisma",
                targetSubpath: "generated/enums",
              },
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture, {
        excludes: ["packages/prisma/generated"],
        generatedArtifactsMode: "prepared",
      });

      expect(evidence.generatedArtifacts?.mode).toBe("prepared");
      expect(
        evidence.generatedArtifacts?.presentButOutOfScope?.[0]?.staleStatus,
      ).toBe("unknown");
      expect(
        evidence.generatedArtifacts?.presentButOutOfScope?.[0]?.staleReason,
      ).toBe("generator-input-hash-not-recorded");
    }));

  it("AMES5. buildManifestEvidence summarizes resolver unresolved roots and reasons", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "symbols.json",
        {
          uses: {
            resolvedInternal: 7,
            unresolvedInternalRatio: 0.31,
            unresolvedInternal: 4,
            external: 2,
          },
          topUnresolvedSpecifiers: [
            {
              specifierPrefix: "@scope/orm",
              count: 3,
              example: "@scope/orm/client",
            },
          ],
          unresolvedInternalSpecifierRecords: [
            {
              specifier: "@scope/orm/client",
              consumerFile: "apps/api/src/a.ts",
              reason: "workspace-generated-artifact-missing",
              resolverStage: "workspace-package-subpath",
              hint: "generated-artifact-missing",
              typeOnly: false,
            },
            {
              specifier: "@scope/orm/client",
              consumerFile: "apps/web/src/b.ts",
              reason: "workspace-generated-artifact-missing",
              resolverStage: "workspace-package-subpath",
              hint: "generated-artifact-missing",
              typeOnly: true,
            },
            {
              specifier: "@scope/orm/helpers",
              consumerFile: "apps/web/src/c.ts",
              reason: "workspace-package-subpath-target-missing",
              resolverStage: "workspace-package-subpath",
            },
            {
              specifier: "app/routes/root",
              consumerFile: "apps/web/src/d.ts",
              reason: "tsconfig-path-target-missing",
              resolverStage: "tsconfig-paths",
            },
          ],
          unresolvedInternalSummaryByReason: {
            "workspace-generated-artifact-missing": { count: 2 },
            "workspace-package-subpath-target-missing": { count: 1 },
            "tsconfig-path-target-missing": { count: 1 },
          },
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.resolverDiagnostics?.unresolvedInternal).toBe(4);
      expect(evidence.resolverDiagnostics?.unresolvedInternalRatio).toBe(0.31);
      expect(evidence.resolverDiagnostics?.topUnresolvedReasons).toEqual([
        { reason: "workspace-generated-artifact-missing", count: 2 },
        { reason: "tsconfig-path-target-missing", count: 1 },
        { reason: "workspace-package-subpath-target-missing", count: 1 },
      ]);
      expect(evidence.resolverDiagnostics?.topSpecifierRoots).toEqual([
        {
          specifierRoot: "@scope/orm",
          count: 3,
          reasons: {
            "workspace-generated-artifact-missing": 2,
            "workspace-package-subpath-target-missing": 1,
          },
          examples: [
            {
              specifier: "@scope/orm/client",
              consumerFile: "apps/api/src/a.ts",
            },
            {
              specifier: "@scope/orm/client",
              consumerFile: "apps/web/src/b.ts",
            },
            {
              specifier: "@scope/orm/helpers",
              consumerFile: "apps/web/src/c.ts",
            },
          ],
        },
        {
          specifierRoot: "app",
          count: 1,
          reasons: {
            "tsconfig-path-target-missing": 1,
          },
          examples: [
            { specifier: "app/routes/root", consumerFile: "apps/web/src/d.ts" },
          ],
        },
      ]);
      expect(evidence.resolverDiagnostics?.topUnresolvedSpecifiers).toEqual([
        {
          specifierPrefix: "@scope/orm",
          count: 3,
          example: "@scope/orm/client",
        },
      ]);
    }));

  it("AMES5s. buildManifestEvidence summarizes SFC evidence counts without raw records", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "symbols.json",
        {
          uses: {
            sfcScriptConsumers: 4,
            sfcScriptSrcReachability: 2,
            sfcStyleAssetReferences: 3,
            sfcTemplateComponentRefs: 5,
            sfcGlobalComponentRegistrations: 2,
            sfcGeneratedComponentManifests: 1,
            sfcFrameworkConventionComponents: 2,
          },
          sfcTemplateComponentRefs: [
            {
              tagName: "SecretCard",
              consumerFile: "src/App.vue",
            },
          ],
          sfcGlobalComponentRegistrations: [
            {
              componentName: "GlobalSecret",
              consumerFile: "src/main.ts",
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);

      expect(evidence.sfcEvidence).toEqual({
        artifact: "symbols.json",
        status: "complete",
        scriptImportConsumerCount: 4,
        reachabilityOnlyCount: 2,
        reviewOnlyEvidenceCount: 13,
        totalEvidenceCount: 19,
        byLane: {
          scriptImportConsumers: 4,
          scriptSrcReachability: 2,
          styleAssetReferences: 3,
          templateComponentRefs: 5,
          globalComponentRegistrations: 2,
          generatedComponentManifests: 1,
          frameworkConventionComponents: 2,
        },
        scanGapStillApplies: true,
      });
      expect(JSON.stringify(evidence.sfcEvidence)).not.toContain("SecretCard");
      expect(JSON.stringify(evidence.sfcEvidence)).not.toContain(
        "GlobalSecret",
      );
    }));

  it("AMES5b. manifest resolver blind zone uses resolver-diagnostics summary when present", () =>
    withManifestFixture((fixture) => {
      fixture.writeJson(
        "symbols.json",
        {
          uses: {
            resolvedInternal: 7,
            unresolvedInternalRatio: 0.31,
            unresolvedInternal: 4,
            external: 2,
          },
          topUnresolvedSpecifiers: [
            {
              specifierPrefix: "@legacy/fallback",
              count: 4,
              example: "@legacy/fallback/a",
            },
          ],
          unresolvedInternalSpecifierRecords: [
            {
              specifier: "@legacy/fallback/a",
              consumerFile: "apps/api/src/legacy.ts",
              reason: "legacy-symbols-fallback",
            },
          ],
          unresolvedInternalSummaryByReason: {
            "legacy-symbols-fallback": { count: 4 },
          },
        },
        { to: "output" },
      );
      fixture.writeJson(
        "resolver-diagnostics.json",
        {
          schemaVersion: "resolver-diagnostics.v1",
          resolverVersion: "resolver-2026-05-v1",
          summary: {
            unresolvedInternal: 4,
            unresolvedInternalRatio: 0.31,
            blindZoneCount: 3,
            candidateTargetCount: 2,
            unresolvedImportCount: 4,
            blockedCandidateHintCount: 3,
            reasonCounts: {
              "workspace-package-subpath-target-missing": 3,
              "hash-import-target-missing": 1,
            },
            topFamilies: [
              { family: "workspace-packages", count: 3 },
              { family: "node-imports", count: 1 },
            ],
            topAffectedPackageScopes: [
              { affectedPackageScope: "packages/lib", count: 2 },
              { affectedPackageScope: "packages/app", count: 1 },
            ],
            topUnresolvedReasons: [
              { reason: "workspace-package-subpath-target-missing", count: 3 },
              { reason: "hash-import-target-missing", count: 1 },
            ],
            topSpecifierRoots: [
              {
                specifierRoot: "@scope/lib",
                count: 3,
                reasons: {
                  "workspace-package-subpath-target-missing": 3,
                },
                examples: [
                  {
                    specifier: "@scope/lib/missing",
                    consumerFile: "apps/web/src/a.ts",
                  },
                ],
              },
            ],
          },
          blindZones: [
            {
              family: "workspace-packages",
              reason: "workspace-package-subpath-target-missing",
              specifier: "@scope/lib/missing",
              importer: "apps/web/src/a.ts",
              affectedPackageScope: "packages/lib",
              blocksAbsenceClaims: true,
              relevance: "target-candidate-package-scope",
            },
            {
              family: "node-imports",
              reason: "hash-import-target-missing",
              specifier: "#config",
              importer: "packages/app/src/a.ts",
              affectedPackageScope: "packages/app",
              blocksAbsenceClaims: true,
              relevance: "affected-package-scope",
            },
          ],
          blockedCandidateHints: [
            {
              family: "workspace-packages",
              reason: "workspace-package-subpath-target-missing",
              specifier: "@scope/lib/missing",
              importer: "apps/web/src/a.ts",
              affectedPackageScope: "packages/lib",
              blockingScope: "candidate-relevant",
              relevance: "target-candidate-package-scope",
              proofUse: "blocks-absence-claim",
              candidatePath: "packages/lib/missing.ts",
            },
            {
              family: "node-imports",
              reason: "hash-import-target-missing",
              specifier: "#config",
              importer: "packages/app/src/a.ts",
              affectedPackageScope: "packages/app",
              blockingScope: "candidate-relevant",
              relevance: "affected-package-scope",
              proofUse: "blocks-absence-claim",
              candidatePath: "packages/app/src/config.ts",
            },
            {
              family: "generated-artifacts",
              reason: "generated-consumer-blind-zone",
              specifier: "@scope/generated/client",
              importer: "packages/app/src/use-client.ts",
              affectedPackageScope: "packages/generated",
              blockingScope: "candidate-relevant",
              relevance: "generated-consumer-scope",
              proofUse: "blocks-absence-claim",
              candidatePath: "packages/generated/client.ts",
            },
          ],
          candidateTargets: [
            {
              specifier: "@scope/lib/missing",
              candidates: ["packages/lib/missing.ts"],
            },
            {
              specifier: "#config",
              candidates: ["packages/app/src/config.ts"],
            },
          ],
          unresolvedImports: [
            {
              specifier: "@scope/lib/missing",
              consumerFile: "apps/web/src/a.ts",
            },
          ],
        },
        { to: "output" },
      );

      const evidence = buildManifestEvidence(fixture);
      const resolverZone = evidence.blindZones.find(
        (zone) => zone?.area === "resolver",
      );

      expect(resolverZone).toBeTruthy();
      expect(resolverZone.details?.sourceArtifact).toBe(
        "resolver-diagnostics.json",
      );
      expect(resolverZone.details?.resolverVersion).toBe("resolver-2026-05-v1");
      expect(resolverZone.details?.blindZoneCount).toBe(3);
      expect(resolverZone.details?.candidateTargetCount).toBe(2);
      expect(resolverZone.details?.unresolvedImportCount).toBe(4);
      expect(evidence.resolverDiagnostics?.blockedCandidateHintCount).toBe(3);
      expect(
        evidence.resolverDiagnostics?.blockedCandidateHintSampleLimit,
      ).toBe(10);
      expect(evidence.resolverDiagnostics?.blockedCandidateHints).toEqual([
        {
          family: "workspace-packages",
          reason: "workspace-package-subpath-target-missing",
          specifier: "@scope/lib/missing",
          importer: "apps/web/src/a.ts",
          affectedPackageScope: "packages/lib",
          blockingScope: "candidate-relevant",
          relevance: "target-candidate-package-scope",
          proofUse: "blocks-absence-claim",
          candidatePath: "packages/lib/missing.ts",
        },
        {
          family: "node-imports",
          reason: "hash-import-target-missing",
          specifier: "#config",
          importer: "packages/app/src/a.ts",
          affectedPackageScope: "packages/app",
          blockingScope: "candidate-relevant",
          relevance: "affected-package-scope",
          proofUse: "blocks-absence-claim",
          candidatePath: "packages/app/src/config.ts",
        },
        {
          family: "generated-artifacts",
          reason: "generated-consumer-blind-zone",
          specifier: "@scope/generated/client",
          importer: "packages/app/src/use-client.ts",
          affectedPackageScope: "packages/generated",
          blockingScope: "candidate-relevant",
          relevance: "generated-consumer-scope",
          proofUse: "blocks-absence-claim",
          candidatePath: "packages/generated/client.ts",
        },
      ]);
      expect(
        evidence.resolverDiagnostics?.blockedCandidateHintReasonCounts,
      ).toEqual([
        {
          reason: "generated-consumer-blind-zone",
          count: 1,
          families: { "generated-artifacts": 1 },
        },
        {
          reason: "hash-import-target-missing",
          count: 1,
          families: { "node-imports": 1 },
        },
        {
          reason: "workspace-package-subpath-target-missing",
          count: 1,
          families: { "workspace-packages": 1 },
        },
      ]);
      expect(
        evidence.resolverDiagnostics?.blockedCandidateHintFamilyCounts,
      ).toEqual([
        {
          family: "generated-artifacts",
          count: 1,
          reasons: { "generated-consumer-blind-zone": 1 },
        },
        {
          family: "node-imports",
          count: 1,
          reasons: { "hash-import-target-missing": 1 },
        },
        {
          family: "workspace-packages",
          count: 1,
          reasons: { "workspace-package-subpath-target-missing": 1 },
        },
      ]);
      expect(resolverZone.details?.reasonCounts).toEqual({
        "workspace-package-subpath-target-missing": 3,
        "hash-import-target-missing": 1,
      });
      expect(resolverZone.details?.topFamilies).toEqual([
        { family: "workspace-packages", count: 3 },
        { family: "node-imports", count: 1 },
      ]);
      expect(evidence.resolverDiagnostics?.topAffectedPackageScopes).toEqual([
        { affectedPackageScope: "packages/lib", count: 2 },
        { affectedPackageScope: "packages/app", count: 1 },
      ]);
      expect(resolverZone.details?.topAffectedPackageScopes).toEqual([
        { affectedPackageScope: "packages/lib", count: 2 },
        { affectedPackageScope: "packages/app", count: 1 },
      ]);
      expect(resolverZone.details?.topUnresolvedReasons).toEqual([
        { reason: "workspace-package-subpath-target-missing", count: 3 },
        { reason: "hash-import-target-missing", count: 1 },
      ]);
      expect(resolverZone.details?.topSpecifierRoots).toEqual([
        {
          specifierRoot: "@scope/lib",
          count: 3,
          reasons: {
            "workspace-package-subpath-target-missing": 3,
          },
          examples: [
            {
              specifier: "@scope/lib/missing",
              consumerFile: "apps/web/src/a.ts",
            },
          ],
        },
      ]);
    }));
});
