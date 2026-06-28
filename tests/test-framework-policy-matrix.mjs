import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import {
  ACTION_MUTE,
  ACTION_NONE,
  ACTION_REVIEW_HINT,
  classifyFrameworkPolicy,
  createFrameworkPolicyContext,
  createFrameworkPolicyCounters,
  recordFrameworkPolicyDecision,
} from '../_lib/framework-policy-matrix.mjs';
import { createFrameworkPolicyContextForRepo } from '../_lib/classify-policies.mjs';
import { detectRepoMode } from '../_lib/repo-mode.mjs';

let passed = 0;
let failed = 0;

function check(label, fn) {
  try {
    fn();
    passed++;
    console.log(`  PASS  ${label}`);
  } catch (error) {
    failed++;
    console.log(`  FAIL  ${label}\n        ${error?.message ?? error}`);
  }
}

const ROOT = 'C:/repo';

function packageRecord(relRoot, packageJson) {
  return {
    root: relRoot === '.' ? ROOT : `${ROOT}/${relRoot}`,
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

function classify(policyContext, file, exportName = 'default', kind = 'function') {
  return classifyFrameworkPolicy(policyContext, { file, exportName, kind });
}

function pkgWithDeps(dependencies) {
  return { name: 'fixture', dependencies };
}

check('T1. root Next evidence does not activate nested package with its own package.json', () => {
  const policyContext = context({
    packageRecords: [
      packageRecord('.', pkgWithDeps({ next: '15.0.0' })),
      packageRecord('packages/tool', pkgWithDeps({})),
    ],
    files: ['app/page.tsx', 'packages/tool/app/page.tsx'],
  });

  assert.equal(classify(policyContext, 'app/page.tsx').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'packages/tool/app/page.tsx').action, ACTION_NONE);
});

check('T2. nested Next package protects src app, pages, proxy, and instrumentation exports', () => {
  const policyContext = context({
    packageRecords: [
      packageRecord('.', pkgWithDeps({})),
      packageRecord('packages/web', pkgWithDeps({ next: '15.0.0' })),
    ],
    files: [
      'packages/web/src/app/page.tsx',
      'packages/web/src/pages/index.tsx',
      'packages/web/src/proxy.ts',
      'packages/web/src/instrumentation.ts',
      'packages/web/src/instrumentation-client.ts',
    ],
  });

  const page = classify(policyContext, 'packages/web/src/app/page.tsx');
  assert.equal(page.action, ACTION_MUTE);
  assert.equal(page.framework, 'next');
  assert.equal(page.reason, 'frameworkSentinel_FP27');

  assert.equal(classify(policyContext, 'packages/web/src/pages/index.tsx').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'packages/web/src/proxy.ts', 'proxy').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'packages/web/src/proxy.ts', 'config').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'packages/web/src/instrumentation.ts', 'register').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'packages/web/src/instrumentation.ts', 'onRequestError').action, ACTION_MUTE);

  const client = classify(policyContext, 'packages/web/src/instrumentation-client.ts', 'default');
  assert.equal(client.action, ACTION_REVIEW_HINT);
  assert.equal(client.framework, 'next');
});

check('T2b. non-workspace nested Next package protects app router files', () => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'lrl-nested-next-policy-'));
  try {
    writeFileSync(path.join(root, 'package.json'), JSON.stringify({
      name: 'fixture-root',
      private: true,
      workspaces: ['packages/*'],
    }));
    mkdirSync(path.join(root, 'apps/dashboard/app'), { recursive: true });
    writeFileSync(path.join(root, 'apps/dashboard/package.json'), JSON.stringify({
      name: 'dashboard',
      private: true,
      dependencies: { next: '15.0.0' },
    }));
    writeFileSync(path.join(root, 'apps/dashboard/app/page.tsx'), 'export default function Page() { return null; }\n');

    const repoMode = detectRepoMode(root);
    const policyContext = createFrameworkPolicyContextForRepo({
      root,
      repoMode,
      symbolsData: { defIndex: { 'apps/dashboard/app/page.tsx': [] } },
      deadList: [{ file: 'apps/dashboard/app/page.tsx', symbol: 'default' }],
      includeTests: true,
      exclude: [],
    });

    const page = classifyFrameworkPolicy(policyContext, {
      file: 'apps/dashboard/app/page.tsx',
      exportName: 'default',
      kind: 'FunctionDeclaration',
    });
    assert.equal(page.action, ACTION_MUTE);
    assert.equal(page.framework, 'next');
    assert.equal(page.reason, 'frameworkSentinel_FP27');
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

check('T2c. non-workspace nested package boundary still blocks root Next leakage', () => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'lrl-nested-next-boundary-'));
  try {
    writeFileSync(path.join(root, 'package.json'), JSON.stringify({
      name: 'fixture-root',
      private: true,
      dependencies: { next: '15.0.0' },
    }));
    mkdirSync(path.join(root, 'apps/tool/app'), { recursive: true });
    writeFileSync(path.join(root, 'apps/tool/package.json'), JSON.stringify({
      name: 'tool',
      private: true,
      dependencies: {},
    }));
    writeFileSync(path.join(root, 'apps/tool/app/page.tsx'), 'export default function Page() { return null; }\n');

    const repoMode = detectRepoMode(root);
    const policyContext = createFrameworkPolicyContextForRepo({
      root,
      repoMode,
      symbolsData: { defIndex: { 'apps/tool/app/page.tsx': [] } },
      deadList: [{ file: 'apps/tool/app/page.tsx', symbol: 'default' }],
      includeTests: true,
      exclude: [],
    });

    const page = classifyFrameworkPolicy(policyContext, {
      file: 'apps/tool/app/page.tsx',
      exportName: 'default',
      kind: 'FunctionDeclaration',
    });
    assert.equal(page.action, ACTION_NONE);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

check('T2d. repo mode merges package workspaces with pnpm-workspace.yaml patterns', () => {
  const root = mkdtempSync(path.join(os.tmpdir(), 'lrl-workspace-pattern-merge-'));
  try {
    writeFileSync(path.join(root, 'package.json'), JSON.stringify({
      name: 'fixture-root',
      private: true,
      workspaces: ['packages/*'],
    }));
    writeFileSync(path.join(root, 'pnpm-workspace.yaml'), 'packages:\n  - apps/*\n  - bench/*\n');
    mkdirSync(path.join(root, 'packages/core'), { recursive: true });
    mkdirSync(path.join(root, 'apps/dashboard'), { recursive: true });
    mkdirSync(path.join(root, 'bench/heavy-npm-deps'), { recursive: true });
    writeFileSync(path.join(root, 'packages/core/package.json'), JSON.stringify({ name: '@fixture/core' }));
    writeFileSync(path.join(root, 'apps/dashboard/package.json'), JSON.stringify({
      name: '@fixture/dashboard',
      dependencies: { next: '15.0.0' },
    }));
    writeFileSync(path.join(root, 'bench/heavy-npm-deps/package.json'), JSON.stringify({
      name: '@fixture/bench',
      dependencies: { next: '15.0.0' },
    }));

    const repoMode = detectRepoMode(root);
    const relWorkspaces = repoMode.workspaceDirs
      .map((dir) => path.relative(root, dir).replace(/\\/g, '/'))
      .sort();

    assert.deepEqual(relWorkspaces, [
      'apps/dashboard',
      'bench/heavy-npm-deps',
      'packages/core',
    ]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

check('T3. arbitrary nested Next middleware path stays visible', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ next: '15.0.0' }))],
    files: ['app/page.tsx', 'app/foo/middleware.ts'],
  });

  const decision = classify(policyContext, 'app/foo/middleware.ts', 'middleware');
  assert.equal(decision.action, ACTION_NONE);
});

check('T4. Nuxt rejected signals do not activate Nitro muting', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ '@nuxt/opencollective': '0.4.1', h3: '^1.0.0' }))],
    files: ['middleware/logger.ts', 'plugins/logger.ts'],
  });

  assert.equal(classify(policyContext, 'middleware/logger.ts', 'LoggerMiddleware').action, ACTION_NONE);

  const counters = createFrameworkPolicyCounters(policyContext);
  assert.deepEqual(counters.rejectedSignalOccurrences['@nuxt/opencollective'], {
    packages: 1,
    findingsAffected: 0,
  });
});

check('T5. Nuxt top-level composable may mute but nested composable stays visible', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ nuxt: '^4.0.0' }))],
    files: ['app/composables/useThing.ts', 'app/composables/nested/useThing.ts'],
  });

  assert.equal(classify(policyContext, 'app/composables/useThing.ts', 'useThing').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'app/composables/nested/useThing.ts', 'useThing').action, ACTION_NONE);
});

check('T6. SvelteKit protects route exports and narrows entries to dynamic routes', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ '@sveltejs/kit': '^2.0.0' }))],
    files: [
      'src/routes/+layout.ts',
      'src/routes/blog/[slug]/+page.server.ts',
      'src/routes/about/+page.ts',
      'src/routes/api/+server.ts',
    ],
  });

  assert.equal(classify(policyContext, 'src/routes/+layout.ts', 'load').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'src/routes/api/+server.ts', 'GET').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'src/routes/blog/[slug]/+page.server.ts', 'entries').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'src/routes/about/+page.ts', 'entries').action, ACTION_NONE);
});

check('T7. Astro protects endpoint exports but not arbitrary defaults', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ astro: '^5.0.0' }))],
    files: ['src/pages/api/user.ts', 'src/pages/[slug].ts'],
  });

  assert.equal(classify(policyContext, 'src/pages/api/user.ts', 'GET').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'src/pages/api/user.ts', 'ALL').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'src/pages/[slug].ts', 'getStaticPaths').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'src/pages/api/user.ts', 'default').action, ACTION_NONE);
});

check('T8. React Router keeps newer route-module exports review-visible', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ '@react-router/dev': '^7.0.0' }))],
    files: ['app/routes/home.tsx'],
  });

  assert.equal(classify(policyContext, 'app/routes/home.tsx', 'loader').action, ACTION_MUTE);
  assert.equal(classify(policyContext, 'app/routes/home.tsx', 'clientLoader').action, ACTION_REVIEW_HINT);
});

check('T9. Hono muting requires route registration facts, not path shape', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ hono: '^4.0.0' }))],
    files: ['routes/health.ts', 'src/handlers.ts'],
    frameworkFacts: {
      honoRouteRegistrations: [
        {
          file: 'src/server.ts',
          callee: 'app.get',
          route: '/health',
          handlerRefs: [{ file: 'src/handlers.ts', exportName: 'handler' }],
        },
      ],
    },
  });

  assert.equal(classify(policyContext, 'routes/health.ts', 'handler').action, ACTION_NONE);
  assert.equal(classify(policyContext, 'src/handlers.ts', 'handler').action, ACTION_MUTE);
});

check('T10. NestJS dependencies and paths do not framework-mute helpers', () => {
  const policyContext = context({
    packageRecords: [packageRecord('.', pkgWithDeps({ '@nestjs/common': '^10.0.0' }))],
    files: ['src/middleware/logger.middleware.ts', 'src/plugins/logging.plugin.ts'],
  });

  assert.equal(classify(policyContext, 'src/middleware/logger.middleware.ts', 'LoggerMiddleware').action, ACTION_NONE);
  assert.equal(classify(policyContext, 'src/plugins/logging.plugin.ts', 'LoggingPlugin').action, ACTION_NONE);
});

check('T11. counters count muted, review-hint, rejected, and kept-visible findings separately', () => {
  const policyContext = context({
    packageRecords: [
      packageRecord('.', pkgWithDeps({ next: '15.0.0', '@nuxt/opencollective': '0.4.1' })),
    ],
    files: ['app/page.tsx', 'app/foo/middleware.ts', 'instrumentation-client.ts'],
  });

  const counters = createFrameworkPolicyCounters(policyContext);
  const muted = classify(policyContext, 'app/page.tsx');
  const hinted = classify(policyContext, 'instrumentation-client.ts', 'default');
  const visible = classify(policyContext, 'app/foo/middleware.ts', 'middleware');
  recordFrameworkPolicyDecision(counters, muted, { file: 'app/page.tsx' });
  recordFrameworkPolicyDecision(counters, hinted, { file: 'instrumentation-client.ts' });
  recordFrameworkPolicyDecision(counters, visible, { file: 'app/foo/middleware.ts' });

  assert.equal(counters.mutedFindings.next, 1);
  assert.equal(counters.reviewHintFindings.next, 1);
  assert.equal(counters.pathShapedCandidatesKeptVisible.middleware, 1);
  assert.deepEqual(counters.rejectedSignalOccurrences['@nuxt/opencollective'], {
    packages: 1,
    findingsAffected: 0,
  });
});

check('T12. Cloudflare Worker package protects module default export entrypoint only', () => {
  const policyContext = context({
    packageRecords: [
      packageRecord('.', pkgWithDeps({})),
      packageRecord('cloudflare/worker', pkgWithDeps({
        wrangler: '^4.0.0',
        '@cloudflare/workers-types': '^4.0.0',
      })),
    ],
    files: ['cloudflare/worker/src/index.js'],
  });

  const defaultEntry = classify(policyContext, 'cloudflare/worker/src/index.js', 'default');
  assert.equal(defaultEntry.action, ACTION_MUTE);
  assert.equal(defaultEntry.framework, 'cloudflare-workers');
  assert.equal(defaultEntry.reason, 'frameworkSentinel_FP27');

  assert.equal(classify(policyContext, 'cloudflare/worker/src/index.js', 'helper').action, ACTION_NONE);

  const nestedWithoutEvidence = context({
    packageRecords: [
      packageRecord('.', pkgWithDeps({ wrangler: '^4.0.0' })),
      packageRecord('packages/tool', pkgWithDeps({})),
    ],
    files: ['packages/tool/src/index.js'],
  });

  assert.equal(classify(nestedWithoutEvidence, 'packages/tool/src/index.js', 'default').action, ACTION_NONE);

  const configScopedWorker = context({
    packageRecords: [packageRecord('.', pkgWithDeps({}))],
    files: [
      'cloudflare/worker/wrangler.toml',
      'cloudflare/worker/src/index.js',
      'cloudflare/worker/src/helper.js',
    ],
  });

  const configScopedEntry = classify(configScopedWorker, 'cloudflare/worker/src/index.js', 'default');
  assert.equal(configScopedEntry.action, ACTION_MUTE);
  assert.equal(configScopedEntry.framework, 'cloudflare-workers');
  assert.deepEqual(configScopedEntry.evidence.activation, ['config:cloudflare/worker/wrangler.toml']);
  assert.equal(classify(configScopedWorker, 'cloudflare/worker/src/helper.js', 'default').action, ACTION_NONE);
});

if (failed > 0) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, 0 failed`);
