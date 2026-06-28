import { execFileSync } from "node:child_process";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";
import {
  buildUnusedDepsArtifact,
  collectPackageScriptToolEvidence,
  packageNameFromSpecifier,
} from "../_lib/unused-deps-artifact.mjs";

function makeFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-unused-deps-",
    packageJson: {
      name: "unused-deps-fixture",
      private: true,
      type: "module",
      scripts: {
        start: "tsx src/server.ts",
        dev: "vite --host 0.0.0.0",
        lint: "eslint .",
        wrapped: "npm run start",
      },
      dependencies: {
        react: "^19.0.0",
        "left-pad": "^1.3.0",
      },
      devDependencies: {
        tsx: "^4.0.0",
        vite: "^7.0.0",
        eslint: "^9.0.0",
        "@types/node": "^22.0.0",
      },
      peerDependencies: {
        "@storybook/react": "^8.0.0",
      },
      optionalDependencies: {
        fsevents: "^2.3.0",
      },
    },
  });

  fixture.write(
    "src/app.tsx",
    'import React from "react";\nexport const App = React.Fragment;\n',
  );
  fixture.write("src/server.ts", "export const server = true;\n");
  return fixture;
}

function runAudit() {
  const fixture = makeFixture();
  try {
    execFileSync(
      process.execPath,
      [
        "audit-repo.mjs",
        "--root",
        fixture.root,
        "--output",
        fixture.output,
        "--profile",
        "quick",
      ],
      {
        cwd: process.cwd(),
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );
    return {
      artifact: fixture.readJson("unused-deps.json", { from: "output" }),
      manifest: fixture.readJson("manifest.json", { from: "output" }),
    };
  } finally {
    fixture.cleanup();
  }
}

function depByName(pkg, name) {
  return pkg.dependencies.find((entry) => entry.name === name);
}

describe("unused deps package identity", () => {
  it("normalizes external package specifiers and rejects non-packages", () => {
    expect(packageNameFromSpecifier("react")).toBe("react");
    expect(packageNameFromSpecifier("react/jsx-runtime")).toBe("react");
    expect(packageNameFromSpecifier("@scope/pkg/sub/path")).toBe("@scope/pkg");
    expect(packageNameFromSpecifier("node:fs")).toBeNull();
    expect(packageNameFromSpecifier("./local")).toBeNull();
    expect(packageNameFromSpecifier("../local")).toBeNull();
    expect(packageNameFromSpecifier("/abs/local")).toBeNull();
    expect(packageNameFromSpecifier("C:/abs/local")).toBeNull();
    expect(packageNameFromSpecifier("https://cdn.example/pkg.js")).toBeNull();
    expect(packageNameFromSpecifier("data:text/javascript,export{}")).toBeNull();
    expect(packageNameFromSpecifier("#internal")).toBeNull();
    expect(packageNameFromSpecifier("virtual:foo")).toBeNull();
    expect(packageNameFromSpecifier("@broken")).toBeNull();
    expect(packageNameFromSpecifier("")).toBeNull();
    expect(packageNameFromSpecifier(null)).toBeNull();
  });
});

describe("unused deps package script tool evidence", () => {
  it("extracts direct package script tools without following wrappers", () => {
    const packageRecord = {
      root: "C:/repo",
      relRoot: ".",
      packageJson: {
        scripts: {
          start: "tsx src/server.ts",
          dev: "vite --host 0.0.0.0",
          lint: "pnpm eslint .",
          bunvite: "bunx vite build",
          npxlint: "npx eslint .",
          npmexec: "npm exec eslint .",
          npmstart: "npm start",
          npmtest: "npm test",
          wrapped: "npm run start",
        },
      },
    };
    const evidence = collectPackageScriptToolEvidence(packageRecord);
    const keys = evidence.map((entry) => `${entry.tool}:${entry.scriptName}`).sort();
    expect(keys).toEqual([
      "eslint:lint",
      "eslint:npmexec",
      "eslint:npxlint",
      "tsx:start",
      "vite:bunvite",
      "vite:dev",
    ]);
    expect(evidence.some((entry) => entry.scriptName === "wrapped")).toBe(false);
    expect(evidence.some((entry) => entry.scriptName === "npmstart")).toBe(false);
    expect(evidence.some((entry) => entry.scriptName === "npmtest")).toBe(false);
  });
});

describe("unused deps artifact policy", () => {
  it("classifies used, muted, and review-unused dependencies deterministically", () => {
    const artifact = buildUnusedDepsArtifact({
      root: "C:/repo",
      includeTests: true,
      exclude: [],
      packageRecords: [
        {
          root: "C:/repo",
          relRoot: ".",
          packageJson: {
            name: "app",
            scripts: { start: "tsx src/server.ts" },
            dependencies: { react: "^19.0.0", "left-pad": "^1.3.0" },
            devDependencies: { tsx: "^4.0.0", "@types/node": "^22.0.0" },
            peerDependencies: { "@storybook/react": "^8.0.0" },
            optionalDependencies: { fsevents: "^2.3.0" },
          },
        },
      ],
      symbols: {
        meta: { supports: { dependencyImportConsumers: true } },
        dependencyImportConsumers: [
          {
            file: "src/app.tsx",
            fromSpec: "react/jsx-runtime",
            depRoot: "react",
            kind: "import",
            source: "source-import",
          },
        ],
      },
    });

    expect(artifact.schemaVersion).toBe("unused-deps.v1");
    expect(artifact.policyVersion).toBe("unused-deps-review-policy-v1");
    expect(artifact.status).toBe("complete");
    expect(artifact.scanRange).toEqual({
      root: "C:/repo",
      includeTests: true,
      exclude: [],
      source: "producer-cli",
    });
    expect(artifact.summary).toMatchObject({
      packageCount: 1,
      declaredDependencyCount: 6,
      usedCount: 1,
      mutedCount: 4,
      reviewUnusedCount: 1,
    });

    const pkg = artifact.packages[0];
    expect(depByName(pkg, "react")).toMatchObject({
      status: "used",
      reason: "external-import-consumer",
    });
    expect(depByName(pkg, "left-pad")).toMatchObject({
      status: "review-unused",
      reason: "no-observed-consumer",
    });
    expect(depByName(pkg, "tsx")).toMatchObject({
      status: "muted",
      reason: "package-script-tool",
    });
    expect(depByName(pkg, "@types/node")).toMatchObject({
      status: "muted",
      reason: "ambient-types",
    });
    expect(depByName(pkg, "@storybook/react")).toMatchObject({
      status: "muted",
      reason: "peer-contract",
    });
    expect(depByName(pkg, "fsevents")).toMatchObject({
      status: "muted",
      reason: "optional-runtime",
    });
  });

  it("attributes consumers to the nearest workspace package and mutes workspace internals", () => {
    const artifact = buildUnusedDepsArtifact({
      root: "C:/repo",
      includeTests: true,
      exclude: [],
      packageRecords: [
        {
          root: "C:/repo",
          relRoot: ".",
          packageJson: {
            name: "root-app",
            dependencies: {
              react: "^19.0.0",
              "@repo/shared": "workspace:*",
            },
          },
        },
        {
          root: "C:/repo/packages/app",
          relRoot: "packages/app",
          packageJson: {
            name: "@repo/app",
            dependencies: {
              react: "^19.0.0",
              "@repo/shared": "workspace:*",
            },
          },
        },
        {
          root: "C:/repo/packages/shared",
          relRoot: "packages/shared",
          packageJson: { name: "@repo/shared" },
        },
      ],
      symbols: {
        meta: { supports: { dependencyImportConsumers: true } },
        dependencyImportConsumers: [
          {
            file: "packages/app/src/App.tsx",
            fromSpec: "react",
            depRoot: "react",
            kind: "import",
            source: "source-import",
          },
        ],
      },
    });

    const rootPkg = artifact.packages.find((entry) => entry.packageDir === ".");
    const appPkg = artifact.packages.find(
      (entry) => entry.packageDir === "packages/app",
    );
    expect(depByName(rootPkg, "react")).toMatchObject({
      status: "review-unused",
      reason: "no-observed-consumer",
    });
    expect(depByName(appPkg, "react").status).toBe("used");
    expect(depByName(appPkg, "@repo/shared")).toMatchObject({
      status: "muted",
      reason: "workspace-internal",
    });
  });

  it("writes unavailable artifact when dependency import consumer support is absent", () => {
    const artifact = buildUnusedDepsArtifact({
      root: "C:/repo",
      includeTests: true,
      exclude: [],
      packageRecords: [
        {
          root: "C:/repo",
          relRoot: ".",
          packageJson: { name: "app", dependencies: { react: "^19.0.0" } },
        },
      ],
      symbols: {
        meta: { supports: {} },
        dependencyImportConsumers: [],
      },
    });

    expect(artifact.status).toBe("unavailable");
    expect(artifact.reason).toBe("input-artifact-missing");
    expect(artifact.inputs.symbols.supportsDependencyImportConsumers).toBe(false);
    expect(artifact.summary.declaredDependencyCount).toBe(0);
    expect(artifact.packages).toEqual([]);
  });
});

describe("unused deps audit pipeline", () => {
  it("emits unused-deps.json and records it as a produced artifact", () => {
    const { artifact, manifest } = runAudit();

    expect(artifact.schemaVersion).toBe("unused-deps.v1");
    expect(artifact.status).toBe("complete");
    expect(manifest.commandsRun).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          step: "build-unused-deps.mjs",
          status: "ok",
        }),
      ]),
    );
    expect(manifest.artifactsProduced).toContain("unused-deps.json");
    expect(manifest.unusedDependencies).toMatchObject({
      artifact: "unused-deps.json",
      schemaVersion: "unused-deps.v1",
      policyVersion: "unused-deps-review-policy-v1",
      status: "complete",
      reviewUnusedCount: 1,
      mutedCount: 6,
    });
    expect(manifest.unusedDependencies?.topReviewUnused).toEqual([
      {
        packageDir: ".",
        manifestPath: "package.json",
        name: "left-pad",
        field: "dependencies",
        reason: "no-observed-consumer",
        confidence: "review",
      },
    ]);

    const pkg = artifact.packages[0];
    expect(depByName(pkg, "react").status).toBe("used");
    expect(depByName(pkg, "left-pad").status).toBe("review-unused");
    expect(depByName(pkg, "tsx").status).toBe("muted");
    expect(depByName(pkg, "vite").status).toBe("muted");
    expect(depByName(pkg, "eslint").status).toBe("muted");
  }, 15_000);
});
