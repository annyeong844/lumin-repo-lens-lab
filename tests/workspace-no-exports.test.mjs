import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { buildAliasMap } from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function runSymbolGraph(fixture) {
  execFileSync(
    process.execPath,
    [
      "build-symbol-graph.mjs",
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    {
      cwd: REPO_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  return fixture.readJson("symbols.json", { from: "output" });
}

function deadSymbols(symbols) {
  return new Set((symbols.deadProdList ?? []).map((entry) => entry.symbol));
}

function deadIdentities(symbols) {
  return new Set(
    (symbols.deadProdList ?? []).map(
      (entry) => `${entry.file}::${entry.symbol}`,
    ),
  );
}

function buildWorkspace(fixture, variant) {
  fixture.mkdir("apps/blog/app");
  fixture.mkdir("packages/libs");
  fixture.writeJson("package.json", {
    name: "root",
    type: "module",
    workspaces: ["apps/*", "packages/*"],
  });

  const pkgShape =
    variant === "main-only"
      ? { name: "@scope/libs", type: "module", main: "./getPost.ts" }
      : variant === "no-main-no-exports"
        ? { name: "@scope/libs", type: "module" }
        : {
            name: "@scope/libs",
            type: "module",
            exports: { ".": "./getPost.ts" },
          };

  fixture.writeJson("packages/libs/package.json", pkgShape);
  fixture.write(
    "packages/libs/getPost.ts",
    [
      "export function getPostBySlug(slug: string) { return { slug }; }",
      "export function getAllCategories() { return []; }",
      "export function getPostsByCategory(c: string) { return []; }",
      "export function unusedInternal() { return 99; }",
      "",
    ].join("\n"),
  );
  fixture.write(
    "packages/libs/getSeries.ts",
    [
      "export function getAllSeries() { return []; }",
      "export function getSeries(n: string) { return { n }; }",
      "",
    ].join("\n"),
  );
  fixture.write(
    "packages/libs/inputs/location.input.ts",
    [
      "export function makeLocationInput() { return { location: 'office' }; }",
      "export function unusedLocationInput() { return { location: 'unused' }; }",
      "",
    ].join("\n"),
  );
  fixture.writeJson("apps/blog/package.json", {
    name: "blog",
    type: "module",
    dependencies: { "@scope/libs": "workspace:*" },
  });
  fixture.write(
    "apps/blog/app/page.tsx",
    [
      "import { getPostBySlug, getAllCategories } from '@scope/libs/getPost';",
      "import { getAllSeries } from '@scope/libs/getSeries';",
      "import { makeLocationInput } from '@scope/libs/inputs/location.input';",
      "export function Page() {",
      "  getPostBySlug('x');",
      "  getAllCategories();",
      "  getAllSeries();",
      "  makeLocationInput();",
      "  return null;",
      "}",
      "",
    ].join("\n"),
  );
}

describe("workspace packages without exports", () => {
  it("F1-F6c. resolves legacy workspace subpaths precisely for main-only packages", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-fp38-main-",
      outputDirName: "artifacts",
    });
    try {
      buildWorkspace(fixture, "main-only");
      const symbols = runSymbolGraph(fixture);
      const dead = deadSymbols(symbols);

      expect(dead).not.toContain("getPostBySlug");
      expect(dead).not.toContain("getAllCategories");
      expect(dead).not.toContain("getAllSeries");
      expect(symbols.uses?.resolvedInternal ?? 0).toBeGreaterThanOrEqual(3);
      expect(symbols.uses?.external ?? 0).toBe(0);
      expect(dead).toContain("getPostsByCategory");
      expect(dead).toContain("unusedInternal");
      expect(
        symbols.fanInByIdentity?.[
          "packages/libs/inputs/location.input.ts::makeLocationInput"
        ],
      ).toBe(1);
      expect(dead).not.toContain("makeLocationInput");
      expect(dead).toContain("unusedLocationInput");
    } finally {
      fixture.cleanup();
    }
  });

  it("F7-F8. keeps explicit exports behavior while adding only intended legacy subpath wildcards", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-fp38-exports-",
      outputDirName: "artifacts",
    });
    try {
      buildWorkspace(fixture, "exports");
      const symbols = runSymbolGraph(fixture);
      const dead = deadSymbols(symbols);

      expect(dead).not.toContain("getPostBySlug");

      const aliasMap = buildAliasMap(
        fixture.root,
        detectRepoMode(fixture.root),
      );
      const legacyKeys = [...aliasMap.keys()].filter((key) =>
        key.includes("__LEGACY_SUBPATH__"),
      );
      expect(
        legacyKeys.some((key) => key.startsWith("@scope/libs/")),
        JSON.stringify(legacyKeys),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("F9-F13. maps dist output targets back to package-root authored source", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-fp38-dist-root-",
      outputDirName: "artifacts",
    });
    try {
      fixture.mkdir("apps/web/app");
      fixture.mkdir("packages/platform-types");
      fixture.writeJson("package.json", {
        name: "root",
        type: "module",
        workspaces: ["apps/*", "packages/*"],
      });
      fixture.writeJson("packages/platform-types/package.json", {
        name: "@scope/platform-types",
        type: "module",
        main: "./dist/index.js",
        types: "./dist/index.d.ts",
        exports: {
          ".": {
            import: "./dist/index.js",
            types: "./dist/index.d.ts",
          },
          "./*": "./dist/*.js",
        },
      });
      fixture.write(
        "packages/platform-types/index.ts",
        [
          "export interface PlatformUser { id: string }",
          "export function makePlatformUser(): PlatformUser { return { id: 'u' }; }",
          "export function unusedPlatformRoot() { return 1; }",
          "",
        ].join("\n"),
      );
      fixture.write(
        "packages/platform-types/api.ts",
        [
          "export interface ApiResponse { ok: boolean }",
          "export function makeApiResponse(): ApiResponse { return { ok: true }; }",
          "export function unusedApiResponse() { return 2; }",
          "",
        ].join("\n"),
      );
      fixture.write(
        "packages/platform-types/bookings/2024-08-13/inputs/location.input.ts",
        [
          "export interface LocationInput { location: string }",
          "export function makeLocationInput(): LocationInput { return { location: 'office' }; }",
          "",
        ].join("\n"),
      );
      fixture.writeJson("apps/web/package.json", {
        name: "web",
        type: "module",
        dependencies: { "@scope/platform-types": "workspace:*" },
      });
      fixture.write(
        "apps/web/app/page.tsx",
        [
          "import { makePlatformUser } from '@scope/platform-types';",
          "import { makeApiResponse } from '@scope/platform-types/api';",
          "import { makeLocationInput } from '@scope/platform-types/bookings/2024-08-13/inputs/location.input';",
          "export function Page() {",
          "  makePlatformUser();",
          "  makeApiResponse();",
          "  makeLocationInput();",
          "  return null;",
          "}",
          "",
        ].join("\n"),
      );

      const symbols = runSymbolGraph(fixture);
      const dead = deadIdentities(symbols);

      expect(
        symbols.fanInByIdentity?.[
          "packages/platform-types/index.ts::makePlatformUser"
        ],
      ).toBe(1);
      expect(
        dead.has("packages/platform-types/index.ts::makePlatformUser"),
      ).toBe(false);
      expect(
        symbols.fanInByIdentity?.[
          "packages/platform-types/api.ts::makeApiResponse"
        ],
      ).toBe(1);
      expect(dead.has("packages/platform-types/api.ts::makeApiResponse")).toBe(
        false,
      );
      expect(symbols.uses?.external ?? 0).toBe(0);
      expect(symbols.uses?.unresolvedInternal ?? 0).toBe(0);
      expect(dead).toContain(
        "packages/platform-types/api.ts::unusedApiResponse",
      );
      expect(
        symbols.fanInByIdentity?.[
          "packages/platform-types/bookings/2024-08-13/inputs/location.input.ts::makeLocationInput"
        ],
      ).toBe(1);
      expect(
        dead.has(
          "packages/platform-types/bookings/2024-08-13/inputs/location.input.ts::makeLocationInput",
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });

  it("F14-F16. maps declarationDir subpaths back to source files", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-fp38-declaration-dir-",
      outputDirName: "artifacts",
    });
    try {
      fixture.mkdir("apps/web/app");
      fixture.mkdir("packages/trpc/server/routers");
      fixture.writeJson("package.json", {
        name: "root",
        type: "module",
        workspaces: ["apps/*", "packages/*"],
      });
      fixture.writeJson("packages/trpc/package.json", {
        name: "@scope/trpc",
        type: "module",
        main: "index.ts",
      });
      fixture.writeJson("packages/trpc/tsconfig.json", {
        compilerOptions: {
          declaration: true,
          emitDeclarationOnly: true,
          declarationDir: "types/server",
        },
        include: ["./server"],
      });
      fixture.write("packages/trpc/index.ts", "export const root = 1;\n");
      fixture.write(
        "packages/trpc/server/createContext.ts",
        [
          "export interface TRPCContext { userId: string }",
          "export function createContext(): TRPCContext { return { userId: 'u' }; }",
          "export function unusedContextHelper() { return 1; }",
          "",
        ].join("\n"),
      );
      fixture.write(
        "packages/trpc/server/routers/_app.ts",
        [
          "export interface AppRouter { routes: string[] }",
          "export function makeAppRouter(): AppRouter { return { routes: [] }; }",
          "export function unusedRouterHelper() { return 2; }",
          "",
        ].join("\n"),
      );
      fixture.writeJson("apps/web/package.json", {
        name: "web",
        type: "module",
        dependencies: { "@scope/trpc": "workspace:*" },
      });
      fixture.write(
        "apps/web/app/page.ts",
        [
          "import type { TRPCContext } from '@scope/trpc/types/server/createContext';",
          "import type { AppRouter } from '@scope/trpc/types/server/routers/_app';",
          "export function Page(_ctx: TRPCContext, _router: AppRouter) { return null; }",
          "",
        ].join("\n"),
      );

      const symbols = runSymbolGraph(fixture);
      const dead = deadIdentities(symbols);
      const unresolved = symbols.unresolvedInternalSpecifierRecords ?? [];

      expect(
        symbols.fanInByIdentity?.[
          "packages/trpc/server/createContext.ts::TRPCContext"
        ],
      ).toBe(1);
      expect(
        dead.has("packages/trpc/server/createContext.ts::TRPCContext"),
      ).toBe(false);
      expect(
        symbols.fanInByIdentity?.[
          "packages/trpc/server/routers/_app.ts::AppRouter"
        ],
      ).toBe(1);
      expect(dead.has("packages/trpc/server/routers/_app.ts::AppRouter")).toBe(
        false,
      );
      expect(
        unresolved.some((record) =>
          record.specifier?.startsWith("@scope/trpc/types/server/"),
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });

  it("F17-F22. resolves source-direct bare package entries without fake-resolving missing generated typings", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-fp38-bare-source-entry-",
      outputDirName: "artifacts",
    });
    try {
      fixture.mkdir("apps/web/src");
      fixture.mkdir("packages/typed-entry");
      fixture.mkdir("packages/index-entry");
      fixture.mkdir("packages/generated-entry");
      fixture.writeJson("package.json", {
        name: "root",
        type: "module",
        workspaces: ["apps/*", "packages/*"],
      });
      fixture.writeJson("packages/typed-entry/package.json", {
        name: "@scope/typed-entry",
        type: "module",
        types: "./index.ts",
      });
      fixture.write(
        "packages/typed-entry/index.ts",
        [
          "export interface TypedEntry { id: string }",
          "export function makeTypedEntry(): TypedEntry { return { id: 'typed' }; }",
          "export function unusedTypedEntry() { return 1; }",
          "",
        ].join("\n"),
      );
      fixture.writeJson("packages/index-entry/package.json", {
        name: "@scope/index-entry",
        type: "module",
      });
      fixture.write(
        "packages/index-entry/index.ts",
        [
          "export function makeIndexEntry() { return { id: 'index' }; }",
          "export function unusedIndexEntry() { return 2; }",
          "",
        ].join("\n"),
      );
      fixture.writeJson("packages/generated-entry/package.json", {
        name: "@scope/generated-entry",
        type: "module",
        typings: "./dist/index.d.ts",
        files: ["dist"],
        scripts: { build: "vite build" },
      });
      fixture.writeJson("apps/web/package.json", {
        name: "web",
        type: "module",
        dependencies: {
          "@scope/typed-entry": "workspace:*",
          "@scope/index-entry": "workspace:*",
          "@scope/generated-entry": "workspace:*",
        },
      });
      fixture.write(
        "apps/web/src/page.ts",
        [
          "import type { TypedEntry } from '@scope/typed-entry';",
          "import { makeTypedEntry } from '@scope/typed-entry';",
          "import { makeIndexEntry } from '@scope/index-entry';",
          "import type { GeneratedEntry } from '@scope/generated-entry';",
          "export function page(): TypedEntry {",
          "  makeIndexEntry();",
          "  return makeTypedEntry() as GeneratedEntry & TypedEntry;",
          "}",
          "",
        ].join("\n"),
      );

      const symbols = runSymbolGraph(fixture);
      const dead = deadIdentities(symbols);
      const unresolved = symbols.unresolvedInternalSpecifierRecords ?? [];

      expect(
        symbols.fanInByIdentity?.["packages/typed-entry/index.ts::TypedEntry"],
      ).toBe(1);
      expect(dead.has("packages/typed-entry/index.ts::TypedEntry")).toBe(false);
      expect(
        symbols.fanInByIdentity?.[
          "packages/typed-entry/index.ts::makeTypedEntry"
        ],
      ).toBe(1);
      expect(dead.has("packages/typed-entry/index.ts::makeTypedEntry")).toBe(
        false,
      );
      expect(
        symbols.fanInByIdentity?.[
          "packages/index-entry/index.ts::makeIndexEntry"
        ],
      ).toBe(1);
      expect(dead.has("packages/index-entry/index.ts::makeIndexEntry")).toBe(
        false,
      );
      expect(symbols.uses?.external ?? 0).toBe(0);
      expect(
        unresolved.some(
          (record) =>
            record.specifier === "@scope/generated-entry" &&
            record.reason === "workspace-generated-artifact-missing" &&
            record.resolverStage === "exact-alias" &&
            record.generatedArtifact?.generatorFamily === "build-output",
        ),
      ).toBe(true);
      expect(dead).toContain("packages/typed-entry/index.ts::unusedTypedEntry");
      expect(dead).toContain("packages/index-entry/index.ts::unusedIndexEntry");
    } finally {
      fixture.cleanup();
    }
  });
});
