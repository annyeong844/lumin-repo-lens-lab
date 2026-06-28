import path from "node:path";

import { describe, expect, it } from "vitest";

import { createFrameworkPolicyContextForRepo } from "../_lib/classify-policies.mjs";
import {
  ACTION_MUTE,
  ACTION_NONE,
  ACTION_REVIEW_HINT,
  classifyFrameworkPolicy,
  createFrameworkPolicyContext,
  createFrameworkPolicyCounters,
  recordFrameworkPolicyDecision,
} from "../_lib/framework-policy-matrix.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = "C:/repo";

function packageRecord(relRoot, packageJson) {
  return {
    root: relRoot === "." ? ROOT : `${ROOT}/${relRoot}`,
    relRoot,
    packageJson,
  };
}

function context({ packageRecords, files = [], frameworkFacts = {} }) {
  return createFrameworkPolicyContext({
    root: ROOT,
    packageRecords,
    files,
    frameworkFacts,
  });
}

function classify(
  policyContext,
  file,
  exportName = "default",
  kind = "function",
) {
  return classifyFrameworkPolicy(policyContext, { file, exportName, kind });
}

function pkgWithDeps(dependencies) {
  return { name: "fixture", dependencies };
}

describe("framework policy matrix", () => {
  it("T1. root Next evidence does not activate nested package with its own package.json", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({ next: "15.0.0" })),
        packageRecord("packages/tool", pkgWithDeps({})),
      ],
      files: ["app/page.tsx", "packages/tool/app/page.tsx"],
    });

    expect(classify(policyContext, "app/page.tsx").action).toBe(ACTION_MUTE);
    expect(classify(policyContext, "packages/tool/app/page.tsx").action).toBe(
      ACTION_NONE,
    );
  });

  it("T2. nested Next package protects src app, pages, proxy, and instrumentation exports", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({})),
        packageRecord("packages/web", pkgWithDeps({ next: "15.0.0" })),
      ],
      files: [
        "packages/web/src/app/page.tsx",
        "packages/web/src/pages/index.tsx",
        "packages/web/src/proxy.ts",
        "packages/web/src/instrumentation.ts",
        "packages/web/src/instrumentation-client.ts",
      ],
    });

    expect(
      classify(policyContext, "packages/web/src/app/page.tsx"),
    ).toMatchObject({
      action: ACTION_MUTE,
      framework: "next",
      reason: "frameworkSentinel_FP27",
    });
    expect(
      classify(policyContext, "packages/web/src/pages/index.tsx").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "packages/web/src/proxy.ts", "proxy").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "packages/web/src/proxy.ts", "config").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "packages/web/src/instrumentation.ts", "register")
        .action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(
        policyContext,
        "packages/web/src/instrumentation.ts",
        "onRequestError",
      ).action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(
        policyContext,
        "packages/web/src/instrumentation-client.ts",
        "default",
      ),
    ).toMatchObject({ action: ACTION_REVIEW_HINT, framework: "next" });
  });

  it("T2b. non-workspace nested Next package protects app router files", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-nested-next-policy-",
      packageJson: {
        name: "fixture-root",
        private: true,
        workspaces: ["packages/*"],
      },
    });

    try {
      fixture.writeJson("apps/dashboard/package.json", {
        name: "dashboard",
        private: true,
        dependencies: { next: "15.0.0" },
      });
      fixture.write(
        "apps/dashboard/app/page.tsx",
        "export default function Page() { return null; }\n",
      );

      const repoMode = detectRepoMode(fixture.root);
      const policyContext = createFrameworkPolicyContextForRepo({
        root: fixture.root,
        repoMode,
        symbolsData: { defIndex: { "apps/dashboard/app/page.tsx": [] } },
        deadList: [{ file: "apps/dashboard/app/page.tsx", symbol: "default" }],
        includeTests: true,
        exclude: [],
      });

      expect(
        classifyFrameworkPolicy(policyContext, {
          file: "apps/dashboard/app/page.tsx",
          exportName: "default",
          kind: "FunctionDeclaration",
        }),
      ).toMatchObject({
        action: ACTION_MUTE,
        framework: "next",
        reason: "frameworkSentinel_FP27",
      });
    } finally {
      fixture.cleanup();
    }
  });

  it("T2c. non-workspace nested package boundary still blocks root Next leakage", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-nested-next-boundary-",
      packageJson: {
        name: "fixture-root",
        private: true,
        dependencies: { next: "15.0.0" },
      },
    });

    try {
      fixture.writeJson("apps/tool/package.json", {
        name: "tool",
        private: true,
        dependencies: {},
      });
      fixture.write(
        "apps/tool/app/page.tsx",
        "export default function Page() { return null; }\n",
      );

      const repoMode = detectRepoMode(fixture.root);
      const policyContext = createFrameworkPolicyContextForRepo({
        root: fixture.root,
        repoMode,
        symbolsData: { defIndex: { "apps/tool/app/page.tsx": [] } },
        deadList: [{ file: "apps/tool/app/page.tsx", symbol: "default" }],
        includeTests: true,
        exclude: [],
      });

      expect(
        classifyFrameworkPolicy(policyContext, {
          file: "apps/tool/app/page.tsx",
          exportName: "default",
          kind: "FunctionDeclaration",
        }).action,
      ).toBe(ACTION_NONE);
    } finally {
      fixture.cleanup();
    }
  });

  it("T2d. repo mode merges package workspaces with pnpm-workspace.yaml patterns", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-workspace-pattern-merge-",
      packageJson: {
        name: "fixture-root",
        private: true,
        workspaces: ["packages/*"],
      },
    });

    try {
      fixture.write(
        "pnpm-workspace.yaml",
        "packages:\n  - apps/*\n  - bench/*\n",
      );
      fixture.writeJson("packages/core/package.json", {
        name: "@fixture/core",
      });
      fixture.writeJson("apps/dashboard/package.json", {
        name: "@fixture/dashboard",
        dependencies: { next: "15.0.0" },
      });
      fixture.writeJson("bench/heavy-npm-deps/package.json", {
        name: "@fixture/bench",
        dependencies: { next: "15.0.0" },
      });

      const repoMode = detectRepoMode(fixture.root);
      const relWorkspaces = repoMode.workspaceDirs
        .map((dir) => path.relative(fixture.root, dir).replace(/\\/g, "/"))
        .sort();

      expect(relWorkspaces).toEqual([
        "apps/dashboard",
        "bench/heavy-npm-deps",
        "packages/core",
      ]);
    } finally {
      fixture.cleanup();
    }
  });

  it("T3. arbitrary nested Next middleware path stays visible", () => {
    const policyContext = context({
      packageRecords: [packageRecord(".", pkgWithDeps({ next: "15.0.0" }))],
      files: ["app/page.tsx", "app/foo/middleware.ts"],
    });

    expect(
      classify(policyContext, "app/foo/middleware.ts", "middleware").action,
    ).toBe(ACTION_NONE);
  });

  it("T4. Nuxt rejected signals do not activate Nitro muting", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(
          ".",
          pkgWithDeps({ "@nuxt/opencollective": "0.4.1", h3: "^1.0.0" }),
        ),
      ],
      files: ["middleware/logger.ts", "plugins/logger.ts"],
    });

    expect(
      classify(policyContext, "middleware/logger.ts", "LoggerMiddleware")
        .action,
    ).toBe(ACTION_NONE);
    expect(
      createFrameworkPolicyCounters(policyContext).rejectedSignalOccurrences[
        "@nuxt/opencollective"
      ],
    ).toEqual({ packages: 1, findingsAffected: 0 });
  });

  it("T5. Nuxt top-level composable may mute but nested composable stays visible", () => {
    const policyContext = context({
      packageRecords: [packageRecord(".", pkgWithDeps({ nuxt: "^4.0.0" }))],
      files: [
        "app/composables/useThing.ts",
        "app/composables/nested/useThing.ts",
      ],
    });

    expect(
      classify(policyContext, "app/composables/useThing.ts", "useThing").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "app/composables/nested/useThing.ts", "useThing")
        .action,
    ).toBe(ACTION_NONE);
  });

  it("T6. SvelteKit protects route exports and narrows entries to dynamic routes", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({ "@sveltejs/kit": "^2.0.0" })),
      ],
      files: [
        "src/routes/+layout.ts",
        "src/routes/blog/[slug]/+page.server.ts",
        "src/routes/about/+page.ts",
        "src/routes/api/+server.ts",
      ],
    });

    expect(
      classify(policyContext, "src/routes/+layout.ts", "load").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "src/routes/api/+server.ts", "GET").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(
        policyContext,
        "src/routes/blog/[slug]/+page.server.ts",
        "entries",
      ).action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "src/routes/about/+page.ts", "entries").action,
    ).toBe(ACTION_NONE);
  });

  it("T7. Astro protects endpoint exports but not arbitrary defaults", () => {
    const policyContext = context({
      packageRecords: [packageRecord(".", pkgWithDeps({ astro: "^5.0.0" }))],
      files: ["src/pages/api/user.ts", "src/pages/[slug].ts"],
    });

    expect(classify(policyContext, "src/pages/api/user.ts", "GET").action).toBe(
      ACTION_MUTE,
    );
    expect(classify(policyContext, "src/pages/api/user.ts", "ALL").action).toBe(
      ACTION_MUTE,
    );
    expect(
      classify(policyContext, "src/pages/[slug].ts", "getStaticPaths").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "src/pages/api/user.ts", "default").action,
    ).toBe(ACTION_NONE);
  });

  it("T8. React Router keeps newer route-module exports review-visible", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({ "@react-router/dev": "^7.0.0" })),
      ],
      files: ["app/routes/home.tsx"],
    });

    expect(
      classify(policyContext, "app/routes/home.tsx", "loader").action,
    ).toBe(ACTION_MUTE);
    expect(
      classify(policyContext, "app/routes/home.tsx", "clientLoader").action,
    ).toBe(ACTION_REVIEW_HINT);
  });

  it("T9. Hono muting requires route registration facts, not path shape", () => {
    const policyContext = context({
      packageRecords: [packageRecord(".", pkgWithDeps({ hono: "^4.0.0" }))],
      files: ["routes/health.ts", "src/handlers.ts"],
      frameworkFacts: {
        honoRouteRegistrations: [
          {
            file: "src/server.ts",
            callee: "app.get",
            route: "/health",
            handlerRefs: [{ file: "src/handlers.ts", exportName: "handler" }],
          },
        ],
      },
    });

    expect(classify(policyContext, "routes/health.ts", "handler").action).toBe(
      ACTION_NONE,
    );
    expect(classify(policyContext, "src/handlers.ts", "handler").action).toBe(
      ACTION_MUTE,
    );
  });

  it("T10. NestJS dependencies and paths do not framework-mute helpers", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({ "@nestjs/common": "^10.0.0" })),
      ],
      files: [
        "src/middleware/logger.middleware.ts",
        "src/plugins/logging.plugin.ts",
      ],
    });

    expect(
      classify(
        policyContext,
        "src/middleware/logger.middleware.ts",
        "LoggerMiddleware",
      ).action,
    ).toBe(ACTION_NONE);
    expect(
      classify(policyContext, "src/plugins/logging.plugin.ts", "LoggingPlugin")
        .action,
    ).toBe(ACTION_NONE);
  });

  it("T11. counters count muted, review-hint, rejected, and kept-visible findings separately", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(
          ".",
          pkgWithDeps({ next: "15.0.0", "@nuxt/opencollective": "0.4.1" }),
        ),
      ],
      files: [
        "app/page.tsx",
        "app/foo/middleware.ts",
        "instrumentation-client.ts",
      ],
    });

    const counters = createFrameworkPolicyCounters(policyContext);
    recordFrameworkPolicyDecision(
      counters,
      classify(policyContext, "app/page.tsx"),
      {
        file: "app/page.tsx",
      },
    );
    recordFrameworkPolicyDecision(
      counters,
      classify(policyContext, "instrumentation-client.ts", "default"),
      { file: "instrumentation-client.ts" },
    );
    recordFrameworkPolicyDecision(
      counters,
      classify(policyContext, "app/foo/middleware.ts", "middleware"),
      { file: "app/foo/middleware.ts" },
    );

    expect(counters.mutedFindings.next).toBe(1);
    expect(counters.reviewHintFindings.next).toBe(1);
    expect(counters.pathShapedCandidatesKeptVisible.middleware).toBe(1);
    expect(counters.rejectedSignalOccurrences["@nuxt/opencollective"]).toEqual({
      packages: 1,
      findingsAffected: 0,
    });
  });

  it("T12. Cloudflare Worker package protects module default export entrypoint only", () => {
    const policyContext = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({})),
        packageRecord(
          "cloudflare/worker",
          pkgWithDeps({
            wrangler: "^4.0.0",
            "@cloudflare/workers-types": "^4.0.0",
          }),
        ),
      ],
      files: ["cloudflare/worker/src/index.js"],
    });

    expect(
      classify(policyContext, "cloudflare/worker/src/index.js", "default"),
    ).toMatchObject({
      action: ACTION_MUTE,
      framework: "cloudflare-workers",
      reason: "frameworkSentinel_FP27",
    });
    expect(
      classify(policyContext, "cloudflare/worker/src/index.js", "helper")
        .action,
    ).toBe(ACTION_NONE);

    const nestedWithoutEvidence = context({
      packageRecords: [
        packageRecord(".", pkgWithDeps({ wrangler: "^4.0.0" })),
        packageRecord("packages/tool", pkgWithDeps({})),
      ],
      files: ["packages/tool/src/index.js"],
    });
    expect(
      classify(nestedWithoutEvidence, "packages/tool/src/index.js", "default")
        .action,
    ).toBe(ACTION_NONE);

    const configScopedWorker = context({
      packageRecords: [packageRecord(".", pkgWithDeps({}))],
      files: [
        "cloudflare/worker/wrangler.toml",
        "cloudflare/worker/src/index.js",
        "cloudflare/worker/src/helper.js",
      ],
    });

    expect(
      classify(configScopedWorker, "cloudflare/worker/src/index.js", "default"),
    ).toMatchObject({
      action: ACTION_MUTE,
      framework: "cloudflare-workers",
      evidence: { activation: ["config:cloudflare/worker/wrangler.toml"] },
    });
    expect(
      classify(configScopedWorker, "cloudflare/worker/src/helper.js", "default")
        .action,
    ).toBe(ACTION_NONE);
  });
});
