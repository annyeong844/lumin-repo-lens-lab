import { describe, expect, it } from "vitest";

import {
  getPublicDeepImportRisk,
  hasPublicDeepImportRisk,
} from "../_lib/package-exports.mjs";

describe("public deep-import risk", () => {
  it("D1-D8. classifies private, absent, root-only, wildcard, explicit, null, and array export risk", () => {
    expect(
      hasPublicDeepImportRisk(
        {
          private: true,
          exports: { "./*": "./src/*" },
        },
        "src/internal.ts",
      ),
    ).toBe(false);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          main: "./dist/index.js",
        },
        "src/internal.ts",
      ),
    ).toBe(true);

    expect(
      hasPublicDeepImportRisk(
        {
          type: "module",
          main: "./src/index.js",
        },
        "src/internal.ts",
      ),
    ).toBe(false);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          exports: {
            ".": {
              types: "./dist/index.d.ts",
              import: "./dist/index.mjs",
              require: "./dist/index.cjs",
            },
            "./package.json": "./package.json",
          },
        },
        "src/internal.ts",
      ),
    ).toBe(false);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          exports: { "./src/*": "./src/*" },
        },
        "src/internal.ts",
      ),
    ).toBe(true);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          exports: {
            "./features/*": {
              import: "./src/features/*.ts",
              types: "./src/features/*.d.ts",
            },
          },
        },
        "src/features/foo.ts",
      ),
    ).toBe(true);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          exports: { "./internals/foo": "./src/internals/foo.ts" },
        },
        "src/internals/foo.ts",
      ),
    ).toBe(true);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          exports: { "./internals/*": null, ".": "./dist/index.js" },
        },
        "src/internals/foo.ts",
      ),
    ).toBe(false);

    expect(
      hasPublicDeepImportRisk(
        {
          name: "pkg",
          exports: { "./x": ["./dist/x.mjs", "./dist/x.cjs"] },
        },
        "dist/x.mjs",
      ),
    ).toBe(true);
  });

  it("D9-D12. reports detailed reasons for unknown publish surface, wildcard, explicit, and no-name packages", () => {
    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          main: "./dist/index.js",
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-publish-surface-unknown",
      publishSurfaceSource: "implicit-npm-surface",
      packageName: "pkg",
      relFileFromPkgRoot: "src/internal.ts",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          exports: { "./src/*": "./src/*" },
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "wildcard-exposes-file",
      matchedExport: "./src/*",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          exports: { "./internals/foo": "./src/internals/foo.ts" },
        },
        "src/internals/foo.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "explicitly-exposed-file",
      matchedExport: "./src/internals/foo.ts",
    });

    expect(
      getPublicDeepImportRisk(
        {
          type: "module",
          main: "./src/index.js",
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: false,
      reason: "package-name-absent",
    });
  });

  it("D13-D17. applies package files inclusion and exclusion precisely", () => {
    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["dist"],
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: false,
      reason: "files-excludes-file",
      publishSurfaceSource: "package-json-files",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["src"],
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published",
      publishSurfaceSource: "package-json-files",
      matchedFilesEntry: "src",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["src/index.ts"],
        },
        "src/index.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published",
      matchedFilesEntry: "src/index.ts",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["src/index.ts"],
        },
        "src/other.ts",
      ),
    ).toMatchObject({
      risk: false,
      reason: "files-excludes-file",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: [],
        },
        "src/internal.js",
      ),
    ).toMatchObject({
      risk: false,
      reason: "files-excludes-file",
    });
  });

  it("D18-D22. keeps npm always-included files public even when files excludes them", () => {
    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          main: "src/index.js",
          files: ["dist"],
        },
        "src/index.js",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published-always-included",
      publishSurfaceSource: "npm-always-included",
      matchedAlwaysIncludedRule: "main",
      matchedPackageJsonField: "main",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["dist"],
        },
        "index.js",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published-always-included",
      matchedAlwaysIncludedRule: "default-main",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          bin: { cli: "src/cli.js" },
          files: ["dist"],
        },
        "src/cli.js",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published-always-included",
      matchedAlwaysIncludedRule: "bin",
      matchedPackageJsonField: "bin",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          directories: { bin: "bin" },
          files: ["dist"],
        },
        "bin/tool.js",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published-always-included",
      matchedAlwaysIncludedRule: "directories.bin",
      matchedPackageJsonField: "directories.bin",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: [],
        },
        "README.md",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published-always-included",
      matchedAlwaysIncludedRule: "readme",
    });
  });

  it("D23-D28. handles globstar and unsafe package files entries without clearing risk incorrectly", () => {
    const directSingleStar = getPublicDeepImportRisk(
      {
        name: "pkg",
        files: ["src/*"],
      },
      "src/a.ts",
    );
    const nestedSingleStar = getPublicDeepImportRisk(
      {
        name: "pkg",
        files: ["src/*"],
      },
      "src/nested/a.ts",
    );

    expect(directSingleStar).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published",
    });
    expect(nestedSingleStar).toMatchObject({
      risk: false,
      reason: "files-excludes-file",
    });

    const directGlobstar = getPublicDeepImportRisk(
      {
        name: "pkg",
        files: ["src/**/*.ts"],
      },
      "src/a.ts",
    );
    const nestedGlobstar = getPublicDeepImportRisk(
      {
        name: "pkg",
        files: ["src/**/*.ts"],
      },
      "src/nested/a.ts",
    );

    expect(directGlobstar).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published",
    });
    expect(nestedGlobstar).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["dist", { bad: true }],
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-files-unsupported",
      publishSurfaceSource: "package-json-files",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["dist", { bad: true }],
        },
        "dist/index.js",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-file-published",
      matchedFilesEntry: "dist",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["C:/repo/src/internal.ts"],
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-files-unsupported",
    });

    expect(
      getPublicDeepImportRisk(
        {
          name: "pkg",
          files: ["..\\src\\internal.ts"],
        },
        "src/internal.ts",
      ),
    ).toMatchObject({
      risk: true,
      reason: "exports-absent-files-unsupported",
    });
  });
});
