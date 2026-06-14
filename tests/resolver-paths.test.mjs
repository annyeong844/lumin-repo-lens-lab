import path from "node:path";

import { describe, expect, it } from "vitest";

import { buildAliasMap } from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import {
  explainUnresolvedSpecifier,
  isGeneratedVirtualResolution,
  isNonSourceAssetResolution,
  isResolvedFile,
  makeResolver,
} from "../_lib/resolver-core.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function createResolverPathFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-resolver-paths-",
    packageJson: {
      name: "fx",
      type: "module",
      scripts: {
        tailwind:
          "tailwindcss --input ./src/styles.css --output ./src/tailwind.generated.css",
      },
    },
  });

  fixture.write("src/mod.cjs", "module.exports = 1;\n");
  fixture.write("src/view.jsx", "export const V = () => 1;\n");
  fixture.write("src/util.mts", "export const U = 1;\n");
  fixture.write("src/conf.cts", "export const C = 1;\n");
  fixture.write("src/types.d.ts", "export interface T {}\n");
  fixture.write("src/embed.css", ".embed { color: red; }\n");
  fixture.write("src/embed-cache.css", ".embed-cache { color: blue; }\n");
  fixture.write("dir/index.js", "export const I = 1;\n");
  fixture.write("cjs-dir/index.cjs", "module.exports = 1;\n");
  fixture.write("decl-dir/index.d.ts", "export interface D {}\n");
  fixture.write("src/consumer.ts", "export const x = 1;\n");

  return fixture;
}

function makeFixtureResolver(fixture, aliasMap = null) {
  const mode = detectRepoMode(fixture.root);
  return makeResolver(
    fixture.root,
    aliasMap ?? buildAliasMap(fixture.root, mode),
  );
}

describe("resolver path lookup edge cases", () => {
  it("resolves extensionless files and directory indexes across supported JS/TS extensions", () => {
    const fixture = createResolverPathFixture();
    try {
      const resolve = makeFixtureResolver(fixture);
      const from = fixture.path("src/consumer.ts");

      expect(resolve(from, "./mod")).toBe(fixture.path("src/mod.cjs"));
      expect(resolve(from, "./view")).toBe(fixture.path("src/view.jsx"));
      expect(resolve(from, "./util")).toBe(fixture.path("src/util.mts"));
      expect(resolve(from, "./conf")).toBe(fixture.path("src/conf.cts"));
      expect(resolve(from, "./types")).toBe(fixture.path("src/types.d.ts"));
      expect(resolve(from, "../dir")).toBe(fixture.path("dir/index.js"));
      expect(resolve(from, "../cjs-dir")).toBe(
        fixture.path("cjs-dir/index.cjs"),
      );
      expect(resolve(from, "../decl-dir")).toBe(
        fixture.path("decl-dir/index.d.ts"),
      );
      expect(resolve(from, "./mod.cjs")).toBe(fixture.path("src/mod.cjs"));
      expect(resolve(from, "./view.jsx")).toBe(fixture.path("src/view.jsx"));
      expect(resolve(from, "./nonexistent")).toBeNull();
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps resource-query assets out of source-file resolution and explains generated asset misses", () => {
    const fixture = createResolverPathFixture();
    try {
      const mode = detectRepoMode(fixture.root);
      const aliasMap = buildAliasMap(fixture.root, mode);
      const resolve = makeResolver(fixture.root, aliasMap);
      const from = fixture.path("src/consumer.ts");

      const inlineAsset = resolve(from, "./embed.css?inline");
      expect(isNonSourceAssetResolution(inlineAsset)).toBe(true);
      expect(isResolvedFile(inlineAsset)).toBe(false);

      const generatedAssetExplanation = explainUnresolvedSpecifier(
        fixture.root,
        aliasMap,
        from,
        "./tailwind.generated.css?inline",
      );
      expect(generatedAssetExplanation?.reason).toBe(
        "workspace-generated-artifact-missing",
      );
      expect(generatedAssetExplanation?.targetCandidates?.[0]).toBe(
        "src/tailwind.generated.css",
      );
      expect(generatedAssetExplanation?.generatedArtifact?.evidence).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            kind: "script-output-path",
            field: "scripts.tailwind",
            matched: "src/tailwind.generated.css",
          }),
        ]),
      );
    } finally {
      fixture.cleanup();
    }
  });

  it("discriminates real resolved paths from external, unresolved, null, undefined, and non-string sentinels", () => {
    expect(isResolvedFile(path.resolve("abs/path/file.ts"))).toBe(true);
    expect(isResolvedFile("EXTERNAL")).toBe(false);
    expect(isResolvedFile("UNRESOLVED_INTERNAL")).toBe(false);
    expect(isResolvedFile(null)).toBe(false);
    expect(isResolvedFile(undefined)).toBe(false);
    expect(isResolvedFile(123)).toBe(false);
  });

  it("memoizes null, asset, unresolved-internal, and generated-virtual results without changing their identity", () => {
    const fixture = createResolverPathFixture();
    try {
      const resolve = makeFixtureResolver(fixture);
      const from = fixture.path("src/consumer.ts");
      const memoStart = resolve.memoStats();

      expect(resolve(from, "./memoized-missing")).toBeNull();
      expect(resolve(from, "./memoized-missing")).toBeNull();
      let memoAfter = resolve.memoStats();
      expect(memoAfter.hits - memoStart.hits).toBe(1);
      expect(memoAfter.misses - memoStart.misses).toBe(1);

      const memoBeforeAsset = resolve.memoStats();
      expect(
        isNonSourceAssetResolution(resolve(from, "./embed-cache.css?inline")),
      ).toBe(true);
      expect(
        isNonSourceAssetResolution(resolve(from, "./embed-cache.css?inline")),
      ).toBe(true);
      memoAfter = resolve.memoStats();
      expect(memoAfter.hits - memoBeforeAsset.hits).toBe(1);
      expect(memoAfter.misses - memoBeforeAsset.misses).toBe(1);

      const unresolvedAliasMap = new Map([
        [
          "@missing/internal",
          {
            type: "exact",
            path: fixture.path("src/missing-internal.ts"),
            source: "test",
          },
        ],
      ]);
      const resolveUnresolvedAlias = makeFixtureResolver(
        fixture,
        unresolvedAliasMap,
      );
      const unresolvedMemoStart = resolveUnresolvedAlias.memoStats();
      expect(resolveUnresolvedAlias(from, "@missing/internal")).toBe(
        "UNRESOLVED_INTERNAL",
      );
      expect(resolveUnresolvedAlias(from, "@missing/internal")).toBe(
        "UNRESOLVED_INTERNAL",
      );
      const unresolvedMemoAfter = resolveUnresolvedAlias.memoStats();
      expect(unresolvedMemoAfter.hits - unresolvedMemoStart.hits).toBe(1);
      expect(unresolvedMemoAfter.misses - unresolvedMemoStart.misses).toBe(1);

      const virtualAliasMap = new Map([
        [
          "@virtual/*",
          {
            type: "wildcard",
            matchPrefix: "@virtual/",
            matchSuffix: "",
            targetPattern: "./generated/*",
            pkgDir: fixture.root,
            pkgName: "@virtual",
            source: "test",
            generatedVirtualSurfaces: [
              {
                id: "generated-virtual:test:enums",
                source: "generated-virtual",
                virtual: true,
                runtimeEquivalence: false,
                targetSubpath: "enums",
                exports: [{ name: "GeneratedEnum", spaces: ["value", "type"] }],
              },
            ],
          },
        ],
      ]);
      const resolveVirtual = makeFixtureResolver(fixture, virtualAliasMap);
      const virtualMemoStart = resolveVirtual.memoStats();
      const virtualFirst = resolveVirtual(from, "@virtual/enums");
      const virtualSecond = resolveVirtual(from, "@virtual/enums");
      const virtualMemoAfter = resolveVirtual.memoStats();

      expect(isGeneratedVirtualResolution(virtualFirst)).toBe(true);
      expect(virtualSecond).toBe(virtualFirst);
      expect(virtualMemoAfter.hits - virtualMemoStart.hits).toBe(1);
      expect(virtualMemoAfter.misses - virtualMemoStart.misses).toBe(1);
    } finally {
      fixture.cleanup();
    }
  });

  it("shares scoped baseUrl probe-cache hits across importer files without changing resolved and no-match results", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-baseurl-probe-cache-",
      packageJson: {
        name: "fx-baseurl-probe-cache",
        type: "module",
      },
    });
    try {
      fixture.writeJson("tsconfig.json", {
        compilerOptions: { baseUrl: "." },
        include: ["app/**/*.ts"],
      });
      fixture.write(
        "app/_types.ts",
        "export interface PageProps { slug: string }\n",
      );
      fixture.write(
        "app/a.ts",
        "import type { PageProps } from 'app/_types';\nexport type A = PageProps;\n",
      );
      fixture.write(
        "app/b.ts",
        "import type { PageProps } from 'app/_types';\nexport type B = PageProps;\n",
      );

      const resolve = makeFixtureResolver(fixture);
      const aFile = fixture.path("app/a.ts");
      const bFile = fixture.path("app/b.ts");
      const stageStart = resolve.stageStats();

      expect(resolve(aFile, "app/_types")).toBe(fixture.path("app/_types.ts"));
      expect(resolve(bFile, "app/_types")).toBe(fixture.path("app/_types.ts"));
      expect(
        resolve.stageStats().scopedBaseUrl.cacheHits -
          stageStart.scopedBaseUrl.cacheHits,
      ).toBeGreaterThanOrEqual(1);
      expect(
        resolve.stageStats().scopedBaseUrl.cacheMisses -
          stageStart.scopedBaseUrl.cacheMisses,
      ).toBe(1);

      const externalStageStart = resolve.stageStats();
      expect(resolve(aFile, "react")).toBe("EXTERNAL");
      expect(resolve(bFile, "react")).toBe("EXTERNAL");
      expect(
        resolve.stageStats().scopedBaseUrl.cacheHits -
          externalStageStart.scopedBaseUrl.cacheHits,
      ).toBeGreaterThanOrEqual(1);
      expect(
        resolve.stageStats().scopedBaseUrl.cacheMisses -
          externalStageStart.scopedBaseUrl.cacheMisses,
      ).toBe(1);
    } finally {
      fixture.cleanup();
    }
  });

  it("shares wildcard alias stage-cache hits across importer files for resolved, no-match, unresolved, and generated virtual outcomes", () => {
    const fixture = createResolverPathFixture();
    try {
      fixture.write("src/wild-target.ts", "export const WildTarget = 1;\n");
      const from = fixture.path("src/consumer.ts");
      const siblingFrom = fixture.path("src/other-consumer.ts");
      const wildcardAliasMap = new Map([
        [
          "@wild/*",
          {
            type: "wildcard",
            matchPrefix: "@wild/",
            matchSuffix: "",
            targetPattern: "./src/*",
            pkgDir: fixture.root,
            pkgName: "@wild",
            source: "test",
          },
        ],
      ]);
      const resolveWildcardAlias = makeFixtureResolver(
        fixture,
        wildcardAliasMap,
      );

      const resolvedStart = resolveWildcardAlias.stageStats();
      expect(resolveWildcardAlias(from, "@wild/wild-target")).toBe(
        fixture.path("src/wild-target.ts"),
      );
      expect(resolveWildcardAlias(siblingFrom, "@wild/wild-target")).toBe(
        fixture.path("src/wild-target.ts"),
      );
      let stageAfter = resolveWildcardAlias.stageStats();
      expect(
        stageAfter.wildcardAlias.cacheHits -
          resolvedStart.wildcardAlias.cacheHits,
      ).toBeGreaterThanOrEqual(1);
      expect(
        stageAfter.wildcardAlias.cacheMisses -
          resolvedStart.wildcardAlias.cacheMisses,
      ).toBe(1);

      const noMatchStart = resolveWildcardAlias.stageStats();
      expect(resolveWildcardAlias(from, "react")).toBe("EXTERNAL");
      expect(resolveWildcardAlias(siblingFrom, "react")).toBe("EXTERNAL");
      stageAfter = resolveWildcardAlias.stageStats();
      expect(
        stageAfter.wildcardAlias.cacheHits -
          noMatchStart.wildcardAlias.cacheHits,
      ).toBeGreaterThanOrEqual(1);
      expect(
        stageAfter.wildcardAlias.cacheMisses -
          noMatchStart.wildcardAlias.cacheMisses,
      ).toBe(1);

      const unresolvedStart = resolveWildcardAlias.stageStats();
      expect(resolveWildcardAlias(from, "@wild/missing")).toBe(
        "UNRESOLVED_INTERNAL",
      );
      expect(resolveWildcardAlias(siblingFrom, "@wild/missing")).toBe(
        "UNRESOLVED_INTERNAL",
      );
      stageAfter = resolveWildcardAlias.stageStats();
      expect(
        stageAfter.wildcardAlias.cacheHits -
          unresolvedStart.wildcardAlias.cacheHits,
      ).toBeGreaterThanOrEqual(1);

      const virtualAliasMap = new Map([
        [
          "@virtual/*",
          {
            type: "wildcard",
            matchPrefix: "@virtual/",
            matchSuffix: "",
            targetPattern: "./generated/*",
            pkgDir: fixture.root,
            pkgName: "@virtual",
            source: "test",
            generatedVirtualSurfaces: [
              {
                id: "generated-virtual:test:enums",
                source: "generated-virtual",
                virtual: true,
                runtimeEquivalence: false,
                targetSubpath: "enums",
                exports: [{ name: "GeneratedEnum", spaces: ["value", "type"] }],
              },
            ],
          },
        ],
      ]);
      const resolveVirtualStage = makeFixtureResolver(fixture, virtualAliasMap);
      const virtualStageStart = resolveVirtualStage.stageStats();
      const virtualFirst = resolveVirtualStage(from, "@virtual/enums");
      const virtualSecond = resolveVirtualStage(
        fixture.path("src/virtual-consumer.ts"),
        "@virtual/enums",
      );
      const virtualStageAfter = resolveVirtualStage.stageStats();
      expect(isGeneratedVirtualResolution(virtualFirst)).toBe(true);
      expect(virtualSecond).toBe(virtualFirst);
      expect(Object.isFrozen(virtualSecond)).toBe(true);
      expect(() => {
        virtualSecond.aliasSource = "mutated";
      }).toThrow();
      expect(
        virtualStageAfter.wildcardAlias.cacheHits -
          virtualStageStart.wildcardAlias.cacheHits,
      ).toBeGreaterThanOrEqual(1);
      expect(
        virtualStageAfter.wildcardAlias.cacheMisses -
          virtualStageStart.wildcardAlias.cacheMisses,
      ).toBe(1);
    } finally {
      fixture.cleanup();
    }
  });
});
