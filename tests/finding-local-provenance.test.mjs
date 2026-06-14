import { describe, expect, it } from "vitest";

import {
  computeFindingProvenance,
  specifierCouldMatchFile,
} from "../_lib/finding-provenance.mjs";
import { tierForFinding } from "../_lib/ranking.mjs";

function safeAction() {
  return {
    kind: "demote_export_declaration",
    proofComplete: true,
    actionBlockers: [],
    strongerActionBlockers: [],
  };
}

const aliasMap = {
  scopedTsconfigPaths: [
    {
      scopeDir: "apps/web",
      baseUrlDir: "apps/web",
      key: "@/*",
      matchPrefix: "@/",
      matchSuffix: "",
      targets: ["./*"],
      wildcard: true,
    },
  ],
  scopedTsconfigBaseUrls: [
    {
      scopeDir: "apps/web",
      baseUrlDir: "apps/web",
    },
  ],
};

const submoduleOf = (file) =>
  file.replace(/\\/g, "/").split("/").slice(0, 2).join("/");

const cleanOptions = {
  filesWithParseErrors: [],
  unresolvedInternalSpecifiers: [],
  astEvidence: "ast-ident-ref-count",
  astCount: 0,
};

const strongEvidence = {
  runtime: { status: "dead-confirmed", grounding: "grounded", hitsInSymbol: 0 },
  staleness: { tier: "fossil", grounding: "grounded" },
  resolver: { unresolvedRatio: 0.05 },
};

describe("finding-local provenance specifier matching", () => {
  it("S1. known alias matches only inside alias scope", () => {
    expect(
      specifierCouldMatchFile(
        "@/components/auth-control",
        "apps/web/components/auth-control.tsx",
        {
          aliasMap,
          fromHint: "apps/web/page.ts",
          submoduleOf,
        },
      ),
    ).toBe("match");
  });

  it("S2. known alias does not taint a different scope", () => {
    expect(
      specifierCouldMatchFile(
        "@/components/auth-control",
        "apps/api/components/auth-control.ts",
        {
          aliasMap,
          fromHint: "apps/web/page.ts",
          submoduleOf,
        },
      ),
    ).toBe("no-match");
  });

  it("S3. known alias does not match unrelated target in scope", () => {
    expect(
      specifierCouldMatchFile(
        "@/components/auth-control",
        "apps/web/utils/logger.ts",
        {
          aliasMap,
          fromHint: "apps/web/page.ts",
          submoduleOf,
        },
      ),
    ).toBe("no-match");
  });

  it("S4. bare specifier without slash does not match anything", () => {
    expect(
      specifierCouldMatchFile("@scope", "any/file.ts", {
        aliasMap,
        submoduleOf,
      }),
    ).toBe("no-match");
  });

  it("S5. unknown alias-like spec is unknown only in same submodule", () => {
    expect(
      specifierCouldMatchFile("~/config", "apps/web/config.ts", {
        aliasMap,
        fromHint: "apps/web/page.ts",
        submoduleOf,
      }),
    ).toBe("unknown");
  });

  it("S6. unknown alias-like spec does not taint other submodule", () => {
    expect(
      specifierCouldMatchFile("~/config", "apps/api/config.ts", {
        aliasMap,
        fromHint: "apps/web/page.ts",
        submoduleOf,
      }),
    ).toBe("no-match");
  });

  it("S7. baseUrl-like spec is unknown in matching baseUrl scope", () => {
    expect(
      specifierCouldMatchFile("app/_types", "apps/web/app/_types.ts", {
        aliasMap,
        fromHint: "apps/web/page.ts",
        submoduleOf,
      }),
    ).toBe("unknown");
  });

  it("S8. baseUrl-like spec does not taint outside matching scope", () => {
    expect(
      specifierCouldMatchFile("app/_types", "apps/api/app/_types.ts", {
        aliasMap,
        fromHint: "apps/web/page.ts",
        submoduleOf,
      }),
    ).toBe("no-match");
  });

  it("S9. relative specifier matches importer-normalized path", () => {
    expect(
      specifierCouldMatchFile(
        "../components/auth-control",
        "apps/web/components/auth-control.tsx",
        {
          fromHint: "apps/web/pages/page.ts",
          submoduleOf,
        },
      ),
    ).toBe("match");
  });

  it("S10. Windows-style backslash target path is normalized", () => {
    expect(
      specifierCouldMatchFile(
        "@/components/auth-control",
        "apps\\web\\components\\auth-control.tsx",
        {
          aliasMap,
          fromHint: "apps/web/page.ts",
          submoduleOf,
        },
      ),
    ).toBe("match");
  });
});

describe("finding-local provenance taints", () => {
  it("P1. no taint when clean", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      cleanOptions,
    );

    expect(provenance.taintedBy).toEqual([]);
    expect(provenance.resolverConfidence).toBe("high");
    expect(provenance.parseStatus).toBe("ok");
    expect(provenance.supportedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "ast-ident-ref-count" }),
      ]),
    );
  });

  it("P2. parse-errors-elsewhere produces parse-errors-present taint", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["src/other.ts"],
      },
    );

    expect(provenance.taintedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "parse-errors-present" }),
      ]),
    );
    expect(provenance.resolverConfidence).toBe("medium");
  });

  it("P2c. parse error in unrelated submodule does not taint finding", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/api/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["apps/web/broken.ts"],
        submoduleOf,
      },
    );

    expect(provenance.taintedBy).toEqual([]);
  });

  it("P2d. parse error in same submodule remains relevant soft taint", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["apps/web/broken.ts"],
        submoduleOf,
      },
    );

    expect(provenance.taintedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "parse-errors-present" }),
      ]),
    );
  });

  it("P3. defining-file-parse-error taint emitted when file in list", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["src/foo.ts"],
      },
    );

    expect(provenance.taintedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "defining-file-parse-error" }),
      ]),
    );
    expect(provenance.parseStatus).toBe("error");
    expect(provenance.resolverConfidence).toBe("low");
  });

  it("P4. unresolved-specifier-could-match detected via scoped alias", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/components/auth-control.tsx", symbol: "AuthControl" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@/components/auth-control",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
          {
            specifier: "@/other/thing",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );
    const match = provenance.taintedBy.find(
      (taint) => taint.kind === "unresolved-specifier-could-match",
    );

    expect(match).toEqual(expect.any(Object));
    expect(match.specifiers).toContain("@/components/auth-control");
    expect(match.specifiers).not.toContain("@/other/thing");
    expect(provenance.resolverConfidence).toBe("low");
  });

  it("P4e. unknown alias in same submodule emits weak unresolved taint", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/config.ts", symbol: "config" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "~/config",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );
    const unknown = provenance.taintedBy.find(
      (taint) => taint.kind === "unresolved-specifier-could-match-unknown",
    );

    expect(unknown).toEqual(expect.any(Object));
    expect(unknown.consumerFile).toBe("apps/web/page.ts");
    expect(provenance.resolverConfidence).toBe("medium");
  });

  it("P4h. unknown alias from other submodule does not taint finding", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/api/config.ts", symbol: "config" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "~/config",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.taintedBy).toEqual([]);
  });

  it("P5. affected file is tainted", () => {
    const unresolvedInRepo = [
      {
        specifier: "@/components/auth-control",
        consumerFile: "apps/web/page.ts",
        fromHint: "apps/web/page.ts",
      },
    ];
    const affected = computeFindingProvenance(
      { file: "apps/web/components/auth-control.tsx", symbol: "AuthControl" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: unresolvedInRepo,
        aliasMap,
        submoduleOf,
      },
    );
    const unaffected = computeFindingProvenance(
      { file: "apps/web/utils/logger.ts", symbol: "log" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: unresolvedInRepo,
        aliasMap,
        submoduleOf,
      },
    );

    expect(affected.taintedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "unresolved-specifier-could-match",
        }),
      ]),
    );
    expect(unaffected.taintedBy).toEqual([]);
  });

  it("P6. generated artifact miss in candidate package emits relevant soft taint", () => {
    const provenance = computeFindingProvenance(
      { file: "packages/prisma/index.ts", symbol: "PrismaEnums" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@scope/prisma/enums",
            consumerFile: "apps/web/page.ts",
            reason: "workspace-generated-artifact-missing",
            hint: "generated-artifact-missing",
            targetCandidates: [
              "packages/prisma/enums.ts",
              "packages/prisma/enums/index.ts",
            ],
            generatedArtifact: {
              policyVersion: "generated-artifact-policy-v1",
              matchedPackage: "@scope/prisma",
              targetSubpath: "enums",
              generatorFamily: "prisma",
              confidence: "strong",
            },
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );
    const generatedTaint = provenance.taintedBy.find(
      (taint) => taint.kind === "generated-artifact-missing-relevant",
    );

    expect(generatedTaint).toEqual(
      expect.objectContaining({
        specifier: "@scope/prisma/enums",
        matchedPackage: "@scope/prisma",
        targetSubpath: "enums",
        impact: "provider-surface-unresolved",
      }),
    );
    expect(provenance.resolverConfidence).toBe("medium");
  });

  it("P7. unrelated generated artifact miss does not taint another package", () => {
    const provenance = computeFindingProvenance(
      { file: "packages/ui/Button.tsx", symbol: "Button" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@scope/prisma/enums",
            consumerFile: "packages/prisma/client.ts",
            reason: "workspace-generated-artifact-missing",
            hint: "generated-artifact-missing",
            generatedArtifact: {
              policyVersion: "generated-artifact-policy-v1",
              matchedPackage: "@scope/prisma",
              packageRoot: "packages/prisma",
              targetSubpath: "enums",
              generatorFamily: "prisma",
              confidence: "strong",
            },
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.taintedBy).toEqual([]);
    expect(provenance.resolverConfidence).toBe("high");
  });

  it("P8. generated provider miss in consumer submodule alone stays clean", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/app/_types.ts", symbol: "LayoutProps" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@scope/prisma/enums",
            consumerFile: "apps/web/page.ts",
            reason: "workspace-generated-artifact-missing",
            hint: "generated-artifact-missing",
            targetCandidates: [
              "packages/prisma/enums.ts",
              "packages/prisma/enums/index.ts",
            ],
            generatedArtifact: {
              policyVersion: "generated-artifact-policy-v1",
              matchedPackage: "@scope/prisma",
              targetSubpath: "enums",
              generatorFamily: "prisma",
              confidence: "strong",
            },
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.taintedBy).toEqual([]);
    expect(provenance.resolverConfidence).toBe("high");
  });

  it("P9. generated consumer blind zone emits consumer-surface taint", () => {
    const provenance = computeFindingProvenance(
      { file: "packages/prisma/model.ts", symbol: "ModelName" },
      {
        ...cleanOptions,
        aliasMap,
        submoduleOf,
        generatedConsumerBlindZones: [
          {
            reason: "generated-consumer-blind-zone",
            sourceReason: "workspace-generated-artifact-missing",
            specifier: "@scope/prisma/enums",
            consumerFile: "apps/web/page.ts",
            matchedPackage: "@scope/prisma",
            targetSubpath: "enums",
            candidatePath: "packages/prisma/generated/enums.ts",
            scopePackageRoot: "packages/prisma",
            status: "missing",
            mode: "default",
          },
        ],
      },
    );

    expect(provenance.taintedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "generated-artifact-missing-relevant",
          reason: "generated-consumer-blind-zone",
          impact: "consumer-surface-unresolved",
          relevance: "generated-consumer-scope",
        }),
      ]),
    );
    expect(provenance.resolverConfidence).toBe("medium");
  });

  it("P1b. resolverConfidence=high when clean", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      cleanOptions,
    );

    expect(provenance.resolverConfidence).toBe("high");
  });

  it("P1c. parseStatus=ok when file not in error list", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      cleanOptions,
    );

    expect(provenance.parseStatus).toBe("ok");
  });

  it("P1d. supportedBy includes ast-ident-ref-count", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      cleanOptions,
    );

    expect(provenance.supportedBy).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "ast-ident-ref-count" }),
      ]),
    );
  });

  it("P2b. resolverConfidence=medium when only soft taint", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["src/other.ts"],
      },
    );

    expect(provenance.resolverConfidence).toBe("medium");
  });

  it("P3b. parseStatus=error when file is in the error list", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["src/foo.ts"],
      },
    );

    expect(provenance.parseStatus).toBe("error");
  });

  it("P3c. resolverConfidence=low for blocking taint", () => {
    const provenance = computeFindingProvenance(
      { file: "src/foo.ts", symbol: "foo" },
      {
        ...cleanOptions,
        filesWithParseErrors: ["src/foo.ts"],
      },
    );

    expect(provenance.resolverConfidence).toBe("low");
  });

  it("P4b. matched specifier listed", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/components/auth-control.tsx", symbol: "AuthControl" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@/components/auth-control",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
          {
            specifier: "@/other/thing",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );
    const match = provenance.taintedBy.find(
      (taint) => taint.kind === "unresolved-specifier-could-match",
    );

    expect(match.specifiers).toContain("@/components/auth-control");
  });

  it("P4c. non-matching specifier not listed", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/components/auth-control.tsx", symbol: "AuthControl" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@/components/auth-control",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
          {
            specifier: "@/other/thing",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );
    const match = provenance.taintedBy.find(
      (taint) => taint.kind === "unresolved-specifier-could-match",
    );

    expect(match.specifiers).not.toContain("@/other/thing");
  });

  it("P4d. resolverConfidence=low on blocking spec match", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/components/auth-control.tsx", symbol: "AuthControl" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@/components/auth-control",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.resolverConfidence).toBe("low");
  });

  it("P4f. weak unresolved taint records consumer file", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/config.ts", symbol: "config" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "~/config",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );
    const unknown = provenance.taintedBy.find(
      (taint) => taint.kind === "unresolved-specifier-could-match-unknown",
    );

    expect(unknown.consumerFile).toBe("apps/web/page.ts");
  });

  it("P4g. weak unresolved taint is medium confidence, not low", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/config.ts", symbol: "config" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "~/config",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.resolverConfidence).toBe("medium");
  });

  it("P5b. unaffected file stays clean (the P1 win)", () => {
    const provenance = computeFindingProvenance(
      { file: "apps/web/utils/logger.ts", symbol: "log" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@/components/auth-control",
            consumerFile: "apps/web/page.ts",
            fromHint: "apps/web/page.ts",
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.taintedBy).toEqual([]);
  });

  it("P6b. generated artifact relevant taint lowers resolver confidence to medium", () => {
    const provenance = computeFindingProvenance(
      { file: "packages/prisma/index.ts", symbol: "PrismaEnums" },
      {
        ...cleanOptions,
        unresolvedInternalSpecifiers: [
          {
            specifier: "@scope/prisma/enums",
            consumerFile: "apps/web/page.ts",
            reason: "workspace-generated-artifact-missing",
            hint: "generated-artifact-missing",
            targetCandidates: [
              "packages/prisma/enums.ts",
              "packages/prisma/enums/index.ts",
            ],
            generatedArtifact: {
              policyVersion: "generated-artifact-policy-v1",
              matchedPackage: "@scope/prisma",
              targetSubpath: "enums",
              generatorFamily: "prisma",
              confidence: "strong",
            },
          },
        ],
        aliasMap,
        submoduleOf,
      },
    );

    expect(provenance.resolverConfidence).toBe("medium");
  });
});

describe("finding-local provenance ranking effects", () => {
  it("T1. empty taintedBy plus strong evidence promotes to SAFE_FIX", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      taintedBy: [],
      safeAction: safeAction(),
    };

    expect(tierForFinding(finding, strongEvidence).tier).toBe("SAFE_FIX");
  });

  it("T2. unresolved-specifier-could-match → DEGRADED (overrides strong evidence)", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "unresolved-specifier-could-match",
          specifiers: ["@/components/a"],
          total: 1,
          effect: "...",
        },
      ],
    };
    const { tier, reason } = tierForFinding(finding, strongEvidence);

    expect(tier).toBe("DEGRADED");
    expect(reason).toContain("@/components/a");
  });

  it("T2c. unknown unresolved specifier demotes SAFE_FIX to REVIEW_FIX", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "unresolved-specifier-could-match-unknown",
          specifiers: ["~/a"],
          consumerFile: "src/consumer.ts",
          total: 1,
          effect: "...",
        },
      ],
    };

    expect(tierForFinding(finding, strongEvidence).tier).toBe("REVIEW_FIX");
  });

  it("T3. defining-file parse error degrades to DEGRADED", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "defining-file-parse-error",
          file: "src/a.ts",
          effect: "...",
        },
      ],
    };

    expect(tierForFinding(finding, strongEvidence).tier).toBe("DEGRADED");
  });

  it("T4. parse-errors-present demotes SAFE_FIX → REVIEW_FIX (soft taint)", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "parse-errors-present",
          scope: "repo-wide",
          affected: 2,
          sample: ["src/other.ts"],
          effect: "...",
        },
      ],
    };
    const { tier, reason } = tierForFinding(finding, strongEvidence);

    expect(tier).toBe("REVIEW_FIX");
    expect(reason).toContain("parse-errors-elsewhere");
  });

  it("T4c. relevant generated artifact miss demotes SAFE_FIX → REVIEW_FIX", () => {
    const finding = {
      file: "packages/prisma/index.ts",
      line: 1,
      symbol: "PrismaEnums",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "generated-artifact-missing-relevant",
          specifier: "@scope/prisma/enums",
          matchedPackage: "@scope/prisma",
          targetSubpath: "enums",
          impact: "provider-surface-unresolved",
          effect: "...",
        },
      ],
    };
    const { tier, reason } = tierForFinding(finding, strongEvidence);

    expect(tier).toBe("REVIEW_FIX");
    expect(reason).toContain("generated-artifact-missing");
  });

  it("T5. clean finding in high-global-ratio repo remains SAFE_FIX", () => {
    const finding = {
      file: "src/clean.ts",
      line: 1,
      symbol: "x",
      bucket: "C",
      taintedBy: [],
      safeAction: safeAction(),
    };
    const evidence = {
      runtime: {
        status: "dead-confirmed",
        grounding: "grounded",
        hitsInSymbol: 0,
      },
      staleness: { tier: "fossil", grounding: "grounded" },
      resolver: { unresolvedRatio: 0.45 },
    };

    expect(tierForFinding(finding, evidence).tier).toBe("SAFE_FIX");
  });

  it("T6. legacy finding (no taintedBy) falls back to global ratio gate", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
    };
    const evidence = {
      runtime: {
        status: "dead-confirmed",
        grounding: "grounded",
        hitsInSymbol: 0,
      },
      staleness: { tier: "fossil", grounding: "grounded" },
      resolver: { unresolvedRatio: 0.45 },
    };
    const { tier, reason } = tierForFinding(finding, evidence);

    expect(tier).toBe("DEGRADED");
    expect(reason).toContain("resolver-blind");
  });

  it("T2b. reason surfaces the matching specifier", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "unresolved-specifier-could-match",
          specifiers: ["@/components/a"],
          total: 1,
          effect: "...",
        },
      ],
    };
    const { reason } = tierForFinding(finding, strongEvidence);

    expect(reason).toContain("@/components/a");
  });

  it("T4b. reason mentions parse-errors-elsewhere", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "parse-errors-present",
          scope: "repo-wide",
          affected: 2,
          sample: ["src/other.ts"],
          effect: "...",
        },
      ],
    };
    const { reason } = tierForFinding(finding, strongEvidence);

    expect(reason).toContain("parse-errors-elsewhere");
  });

  it("T4d. reason mentions generated-artifact-missing", () => {
    const finding = {
      file: "packages/prisma/index.ts",
      line: 1,
      symbol: "PrismaEnums",
      bucket: "C",
      safeAction: safeAction(),
      taintedBy: [
        {
          kind: "generated-artifact-missing-relevant",
          specifier: "@scope/prisma/enums",
          matchedPackage: "@scope/prisma",
          targetSubpath: "enums",
          impact: "provider-surface-unresolved",
          effect: "...",
        },
      ],
    };
    const { reason } = tierForFinding(finding, strongEvidence);

    expect(reason).toContain("generated-artifact-missing");
  });

  it("T6b. reason mentions resolver-blind fallback", () => {
    const finding = {
      file: "src/a.ts",
      line: 1,
      symbol: "a",
      bucket: "C",
    };
    const evidence = {
      runtime: {
        status: "dead-confirmed",
        grounding: "grounded",
        hitsInSymbol: 0,
      },
      staleness: { tier: "fossil", grounding: "grounded" },
      resolver: { unresolvedRatio: 0.45 },
    };
    const { reason } = tierForFinding(finding, evidence);

    expect(reason).toContain("resolver-blind");
  });
});
