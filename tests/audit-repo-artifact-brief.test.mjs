import { describe, expect, it } from "vitest";
import { spawnSync } from "node:child_process";
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
import { fileURLToPath } from "node:url";

import { renderAuditReviewPack } from "../_lib/audit-review-pack.mjs";
import { renderAuditSummary } from "../_lib/audit-summary.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const NODE = process.execPath;
const AUDIT_REPO = path.join(ROOT, "audit-repo.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildTinyRepo(root) {
  write(root, "package.json", JSON.stringify({ name: "brief-fixture" }));
  write(root, "src/a.ts", "export const alive = 1;\n");
}

function runAudit(root, output, args = []) {
  return spawnSync(
    NODE,
    [AUDIT_REPO, "--root", root, "--output", output, ...args],
    {
      encoding: "utf8",
    },
  );
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

describe("audit-repo artifact brief split track", () => {
  it("A0pre. rejects invalid audit-repo options before applying defaults", () => {
    const typo = spawnSync(NODE, [AUDIT_REPO, "--profil", "full"], {
      encoding: "utf8",
    });
    expect(typo.status).toBe(2);
    expect(typo.stderr).toMatch(/unknown option\(s\): --profil/);

    const badGeneratedMode = spawnSync(
      NODE,
      [AUDIT_REPO, "--root", ROOT, "--generated-artifacts", "run"],
      { encoding: "utf8" },
    );
    expect(badGeneratedMode.status).toBe(2);
    expect(badGeneratedMode.stderr).toMatch(
      /unsupported --generated-artifacts mode: run/,
    );
  });

  it("A0. renders summary and review-pack text as artifact evidence, not recommendation prose", () => {
    const manifest = {
      meta: { generated: "2026-04-28T00:00:00.000Z" },
      profile: "full",
      scanRange: { files: 10, languages: ["ts"], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
      blindZones: [],
      livingAudit: {
        existingDocs: [
          { path: "docs/current/audit/lumin-structural-audit.md" },
        ],
      },
    };
    const symbols = {
      meta: { supports: { anyContamination: true } },
      helperOwnersByIdentity: {
        "src/dirty.ts::dirtyHelper": {
          ownerFile: "src/dirty.ts",
          exportedName: "dirtyHelper",
          kind: "FunctionDeclaration",
          line: 7,
          anyContamination: {
            label: "severely-any-contaminated",
            labels: [
              "has-any",
              "any-contaminated",
              "severely-any-contaminated",
            ],
            measurements: { explicitAnyCount: 3 },
          },
        },
      },
      typeOwnersByIdentity: {
        "src/dirty.ts::DirtyShape": {
          ownerFile: "src/dirty.ts",
          exportedName: "DirtyShape",
          kind: "TSInterfaceDeclaration",
          line: 1,
          anyContamination: {
            label: "severely-any-contaminated",
            labels: [
              "has-any",
              "any-contaminated",
              "severely-any-contaminated",
            ],
            measurements: { explicitAnyCount: 3 },
          },
        },
      },
    };

    const summary = renderAuditSummary({
      manifest,
      checklistFacts: {
        E2_silent_catch: {
          gate: "watch",
          count: 0,
          nonEmptyAnonymousCount: 1,
          unusedParamCount: 0,
          nonEmptyAnonymousSites: [{ file: "src/errors.ts", line: 10 }],
        },
      },
      fixPlan: {
        summary: { REVIEW_FIX: 7 },
        reviewFixes: [
          {
            finding: { file: "src/dead.ts", line: 1, symbol: "Dead" },
          },
        ],
      },
      topology: {
        summary: { sccCount: 1 },
        sccs: [{ members: ["src/a.ts", "src/b.ts"] }],
        largestFiles: [],
      },
      discipline: {
        totals: {
          ":any": 41,
          "as any": 0,
          "@ts-ignore": 0,
          "@ts-expect-error": 108,
        },
        overallTopOffenders: [
          { file: "src/types.ts", total: 20, breakdown: { ":any": 20 } },
        ],
      },
      callGraph: {
        summary: { semiDead: 2 },
        semiDeadList: [
          { file: "src/test.ts", symbol: "unusedImport", source: "./helper" },
        ],
      },
      symbols,
    });

    expect(summary).toContain("# Audit Artifact Brief");
    expect(summary).toContain("not a recommendation engine");
    expect(summary).toContain("Do not paste it as the final user answer");
    expect(summary).toContain("## Measured Cues (Unranked)");
    expect(summary).toContain("Runtime cycles: 1");
    expect(summary).toContain("Type-check escapes: 149");
    expect(summary).toContain("Call graph: semi-dead imports 2");
    expect(summary).toContain("`discipline.json`");
    expect(summary).toContain("`call-graph.json`");
    expect(summary).toContain("`symbols.json`");
    expect(summary).toContain(
      "Exported any-contamination: 1 severe type owner, 1 severe helper owner",
    );
    expect(summary).toContain("## Expansion Hint");
    expect(summary).toContain("## Living Audit Tracking");
    expect(summary).toContain("NOT_RECHECKED");
    expect(summary).not.toContain("Ask the coding agent:");
    expect(summary).not.toMatch(/^\d+\. /m);

    const caveatedPostWriteSummary = renderAuditSummary({
      manifest: {
        meta: { generated: "2026-04-28T00:00:00.000Z" },
        profile: "quick",
        scanRange: { files: 1, languages: ["ts"], includeTests: true },
        confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
        blindZones: [],
        postWrite: {
          requested: true,
          ran: true,
          silentNew: 0,
          baselineStatus: "missing",
          scanRangeParity: "baseline-missing",
          afterComplete: true,
        },
      },
    });
    expect(caveatedPostWriteSummary).toContain("delta confidence is limited");
    expect(caveatedPostWriteSummary).toContain("baseline=missing");
    expect(caveatedPostWriteSummary).not.toContain(
      "found 0 new unplanned any-like escapes",
    );

    const reviewPack = renderAuditReviewPack({
      manifest: {
        scanRange: { files: 10, languages: ["ts"], includeTests: true },
        resolverDiagnostics: {
          blockedCandidateHintCount: 2,
          blockedCandidateHintSampleLimit: 10,
          blockedCandidateHints: [
            {
              candidatePath: "src/generated.ts",
              specifier: "@pkg/generated",
              reason: "generated-consumer-blind-zone",
            },
          ],
        },
      },
      discipline: { totals: { ":any": 41 } },
      symbols,
    });

    expect(reviewPack).toContain(
      "Identity-level anyContamination: 1 severe type owner, 1 severe helper owner",
    );
    expect(reviewPack).toContain("Inspect symbols.json owner maps");
    expect(reviewPack).toContain("Resolver blocked absence hints");
    expect(reviewPack).not.toContain("Ask the coding agent:");

    const manifestWithFrameworkResourceSurfaces = {
      meta: { generated: "2026-05-09T00:00:00.000Z" },
      profile: "full",
      scanRange: { files: 12, languages: ["ts", "js"], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
      blindZones: [],
      frameworkResourceSurfaces: {
        artifact: "framework-resource-surfaces.json",
        totalFilesWithSurfaces: 4,
        byLane: {
          "framework-dispatch-entry": 2,
          "scaffold-template-resource": 1,
          "bundled-build-artifact": 1,
        },
        byConfidence: {
          grounded: 2,
          "resource-only": 1,
          "generated-output-review": 1,
        },
        topExamples: [
          {
            file: "src/Button.stories.tsx",
            lanes: ["framework-dispatch-entry"],
            reasons: ["storybook-story-file"],
          },
        ],
      },
    };
    const summaryWithFrameworkResources = renderAuditSummary({
      manifest: manifestWithFrameworkResourceSurfaces,
    });
    expect(summaryWithFrameworkResources).toContain(
      "Framework/resource surfaces: 4 files",
    );
    expect(summaryWithFrameworkResources).toContain(
      "framework-dispatch-entry 2",
    );
    expect(summaryWithFrameworkResources).toContain(
      "scaffold-template-resource 1",
    );
    expect(summaryWithFrameworkResources).toContain("bundled-build-artifact 1");
    expect(summaryWithFrameworkResources).toContain(
      "framework-resource-surfaces.json",
    );
    expect(summaryWithFrameworkResources).toContain(
      "before treating import absence as deadness",
    );

    const reviewPackWithFrameworkResources = renderAuditReviewPack({
      manifest: manifestWithFrameworkResourceSurfaces,
      fixPlan: {
        summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 },
      },
      deadClassify: { summary: { excluded: {} } },
    });
    expect(reviewPackWithFrameworkResources).toContain(
      "Framework/resource surfaces: 4 files",
    );
    expect(reviewPackWithFrameworkResources).toContain(
      "framework-dispatch-entry 2",
    );
    expect(reviewPackWithFrameworkResources).toContain(
      "Read manifest.json.frameworkResourceSurfaces and framework-resource-surfaces.json",
    );

    const manifestWithUnusedDependencies = {
      meta: { generated: "2026-05-24T00:00:00.000Z" },
      profile: "full",
      scanRange: { files: 12, languages: ["ts", "js"], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
      blindZones: [],
      artifactsProduced: ["unused-deps.json"],
      unusedDependencies: {
        artifact: "unused-deps.json",
        schemaVersion: "unused-deps.v1",
        policyVersion: "unused-deps-review-policy-v1",
        status: "complete",
        reviewUnusedCount: 2,
        mutedCount: 3,
        confidenceLimitedCount: 0,
        topReviewUnused: [
          {
            name: "left-pad",
            packageRoot: ".",
            dependencyField: "dependencies",
          },
        ],
      },
    };
    const summaryWithUnusedDependencies = renderAuditSummary({
      manifest: manifestWithUnusedDependencies,
    });
    const dependencySummaryLines = summaryWithUnusedDependencies
      .split("\n")
      .filter((line) => line.includes("Dependency hygiene"));
    expect(dependencySummaryLines.join("\n")).toContain(
      "2 review-only dependency declarations need inspection",
    );
    expect(dependencySummaryLines.join("\n")).toContain(
      "3 muted explanations",
    );
    expect(dependencySummaryLines.join("\n")).toContain(
      "manifest.json.unusedDependencies",
    );
    expect(dependencySummaryLines.join("\n")).toContain("unused-deps.json");
    expect(summaryWithUnusedDependencies).toContain(
      "`unused-deps.json`: review-only dependency declaration evidence",
    );
    expect(summaryWithUnusedDependencies).not.toContain("left-pad");
    expect(dependencySummaryLines.join("\n")).not.toMatch(
      /\b(safe|remove|delete|uninstall|drop|fix)\b/i,
    );

    const reviewPackWithUnusedDependencies = renderAuditReviewPack({
      manifest: manifestWithUnusedDependencies,
      fixPlan: {
        summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 },
      },
      deadClassify: { summary: { excluded: {} } },
    });
    const dependencyReviewLines = reviewPackWithUnusedDependencies
      .split("\n")
      .filter((line) => line.includes("Dependency hygiene"));
    expect(dependencyReviewLines.join("\n")).toContain(
      "Dependency hygiene review: inspect unused-deps.json before changing package manifests",
    );
    expect(dependencyReviewLines.join("\n")).toContain(
      "review-only=2; muted=3; confidence-limited=0",
    );
    expect(reviewPackWithUnusedDependencies).toContain("unused-deps.json");
    expect(reviewPackWithUnusedDependencies).not.toContain("left-pad");
    expect(dependencyReviewLines.join("\n")).not.toMatch(
      /\b(safe|remove|delete|uninstall|drop|fix)\b/i,
    );

    const manifestWithUnavailableUnusedDependencies = {
      ...manifestWithUnusedDependencies,
      unusedDependencies: {
        artifact: "unused-deps.json",
        schemaVersion: "unused-deps.v1",
        status: "unavailable",
        reason: "input-artifact-missing",
        reviewUnusedCount: 0,
        mutedCount: 0,
        confidenceLimitedCount: 0,
      },
    };
    const unavailableSummary = renderAuditSummary({
      manifest: manifestWithUnavailableUnusedDependencies,
    });
    const unavailableReviewPack = renderAuditReviewPack({
      manifest: manifestWithUnavailableUnusedDependencies,
      fixPlan: {
        summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 },
      },
      deadClassify: { summary: { excluded: {} } },
    });
    expect(unavailableSummary).toContain(
      "Dependency hygiene: evidence incomplete; do not infer dependency declaration absence",
    );
    expect(unavailableReviewPack).toContain(
      "Dependency hygiene review: evidence incomplete; do not infer dependency declaration absence",
    );

    const manifestWithSfcEvidence = {
      meta: { generated: "2026-05-30T00:00:00.000Z" },
      profile: "full",
      scanRange: {
        files: 12,
        languages: ["ts", "vue", "svelte", "astro"],
        includeTests: true,
      },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
      blindZones: [
        {
          area: "sfc-scan-gap",
          severity: "scan-gap",
          details: { files: 3 },
        },
      ],
      sfcEvidence: {
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
      },
    };
    const summaryWithSfcEvidence = renderAuditSummary({
      manifest: manifestWithSfcEvidence,
    });
    const sfcSummaryLines = summaryWithSfcEvidence
      .split("\n")
      .filter((line) => line.includes("SFC evidence"));
    expect(sfcSummaryLines.join("\n")).toContain("19 records");
    expect(sfcSummaryLines.join("\n")).toContain("script imports 4");
    expect(sfcSummaryLines.join("\n")).toContain("template refs 5");
    expect(sfcSummaryLines.join("\n")).toContain("framework conventions 2");
    expect(sfcSummaryLines.join("\n")).toContain(
      "manifest.json.sfcEvidence",
    );
    expect(sfcSummaryLines.join("\n")).toContain("SFC arrays in `symbols.json`");
    expect(sfcSummaryLines.join("\n")).toContain(
      "sfc-scan-gap still applies",
    );
    expect(sfcSummaryLines.join("\n")).not.toMatch(
      /\b(safe|remove|delete|uninstall|drop|fix)\b/i,
    );

    const reviewPackWithSfcEvidence = renderAuditReviewPack({
      manifest: manifestWithSfcEvidence,
      fixPlan: {
        summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 },
      },
      deadClassify: { summary: { excluded: {} } },
    });
    const sfcReviewLines = reviewPackWithSfcEvidence
      .split("\n")
      .filter((line) => line.includes("SFC evidence review"));
    expect(sfcReviewLines.join("\n")).toContain(
      "inspect manifest.json.sfcEvidence and SFC arrays in symbols.json",
    );
    expect(sfcReviewLines.join("\n")).toContain("template-refs=5");
    expect(sfcReviewLines.join("\n")).toContain("review-only=13");
    expect(sfcReviewLines.join("\n")).toContain(
      "sfc-scan-gap still applies",
    );
    expect(sfcReviewLines.join("\n")).not.toMatch(
      /\b(safe|remove|delete|uninstall|drop|fix)\b/i,
    );
  });

  it("O4/O7. quick audit writes artifact brief outputs and console preview without recommendation wording", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-brief-quick-"));
    const output = path.join(repo, ".audit");
    try {
      buildTinyRepo(repo);
      const result = runAudit(repo, output, ["--profile", "quick"]);
      expect(result.status, result.stderr.slice(0, 800)).toBe(0);

      const manifest = readJson(path.join(output, "manifest.json"));
      const summary = readFileSync(
        path.join(output, "audit-summary.latest.md"),
        "utf8",
      );

      expect(manifest.artifactsProduced).toEqual(
        expect.arrayContaining([
          "audit-summary.latest.md",
          "topology.mermaid.md",
        ]),
      );
      expect(existsSync(path.join(output, "topology.mermaid.md"))).toBe(true);
      expect(summary).toContain("# Audit Artifact Brief");
      expect(summary).toContain("not a recommendation engine");
      expect(summary).not.toContain("Ask the coding agent:");

      expect(result.stdout).toContain("audit-summary.latest.md");
      expect(result.stdout).toContain("artifact brief preview");
      expect(result.stdout).not.toMatch(/produced \d+\/\d+ artifacts/i);
      expect(result.stderr).not.toContain("artifact brief preview");
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 30_000);

  it("O10c2. full audit writes the review-pack artifact without turning it into final-answer guidance", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-brief-full-"));
    const output = path.join(repo, ".audit");
    try {
      buildTinyRepo(repo);
      const result = runAudit(repo, output, ["--profile", "full"]);
      expect(result.status, result.stderr.slice(0, 800)).toBe(0);

      const manifest = readJson(path.join(output, "manifest.json"));
      const reviewPackPath = path.join(output, "audit-review-pack.latest.md");
      const reviewPack = readFileSync(reviewPackPath, "utf8");

      expect(existsSync(reviewPackPath)).toBe(true);
      expect(manifest.artifactsProduced).toContain(
        "audit-review-pack.latest.md",
      );
      expect(reviewPack).toContain("Lane");
      expect(reviewPack).not.toContain(
        "Do not paste it as the final user answer",
      );
      expect(reviewPack).not.toContain("Ask the coding agent:");
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 30_000);
});
