import { describe, expect, it } from "vitest";

import {
  buildGeneratedConsumerBlindZones,
  generatedArtifactRelevance,
  generatedArtifactRelevantTaint,
  generatedConsumerBlindZoneRelevance,
} from "../_lib/generated-blind-zone-relevance.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function submoduleOf(file) {
  const parts = String(file ?? "").replace(/\\/g, "/").split("/");
  if ((parts[0] === "apps" || parts[0] === "packages") && parts.length >= 2) {
    return `${parts[0]}/${parts[1]}`;
  }
  return parts[0] || "root";
}

function generatedRecord(overrides = {}) {
  return {
    specifier: "@scope/prisma/enums",
    consumerFile: "apps/web/page.ts",
    reason: "workspace-generated-artifact-missing",
    hint: "generated-artifact-missing",
    targetCandidates: ["packages/prisma/generated/enums.ts"],
    generatedArtifact: {
      policyVersion: "generated-artifact-policy-v1",
      matchedPackage: "@scope/prisma",
      packageRoot: "packages/prisma",
      targetSubpath: "generated/enums",
      generatorFamily: "prisma",
      confidence: "strong",
    },
    ...overrides,
  };
}

describe("generated blind-zone relevance policy", () => {
  describe("provider-surface relevance", () => {
    it("treats a candidate inside the generated package root as relevant", () => {
      const relevance = generatedArtifactRelevance(
        { file: "packages/prisma/index.ts", symbol: "PrismaEnums" },
        generatedRecord(),
        { submoduleOf },
      );

      expect(relevance).toEqual({
        impact: "provider-surface-unresolved",
        relevance: "matched-package-root",
      });
    });

    it("uses target-candidate submodule relevance when package root is absent", () => {
      const relevance = generatedArtifactRelevance(
        { file: "packages/prisma/client.ts", symbol: "PrismaClient" },
        generatedRecord({
          generatedArtifact: {
            policyVersion: "generated-artifact-policy-v1",
            matchedPackage: "@scope/prisma",
            targetSubpath: "generated/enums",
            generatorFamily: "prisma",
            confidence: "strong",
          },
        }),
        { submoduleOf },
      );

      expect(relevance).toEqual({
        impact: "provider-surface-unresolved",
        relevance: "target-candidate-submodule",
      });
    });
  });

  describe("consumer-only non-relevance", () => {
    it("does not treat consumer submodule overlap as provider-surface proof", () => {
      const relevance = generatedArtifactRelevance(
        { file: "apps/web/components/Button.tsx", symbol: "Button" },
        generatedRecord(),
        { submoduleOf },
      );

      expect(relevance).toBeNull();
    });

    it("does not create finding taint from consumer-only generated misses", () => {
      const taint = generatedArtifactRelevantTaint(
        { file: "apps/web/components/Button.tsx", symbol: "Button" },
        [generatedRecord()],
        { submoduleOf },
      );

      expect(taint).toBeNull();
    });
  });

  describe("blind-zone inventory shape", () => {
    it("records missing generated target scope for generated consumers", () => {
      const zones = buildGeneratedConsumerBlindZones(
        {
          unresolvedInternalSpecifierRecords: [generatedRecord()],
        },
        {
          root: "C:/repo",
          includeTests: true,
          exclude: [],
          mode: "default",
        },
      );

      expect(zones).toEqual([
        {
          reason: "generated-consumer-blind-zone",
          sourceReason: "workspace-generated-artifact-missing",
          specifier: "@scope/prisma/enums",
          consumerFile: "apps/web/page.ts",
          matchedPackage: "@scope/prisma",
          targetSubpath: "generated/enums",
          generatorFamily: "prisma",
          confidence: "strong",
          candidatePath: "packages/prisma/generated/enums.ts",
          status: "missing",
          scopePackageRoot: "packages/prisma",
          mode: "default",
        },
      ]);
    });

    it("keeps present generated files excluded by scan policy as blind zones", () => {
      const fixture = createTempRepoFixture({
        prefix: "fx-vitest-generated-blind-zone-relevance-",
      });

      try {
        const generatedFile = fixture.write(
          "packages/prisma/generated/enums.ts",
          "export const GeneratedEnum = 1;\n",
        );

        const zones = buildGeneratedConsumerBlindZones(
          {
            unresolvedInternalSpecifierRecords: [
              generatedRecord({
                targetCandidates: [generatedFile],
              }),
            ],
          },
          {
            root: fixture.root,
            includeTests: true,
            exclude: ["packages/prisma/generated"],
            mode: "prepared",
          },
        );

        expect(zones).toHaveLength(1);
        expect(zones[0]).toMatchObject({
          status: "present-but-out-of-scope",
          scanScopeReason: "excluded",
          staleStatus: "unknown",
          staleReason: "generator-input-hash-not-recorded",
        });
      } finally {
        fixture.cleanup();
      }
    });
  });

  describe("generated consumer relevance and soft taint", () => {
    it("scopes generated consumer blind-zone relevance to the generated package surface", () => {
      const zone = buildGeneratedConsumerBlindZones(
        {
          unresolvedInternalSpecifierRecords: [generatedRecord()],
        },
        {
          root: "C:/repo",
        },
      )[0];

      expect(
        generatedConsumerBlindZoneRelevance(
          { file: "packages/prisma/model.ts", symbol: "ModelName" },
          zone,
          { submoduleOf },
        ),
      ).toEqual({
        impact: "consumer-surface-unresolved",
        relevance: "generated-consumer-scope",
      });
      expect(
        generatedConsumerBlindZoneRelevance(
          { file: "apps/web/components/Button.tsx", symbol: "Button" },
          zone,
          { submoduleOf },
        ),
      ).toBeNull();
    });

    it("creates structured soft taint for relevant generated consumer blind zones", () => {
      const zone = buildGeneratedConsumerBlindZones(
        {
          unresolvedInternalSpecifierRecords: [generatedRecord()],
        },
        {
          root: "C:/repo",
        },
      )[0];
      const taint = generatedArtifactRelevantTaint(
        { file: "packages/prisma/model.ts", symbol: "ModelName" },
        [],
        { submoduleOf, generatedConsumerBlindZones: [zone] },
      );

      expect(taint).toMatchObject({
        kind: "generated-artifact-missing-relevant",
        reason: "generated-consumer-blind-zone",
        impact: "consumer-surface-unresolved",
        relevance: "generated-consumer-scope",
      });
    });
  });
});
