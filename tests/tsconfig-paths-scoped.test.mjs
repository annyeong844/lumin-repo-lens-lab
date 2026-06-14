import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { buildAliasMap } from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { makeResolver } from "../_lib/resolver-core.mjs";
import {
  discoverScopedTsconfigPaths,
  discoverScopedTsconfigResolution,
} from "../_lib/tsconfig-paths.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function runSymbolGraph(fixture) {
  execFileSync(
    "node",
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
  return fixture.readJson("symbols.json", { from: "output" });
}

function createResolver(fixture) {
  const repoMode = detectRepoMode(fixture.root);
  return makeResolver(fixture.root, buildAliasMap(fixture.root, repoMode));
}

function withFixture(prefix, fn) {
  const fixture = createTempRepoFixture({
    prefix,
    packageJson: { name: "root", type: "module", workspaces: ["apps/*"] },
  });
  try {
    return fn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function slash(value) {
  return String(value).replace(/\\/g, "/");
}

describe("scoped tsconfig path and baseUrl resolution", () => {
  it("resolves identical @/* aliases to different app-local files by importer scope", () => {
    withFixture("fx-vitest-tsconfig-scoped-", (fixture) => {
      fixture.writeJson("apps/agents/package.json", {
        name: "agents",
        type: "module",
      });
      fixture.write(
        "apps/agents/tsconfig.json",
        `{
  "$schema": "https://json.schemastore.org/tsconfig",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  },
  "include": ["**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules", "**/*.test.ts"]
}
`,
      );
      fixture.write(
        "apps/agents/components/auth-control.tsx",
        "export function AuthControl() { return null; }\n",
      );
      fixture.write(
        "apps/agents/app/chat-top-bar.tsx",
        "import { AuthControl } from '@/components/auth-control';\n" +
          "export function ChatTopBar() { return AuthControl(); }\n",
      );

      fixture.writeJson("apps/admin/package.json", {
        name: "admin",
        type: "module",
      });
      fixture.write(
        "apps/admin/tsconfig.json",
        `{
  "$schema": "https://json.schemastore.org/tsconfig",
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@/*": ["./*"] }
  },
  "include": ["**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules", "**/*.test.ts"]
}
`,
      );
      fixture.write(
        "apps/admin/components/auth-control.tsx",
        'export function AuthControl() { return "admin"; }\n',
      );
      fixture.write(
        "apps/admin/app/sidebar.tsx",
        "import { AuthControl } from '@/components/auth-control';\n" +
          "export function Sidebar() { return AuthControl(); }\n",
      );

      const symbols = runSymbolGraph(fixture);
      const resolve = createResolver(fixture);
      const agentsResolved = resolve(
        fixture.path("apps/agents/app/chat-top-bar.tsx"),
        "@/components/auth-control",
      );
      const adminResolved = resolve(
        fixture.path("apps/admin/app/sidebar.tsx"),
        "@/components/auth-control",
      );

      expect(slash(agentsResolved)).toContain(
        "apps/agents/components/auth-control.tsx",
      );
      expect(slash(adminResolved)).toContain(
        "apps/admin/components/auth-control.tsx",
      );
      expect(agentsResolved).not.toBe(adminResolved);
      expect(
        new Set((symbols.deadProdList ?? []).map((d) => d.symbol)),
      ).not.toContain("AuthControl");
      expect(symbols.uses?.unresolvedInternal).toBe(0);
      expect(symbols.uses?.unresolvedInternalRatio).toBe(0);
      expect(
        resolve(
          fixture.path("apps/agents/app/chat-top-bar.tsx"),
          "@/components/does-not-exist",
        ),
      ).toBe("UNRESOLVED_INTERNAL");
      expect(
        resolve(fixture.path("apps/agents/app/chat-top-bar.tsx"), "react"),
      ).toBe("EXTERNAL");

      const statsBeforeCache = resolve.stageStats().scopedTsconfig;
      expect(
        resolve(
          fixture.path("apps/agents/app/chat-side-panel.tsx"),
          "@/components/auth-control",
        ),
      ).toBe(agentsResolved);
      const statsAfterCache = resolve.stageStats().scopedTsconfig;
      expect(statsAfterCache.cacheHits).toBeGreaterThan(
        statsBeforeCache.cacheHits,
      );
      expect(statsAfterCache.probeHits).toBe(statsBeforeCache.probeHits);
    });
  });

  it("parses realistic JSONC tsconfigs and hoisted extends with TypeScript path replacement semantics", () => {
    withFixture("fx-vitest-tsconfig-extends-", (fixture) => {
      fixture.write(
        "node_modules/@shared/tsconfig/package.json",
        '{"name":"@shared/tsconfig"}\n',
      );
      fixture.writeJson("node_modules/@shared/tsconfig/base.json", {
        compilerOptions: { paths: { "@shared/*": ["./*"] } },
      });
      fixture.writeJson("apps/agents/package.json", { name: "agents" });
      fixture.writeJson("apps/agents/tsconfig.json", {
        extends: "@shared/tsconfig/base.json",
        compilerOptions: {
          baseUrl: ".",
          paths: { "@/*": ["./*"] },
        },
      });

      let entries = discoverScopedTsconfigPaths(fixture.root);
      let agentsEntries = entries.filter((entry) =>
        slash(entry.configPath).includes("apps/agents/tsconfig.json"),
      );

      expect(agentsEntries.some((entry) => entry.key === "@/*")).toBe(true);
      expect(agentsEntries.some((entry) => entry.key === "@shared/*")).toBe(
        false,
      );

      fixture.writeJson("apps/agents/tsconfig.json", {
        extends: "@shared/tsconfig/base.json",
        compilerOptions: { baseUrl: "." },
      });
      entries = discoverScopedTsconfigPaths(fixture.root);
      agentsEntries = entries.filter((entry) =>
        slash(entry.configPath).includes("apps/agents/tsconfig.json"),
      );

      expect(agentsEntries.some((entry) => entry.key === "@shared/*")).toBe(
        true,
      );
    });
  });

  it("resolves baseUrl-only type and value imports in the correct identity spaces", () => {
    withFixture("fx-vitest-baseurl-only-", (fixture) => {
      fixture.writeJson("apps/web/package.json", {
        name: "web",
        type: "module",
      });
      fixture.writeJson("apps/web/tsconfig.json", {
        compilerOptions: { baseUrl: "." },
        include: ["**/*.ts", "**/*.tsx"],
      });
      fixture.write(
        "apps/web/app/_types.ts",
        "export interface PageProps { params: { slug: string } }\n",
      );
      fixture.write(
        "apps/web/app/_trpc/context.ts",
        "export function getTRPCContext() { return { ok: true }; }\n",
      );
      fixture.write(
        "apps/web/app/page.ts",
        "import type { PageProps } from 'app/_types';\n" +
          "import { getTRPCContext } from 'app/_trpc/context';\n" +
          "export function Page(_props: PageProps) { return getTRPCContext().ok; }\n",
      );

      const symbols = runSymbolGraph(fixture);
      const resolve = createResolver(fixture);
      const importer = fixture.path("apps/web/app/page.ts");
      const deadIdentities = new Set(
        (symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`),
      );

      expect(slash(resolve(importer, "app/_types"))).toMatch(
        /apps\/web\/app\/_types\.ts$/,
      );
      expect(slash(resolve(importer, "app/_trpc/context"))).toMatch(
        /apps\/web\/app\/_trpc\/context\.ts$/,
      );
      expect(
        symbols.fanInByIdentity?.["apps/web/app/_types.ts::PageProps"],
      ).toBe(1);
      expect(deadIdentities).not.toContain("apps/web/app/_types.ts::PageProps");
      expect(
        symbols.fanInByIdentitySpace?.["apps/web/app/_types.ts::PageProps"],
      ).toMatchObject({ type: 1, value: 0, broad: 0 });
      expect(
        symbols.fanInByIdentity?.[
          "apps/web/app/_trpc/context.ts::getTRPCContext"
        ],
      ).toBe(1);
      expect(deadIdentities).not.toContain(
        "apps/web/app/_trpc/context.ts::getTRPCContext",
      );
      expect(
        symbols.fanInByIdentitySpace?.[
          "apps/web/app/_trpc/context.ts::getTRPCContext"
        ],
      ).toMatchObject({ value: 1, type: 0, broad: 0 });
      expect(resolve(importer, "app/does-not-exist")).toBe(
        "UNRESOLVED_INTERNAL",
      );
      expect(resolve(importer, "react")).toBe("EXTERNAL");
      expect(symbols.uses?.unresolvedInternal).toBe(0);
      expect(symbols.uses?.unresolvedInternalRatio).toBe(0);
    });
  });

  it("skips invalid tsconfig fixtures without dropping valid sibling entries", () => {
    withFixture("fx-vitest-invalid-tsconfig-", (fixture) => {
      fixture.writeJson("app/tsconfig.json", {
        compilerOptions: {
          baseUrl: ".",
          paths: { "@/*": ["./src/*"] },
        },
      });
      fixture.write(
        "packages/pkg/test/fixtures/tsconfig-handling/invalid/tsconfig.json",
        '{ "compilerOptions": { "baseUrl": ".", \n',
      );

      expect(() =>
        discoverScopedTsconfigResolution(fixture.root),
      ).not.toThrow();
      const resolution = discoverScopedTsconfigResolution(fixture.root);
      expect(
        resolution.paths.some(
          (entry) =>
            entry.key === "@/*" &&
            slash(entry.configPath).endsWith("/app/tsconfig.json"),
        ),
      ).toBe(true);
      expect(
        resolution.baseUrls.some((entry) =>
          slash(entry.configPath).endsWith("/app/tsconfig.json"),
        ),
      ).toBe(true);
    });
  });

  it("honors scan excludes when discovering scoped tsconfig resolution", () => {
    withFixture("fx-vitest-tsconfig-excludes-", (fixture) => {
      fixture.writeJson("app/tsconfig.json", {
        compilerOptions: {
          baseUrl: ".",
          paths: { "@app/*": ["./src/*"] },
        },
      });
      fixture.writeJson("p6-corpus/mirror/tsconfig.json", {
        compilerOptions: {
          baseUrl: ".",
          paths: { "@mirror/*": ["./src/*"] },
        },
      });

      const unscoped = discoverScopedTsconfigResolution(fixture.root);
      expect(
        unscoped.paths.some((entry) =>
          slash(entry.configPath).includes("/p6-corpus/mirror/tsconfig.json"),
        ),
      ).toBe(true);

      const scoped = discoverScopedTsconfigResolution(fixture.root, {
        exclude: ["p6-corpus"],
      });
      expect(
        scoped.paths.some((entry) =>
          slash(entry.configPath).includes("/app/tsconfig.json"),
        ),
      ).toBe(true);
      expect(
        scoped.paths.some((entry) =>
          slash(entry.configPath).includes("/p6-corpus/mirror/tsconfig.json"),
        ),
      ).toBe(false);

      const aliasMap = buildAliasMap(fixture.root, detectRepoMode(fixture.root), {
        exclude: ["p6-corpus"],
      });
      expect(
        aliasMap.scopedTsconfigPaths.some((entry) =>
          slash(entry.configPath).includes("/p6-corpus/mirror/tsconfig.json"),
        ),
      ).toBe(false);
    });
  });

  it("records unresolved tsconfig, workspace, and generated artifact reasons separately", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-unresolved-reasons-",
      packageJson: {
        name: "root",
        type: "module",
        workspaces: ["apps/*", "packages/*"],
      },
    });
    try {
      fixture.writeJson("apps/web/package.json", {
        name: "web",
        type: "module",
      });
      fixture.writeJson("apps/web/tsconfig.json", {
        compilerOptions: {
          baseUrl: ".",
          paths: {
            "@scope/generated-client": [
              "../../packages/generated/generated/client",
            ],
          },
        },
      });
      fixture.writeJson("packages/generated/package.json", {
        name: "@scope/generated",
        type: "module",
      });
      fixture.writeJson("packages/prisma/package.json", {
        name: "@scope/prisma",
        type: "module",
        main: "index.ts",
        bin: { "prisma-enum-generator": "./run-enum-generator.js" },
        prisma: { seed: "node seed.mjs" },
        scripts: { generate: "prisma generate" },
        dependencies: { "@prisma/client": "1.0.0" },
      });
      fixture.write(
        "packages/prisma/index.ts",
        "export const prismaRoot = 1;\n",
      );
      fixture.writeJson("packages/types/package.json", {
        name: "@scope/types",
        type: "module",
        main: "index.ts",
      });
      fixture.write("packages/types/index.ts", "export const root = 1;\n");
      fixture.write(
        "apps/web/src/consumer.ts",
        "import { missingGenerated } from '@scope/generated-client';\n" +
          "import { BookingStatus } from '@scope/prisma/enums';\n" +
          "import type { Thing } from '@scope/types/thing';\n" +
          "export const uses = [missingGenerated, BookingStatus] as Thing[];\n",
      );

      const symbols = runSymbolGraph(fixture);
      const records = symbols.unresolvedInternalSpecifierRecords ?? [];
      const generated = records.find(
        (r) => r.specifier === "@scope/generated-client",
      );
      const generatedWorkspace = records.find(
        (r) => r.specifier === "@scope/prisma/enums",
      );
      const workspace = records.find(
        (r) => r.specifier === "@scope/types/thing",
      );

      expect(generated).toMatchObject({
        reason: "tsconfig-path-target-missing",
        hint: "generated-artifact-missing",
        matchedPattern: "@scope/generated-client",
      });
      expect(generated?.generatedArtifact).toMatchObject({
        policyVersion: "generated-artifact-policy-v1",
        generatorFamily: "path-segment",
        confidence: "supporting",
      });
      expect(workspace).toMatchObject({
        reason: "workspace-package-subpath-target-missing",
        matchedPattern: "@scope/types/*",
        typeOnly: true,
      });
      expect(generatedWorkspace).toMatchObject({
        reason: "workspace-generated-artifact-missing",
        hint: "generated-artifact-missing",
        matchedPattern: "@scope/prisma/*",
      });
      expect(generatedWorkspace?.generatedArtifact).toMatchObject({
        policyVersion: "generated-artifact-policy-v1",
        generatorFamily: "prisma",
        confidence: "strong",
        matchedPackage: "@scope/prisma",
        targetSubpath: "enums",
      });
      expect(
        symbols.unresolvedInternalSummaryByReason?.[
          "tsconfig-path-target-missing"
        ]?.count,
      ).toBe(1);
      expect(
        symbols.unresolvedInternalSummaryByReason?.[
          "workspace-package-subpath-target-missing"
        ]?.count,
      ).toBe(1);
      expect(
        symbols.unresolvedInternalSummaryByReason?.[
          "workspace-generated-artifact-missing"
        ]?.count,
      ).toBe(1);
    } finally {
      fixture.cleanup();
    }
  });

  it("falls back from a missing tsconfig generated target to a concrete workspace package source", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-tsconfig-workspace-fallback-",
      packageJson: {
        name: "root",
        type: "module",
        workspaces: ["apps/*", "packages/*"],
      },
    });
    try {
      fixture.writeJson("apps/api/package.json", {
        name: "api",
        type: "module",
        dependencies: { "@scope/orm": "workspace:*" },
      });
      fixture.writeJson("apps/api/tsconfig.json", {
        compilerOptions: {
          baseUrl: ".",
          paths: {
            "@scope/orm/client": ["../../packages/orm/generated/client"],
          },
        },
      });
      fixture.writeJson("packages/orm/package.json", {
        name: "@scope/orm",
        type: "module",
        main: "index.ts",
      });
      fixture.write("packages/orm/index.ts", "export const root = 1;\n");
      fixture.write(
        "packages/orm/client/index.ts",
        "export interface OrmUser { id: string }\n" +
          "export function makeOrmUser(): OrmUser { return { id: 'u' }; }\n" +
          "export function unusedOrmClient() { return 1; }\n",
      );
      fixture.write(
        "apps/api/src/service.ts",
        "import type { OrmUser } from '@scope/orm/client';\n" +
          "import { makeOrmUser } from '@scope/orm/client';\n" +
          "export function service(): OrmUser { return makeOrmUser(); }\n",
      );

      const symbols = runSymbolGraph(fixture);
      const unresolved = symbols.unresolvedInternalSpecifierRecords ?? [];
      const dead = new Set(
        (symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`),
      );

      expect(
        symbols.fanInByIdentity?.["packages/orm/client/index.ts::OrmUser"],
      ).toBe(1);
      expect(
        symbols.fanInByIdentity?.["packages/orm/client/index.ts::makeOrmUser"],
      ).toBe(1);
      expect(dead).not.toContain("packages/orm/client/index.ts::OrmUser");
      expect(dead).not.toContain("packages/orm/client/index.ts::makeOrmUser");
      expect(unresolved.some((r) => r.specifier === "@scope/orm/client")).toBe(
        false,
      );
      expect(symbols.uses?.unresolvedInternal).toBe(0);
    } finally {
      fixture.cleanup();
    }
  });
});
