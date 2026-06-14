import { readFileSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import * as evidence from "../_lib/generated-artifact-evidence.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

describe("generated artifact evidence policy", () => {
  it("requires files coverage plus build-like script evidence for strong build output", () => {
    const packet = evidence.generatedOutputArtifactEvidence(
      {
        name: "@scope/bundle",
        files: ["dist"],
        scripts: { build: "vite build" },
      },
      "./dist/bundle.js",
      'exports["."]',
    );

    expect(packet).toMatchObject({
      policyVersion: "generated-artifact-policy-v1",
      generatorFamily: "build-output",
      confidence: "strong",
      matchedPackage: "@scope/bundle",
      targetSubpath: "dist/bundle.js",
    });
    expect(packet.evidence).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "package-files",
          field: "files",
          matched: "dist",
        }),
        expect.objectContaining({
          kind: "package-script",
          field: "scripts.build",
          matched: "vite build",
        }),
      ]),
    );
  });

  it("keeps files-only build output weak instead of producing strong generated evidence", () => {
    const packet = evidence.generatedOutputArtifactEvidence(
      {
        name: "@scope/bundle",
        files: ["dist"],
      },
      "./dist/bundle.js",
      'exports["."]',
    );

    expect(packet).toBeNull();
  });

  it("requires explicit package script output path evidence for static artifacts", () => {
    const packet = evidence.generatedOutputArtifactEvidence(
      {
        name: "@scope/css-output",
        files: ["style.min.css"],
        scripts: { build: "postcss ./style.css -o ./style.min.css" },
      },
      "./style.min.css",
      'exports["./style.min.css"]',
    );

    expect(packet).toMatchObject({
      generatorFamily: "static-artifact",
      confidence: "strong",
      targetSubpath: "style.min.css",
    });
    expect(packet.evidence).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "script-output-path",
          field: "scripts.build",
          matched: "style.min.css",
        }),
      ]),
    );
  });

  it("requires an exact package script output path for relative generated assets", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-generated-artifact-evidence-",
      packageJson: {
        name: "relative-generated-fixture",
        type: "module",
        scripts: {
          tailwind:
            "tailwindcss --input ./src/styles.css --output ./src/tailwind.generated.css",
        },
      },
    });

    try {
      fixture.write("src/consumer.ts", "import './tailwind.generated.css';\n");
      const fromFile = fixture.path("src/consumer.ts");
      const generatedTarget = fixture.path("src/tailwind.generated.css");
      const ordinaryTarget = fixture.path("src/ordinary.css");

      const packet = evidence.generatedRelativeArtifactEvidence(
        fixture.root,
        fromFile,
        generatedTarget,
      );

      expect(packet).toMatchObject({
        policyVersion: "generated-artifact-policy-v1",
        generatorFamily: "local-generated-asset",
        confidence: "strong",
        matchedPackage: "relative-generated-fixture",
        packageRoot: ".",
        targetSubpath: "src/tailwind.generated.css",
      });
      expect(packet.evidence).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            kind: "script-output-path",
            field: "scripts.tailwind",
            matched: "src/tailwind.generated.css",
          }),
        ]),
      );
      expect(
        evidence.generatedRelativeArtifactEvidence(
          fixture.root,
          fromFile,
          ordinaryTarget,
        ),
      ).toBeNull();
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps path-segment evidence supporting-only while exposing the generated hint", () => {
    const root = path.resolve("repo");
    const candidate = path.join(root, "packages/generated/generated/client");
    const packet = evidence.generatedArtifactForTargetCandidates(root, [
      candidate,
    ]);

    expect(evidence.GENERATED_ARTIFACT_MISSING_HINT).toBe(
      "generated-artifact-missing",
    );
    expect(packet).toMatchObject({
      policyVersion: "generated-artifact-policy-v1",
      generatorFamily: "path-segment",
      confidence: "supporting",
      targetSubpath: "packages/generated/generated/client",
    });
    expect(evidence.isStrongGeneratedArtifact(packet)).toBe(false);
    expect(evidence.unresolvedGeneratedArtifactHintForCandidates([candidate])).toBe(
      "generated-artifact-missing",
    );
  });

  it("matches workspace subpath evidence only against normalized target subpaths", () => {
    const packet = {
      policyVersion: "generated-artifact-policy-v1",
      generatorFamily: "prisma",
      confidence: "strong",
      targetSubpath: "enums",
      evidence: [],
    };
    const entry = {
      legacySubpath: true,
      generatedSubpathEvidence: [packet],
    };

    expect(evidence.generatedWorkspaceSubpathEvidence(entry, "enums.ts")).toBe(
      packet,
    );
    expect(evidence.generatedWorkspaceSubpathEvidence(entry, "client")).toBeNull();
  });

  it("exports generated artifact identity constants from the policy module", () => {
    expect(evidence.GENERATED_ARTIFACT_POLICY_VERSION).toBe(
      "generated-artifact-policy-v1",
    );
    expect(evidence.GENERATED_ARTIFACT_MISSING_HINT).toBe(
      "generated-artifact-missing",
    );
    expect(evidence.GENERATED_ARTIFACT_MISSING_REASON).toBe(
      "workspace-generated-artifact-missing",
    );
  });

  it("keeps generated artifact identity strings out of downstream modules", () => {
    const sourceFiles = [
      "_lib/audit-manifest.mjs",
      "_lib/resolver-core.mjs",
      "_lib/finding-provenance.mjs",
      "_lib/generated-blind-zone-relevance.mjs",
      "_lib/ranking.mjs",
    ];

    for (const file of sourceFiles) {
      const src = readFileSync(file, "utf8");
      expect(src, file).not.toContain(
        "'workspace-generated-artifact-missing'",
      );
      expect(src, file).not.toContain(
        '"workspace-generated-artifact-missing"',
      );
      expect(src, file).not.toContain("'generated-artifact-missing'");
      expect(src, file).not.toContain('"generated-artifact-missing"');
      expect(src, file).not.toContain("'generated-artifact-policy-v1'");
      expect(src, file).not.toContain('"generated-artifact-policy-v1"');
    }
  });
});
