// Regression guard for v1.9.5 fix-plan ranking layer.
//
// Tests two surfaces:
//   (a) _lib/ranking.mjs tierForFinding — pure predicate, no I/O
//   (b) rank-fixes.mjs — merges synthesized artifacts, produces fix-plan.json
//
// The ranking predicate is pure, so we unit-test it with hand-built
// finding + evidence objects. No fixtures or child processes needed
// for (a). For (b) we build a tiny temp output directory with
// synthesized dead-classify.json / runtime-evidence.json /
// staleness.json / symbols.json and run rank-fixes against it.

import { execSync } from "node:child_process";
import { expect, it } from "vitest";
import { writeFileSync, readFileSync, mkdirSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { tierForFinding, TIER_TO_SARIF_LEVEL } from "../_lib/ranking.mjs";
import { TAINT } from "../_lib/vocab.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");
const OUT = "/tmp/fx-rank-fixes";
const OUT_C = "/tmp/fx-rank-fixes-groups";
const OUT_D = "/tmp/fx-rank-fixes-public-deep-import";
const OUT_E = "/tmp/fx-rank-fixes-call-graph-callbacks";
const OUT_F = "/tmp/fx-rank-fixes-generated-blocks";
const OUT_G = "/tmp/fx-rank-fixes-call-graph-bounded-missing";

function assert(label, ok, detail = "") {
  it(label, () => {
    expect(ok, detail).toBe(true);
  });
}

// ───────────────────────────────────────────────────────────
// A. Unit tests on the ranking predicate
// ───────────────────────────────────────────────────────────

// Canonical finding shape for proposal_C_remove_symbol
const findingC = {
  file: "src/foo.ts",
  line: 10,
  symbol: "Bar",
  kind: "FunctionDeclaration",
  bucket: "C",
};
const findingA = { ...findingC, bucket: "A", fileInternalUses: 1 };
const findingB = {
  ...findingC,
  bucket: "B",
  predicatePartner: { file: "x", line: 1, symbol: "y" },
};
const findingDeclarationRisk = {
  ...findingC,
  bucket: "B",
  kind: "TSTypeAliasDeclaration",
  declarationExportDependency: true,
  declarationExportRefs: { count: 3, lines: [4, 9, 12] },
};
const findingSpec = { ...findingC, bucket: "specifier" };

function safeAction(kind = "demote_export_declaration", overrides = {}) {
  return {
    kind,
    proofComplete: true,
    actionBlockers: [],
    strongerActionBlockers: [],
    requiresModuleMarker: false,
    preservesModuleSyntax: true,
    preservesSideEffects: true,
    preservesTypes: true,
    ...overrides,
  };
}

const findingCSafe = { ...findingC, safeAction: safeAction() };
const findingASafe = { ...findingA, safeAction: safeAction() };
const findingSpecSafe = {
  ...findingSpec,
  safeAction: safeAction("remove_export_specifier"),
};

// R1: full evidence convergence → SAFE_FIX
{
  const { tier, reason } = tierForFinding(findingCSafe, {
    runtime: {
      status: "dead-confirmed",
      grounding: "grounded",
      confidence: "high",
    },
    staleness: {
      tier: "fossil",
      grounding: "grounded",
      lineLastTouchedDaysAgo: 900,
    },
    policy: { excluded: false },
  });
  assert(
    "R1. C + runtime-dead + fossil → SAFE_FIX",
    tier === "SAFE_FIX",
    `got ${tier} (${reason})`,
  );
}

// R2: runtime executed → DEGRADED (must override anything else)
{
  const { tier } = tierForFinding(findingCSafe, {
    runtime: {
      status: "executed",
      grounding: "grounded",
      confidence: "high",
      hitsInSymbol: 42,
    },
    staleness: { tier: "fossil" },
    policy: { excluded: false },
  });
  assert(
    "R2. runtime executed → DEGRADED (overrides SAFE_FIX)",
    tier === "DEGRADED",
    `got ${tier}`,
  );
}

// R3: policy-excluded → MUTED regardless of other evidence
{
  const { tier } = tierForFinding(findingCSafe, {
    runtime: {
      status: "dead-confirmed",
      grounding: "grounded",
      confidence: "high",
    },
    staleness: { tier: "fossil" },
    policy: { excluded: true, reason: "publicApi_FP23" },
  });
  assert(
    "R3. policy-excluded → MUTED regardless of evidence",
    tier === "MUTED",
    `got ${tier}`,
  );
}

// R4: AST dead with clean static evidence → SAFE_FIX even without
// runtime/staleness. Runtime/git evidence strengthens confidence, but
// ordinary TS/JS static cleanup tools must still be useful without it.
{
  const { tier, reason } = tierForFinding(findingCSafe, {
    policy: { excluded: false },
  });
  assert(
    "R4. C with safeAction and without runtime/staleness → SAFE_FIX under static graph",
    tier === "SAFE_FIX" && reason.includes("safe-action"),
    `got ${tier} (${reason})`,
  );
}

// R4b: deadness alone is no longer enough. PCEF SAFE_FIX requires
// a concrete safeAction proof.
{
  const { tier, reason } = tierForFinding(findingC, {
    policy: { excluded: false },
  });
  assert(
    "R4b. C without safeAction proof → REVIEW_FIX",
    tier === "REVIEW_FIX" && reason.includes("missing-safe-action"),
    `got ${tier} (${reason})`,
  );
}

// R4c: bounded classify outputs must degrade, not review or safe-fix.
{
  const { tier, reason } = tierForFinding(
    {
      ...findingC,
      bucket: "unprocessed",
      action:
        "classification incomplete; rerun with a larger classify time budget",
    },
    { policy: { excluded: false } },
  );
  assert(
    "R4c. classify-incomplete bucket → DEGRADED",
    tier === "DEGRADED" && reason.includes("classify-incomplete"),
    `got ${tier} (${reason})`,
  );
}

// R5: resolver blindness >= 15% → DEGRADED (global gate)
{
  const { tier } = tierForFinding(findingCSafe, {
    runtime: {
      status: "dead-confirmed",
      grounding: "grounded",
      confidence: "high",
    },
    staleness: { tier: "fossil" },
    resolver: { unresolvedRatio: 0.25 },
    policy: { excluded: false },
  });
  assert(
    "R5. resolver unresolved >= 15% → DEGRADED even with strong local evidence",
    tier === "DEGRADED",
    `got ${tier}`,
  );
}

// R6: recent staleness no longer blocks static SAFE_FIX. It is context,
// not a contradiction. Runtime-executed and local taint remain blockers.
{
  const { tier, reason } = tierForFinding(findingCSafe, {
    runtime: {
      status: "dead-confirmed",
      grounding: "grounded",
      confidence: "high",
    },
    staleness: {
      tier: "recent",
      grounding: "grounded",
      lineLastTouchedDaysAgo: 3,
    },
    policy: { excluded: false },
  });
  assert(
    "R6. recent edits do not block static SAFE_FIX",
    tier === "SAFE_FIX" && reason.includes("staleness-recent"),
    `got ${tier} (${reason})`,
  );
}

// R7: A bucket is a mechanical export demotion under the constructed
// static graph: keep the definition, remove only the unused export.
{
  const { tier, reason } = tierForFinding(findingASafe, {
    policy: { excluded: false },
  });
  assert(
    "R7. A bucket export demotion can be SAFE_FIX when safeAction proof exists",
    tier === "SAFE_FIX" && reason.includes("safe-action"),
    `got ${tier} (${reason})`,
  );
}

// R8: specifier bucket with strong evidence → SAFE_FIX (mechanical)
{
  const { tier } = tierForFinding(findingSpecSafe, {
    runtime: {
      status: "dead-confirmed",
      grounding: "grounded",
      confidence: "high",
    },
    staleness: { tier: "stale" },
    policy: { excluded: false },
  });
  assert(
    "R8. specifier + strong evidence → SAFE_FIX",
    tier === "SAFE_FIX",
    `got ${tier}`,
  );
}

// R8b: strongerActionBlockers explain why deletion is not safe; they
// must not block a weaker selected safe action such as demotion.
{
  const { tier, reason } = tierForFinding(
    {
      ...findingC,
      safeAction: safeAction("demote_export_declaration", {
        strongerActionBlockers: ["side-effect-initializer"],
      }),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8b. strongerActionBlockers do not block selected safeAction",
    tier === "SAFE_FIX" && reason.includes("safe-action"),
    `got ${tier} (${reason})`,
  );
}

// R8c: selected-action blockers do block SAFE_FIX.
{
  const { tier, reason } = tierForFinding(
    {
      ...findingC,
      safeAction: safeAction("demote_export_declaration", {
        actionBlockers: ["partial-multi-declarator"],
      }),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8c. actionBlockers force REVIEW_FIX",
    tier === "REVIEW_FIX" && reason.includes("action-blockers"),
    `got ${tier} (${reason})`,
  );
}

// R8d: module marker insertion is part of the safe action, not a
// reason to demote to review.
{
  const { tier, reason } = tierForFinding(
    {
      ...findingC,
      safeAction: safeAction("delete_value_declaration", {
        requiresModuleMarker: true,
      }),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8d. requiresModuleMarker keeps SAFE_FIX when proof is complete",
    tier === "SAFE_FIX" && reason.includes("safe-action"),
    `got ${tier} (${reason})`,
  );
}

// R8e: P2 entry-unreachable evidence is a medium confidence booster
// only. It must not create high confidence by itself.
{
  const { tier, reason, confidence, confidenceDetail } = tierForFinding(
    {
      ...findingC,
      supportedBy: [{ kind: "entry-unreachable" }],
      safeAction: safeAction("demote_export_declaration"),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8e. entry-unreachable gives SAFE_FIX medium_with_evidence, not high",
    tier === "SAFE_FIX" &&
      confidence === "medium" &&
      confidenceDetail === "medium_with_evidence" &&
      reason.includes("entry-unreachable") &&
      !reason.includes("high"),
    `got ${tier} (${reason}) confidence=${confidence} detail=${confidenceDetail}`,
  );
}

// R8e2: P3 high confidence requires two compatible positive evidence
// lenses. A single lens stays medium_with_evidence; both entry and
// independent call-graph support can become high.
{
  const { tier, reason, confidence, confidenceDetail } = tierForFinding(
    {
      ...findingC,
      supportedBy: [
        { kind: "entry-unreachable" },
        { kind: "call-graph-no-observed-callers" },
      ],
      safeAction: safeAction("demote_export_declaration"),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8e2. entry + call graph evidence gives SAFE_FIX high confidence",
    tier === "SAFE_FIX" &&
      confidence === "high" &&
      confidenceDetail === "high_two_lens_evidence" &&
      reason.includes("entry-unreachable") &&
      reason.includes("no-observed-callers"),
    `got ${tier} (${reason}) confidence=${confidence} detail=${confidenceDetail}`,
  );
}

// R8f: positive evidence cannot override relevant soft taint.
{
  const { tier, reason } = tierForFinding(
    {
      ...findingC,
      supportedBy: [{ kind: "entry-unreachable" }],
      taintedBy: [
        { kind: TAINT.PARSE_ERRORS_ELSEWHERE, file: "src/parse-error.ts" },
      ],
      safeAction: safeAction("demote_export_declaration"),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8f. soft taint still blocks entry-unreachable SAFE_FIX boost",
    tier === "REVIEW_FIX" && reason.includes("parse-errors-elsewhere"),
    `got ${tier} (${reason})`,
  );
}

{
  const { tier, reason } = tierForFinding(
    {
      ...findingC,
      supportedBy: [{ kind: "entry-unreachable" }],
      taintedBy: [
        {
          kind: TAINT.UNRESOLVED_SPEC_MATCH_UNKNOWN,
          specifiers: ["@scope/kysely/types"],
          total: 8,
          consumerFile: "apps/api/v2/src/modules/kysely/kysely-read.service.ts",
        },
      ],
      safeAction: safeAction("demote_export_declaration"),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8f2. resolver soft taint reason is not mislabeled as parse errors",
    tier === "REVIEW_FIX" &&
      reason.includes("unresolved-specifier-could-match-unknown") &&
      !reason.includes("parse-errors-elsewhere"),
    `got ${tier} (${reason})`,
  );
}

{
  const { tier, blockedPromotion, blockedBy } = tierForFinding(
    {
      ...findingC,
      supportedBy: [{ kind: "entry-unreachable" }],
      taintedBy: [
        {
          kind: TAINT.GENERATED_ARTIFACT_MISSING_RELEVANT,
          specifier: "@scope/prisma/enums",
          specifiers: ["@scope/prisma/enums"],
          total: 1,
          consumerFile: "apps/web/page.ts",
          matchedPackage: "@scope/prisma",
          targetSubpath: "enums",
          generatorFamily: "prisma",
          confidence: "strong",
          impact: "provider-surface-unresolved",
          relevance: "matched-package-root",
          effect: "...",
        },
      ],
      safeAction: safeAction("demote_export_declaration"),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8f3. generated artifact soft taint returns structured blocking diagnostics",
    tier === "REVIEW_FIX" &&
      blockedPromotion === true &&
      blockedBy?.[0]?.reason === "workspace-generated-artifact-missing" &&
      blockedBy?.[0]?.specifier === "@scope/prisma/enums" &&
      blockedBy?.[0]?.matchedPackage === "@scope/prisma" &&
      blockedBy?.[0]?.targetSubpath === "enums" &&
      blockedBy?.[0]?.impact === "provider-surface-unresolved" &&
      blockedBy?.[0]?.relevance === "matched-package-root",
    JSON.stringify({ tier, blockedPromotion, blockedBy }),
  );
}

{
  const { tier, blockedPromotion, blockedBy } = tierForFinding(
    {
      ...findingC,
      supportedBy: [{ kind: "entry-unreachable" }],
      taintedBy: [
        {
          kind: TAINT.GENERATED_ARTIFACT_MISSING_RELEVANT,
          reason: "generated-consumer-blind-zone",
          specifier: "@scope/prisma/enums",
          specifiers: ["@scope/prisma/enums"],
          total: 1,
          consumerFile: "apps/web/page.ts",
          matchedPackage: "@scope/prisma",
          targetSubpath: "enums",
          generatorFamily: "prisma",
          confidence: "strong",
          candidatePath: "packages/prisma/generated/enums.ts",
          status: "missing",
          scopePackageRoot: "packages/prisma",
          impact: "consumer-surface-unresolved",
          relevance: "generated-consumer-scope",
          effect: "...",
        },
      ],
      safeAction: safeAction("demote_export_declaration"),
    },
    { policy: { excluded: false } },
  );
  assert(
    "R8f4. generated consumer blind-zone returns structured blocking diagnostics",
    tier === "REVIEW_FIX" &&
      blockedPromotion === true &&
      blockedBy?.[0]?.reason === "generated-consumer-blind-zone" &&
      blockedBy?.[0]?.specifier === "@scope/prisma/enums" &&
      blockedBy?.[0]?.candidatePath === "packages/prisma/generated/enums.ts" &&
      blockedBy?.[0]?.status === "missing" &&
      blockedBy?.[0]?.scopePackageRoot === "packages/prisma" &&
      blockedBy?.[0]?.impact === "consumer-surface-unresolved" &&
      blockedBy?.[0]?.relevance === "generated-consumer-scope",
    JSON.stringify({ tier, blockedPromotion, blockedBy }),
  );
}

// R8g: public deep-import risk blocks SAFE_FIX because demotion
// removes an externally observable export contract.
{
  const { tier, reason } = tierForFinding(findingCSafe, {
    policy: { excluded: false },
    contract: { publicDeepImportRisk: true },
  });
  assert(
    "R8g. public deep-import risk blocks SAFE_FIX",
    tier === "REVIEW_FIX" && reason.includes("public-deep-import-risk"),
    `got ${tier} (${reason})`,
  );
}

// R9: B bucket always needs review regardless of evidence
{
  const { tier } = tierForFinding(findingB, {
    runtime: {
      status: "dead-confirmed",
      grounding: "grounded",
      confidence: "high",
    },
    staleness: { tier: "fossil" },
    policy: { excluded: false },
  });
  assert(
    "R9. B bucket (predicate partner) always REVIEW_FIX",
    tier === "REVIEW_FIX",
    `got ${tier}`,
  );
}

// R10: TIER_TO_SARIF_LEVEL shape
assert(
  "R10. SAFE_FIX → warning; REVIEW_FIX/DEGRADED → note; MUTED → null",
  TIER_TO_SARIF_LEVEL.SAFE_FIX === "warning" &&
    TIER_TO_SARIF_LEVEL.REVIEW_FIX === "note" &&
    TIER_TO_SARIF_LEVEL.DEGRADED === "note" &&
    TIER_TO_SARIF_LEVEL.MUTED === null,
  JSON.stringify(TIER_TO_SARIF_LEVEL),
);

// R11: exported declaration dependencies block deletion, not demotion.
// A local type dependency used by an exported declaration can keep the
// binding/type in place while removing the export edge.
{
  const { tier, reason } = tierForFinding(findingDeclarationRisk, {
    policy: { excluded: false },
  });
  assert(
    "R11. exported declaration dependency without safe action → REVIEW_FIX",
    tier === "REVIEW_FIX" && reason === "missing-safe-action-proof",
    `got ${tier} (${reason})`,
  );
}

{
  const { tier, reason } = tierForFinding(
    {
      ...findingDeclarationRisk,
      safeAction: safeAction("demote_export_declaration", {
        strongerActionBlockers: ["local-refs-present"],
      }),
    },
    {
      policy: { excluded: false },
    },
  );
  assert(
    "R11b. local declaration dependency + demote safe action → SAFE_FIX",
    tier === "SAFE_FIX" && reason.includes("safe-action"),
    `got ${tier} (${reason})`,
  );
}

{
  const { tier, reason } = tierForFinding(
    {
      ...findingDeclarationRisk,
      safeAction: safeAction("delete_type_declaration"),
    },
    {
      policy: { excluded: false },
    },
  );
  assert(
    "R11c. exported declaration dependency still blocks delete action",
    tier === "REVIEW_FIX" &&
      reason.startsWith("declaration-dependency-not-preserved"),
    `got ${tier} (${reason})`,
  );
}

// ───────────────────────────────────────────────────────────
// B. Integration: synthesize artifacts, run rank-fixes.mjs
// ───────────────────────────────────────────────────────────

rmSync(OUT, { recursive: true, force: true });
mkdirSync(OUT, { recursive: true });

// Three findings: two will pass SAFE_FIX filter, one will carry a
// binding-preserving declaration dependency demotion.
// One excludedCandidate materializes as MUTED (v1.9.6).
writeFileSync(
  path.join(OUT, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 3, category_C: 2, category_B: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/dead.ts",
        line: 10,
        symbol: "Fossil",
        kind: "FunctionDeclaration",
        action: "정의 자체 제거 가능.",
      },
      {
        file: "src/recent.ts",
        line: 5,
        symbol: "Active",
        kind: "FunctionDeclaration",
        action: "정의 자체 제거 가능.",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [
      {
        file: "src/public-types.ts",
        line: 2,
        symbol: "PublicDependency",
        kind: "TSInterfaceDeclaration",
        action: "파일 내 중심 타입.",
        declarationExportDependency: true,
        declarationExportRefs: { count: 2, lines: [5, 9] },
      },
    ],
    proposal_remove_export_specifier: [],
    excludedCandidates: [
      {
        file: "eslint.config.mjs",
        line: 1,
        symbol: "default",
        kind: "default",
        reason: "config_FP22",
      },
    ],
  }),
);

writeFileSync(
  path.join(OUT, "runtime-evidence.json"),
  JSON.stringify({
    meta: { tool: "test" },
    summary: {},
    merged: [
      {
        file: "src/dead.ts",
        line: 10,
        symbol: "Fossil",
        kind: "FunctionDeclaration",
        runtimeStatus: "dead-confirmed",
        grounding: "grounded",
        confidence: "high",
        hitsInSymbol: 0,
      },
      {
        file: "src/recent.ts",
        line: 5,
        symbol: "Active",
        kind: "FunctionDeclaration",
        runtimeStatus: "dead-confirmed",
        grounding: "grounded",
        confidence: "high",
        hitsInSymbol: 0,
      },
    ],
  }),
);

writeFileSync(
  path.join(OUT, "staleness.json"),
  JSON.stringify({
    meta: {},
    summary: {},
    enriched: [
      {
        file: "src/dead.ts",
        line: 10,
        symbol: "Fossil",
        stalenessTier: "fossil",
        grounding: "grounded",
        lineLastTouchedDaysAgo: 900,
      },
      {
        file: "src/recent.ts",
        line: 5,
        symbol: "Active",
        stalenessTier: "recent",
        grounding: "grounded",
        lineLastTouchedDaysAgo: 3,
      },
    ],
  }),
);

// Low resolver blindness so the gate doesn't trip the fixture
writeFileSync(
  path.join(OUT, "symbols.json"),
  JSON.stringify({
    meta: {},
    totalUsesResolved: 1000,
    unresolvedUses: 10,
    files: {},
    totalDefs: 0,
    deadTotal: 0,
    trulyDead: 0,
    deadInProd: 0,
    deadInTest: 0,
    topSymbolFanIn: [],
    deadProdList: [],
    reExportsByFile: {},
    fanInByIdentity: {
      "src/dead.ts::Fossil": 0,
      "src/recent.ts::Active": 0,
    },
  }),
);

writeFileSync(
  path.join(OUT, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/dead.ts:Fossil:10",
          file: "src/dead.ts",
          line: 10,
          symbol: "Fossil",
          safeAction: safeAction("demote_export_declaration", {
            target: { definitionId: "src/dead.ts#FunctionDeclaration:1-40" },
          }),
          actionBlockers: [],
        },
        {
          id: "dead-export:src/recent.ts:Active:5",
          file: "src/recent.ts",
          line: 5,
          symbol: "Active",
          safeAction: safeAction("demote_export_declaration", {
            target: { definitionId: "src/recent.ts#FunctionDeclaration:1-40" },
            strongerActionBlockers: ["side-effect-initializer"],
          }),
          actionBlockers: [],
        },
        {
          id: "dead-export:src/public-types.ts:PublicDependency:2",
          file: "src/public-types.ts",
          line: 2,
          symbol: "PublicDependency",
          safeAction: safeAction("demote_export_declaration", {
            strongerActionBlockers: ["local-refs-present"],
          }),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);

writeFileSync(
  path.join(OUT, "call-graph.json"),
  JSON.stringify(
    {
      meta: {
        tool: "build-call-graph.mjs",
        supports: {
          callFanInByDefinitionId: true,
          callFanInByIdentity: true,
          boundedMemberCallResolution: true,
        },
      },
      callFanInByDefinitionId: {
        "src/dead.ts#FunctionDeclaration:1-40": 0,
        "src/recent.ts#FunctionDeclaration:1-40": 1,
      },
      callFanInByIdentity: {
        "src/dead.ts::Fossil": 0,
        "src/recent.ts::Active": 1,
      },
      boundedOutMemberCallsByFile: {
        "src/dead.ts": 0,
        "src/recent.ts": 0,
      },
      memberCallsByFile: {
        "src/dead.ts": 0,
        "src/recent.ts": 0,
      },
    },
    null,
    2,
  ),
);

writeFileSync(
  path.join(OUT, "module-reachability.json"),
  JSON.stringify(
    {
      meta: {
        tool: "build-module-reachability.mjs",
        completenessBySubmodule: { src: "high" },
        boundedOutReason: null,
      },
      runtimeReachableFiles: ["src/recent.ts"],
      typeReachableFiles: ["src/recent.ts"],
      reachableFiles: ["src/recent.ts"],
      boundedOutFiles: [],
      unreachableFiles: ["src/dead.ts", "src/public-types.ts"],
      summary: { unreachable: 2 },
    },
    null,
    2,
  ),
);

writeFileSync(
  path.join(OUT, "entry-surface.json"),
  JSON.stringify(
    {
      meta: { tool: "build-entry-surface.mjs" },
      publicApiFiles: [],
      scriptEntrypointFiles: [],
      htmlEntrypointFiles: [],
      frameworkEntrypointFiles: [],
      configEntrypointFiles: [],
      entryFiles: [],
      evidenceByFile: {},
      globalCompleteness: "high",
      completenessBySubmodule: { src: "high" },
    },
    null,
    2,
  ),
);

execSync(`node rank-fixes.mjs --root ${OUT} --output ${OUT}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});

const plan = JSON.parse(readFileSync(path.join(OUT, "fix-plan.json"), "utf8"));

// I1: summary shape. Total is 4: 3 classifier proposals + 1 excluded.
assert(
  "I1. fix-plan.json has 4-tier summary + total (includes MUTED from excludedCandidates)",
  plan.summary.SAFE_FIX !== undefined &&
    plan.summary.REVIEW_FIX !== undefined &&
    plan.summary.DEGRADED !== undefined &&
    plan.summary.MUTED !== undefined &&
    plan.summary.total === 4,
  JSON.stringify(plan.summary),
);

// I1b: excludedCandidates materialize as MUTED (v1.9.6 pipeline fix)
assert(
  "I1b. excludedCandidates materialize as MUTED findings (not silently dropped)",
  plan.summary.MUTED === 1 &&
    plan.muted?.[0]?.finding?.file === "eslint.config.mjs" &&
    plan.muted?.[0]?.tier === "MUTED" &&
    plan.muted?.[0]?.evidence?.policy?.reason === "config_FP22",
  `muted: ${JSON.stringify(plan.muted)}`,
);

// I2: the fossil goes to SAFE_FIX
const fossilEntry = plan.safeFixes.find((s) => s.finding.symbol === "Fossil");
assert(
  "I2. Fossil (C + runtime-dead + fossil) ranks SAFE_FIX",
  !!fossilEntry,
  `safeFixes: ${JSON.stringify(plan.safeFixes.map((s) => s.finding.symbol))}`,
);

// I3: the recent edit still reaches SAFE_FIX; staleness is evidence
// context, not a hard blocker for static safe cleanup.
const activeInSafe = plan.safeFixes.find((s) => s.finding.symbol === "Active");
assert(
  "I3. Active (recent staleness) ranks SAFE_FIX under static graph",
  !!activeInSafe && activeInSafe.reason.includes("staleness-recent"),
  `safeFixes: ${JSON.stringify(plan.safeFixes.map((s) => s.finding.symbol))}`,
);

// I4: no clean static C candidate is forced down to REVIEW_FIX just
// because optional runtime/staleness axes are incomplete or recent.
const activeEntry = plan.reviewFixes.find((s) => s.finding.symbol === "Active");
assert(
  "I4. Active is not review-only when static evidence is clean",
  !activeEntry,
  `reviewFixes: ${JSON.stringify(plan.reviewFixes.map((s) => s.finding.symbol))}`,
);

// I5: meta records which inputs were present
assert(
  "I5. fix-plan.meta.inputs flags every optional input",
  plan.meta.inputs["dead-classify.json"] === true &&
    plan.meta.inputs["runtime-evidence.json"] === true &&
    plan.meta.inputs["staleness.json"] === true &&
    plan.meta.inputs["symbols.json"] === true &&
    plan.meta.inputs["export-action-safety.json"] === true &&
    plan.meta.inputs["call-graph.json"] === true &&
    plan.meta.inputs["entry-surface.json"] === true &&
    plan.meta.inputs["module-reachability.json"] === true,
  JSON.stringify(plan.meta.inputs),
);

// I6: resolverBlindness meta present and gate not tripped
assert(
  "I6. resolverBlindness gate = ok on healthy fixture",
  plan.meta.resolverBlindness?.gate === "ok",
  JSON.stringify(plan.meta.resolverBlindness),
);

// I6b: declaration-surface dependency survives flattening, but a safe demote
// action can preserve the local type while removing only the export edge.
const publicDependency = plan.safeFixes.find(
  (s) => s.finding.symbol === "PublicDependency",
);
assert(
  "I6b. local declaration export dependency + demote action ranks SAFE_FIX",
  !!publicDependency &&
    publicDependency.finding.declarationExportDependency === true &&
    publicDependency.finding.safeAction?.kind === "demote_export_declaration",
  JSON.stringify({ safeFixes: plan.safeFixes, degraded: plan.degraded }),
);

assert(
  "I6c. two evidence lenses give unreachable file high confidence",
  fossilEntry?.finding?.supportedBy?.some(
    (s) => s.kind === "entry-unreachable",
  ) &&
    fossilEntry?.confidence === "high" &&
    fossilEntry?.confidenceDetail === "high_two_lens_evidence" &&
    fossilEntry?.reason.includes("entry-unreachable"),
  JSON.stringify(fossilEntry),
);

assert(
  "I6d. reachable file does not get entry-unreachable support",
  !activeInSafe?.finding?.supportedBy?.some(
    (s) => s.kind === "entry-unreachable",
  ) &&
    activeInSafe?.confidence === "medium" &&
    activeInSafe?.confidenceDetail === undefined,
  JSON.stringify(activeInSafe),
);

assert(
  "I6e. call graph no observed callers adds independent support",
  fossilEntry?.finding?.supportedBy?.some(
    (s) =>
      s.kind === "call-graph-no-observed-callers" &&
      s.artifact === "call-graph.json",
  ) &&
    fossilEntry?.reason.includes("no-observed-callers") &&
    !activeInSafe?.finding?.supportedBy?.some(
      (s) => s.kind === "call-graph-no-observed-callers",
    ),
  JSON.stringify({ fossilEntry, activeInSafe }),
);

// I7: regression on reviewer's key concern — when runtime hits exist,
// finding must NOT reach SAFE_FIX (CI noise prevention)
// Build a separate OUT_B to keep isolation
const OUT_B = "/tmp/fx-rank-fixes-b";
rmSync(OUT_B, { recursive: true, force: true });
mkdirSync(OUT_B, { recursive: true });
writeFileSync(
  path.join(OUT_B, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 1, category_C: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/x.ts",
        line: 1,
        symbol: "Hit",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_B, "runtime-evidence.json"),
  JSON.stringify({
    meta: {},
    summary: {},
    merged: [
      {
        file: "src/x.ts",
        line: 1,
        symbol: "Hit",
        kind: "FunctionDeclaration",
        runtimeStatus: "executed",
        grounding: "grounded",
        confidence: "high",
        hitsInSymbol: 7,
      },
    ],
  }),
);
writeFileSync(
  path.join(OUT_B, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 10,
  }),
);
execSync(`node rank-fixes.mjs --root ${OUT_B} --output ${OUT_B}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planB = JSON.parse(
  readFileSync(path.join(OUT_B, "fix-plan.json"), "utf8"),
);
assert(
  "I7. runtime-hit findings never reach SAFE_FIX (CI-noise prevention)",
  planB.summary.SAFE_FIX === 0 && planB.summary.DEGRADED === 1,
  JSON.stringify(planB.summary),
);
rmSync(OUT_B, { recursive: true, force: true });

// I7b: publishable packages without an exports map allow external
// deep imports. rank-fixes must keep those cleanup actions review-visible
// instead of SAFE_FIX, even when deadness/action proof is otherwise clean.
rmSync(OUT_D, { recursive: true, force: true });
mkdirSync(path.join(OUT_D, "src"), { recursive: true });
writeFileSync(
  path.join(OUT_D, "package.json"),
  JSON.stringify(
    {
      name: "public-no-exports",
      version: "1.0.0",
      main: "./src/index.js",
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_D, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 1, category_C: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/internal.js",
        line: 1,
        symbol: "internalThing",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_D, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
  }),
);
writeFileSync(
  path.join(OUT_D, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/internal.js:internalThing:1",
          file: "src/internal.js",
          line: 1,
          symbol: "internalThing",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_D} --output ${OUT_D}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planD = JSON.parse(
  readFileSync(path.join(OUT_D, "fix-plan.json"), "utf8"),
);
assert(
  "I7b. rank-fixes keeps public deep-import risk as REVIEW_FIX",
  planD.summary.SAFE_FIX === 0 &&
    planD.summary.REVIEW_FIX === 1 &&
    planD.summary.reviewReasons?.publicDeepImportRisk?.[
      "exports-absent-publish-surface-unknown"
    ] === 1 &&
    planD.reviewFixes?.[0]?.reason.includes(
      "public-deep-import-risk: exports-absent-publish-surface-unknown",
    ) &&
    planD.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail
      ?.reason === "exports-absent-publish-surface-unknown" &&
    planD.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail
      ?.publishSurfaceSource === "implicit-npm-surface" &&
    planD.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail
      ?.packageName === "public-no-exports",
  JSON.stringify({
    summary: planD.summary,
    safe: planD.safeFixes,
    review: planD.reviewFixes,
  }),
);
rmSync(OUT_D, { recursive: true, force: true });

// I7b2: packages without exports can still clear public deep-import risk
// when package.json#files explicitly excludes the candidate source file.
rmSync(OUT_D, { recursive: true, force: true });
mkdirSync(path.join(OUT_D, "src"), { recursive: true });
writeFileSync(
  path.join(OUT_D, "package.json"),
  JSON.stringify(
    {
      name: "public-files-dist-only",
      version: "1.0.0",
      files: ["dist"],
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_D, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 1, category_C: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/internal.js",
        line: 1,
        symbol: "internalThing",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_D, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
  }),
);
writeFileSync(
  path.join(OUT_D, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/internal.js:internalThing:1",
          file: "src/internal.js",
          line: 1,
          symbol: "internalThing",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_D} --output ${OUT_D}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planD3 = JSON.parse(
  readFileSync(path.join(OUT_D, "fix-plan.json"), "utf8"),
);
assert(
  "I7b2. package files exclusion allows SAFE_FIX when other proof is clean",
  planD3.summary.SAFE_FIX === 1 &&
    planD3.summary.REVIEW_FIX === 0 &&
    planD3.safeFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail
      ?.reason === "files-excludes-file" &&
    planD3.safeFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail
      ?.publishSurfaceSource === "package-json-files",
  JSON.stringify({
    summary: planD3.summary,
    safe: planD3.safeFixes,
    review: planD3.reviewFixes,
  }),
);
rmSync(OUT_D, { recursive: true, force: true });

// I7b3: package.json#files cannot clear public risk for npm always-included
// entrypoint files such as package main.
rmSync(OUT_D, { recursive: true, force: true });
mkdirSync(path.join(OUT_D, "src"), { recursive: true });
writeFileSync(
  path.join(OUT_D, "package.json"),
  JSON.stringify(
    {
      name: "public-main-source",
      version: "1.0.0",
      main: "./src/index.js",
      files: ["dist"],
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_D, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 1, category_C: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/index.js",
        line: 1,
        symbol: "mainThing",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_D, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
  }),
);
writeFileSync(
  path.join(OUT_D, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/index.js:mainThing:1",
          file: "src/index.js",
          line: 1,
          symbol: "mainThing",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_D} --output ${OUT_D}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planD4 = JSON.parse(
  readFileSync(path.join(OUT_D, "fix-plan.json"), "utf8"),
);
assert(
  "I7b3. npm always-included main file keeps REVIEW_FIX",
  planD4.summary.SAFE_FIX === 0 &&
    planD4.summary.REVIEW_FIX === 1 &&
    planD4.summary.reviewReasons?.publicDeepImportRisk?.[
      "exports-absent-file-published-always-included"
    ] === 1 &&
    planD4.reviewFixes?.[0]?.evidence?.contract?.publicDeepImportRiskDetail
      ?.matchedAlwaysIncludedRule === "main",
  JSON.stringify({
    summary: planD4.summary,
    safe: planD4.safeFixes,
    review: planD4.reviewFixes,
  }),
);
rmSync(OUT_D, { recursive: true, force: true });

// I7c: package.json files without a package name are not externally
// addressable package contracts. They should not blanket-demote clean
// safeAction findings under public-deep-import-risk.
rmSync(OUT_D, { recursive: true, force: true });
mkdirSync(path.join(OUT_D, "src"), { recursive: true });
writeFileSync(
  path.join(OUT_D, "package.json"),
  JSON.stringify(
    {
      type: "module",
      main: "./src/index.js",
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_D, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 1, category_C: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/dead-truly.ts",
        line: 1,
        symbol: "neverUsed1",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_D, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
  }),
);
writeFileSync(
  path.join(OUT_D, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/dead-truly.ts:neverUsed1:1",
          file: "src/dead-truly.ts",
          line: 1,
          symbol: "neverUsed1",
          safeAction: safeAction("delete_value_declaration"),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_D} --output ${OUT_D}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planD2 = JSON.parse(
  readFileSync(path.join(OUT_D, "fix-plan.json"), "utf8"),
);
assert(
  "I7c. no package name means no public deep-import risk blanket review",
  planD2.summary.SAFE_FIX === 1 &&
    planD2.summary.REVIEW_FIX === 0 &&
    planD2.safeFixes?.[0]?.reason.includes("safe-action"),
  JSON.stringify({
    summary: planD2.summary,
    safe: planD2.safeFixes,
    review: planD2.reviewFixes,
  }),
);
rmSync(OUT_D, { recursive: true, force: true });

// I7d: call-graph no-observed-callers is an independent evidence booster,
// not a framework-callback detector. React components, hooks, and route
// handlers should not gain that support solely because the call graph has
// no direct call edge.
rmSync(OUT_E, { recursive: true, force: true });
mkdirSync(OUT_E, { recursive: true });
writeFileSync(
  path.join(OUT_E, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 3, category_C: 3 },
    proposal_C_remove_symbol: [
      {
        file: "src/components/Button.tsx",
        line: 1,
        symbol: "Button",
        kind: "FunctionDeclaration",
        action: "",
      },
      {
        file: "src/hooks/useThing.ts",
        line: 1,
        symbol: "useThing",
        kind: "FunctionDeclaration",
        action: "",
      },
      {
        file: "src/app/api/users/route.ts",
        line: 1,
        symbol: "default",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_E, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
    fanInByIdentity: {
      "src/components/Button.tsx::Button": 0,
      "src/hooks/useThing.ts::useThing": 0,
      "src/app/api/users/route.ts::default": 0,
    },
  }),
);
writeFileSync(
  path.join(OUT_E, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/components/Button.tsx:Button:1",
          file: "src/components/Button.tsx",
          line: 1,
          symbol: "Button",
          safeAction: safeAction("demote_export_declaration", {
            target: {
              definitionId:
                "src/components/Button.tsx#FunctionDeclaration:1-40",
            },
          }),
          actionBlockers: [],
        },
        {
          id: "dead-export:src/hooks/useThing.ts:useThing:1",
          file: "src/hooks/useThing.ts",
          line: 1,
          symbol: "useThing",
          safeAction: safeAction("demote_export_declaration", {
            target: {
              definitionId: "src/hooks/useThing.ts#FunctionDeclaration:1-40",
            },
          }),
          actionBlockers: [],
        },
        {
          id: "dead-export:src/app/api/users/route.ts:default:1",
          file: "src/app/api/users/route.ts",
          line: 1,
          symbol: "default",
          safeAction: safeAction("demote_export_declaration", {
            target: {
              definitionId:
                "src/app/api/users/route.ts#FunctionDeclaration:1-40",
            },
          }),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_E, "call-graph.json"),
  JSON.stringify(
    {
      meta: {
        tool: "build-call-graph.mjs",
        supports: {
          callFanInByDefinitionId: true,
          callFanInByIdentity: true,
          boundedMemberCallResolution: true,
        },
      },
      callFanInByDefinitionId: {
        "src/components/Button.tsx#FunctionDeclaration:1-40": 0,
        "src/hooks/useThing.ts#FunctionDeclaration:1-40": 0,
        "src/app/api/users/route.ts#FunctionDeclaration:1-40": 0,
      },
      callFanInByIdentity: {
        "src/components/Button.tsx::Button": 0,
        "src/hooks/useThing.ts::useThing": 0,
        "src/app/api/users/route.ts::default": 0,
      },
      boundedOutMemberCallsByFile: {
        "src/components/Button.tsx": 0,
        "src/hooks/useThing.ts": 0,
        "src/app/api/users/route.ts": 0,
      },
      memberCallsByFile: {
        "src/components/Button.tsx": 0,
        "src/hooks/useThing.ts": 0,
        "src/app/api/users/route.ts": 0,
      },
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_E} --output ${OUT_E}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planE = JSON.parse(
  readFileSync(path.join(OUT_E, "fix-plan.json"), "utf8"),
);
assert(
  "I7d. framework callback-like exports do not get call-graph support",
  planE.safeFixes?.length === 3 &&
    planE.safeFixes.every(
      (s) =>
        !s.finding.supportedBy?.some(
          (support) => support.kind === "call-graph-no-observed-callers",
        ) &&
        !s.reason.includes("no-observed-callers") &&
        s.confidenceDetail === undefined,
    ),
  JSON.stringify(planE.safeFixes, null, 2),
);
rmSync(OUT_E, { recursive: true, force: true });

// I7d2: call-graph no-observed-callers support requires bounded member-call
// stats. Older call-graph artifacts with full fan-in maps but no bounded
// support must not claim independent evidence.
rmSync(OUT_G, { recursive: true, force: true });
mkdirSync(OUT_G, { recursive: true });
writeFileSync(
  path.join(OUT_G, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 1, category_C: 1 },
    proposal_C_remove_symbol: [
      {
        file: "src/worker.ts",
        line: 1,
        symbol: "Worker",
        kind: "FunctionDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_G, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
    fanInByIdentity: { "src/worker.ts::Worker": 0 },
  }),
);
writeFileSync(
  path.join(OUT_G, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:src/worker.ts:Worker:1",
          file: "src/worker.ts",
          line: 1,
          symbol: "Worker",
          safeAction: safeAction("demote_export_declaration", {
            target: { definitionId: "src/worker.ts#FunctionDeclaration:1-40" },
          }),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_G, "call-graph.json"),
  JSON.stringify(
    {
      meta: {
        tool: "build-call-graph.mjs",
        supports: { callFanInByDefinitionId: true, callFanInByIdentity: true },
      },
      callFanInByDefinitionId: {
        "src/worker.ts#FunctionDeclaration:1-40": 0,
      },
      callFanInByIdentity: {
        "src/worker.ts::Worker": 0,
      },
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_G} --output ${OUT_G}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planG = JSON.parse(
  readFileSync(path.join(OUT_G, "fix-plan.json"), "utf8"),
);
assert(
  "I7d2. call graph support is withheld when bounded member-call stats are absent",
  planG.safeFixes?.length === 1 &&
    !planG.safeFixes[0].finding.supportedBy?.some(
      (support) => support.kind === "call-graph-no-observed-callers",
    ) &&
    !planG.safeFixes[0].reason.includes("no-observed-callers") &&
    planG.safeFixes[0].confidenceDetail === undefined,
  JSON.stringify(planG.safeFixes, null, 2),
);
rmSync(OUT_G, { recursive: true, force: true });

// I7e: generated blind-zone review entries should preserve a structured
// blocking diagnostic in fix-plan.json. Consumers should not have to parse
// the prose reason or re-interpret finding.taintedBy to explain why SAFE_FIX
// was held back.
rmSync(OUT_F, { recursive: true, force: true });
mkdirSync(OUT_F, { recursive: true });
writeFileSync(
  path.join(OUT_F, "dead-classify.json"),
  JSON.stringify(
    {
      summary: { total: 1, category_C: 1 },
      proposal_C_remove_symbol: [
        {
          file: "packages/prisma/index.ts",
          line: 1,
          symbol: "PrismaEnums",
          kind: "TSTypeAliasDeclaration",
          action: "",
          taintedBy: [
            {
              kind: TAINT.GENERATED_ARTIFACT_MISSING_RELEVANT,
              specifier: "@scope/prisma/enums",
              specifiers: ["@scope/prisma/enums"],
              total: 1,
              consumerFile: "apps/web/page.ts",
              matchedPackage: "@scope/prisma",
              targetSubpath: "enums",
              generatorFamily: "prisma",
              confidence: "strong",
              impact: "provider-surface-unresolved",
              relevance: "matched-package-root",
              effect: "...",
            },
          ],
        },
      ],
      proposal_A_demote_to_internal: [],
      proposal_B_review: [],
      proposal_remove_export_specifier: [],
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_F, "symbols.json"),
  JSON.stringify(
    {
      totalUsesResolved: 1000,
      unresolvedUses: 0,
    },
    null,
    2,
  ),
);
writeFileSync(
  path.join(OUT_F, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:packages/prisma/index.ts:PrismaEnums:1",
          file: "packages/prisma/index.ts",
          line: 1,
          symbol: "PrismaEnums",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_F} --output ${OUT_F}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planF = JSON.parse(
  readFileSync(path.join(OUT_F, "fix-plan.json"), "utf8"),
);
const generatedReview = planF.reviewFixes?.[0];
assert(
  "I7e. fix-plan review entry carries generated blind-zone blocking diagnostics",
  planF.summary.SAFE_FIX === 0 &&
    planF.summary.REVIEW_FIX === 1 &&
    generatedReview?.blockedPromotion === true &&
    generatedReview?.blockedBy?.[0]?.reason ===
      "workspace-generated-artifact-missing" &&
    generatedReview?.blockedBy?.[0]?.specifier === "@scope/prisma/enums" &&
    generatedReview?.blockedBy?.[0]?.matchedPackage === "@scope/prisma" &&
    generatedReview?.blockedBy?.[0]?.targetSubpath === "enums" &&
    generatedReview?.blockedBy?.[0]?.impact === "provider-surface-unresolved" &&
    generatedReview?.blockedBy?.[0]?.relevance === "matched-package-root",
  JSON.stringify(generatedReview, null, 2),
);
rmSync(OUT_F, { recursive: true, force: true });

// I8: SAFE_FIX grouping is presentation-only evidence. Several
// symbols in one file with the same safe action should surface as
// one reviewable pattern without removing the raw safeFixes list.
rmSync(OUT_C, { recursive: true, force: true });
mkdirSync(OUT_C, { recursive: true });
writeFileSync(
  path.join(OUT_C, "dead-classify.json"),
  JSON.stringify({
    summary: { total: 3, category_C: 3 },
    proposal_C_remove_symbol: [
      {
        file: "apps/server/src/repository.ts",
        line: 10,
        symbol: "CreateTurnInput",
        kind: "TSTypeAliasDeclaration",
        action: "",
      },
      {
        file: "apps/server/src/repository.ts",
        line: 20,
        symbol: "ListLibraryDocsOptions",
        kind: "TSTypeAliasDeclaration",
        action: "",
      },
      {
        file: "apps/server/src/other.ts",
        line: 5,
        symbol: "OtherInput",
        kind: "TSTypeAliasDeclaration",
        action: "",
      },
    ],
    proposal_A_demote_to_internal: [],
    proposal_B_review: [],
    proposal_remove_export_specifier: [],
  }),
);
writeFileSync(
  path.join(OUT_C, "symbols.json"),
  JSON.stringify({
    totalUsesResolved: 1000,
    unresolvedUses: 0,
  }),
);
writeFileSync(
  path.join(OUT_C, "export-action-safety.json"),
  JSON.stringify(
    {
      meta: { tool: "export-action-safety.mjs" },
      findings: [
        {
          id: "dead-export:apps/server/src/repository.ts:CreateTurnInput:10",
          file: "apps/server/src/repository.ts",
          line: 10,
          symbol: "CreateTurnInput",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
        {
          id: "dead-export:apps/server/src/repository.ts:ListLibraryDocsOptions:20",
          file: "apps/server/src/repository.ts",
          line: 20,
          symbol: "ListLibraryDocsOptions",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
        {
          id: "dead-export:apps/server/src/other.ts:OtherInput:5",
          file: "apps/server/src/other.ts",
          line: 5,
          symbol: "OtherInput",
          safeAction: safeAction("demote_export_declaration"),
          actionBlockers: [],
        },
      ],
    },
    null,
    2,
  ),
);
execSync(`node rank-fixes.mjs --root ${OUT_C} --output ${OUT_C}`, {
  cwd: DIR,
  stdio: ["ignore", "pipe", "pipe"],
});
const planC = JSON.parse(
  readFileSync(path.join(OUT_C, "fix-plan.json"), "utf8"),
);
const repoGroup = planC.safeFixGroups?.find(
  (g) =>
    g.file === "apps/server/src/repository.ts" &&
    g.actionKind === "demote_export_declaration",
);
assert(
  "I8. fix-plan groups SAFE_FIX by file and safeAction kind",
  planC.summary.SAFE_FIX === 3 &&
    planC.safeFixes.length === 3 &&
    planC.summary.safeFixGroups === 2 &&
    repoGroup?.count === 2 &&
    repoGroup.symbols.includes("CreateTurnInput") &&
    repoGroup.symbols.includes("ListLibraryDocsOptions"),
  JSON.stringify({
    summary: planC.summary,
    safeFixGroups: planC.safeFixGroups,
  }),
);
rmSync(OUT_C, { recursive: true, force: true });
rmSync(OUT, { recursive: true, force: true });
