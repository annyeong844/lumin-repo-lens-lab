import { describe, expect, it } from "vitest";

import {
  RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
  resolverBlindZoneRelevance,
  resolverBlindZoneRelevantTaint,
} from "../_lib/resolver-blind-zone-relevance.mjs";
import { computeFindingProvenance } from "../_lib/finding-provenance.mjs";
import { tierForFinding } from "../_lib/ranking.mjs";

function submoduleOf(file) {
  const parts = String(file ?? "")
    .replace(/\\/g, "/")
    .replace(/^\.\//, "")
    .split("/");
  if ((parts[0] === "apps" || parts[0] === "packages") && parts.length >= 2) {
    return `${parts[0]}/${parts[1]}`;
  }
  return parts[0] || "root";
}

function workspaceMiss(overrides = {}) {
  return {
    specifier: "@scope/lib/missing",
    consumerFile: "apps/web/src/page.ts",
    reason: "workspace-package-subpath-target-missing",
    resolverStage: "workspace-package-subpath",
    outputLevel: "unresolved_with_reason",
    targetCandidates: ["packages/lib/src/missing.ts"],
    family: "workspace-packages",
    ...overrides,
  };
}

function safeAction() {
  return {
    kind: "demote_export_declaration",
    proofComplete: true,
    actionBlockers: [],
    strongerActionBlockers: [],
  };
}

describe("resolver blind-zone relevance policy", () => {
  it("exports the policy version for diagnostics", () => {
    expect(RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION).toBe(
      "resolver-blind-zone-relevance.v1",
    );
  });

  it("scopes target candidate package relevance to same-package findings", () => {
    expect(
      resolverBlindZoneRelevance(
        { file: "packages/lib/src/foo.ts", symbol: "foo" },
        workspaceMiss(),
        { submoduleOf },
      ),
    ).toEqual({
      impact: "resolver-surface-unresolved",
      relevance: "target-candidate-package-scope",
      severity: "soft",
    });

    expect(
      resolverBlindZoneRelevance(
        { file: "packages/ui/src/Button.tsx", symbol: "Button" },
        workspaceMiss(),
        { submoduleOf },
      ),
    ).toBeNull();
  });

  it("uses affected package scope without repo-global blocking", () => {
    const scopedMiss = workspaceMiss({
      targetCandidates: [],
      affectedPackageScope: "packages/api",
      family: "conditional-exports",
      reason: "condition-profile-ambiguous",
    });

    expect(
      resolverBlindZoneRelevance(
        { file: "packages/api/src/router.ts", symbol: "router" },
        scopedMiss,
        { submoduleOf },
      ),
    ).toEqual({
      impact: "resolver-surface-unresolved",
      relevance: "affected-package-scope",
      severity: "soft",
    });

    expect(
      resolverBlindZoneRelevance(
        { file: "packages/web/src/router.ts", symbol: "router" },
        scopedMiss,
        { submoduleOf },
      ),
    ).toBeNull();
  });

  it("keeps exact target candidate files as blocking unresolved matches", () => {
    expect(
      resolverBlindZoneRelevance(
        { file: "packages/lib/src/missing.ts", symbol: "missing" },
        workspaceMiss(),
        { submoduleOf },
      ),
    ).toEqual({
      impact: "resolver-surface-unresolved",
      relevance: "target-candidate-file",
      severity: "blocking",
    });
  });

  it("leaves generated artifact records to generated relevance helpers", () => {
    expect(
      resolverBlindZoneRelevance(
        { file: "packages/prisma/index.ts", symbol: "PrismaEnums" },
        workspaceMiss({
          reason: "workspace-generated-artifact-missing",
          hint: "generated-artifact-missing",
          generatedArtifact: {
            policyVersion: "generated-artifact-policy-v1",
            packageRoot: "packages/prisma",
          },
          targetCandidates: ["packages/prisma/generated/enums.ts"],
        }),
        { submoduleOf },
      ),
    ).toBeNull();

    expect(
      resolverBlindZoneRelevance(
        { file: "packages/prisma/model.ts", symbol: "ModelName" },
        {
          family: "generated-artifacts",
          reason: "generated-consumer-blind-zone",
          affectedPackageScope: "packages/prisma",
          candidatePath: "packages/prisma/generated/enums.ts",
        },
        { submoduleOf },
      ),
    ).toBeNull();
  });

  it("creates structured soft taint for generic relevant resolver misses", () => {
    const taint = resolverBlindZoneRelevantTaint(
      { file: "packages/lib/src/foo.ts", symbol: "foo" },
      [workspaceMiss()],
      { submoduleOf },
    );

    expect(taint).toMatchObject({
      kind: "resolver-blind-zone-relevant",
      reason: "workspace-package-subpath-target-missing",
      family: "workspace-packages",
      impact: "resolver-surface-unresolved",
      relevance: "target-candidate-package-scope",
      total: 1,
    });
  });

  it("taints only affected findings in provenance", () => {
    const evidence = {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [workspaceMiss()],
      submoduleOf,
      astEvidence: "ast-ident-ref-count",
      astCount: 0,
    };

    const relevant = computeFindingProvenance(
      { file: "packages/lib/src/foo.ts", symbol: "foo" },
      evidence,
    );
    const unrelated = computeFindingProvenance(
      { file: "packages/ui/src/Button.tsx", symbol: "Button" },
      evidence,
    );

    expect(
      relevant.taintedBy.some(
        (taint) => taint.kind === "resolver-blind-zone-relevant",
      ),
    ).toBe(true);
    expect(relevant.resolverConfidence).toBe("medium");
    expect(unrelated.taintedBy).toEqual([]);
    expect(unrelated.resolverConfidence).toBe("high");
  });

  it("demotes SAFE_FIX to REVIEW_FIX with resolver blocker detail", () => {
    const result = tierForFinding(
      {
        file: "packages/lib/src/foo.ts",
        line: 1,
        symbol: "foo",
        bucket: "C",
        safeAction: safeAction(),
        taintedBy: [
          {
            kind: "resolver-blind-zone-relevant",
            reason: "workspace-package-subpath-target-missing",
            family: "workspace-packages",
            specifier: "@scope/lib/missing",
            impact: "resolver-surface-unresolved",
            relevance: "target-candidate-package-scope",
            effect: "...",
          },
        ],
      },
      {
        resolver: { unresolvedRatio: 0.01 },
      },
    );

    expect(result.tier).toBe("REVIEW_FIX");
    expect(result.reason).toMatch(/resolver-blind-zone/);
    expect(result.blockedPromotion).toBe(true);
    expect(result.blockedBy?.[0]).toMatchObject({
      family: "workspace-packages",
      relevance: "target-candidate-package-scope",
    });
  });
});
