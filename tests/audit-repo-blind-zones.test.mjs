import { describe, expect, it } from "vitest";
import {
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

import { runAuditCoreJson } from "../_lib/audit-core.mjs";
import { formatBlindZonesSummary } from "../_lib/blind-zones.mjs";
import { buildManifestEvidence } from "../_lib/audit-manifest.mjs";
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

function runAudit(args) {
  return spawnSync(NODE, [AUDIT_REPO, ...args], {
    cwd: ROOT,
    encoding: "utf8",
  });
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

function rustBlindZones(input) {
  const dir = mkdtempSync(path.join(tmpdir(), "lumin-rust-blind-zones-"));
  const fixture = path.join(dir, "input.json");
  try {
    writeFileSync(fixture, JSON.stringify(input));
    return runAuditCoreJson(
      ["blind-zones-summary", "--input", fixture],
      "rustBlindZones",
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function rustBlindZoneCases(caseInputs) {
  const dir = mkdtempSync(path.join(tmpdir(), "lumin-rust-blind-zone-cases-"));
  const fixture = path.join(dir, "cases.json");
  const cases = Object.entries(caseInputs).map(([name, input]) => ({
    name,
    input,
  }));
  try {
    writeFileSync(fixture, JSON.stringify(cases));
    return Object.fromEntries(
      runAuditCoreJson(
        ["blind-zones-summary", "--cases", fixture],
        "rustBlindZoneCases",
      ).map((result) => [result.name, result.blindZones]),
    );
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

describe("audit-repo blind-zone and confidence split track", () => {
  it("B1-B2c. creates language scan and precision gaps without fake absence claims", () => {
    const zones = rustBlindZoneCases({
      rust: {
        triage: { byLanguage: { ts: 100, rs: 42 } },
      },
      sfc: {
        triage: {
          shape: {
            totalFiles: 4,
            tsFiles: 1,
            jsFiles: 0,
            pyFiles: 0,
            goFiles: 0,
            sfcFiles: 3,
          },
          byLanguage: { ts: 1, vue: 1, svelte: 1, astro: 1 },
        },
      },
      python: {
        triage: { byLanguage: { py: 244 } },
      },
      unavailablePython: {
        triage: {
          shape: {
            totalFiles: 3,
            tsFiles: 2,
            jsFiles: 0,
            pyFiles: 1,
            goFiles: 0,
          },
        },
        symbols: {
          meta: {
            languageSupport: {
              python: { enabled: false, reason: "python executable unavailable" },
            },
          },
        },
      },
      unavailableGo: {
        triage: {
          shape: {
            totalFiles: 3,
            tsFiles: 2,
            jsFiles: 0,
            pyFiles: 0,
            goFiles: 1,
          },
        },
        symbols: {
          meta: {
            languageSupport: {
              go: { enabled: false, reason: "tree-sitter unavailable" },
            },
          },
        },
      },
    });
    const rustZones = zones.rust;
    const rust = rustZones.find((zone) => zone.area === "rs");
    expect(rust).toMatchObject({
      severity: "scan-gap",
    });
    expect(rust.effect).toContain("absence claims");

    const sfcZones = zones.sfc;
    const sfc = sfcZones.find((zone) => zone.area === "sfc-scan-gap");
    expect(sfc).toMatchObject({
      severity: "scan-gap",
      details: {
        files: 3,
        languages: {
          vue: 1,
          svelte: 1,
          astro: 1,
        },
      },
    });
    expect(sfc.effect).toContain("single-file components");
    expect(
      sfcZones.some((zone) => ["vue", "svelte", "astro"].includes(zone.area)),
    ).toBe(false);

    const pythonZones = zones.python;
    const python = pythonZones.find(
      (zone) => zone.area === "python-method-resolution",
    );
    expect(python).toMatchObject({
      severity: "precision-gap",
    });
    expect(python.effect).toContain("Method-level");

    const unavailablePythonZones = zones.unavailablePython;
    const pythonScanGap = unavailablePythonZones.find(
      (zone) => zone.area === "python-scan-gap",
    );
    expect(pythonScanGap).toMatchObject({
      severity: "scan-gap",
    });
    expect(pythonScanGap.details.reason).toContain("python");

    const unavailableGoZones = zones.unavailableGo;
    const goScanGap = unavailableGoZones.find(
      (zone) => zone.area === "go-scan-gap",
    );
    expect(goScanGap).toMatchObject({
      severity: "scan-gap",
    });
    expect(goScanGap.details.reason).toContain("tree-sitter");
  }, 30_000);

  it("B2d. only the current valid Rust analyzer artifact clears Rust blind zones", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-rust-blind-zone-"));
    const output = path.join(repo, "audit-out");
    try {
      mkdirSync(output, { recursive: true });
      write(
        output,
        "triage.json",
        JSON.stringify({
          shape: { totalFiles: 1, rustFiles: 1 },
          byLanguage: { rs: 1 },
        }),
      );

      const validRustArtifact = {
        schemaVersion: "lumin-rust-analyzer.v1",
        policyVersion: "lumin-rust-analyzer-policy.v1",
        meta: {
          producer: "lumin-rust-analyzer",
          mode: "rust-main",
          input: {
            root: repo,
            effectiveSourceHealthProfile: "compact",
            semanticMode: "metadata-only",
            includeTests: false,
            exclude: ["generated"],
          },
        },
        phases: {
          syntax: {
            meta: {
              input: {
                includeTests: false,
                exclude: ["generated"],
                pathPolicy: {
                  exclude: ["**/target/**", "**/vendor/**", "generated"],
                },
              },
            },
          },
        },
        summary: { files: 1, syntaxReviewSignals: 0 },
      };

      write(
        output,
        "rust-analyzer-health.latest.json",
        JSON.stringify(validRustArtifact),
      );
      const staleEvidence = buildManifestEvidence({
        root: repo,
        outDir: output,
        includeTests: false,
        production: true,
        rustAnalysisRun: { requested: false, ran: false, status: "not-requested" },
      });
      expect(staleEvidence.rustAnalysis).toMatchObject({
        status: "complete",
        available: true,
      });
      expect(staleEvidence.blindZones.some((zone) => zone.area === "rs")).toBe(true);

      write(
        output,
        "rust-analyzer-health.latest.json",
        JSON.stringify({
          schemaVersion: "lumin-rust-analyzer.v1",
          meta: validRustArtifact.meta,
          summary: {},
        }),
      );
      const malformedEvidence = buildManifestEvidence({
        root: repo,
        outDir: output,
        includeTests: false,
        production: true,
        rustAnalysisRun: { requested: true, ran: true, status: "complete" },
      });
      expect(malformedEvidence.rustAnalysis).toMatchObject({
        status: "invalid-shape",
        available: false,
      });
      expect(malformedEvidence.blindZones.some((zone) => zone.area === "rs")).toBe(true);

      write(
        output,
        "rust-analyzer-health.latest.json",
        JSON.stringify(validRustArtifact),
      );
      const currentEvidence = buildManifestEvidence({
        root: repo,
        outDir: output,
        includeTests: false,
        production: true,
        rustAnalysisRun: { requested: true, ran: true, status: "complete" },
      });
      expect(currentEvidence.rustAnalysis).toMatchObject({
        status: "complete",
        available: true,
        files: 1,
        scanScope: {
          includeTests: false,
          exclude: ["generated"],
          pathPolicy: {
            exclude: ["**/target/**", "**/vendor/**", "generated"],
          },
        },
      });
      expect(currentEvidence.blindZones.some((zone) => zone.area === "rs")).toBe(false);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("B2e. reused output dirs do not report stale Rust analyzer artifacts as produced", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-rust-stale-artifact-"));
    const output = path.join(repo, ".audit");
    try {
      mkdirSync(output, { recursive: true });
      write(
        repo,
        "package.json",
        JSON.stringify({ name: "rust-stale-artifact-fixture", type: "module" }),
      );
      write(repo, "src/lib.rs", "pub fn live() {}\n");
      write(
        output,
        "rust-analyzer-health.latest.json",
        JSON.stringify({
          schemaVersion: "lumin-rust-analyzer.v1",
          policyVersion: "lumin-rust-analyzer-policy.v1",
          meta: {
            producer: "lumin-rust-analyzer",
            mode: "rust-main",
            input: { root: repo },
          },
          summary: { files: 1 },
        }),
      );

      const result = runAudit([
        "--root",
        repo,
        "--output",
        output,
        "--profile",
        "quick",
        "--production",
      ]);
      expect(result.status, `${result.stdout}\n${result.stderr}`).toBe(0);

      const manifest = readJson(path.join(output, "manifest.json"));
      const summary = readFileSync(
        path.join(output, "audit-summary.latest.md"),
        "utf8",
      );
      expect(manifest.rustAnalysis).toMatchObject({
        requested: false,
        ran: false,
        status: "not-requested",
        artifact: "rust-analyzer-health.latest.json",
        artifactStatus: "complete",
      });
      expect(manifest.artifactsProduced).not.toContain(
        "rust-analyzer-health.latest.json",
      );
      expect(summary).not.toContain(
        "`rust-analyzer-health.latest.json`: Rust-owned",
      );
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 60_000);

  it("B2b. mirrors symbol parse-error warnings into manifest confidence", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-parse-confidence-"));
    try {
      write(
        repo,
        "triage.json",
        JSON.stringify({
          shape: { totalFiles: 2, tsFiles: 1, jsFiles: 1 },
        }),
      );
      write(
        repo,
        "symbols.json",
        JSON.stringify({
          meta: {
            warnings: [{ code: "parse-errors", count: 2 }],
          },
          uses: {
            unresolvedInternalRatio: 0,
            external: 0,
            resolvedInternal: 0,
            unresolvedInternal: 0,
          },
          filesWithParseErrors: ["src/a.js", "src/b.js"],
        }),
      );

      const evidence = buildManifestEvidence({
        root: repo,
        outDir: repo,
        includeTests: true,
        production: false,
      });

      expect(evidence.confidence.parseErrors).toBe(2);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("B3-B4b. preserves resolver confidence policy, absolute counts, grouped reasons, and concentrated roots", () => {
    const zones = rustBlindZoneCases({
      highRatio: {
        triage: { byLanguage: { ts: 100 } },
        symbols: {
          uses: { unresolvedInternalRatio: 0.22, unresolvedInternal: 50 },
          topUnresolvedSpecifiers: [{ specifierPrefix: "@/" }],
        },
      },
      lowRatio: {
        triage: { byLanguage: { ts: 100 } },
        symbols: {
          uses: { unresolvedInternalRatio: 0.02, unresolvedInternal: 3 },
        },
      },
      absoluteCount: {
        triage: { byLanguage: { ts: 5000 } },
        symbols: {
          uses: { unresolvedInternalRatio: 0.07, unresolvedInternal: 1200 },
          topUnresolvedSpecifiers: [
            { specifierPrefix: "@workspace/pkg", count: 120 },
          ],
          unresolvedInternalSpecifierRecords: [
            {
              specifier: "@workspace/pkg/generated",
              reason: "tsconfig-path-target-missing",
            },
            {
              specifier: "@workspace/pkg/generated2",
              reason: "tsconfig-path-target-missing",
            },
            {
              specifier: "@workspace/pkg/subpath",
              reason: "workspace-package-subpath-target-missing",
            },
          ],
        },
      },
      groupedReason: {
        triage: { byLanguage: { ts: 5000 } },
        symbols: {
          uses: { unresolvedInternalRatio: 0.06, unresolvedInternal: 1300 },
          topUnresolvedSpecifiers: [
            { specifierPrefix: "@workspace/", count: 800 },
          ],
          unresolvedInternalSummaryByReason: {
            "workspace-package-subpath-target-missing": {
              count: 12,
              spaces: { type: 12, value: 0, unknown: 0 },
              resolverStages: { workspacePackageSubpath: 12 },
              examples: [
                {
                  specifier: "@workspace/types/foo",
                  consumerFile: "apps/web/src/a.ts",
                },
              ],
            },
            "tsconfig-path-target-missing": {
              count: 4,
              spaces: { type: 1, value: 3, unknown: 0 },
              hints: { "generated-artifact-missing": 4 },
              examples: [
                {
                  specifier: "@/generated/client",
                  consumerFile: "apps/web/src/b.ts",
                },
              ],
            },
          },
          unresolvedInternalSpecifierRecords: [
            { specifier: "@/legacy", reason: "legacy-record-only" },
          ],
        },
      },
      concentratedRoot: {
        triage: { byLanguage: { ts: 5000 } },
        symbols: {
          uses: { unresolvedInternalRatio: 0.05, unresolvedInternal: 220 },
          topUnresolvedSpecifiers: [
            { specifierPrefix: "@workspace/", count: 190 },
            { specifierPrefix: "#/", count: 12 },
          ],
        },
      },
    });
    const highRatioZones = zones.highRatio;
    const highRatio = highRatioZones.find((zone) => zone.area === "resolver");
    expect(highRatio).toMatchObject({
      severity: "confidence-gap",
    });
    expect(highRatio.effect).toContain("FP-36");
    expect(highRatio.details.thresholdPolicy).toMatchObject({
      policyId: "resolver-blind-zone-policy",
      policyVersion: "resolver-blind-zone-policy-v1",
      thresholds: { unresolvedRatio: 0.15 },
    });

    const lowRatioZones = zones.lowRatio;
    expect(lowRatioZones.find((zone) => zone.area === "resolver")).toBeFalsy();

    const absoluteCountZones = zones.absoluteCount;
    const absoluteCount = absoluteCountZones.find(
      (zone) => zone.area === "resolver",
    );
    expect(absoluteCount).toMatchObject({
      severity: "confidence-gap",
      details: {
        unresolvedInternal: 1200,
        trigger: "absolute-count",
      },
    });
    expect(absoluteCount.details.topUnresolvedReasons[0]).toMatchObject({
      reason: "tsconfig-path-target-missing",
      count: 2,
    });

    const groupedReasonZones = zones.groupedReason;
    const groupedReason = groupedReasonZones.find(
      (zone) => zone.area === "resolver",
    );
    expect(groupedReason.details.topUnresolvedReasons[0]).toMatchObject({
      reason: "workspace-package-subpath-target-missing",
      count: 12,
      spaces: { type: 12, value: 0 },
    });
    expect(groupedReason.details.topUnresolvedReasons[1]).toMatchObject({
      reason: "tsconfig-path-target-missing",
      spaces: { type: 1, value: 3 },
    });
    expect(
      groupedReason.details.topUnresolvedReasons.some(
        (item) => item.reason === "legacy-record-only",
      ),
    ).toBe(false);

    const concentratedRootZones = zones.concentratedRoot;
    const concentratedRoot = concentratedRootZones.find(
      (zone) => zone.area === "resolver",
    );
    expect(concentratedRoot).toMatchObject({
      severity: "confidence-gap",
      details: { trigger: "prefix-concentration" },
    });
    expect(concentratedRoot.details.topUnresolvedSpecifiers).toContain(
      "@workspace/",
    );
  }, 30_000);

  it("B5-B5c. degrades precision for parse errors, opaque CJS exports, and dynamic CJS require calls", () => {
    const zones = rustBlindZoneCases({
      parseError: {
        symbols: {
          meta: { warnings: [{ kind: "parse-errors", count: 3, message: "x" }] },
        },
      },
      cjsExport: {
        symbols: {
          cjsExportSurfaceByFile: {
            "src/exact.cjs": {
              exact: [{ name: "foo", kind: "exports-member", line: 1 }],
              opaque: [],
            },
            "src/opaque.cjs": {
              exact: [],
              opaque: [{ kind: "module-exports-assignment", line: 3 }],
            },
          },
        },
      },
      cjsRequire: {
        symbols: {
          cjsRequireOpacity: [
            { consumerFile: "src/consumer.js", line: 2, kind: "dynamic-require" },
          ],
        },
      },
    });
    const parseErrorZones = zones.parseError;
    expect(
      parseErrorZones.find((zone) => zone.area === "parser"),
    ).toMatchObject({
      severity: "precision-gap",
    });

    const cjsExportZones = zones.cjsExport;
    const cjsExport = cjsExportZones.find(
      (zone) => zone.area === "commonjs-export-surface",
    );
    expect(cjsExport).toMatchObject({
      severity: "precision-gap",
      details: {
        files: 1,
        opaqueForms: [{ file: "src/opaque.cjs" }],
      },
    });

    const cjsRequireZones = zones.cjsRequire;
    const cjsRequire = cjsRequireZones.find(
      (zone) => zone.area === "commonjs-dynamic-require",
    );
    expect(cjsRequire).toMatchObject({
      severity: "precision-gap",
      details: {
        files: 1,
        calls: 1,
        examples: [{ consumerFile: "src/consumer.js" }],
      },
    });
  }, 30_000);

  it("B6-B9. keeps clean repos zone-free and formats blind-zone summaries deterministically", () => {
    const cleanZones = rustBlindZones({
      triage: { byLanguage: { ts: 100, tsx: 50 } },
      symbols: {
        uses: { unresolvedInternalRatio: 0.02 },
        meta: { warnings: [] },
      },
    });
    expect(cleanZones).toEqual([]);

    const severitySummary = formatBlindZonesSummary([
      { area: "rs", severity: "scan-gap", effect: "x" },
      { area: "py", severity: "precision-gap", effect: "x" },
      { area: "resolver", severity: "confidence-gap", effect: "x" },
    ]);
    expect(severitySummary).toContain("1 scan-gap");
    expect(severitySummary).toContain("1 precision-gap");
    expect(severitySummary).toContain("1 confidence-gap");
    expect(formatBlindZonesSummary([])).toBeNull();

    const resolverReasonSummary = formatBlindZonesSummary([
      {
        area: "resolver",
        severity: "confidence-gap",
        effect: "x",
        details: {
          topUnresolvedReasons: [
            { reason: "workspace-package-subpath-target-missing", count: 12 },
            { reason: "tsconfig-path-target-missing", count: 4 },
          ],
        },
      },
    ]);
    expect(resolverReasonSummary).toContain(
      "resolver reasons: workspace-package-subpath-target-missing 12, tsconfig-path-target-missing 4",
    );
  }, 30_000);

  it("B10-B10e. surfaces resolver and generated-consumer confidence limits in the audit summary", () => {
    const reasonSummary = renderAuditSummary({
      manifest: {
        meta: { generated: "2026-05-05T00:00:00.000Z" },
        profile: "quick",
        scanRange: { files: 5000, languages: ["ts"], includeTests: true },
        confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
        blindZones: [
          {
            area: "resolver",
            severity: "confidence-gap",
            effect: "x",
            details: {
              topUnresolvedReasons: [
                {
                  reason: "workspace-package-subpath-target-missing",
                  count: 12,
                },
                { reason: "tsconfig-path-target-missing", count: 4 },
              ],
            },
          },
        ],
      },
    });
    expect(reasonSummary).toContain(
      "Resolver blind-zone reasons: workspace-package-subpath-target-missing 12, tsconfig-path-target-missing 4",
    );
    expect(reasonSummary).toContain(
      "symbols.json.unresolvedInternalSummaryByReason",
    );

    const rootSummary = renderAuditSummary({
      manifest: {
        meta: { generated: "2026-05-05T00:00:00.000Z" },
        profile: "quick",
        scanRange: { files: 5000, languages: ["ts"], includeTests: true },
        confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
        resolverDiagnostics: {
          topSpecifierRoots: [
            {
              specifierRoot: "@scope/orm",
              count: 37,
              reasons: {
                "workspace-generated-artifact-missing": 29,
                "workspace-package-subpath-target-missing": 8,
              },
            },
            {
              specifierRoot: "app",
              count: 11,
              reasons: { "tsconfig-path-target-missing": 11 },
            },
          ],
        },
        blindZones: [
          {
            area: "resolver",
            severity: "confidence-gap",
            effect: "x",
            details: {
              topUnresolvedReasons: [
                { reason: "workspace-generated-artifact-missing", count: 29 },
              ],
            },
          },
        ],
      },
    });
    expect(rootSummary).toContain(
      "Top unresolved roots: @scope/orm 37 (workspace-generated-artifact-missing 29, workspace-package-subpath-target-missing 8); app 11 (tsconfig-path-target-missing 11)",
    );
    expect(rootSummary).toContain(
      "manifest.json.resolverDiagnostics.topSpecifierRoots",
    );

    const generatedSummary = renderAuditSummary({
      manifest: {
        meta: { generated: "2026-05-05T00:00:00.000Z" },
        profile: "quick",
        scanRange: { files: 5000, languages: ["ts"], includeTests: true },
        confidence: { parseErrors: 0, unresolvedInternalRatio: 0.02 },
        blindZones: [],
        generatedArtifacts: {
          generatedConsumerBlindZoneCount: 3,
          topGeneratedConsumerBlindZones: [
            {
              scopePackageRoot: "packages/prisma",
              count: 2,
              statuses: {
                missing: 1,
                "present-but-out-of-scope": 1,
              },
              topSpecifiers: [{ specifier: "@scope/prisma/enums", count: 2 }],
            },
            {
              scopePackageRoot: "packages/kysely",
              count: 1,
              statuses: { missing: 1 },
              topSpecifiers: [{ specifier: "@scope/kysely/types", count: 1 }],
            },
          ],
        },
      },
    });
    expect(generatedSummary).toContain("Generated consumer blind zones: 3");
    expect(generatedSummary).toContain(
      "packages/prisma 2 (missing 1, present-but-out-of-scope 1; @scope/prisma/enums 2)",
    );
    expect(generatedSummary).toContain(
      "manifest.json.generatedArtifacts.topGeneratedConsumerBlindZones",
    );
    expect(generatedSummary).toContain(
      "symbols.json.generatedConsumerBlindZones",
    );

    const affectedScopeSummary = renderAuditSummary({
      manifest: {
        meta: { generated: "2026-05-05T00:00:00.000Z" },
        profile: "quick",
        scanRange: { files: 5000, languages: ["ts"], includeTests: true },
        confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
        resolverDiagnostics: {
          topAffectedPackageScopes: [
            { affectedPackageScope: "packages/lib", count: 12 },
            { affectedPackageScope: "apps/web", count: 4 },
          ],
        },
        blindZones: [
          {
            area: "resolver",
            severity: "confidence-gap",
            effect: "x",
            details: {
              topUnresolvedReasons: [
                {
                  reason: "workspace-package-subpath-target-missing",
                  count: 12,
                },
              ],
            },
          },
        ],
      },
    });
    expect(affectedScopeSummary).toContain(
      "Resolver affected scopes: packages/lib 12; apps/web 4",
    );
    expect(affectedScopeSummary).toContain(
      "manifest.json.resolverDiagnostics.topAffectedPackageScopes",
    );

    const blockedHintSummary = renderAuditSummary({
      manifest: {
        meta: { generated: "2026-05-05T00:00:00.000Z" },
        profile: "quick",
        scanRange: { files: 5000, languages: ["ts"], includeTests: true },
        confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
        resolverDiagnostics: {
          blockedCandidateHintCount: 2,
          blockedCandidateHintSampleLimit: 10,
          blockedCandidateHintReasonCounts: [
            {
              reason: "generated-consumer-blind-zone",
              count: 7,
              families: { "generated-artifacts": 7 },
            },
            {
              reason: "hash-import-target-missing",
              count: 2,
              families: { "node-imports": 2 },
            },
          ],
          blockedCandidateHintFamilyCounts: [
            {
              family: "generated-artifacts",
              count: 7,
              reasons: { "generated-consumer-blind-zone": 7 },
            },
            {
              family: "node-imports",
              count: 2,
              reasons: { "hash-import-target-missing": 2 },
            },
          ],
          blockedCandidateHints: [
            {
              specifier: "#app/config",
              candidatePath: "packages/app/src/config",
              affectedPackageScope: "packages/app",
              reason: "hash-import-target-missing",
            },
            {
              specifier: "@scope/orm/client",
              candidatePath: "packages/orm/client",
              affectedPackageScope: "packages/orm",
              reason: "generated-consumer-blind-zone",
            },
          ],
        },
        blindZones: [
          {
            area: "resolver",
            severity: "confidence-gap",
            effect: "x",
            details: {
              topUnresolvedReasons: [
                { reason: "hash-import-target-missing", count: 1 },
                { reason: "generated-consumer-blind-zone", count: 1 },
              ],
            },
          },
        ],
      },
    });
    expect(blockedHintSummary).toContain("Resolver blocked absence hints: 2");
    expect(blockedHintSummary).toContain("manifest sample limit 10");
    expect(blockedHintSummary).toContain(
      "Resolver blocked absence distribution: reasons generated-consumer-blind-zone 7 (generated-artifacts 7), hash-import-target-missing 2 (node-imports 2); families generated-artifacts 7 (generated-consumer-blind-zone 7), node-imports 2 (hash-import-target-missing 2)",
    );
    expect(blockedHintSummary).toContain(
      "packages/app/src/config via #app/config (hash-import-target-missing)",
    );
    expect(blockedHintSummary).toContain(
      "packages/orm/client via @scope/orm/client (generated-consumer-blind-zone)",
    );
    expect(blockedHintSummary).toContain(
      "manifest.json.resolverDiagnostics.blockedCandidateHintReasonCounts",
    );
    expect(blockedHintSummary).toContain(
      "manifest.json.resolverDiagnostics.blockedCandidateHintFamilyCounts",
    );
    expect(blockedHintSummary).toContain(
      "manifest.json.resolverDiagnostics.blockedCandidateHints",
    );
    expect(blockedHintSummary).toContain(
      "resolver-diagnostics.json.blockedCandidateHints",
    );
  });

  it("O5-O6b. real clean TS audits produce zero blind zones and preserve confidence metadata", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-clean-blind-zones-"));
    const output = path.join(repo, "audit-out");
    try {
      write(
        repo,
        "package.json",
        JSON.stringify({ name: "clean-blind-zone-fixture", type: "module" }),
      );
      write(repo, "src/a.ts", "export const live = 1;\n");

      const result = runAudit([
        "--root",
        repo,
        "--output",
        output,
        "--profile",
        "quick",
        "--production",
      ]);
      expect(result.status, `${result.stdout}\n${result.stderr}`).toBe(0);

      const manifest = readJson(path.join(output, "manifest.json"));
      const symbols = readJson(path.join(output, "symbols.json"));

      expect(manifest.blindZones).toEqual([]);
      expect(typeof manifest.confidence.unresolvedInternalRatio).toBe("number");
      expect(typeof manifest.confidence.externalImports).toBe("number");
      expect(symbols.meta?.languageSupport?.ts?.enabled).toBe(true);
      expect(symbols.meta?.languageSupport?.js?.enabled).toBe(true);
      expect(typeof symbols.meta?.languageSupport?.python?.enabled).toBe(
        "boolean",
      );
      expect(typeof symbols.meta?.languageSupport?.go?.enabled).toBe("boolean");
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 60_000);

  it("O8. real Python-containing audits surface a Python precision or scan gap", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-python-blind-zones-"));
    const output = path.join(repo, "audit-out");
    try {
      write(
        repo,
        "package.json",
        JSON.stringify({ name: "python-blind-zone-fixture", type: "module" }),
      );
      write(repo, "src/a.ts", "export const live = 1;\n");
      write(repo, "src/helper.py", "def hello():\n    return 'hi'\n");

      const result = runAudit([
        "--root",
        repo,
        "--output",
        output,
        "--profile",
        "quick",
        "--production",
      ]);
      expect(result.status, `${result.stdout}\n${result.stderr}`).toBe(0);

      const manifest = readJson(path.join(output, "manifest.json"));
      const python = manifest.blindZones.find(
        (zone) =>
          zone.area === "python-method-resolution" ||
          zone.area === "python-scan-gap",
      );
      expect(python).toBeDefined();
      expect(["precision-gap", "scan-gap"]).toContain(python.severity);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 60_000);
});
