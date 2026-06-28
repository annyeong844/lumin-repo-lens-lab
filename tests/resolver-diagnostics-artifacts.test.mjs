import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function runResolverDiagnostics(fixture) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, "build-resolver-diagnostics.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function buildResolverDiagnosticsArtifactFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-resolver-diagnostics-",
  });

  fixture.writeJson(
    "symbols.json",
    {
      uses: {
        resolvedInternal: 7,
        unresolvedInternal: 4,
        unresolvedInternalRatio: 0.3636,
        external: 2,
      },
      topUnresolvedSpecifiers: [
        {
          specifierPrefix: "@scope/orm",
          count: 2,
          example: "@scope/orm/client",
        },
      ],
      unresolvedInternalSpecifierRecords: [
        {
          specifier: "#app/config",
          consumerFile: "packages/app/src/a.ts",
          kind: "import",
          reason: "hash-import-target-missing",
          resolverStage: "hash-imports",
          matchedPattern: "#app/*",
          targetCandidates: ["packages/app/src/config"],
        },
        {
          specifier: "@scope/orm/client",
          consumerFile: "apps/api/src/b.ts",
          kind: "import",
          reason: "workspace-generated-artifact-missing",
          resolverStage: "workspace-package-subpath",
          hint: "generated-artifact-missing",
          targetCandidates: ["packages/orm/client"],
          generatedArtifact: {
            policyVersion: "generated-artifact-policy-v1",
            matchedPackage: "@scope/orm",
            targetSubpath: "client",
            generatorFamily: "prisma",
            confidence: "strong",
            packageRoot: "packages/orm",
          },
        },
        {
          specifier: "app/routes/root",
          consumerFile: "apps/web/src/c.ts",
          kind: "import",
          reason: "tsconfig-path-target-missing",
          resolverStage: "tsconfig-paths",
          matchedPattern: "app/*",
          targetCandidates: ["apps/web/app/routes/root"],
        },
      ],
      generatedConsumerBlindZones: [
        {
          reason: "generated-consumer-blind-zone",
          sourceReason: "workspace-generated-artifact-missing",
          specifier: "@scope/orm/client",
          consumerFile: "apps/api/src/b.ts",
          matchedPackage: "@scope/orm",
          targetSubpath: "client",
          generatorFamily: "prisma",
          confidence: "strong",
          candidatePath: "packages/orm/client",
          status: "missing",
          scopePackageRoot: "packages/orm",
          mode: "prepared",
          staleStatus: "unknown",
          staleReason: "generator-input-hash-not-recorded",
        },
      ],
    },
    { to: "output" },
  );

  runResolverDiagnostics(fixture);

  return {
    fixture,
    capabilities: fixture.readJson("resolver-capabilities.json", {
      from: "output",
    }),
    diagnostics: fixture.readJson("resolver-diagnostics.json", {
      from: "output",
    }),
  };
}

describe("resolver diagnostics artifact contract", () => {
  let fixture;
  let capabilities;
  let diagnostics;

  beforeAll(() => {
    ({ fixture, capabilities, diagnostics } =
      buildResolverDiagnosticsArtifactFixture());
  });

  afterAll(() => {
    fixture?.cleanup();
  });

  it("writes a deterministic resolver capability matrix", () => {
    const nodeImports = capabilities.families?.find(
      (family) => family.family === "node-imports",
    );
    const tsconfig = capabilities.families?.find(
      (family) => family.family === "tsconfig-paths",
    );

    expect(capabilities).toMatchObject({
      schemaVersion: "resolver-capabilities.v1",
    });
    expect(capabilities.resolverVersion).toMatch(
      /^resolver-\d{4}-\d{2}-v\d+$/,
    );
    expect(capabilities.conditionProfiles?.[0]).toMatchObject({
      profileId: "node-esm-default",
    });
    expect(nodeImports).toMatchObject({
      status: "partial",
    });
    expect(nodeImports.reasonCodes).toContain("hash-import-target-missing");
    expect(tsconfig).toMatchObject({
      absenceClaimPolicy: "fail-closed-when-relevant",
    });
  });

  it("keeps per-run diagnostics separate from capability metadata", () => {
    expect(diagnostics).toMatchObject({
      schemaVersion: "resolver-diagnostics.v1",
      resolverVersion: capabilities.resolverVersion,
      capabilityArtifact: "resolver-capabilities.json",
      capabilityReference: {
        artifact: "resolver-capabilities.json",
        schemaVersion: capabilities.schemaVersion,
        resolverVersion: capabilities.resolverVersion,
      },
    });
  });

  it("preserves unresolved imports and generated artifact metadata", () => {
    expect(diagnostics.unresolvedImports).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          specifier: "#app/config",
          family: "node-imports",
          outputLevel: "unresolved_with_reason",
          reason: "hash-import-target-missing",
        }),
        expect.objectContaining({
          specifier: "@scope/orm/client",
          family: "generated-artifacts",
          generatedArtifact: expect.objectContaining({
            generatorFamily: "prisma",
          }),
        }),
      ]),
    );
  });

  it("records candidate targets as diagnostic-only and not graph edges", () => {
    expect(diagnostics.candidateTargets).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          specifier: "#app/config",
          family: "node-imports",
          outputLevel: "candidate",
          proofUse: "diagnostic-only",
          createsGraphEdge: false,
          notResolvedBecause: "hash-import-target-missing",
          candidatePaths: expect.arrayContaining(["packages/app/src/config"]),
        }),
      ]),
    );
  });

  it("declares candidate-relevant resolver and generated blind-zone policies", () => {
    const hashImportZone = diagnostics.blindZones?.find(
      (zone) => zone.specifier === "#app/config",
    );
    const generatedConsumerZone = diagnostics.blindZones?.find(
      (zone) => zone.reason === "generated-consumer-blind-zone",
    );

    expect(hashImportZone).toMatchObject({
      blockingScope: "candidate-relevant",
      relevancePolicy: {
        policyVersion: "resolver-blind-zone-relevance.v1",
        mustNotBlockUnrelatedCandidates: true,
      },
    });
    expect(hashImportZone.relevancePolicy.candidateRelevantWhen).toEqual(
      expect.arrayContaining([
        "target-candidate-file",
        "target-candidate-package-scope",
        "target-candidate-submodule",
      ]),
    );

    expect(generatedConsumerZone).toMatchObject({
      family: "generated-artifacts",
      affectedPackageScope: "packages/orm",
      blocksAbsenceClaims: true,
      staleStatus: "unknown",
      blockingScope: "candidate-relevant",
      relevancePolicy: {
        policyVersion: "generated-blind-zone-relevance.v1",
        mustNotBlockUnrelatedCandidates: true,
      },
    });
    expect(generatedConsumerZone.relevancePolicy.candidateRelevantWhen).toEqual(
      expect.arrayContaining([
        "generated-consumer-scope",
        "generated-consumer-target-submodule",
      ]),
    );
  });

  it("exposes compact blocked candidate hints without action proof", () => {
    expect(diagnostics.summary?.blockedCandidateHintCount).toBe(
      diagnostics.blockedCandidateHints?.length,
    );
    expect(diagnostics.blockedCandidateHints).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          family: "node-imports",
          reason: "hash-import-target-missing",
          specifier: "#app/config",
          candidatePath: "packages/app/src/config",
          affectedPackageScope: "packages/app",
          blockingScope: "candidate-relevant",
          proofUse: "blocks-absence-claim",
        }),
        expect.objectContaining({
          family: "generated-artifacts",
          reason: "generated-consumer-blind-zone",
          specifier: "@scope/orm/client",
          candidatePath: "packages/orm/client",
          affectedPackageScope: "packages/orm",
          relevance: "generated-consumer-scope",
        }),
      ]),
    );
  });

  it("keeps summary pivots machine-readable", () => {
    expect(diagnostics.summary).toMatchObject({
      unresolvedInternal: 4,
      blindZoneCount: diagnostics.blindZones.length,
      blockedCandidateHintCount: diagnostics.blockedCandidateHints.length,
    });
    expect(diagnostics.summary.topFamilies?.[0]).toMatchObject({
      family: "generated-artifacts",
    });
    expect(diagnostics.summary.topAffectedPackageScopes).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          affectedPackageScope: "packages/orm",
          count: 2,
        }),
      ]),
    );
    expect(diagnostics.summary.topUnresolvedReasons).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          reason: "workspace-generated-artifact-missing",
          count: 1,
        }),
      ]),
    );
  });
});
