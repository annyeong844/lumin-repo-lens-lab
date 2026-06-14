import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';

import { collectHonoRouteRegistrations } from '../_lib/framework-policy-facts.mjs';
import { shouldCollectHonoRouteFactsForPackages } from '../_lib/classify-policies.mjs';

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

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function collect(root, files) {
  return collectHonoRouteRegistrations({ root, files });
}

check('T1. collects imported Hono handlers from app.get handlerRefs array', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'hono-facts-'));
  try {
    write(root, 'src/server.ts', [
      "import { Hono } from 'hono';",
      "import { auth } from './middleware';",
      "import { handler } from './handlers';",
      'const app = new Hono();',
      "app.get('/x', auth, handler);",
      '',
    ].join('\n'));
    write(root, 'src/middleware.ts', 'export function auth(c) { return c.next(); }\n');
    write(root, 'src/handlers.ts', 'export function handler(c) { return c.text("ok"); }\n');

    const facts = collect(root, ['src/server.ts', 'src/middleware.ts', 'src/handlers.ts']);
    assert.deepEqual(facts, [
      {
        file: 'src/server.ts',
        callee: 'app.get',
        route: '/x',
        handlerRefs: [
          { file: 'src/middleware.ts', exportName: 'auth' },
          { file: 'src/handlers.ts', exportName: 'handler' },
        ],
      },
    ]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

check('T2. collects app.use and app.route references', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'hono-facts-'));
  try {
    write(root, 'src/server.ts', [
      "import { Hono } from 'hono';",
      "import { auth } from './middleware';",
      "import { apiRoutes } from './api';",
      'const app = new Hono();',
      "app.use('/x', auth);",
      "app.route('/api', apiRoutes);",
      '',
    ].join('\n'));
    write(root, 'src/middleware.ts', 'export const auth = (c, next) => next();\n');
    write(root, 'src/api.ts', 'export const apiRoutes = new Hono();\n');

    const facts = collect(root, ['src/server.ts', 'src/middleware.ts', 'src/api.ts']);
    assert.deepEqual(facts, [
      {
        file: 'src/server.ts',
        callee: 'app.use',
        route: '/x',
        handlerRefs: [{ file: 'src/middleware.ts', exportName: 'auth' }],
      },
      {
        file: 'src/server.ts',
        callee: 'app.route',
        route: '/api',
        handlerRefs: [{ file: 'src/api.ts', exportName: 'apiRoutes' }],
      },
    ]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

check('T3. collects local exported handlers and skips dynamic handler expressions', () => {
  const root = mkdtempSync(path.join(tmpdir(), 'hono-facts-'));
  try {
    write(root, 'src/server.ts', [
      "import { Hono } from 'hono';",
      'const app = new Hono();',
      'export function localHandler(c) { return c.text("local"); }',
      "app.post('/local', localHandler);",
      "app.get('/dynamic', makeHandler());",
      '',
    ].join('\n'));

    const facts = collect(root, ['src/server.ts']);
    assert.deepEqual(facts, [
      {
        file: 'src/server.ts',
        callee: 'app.post',
        route: '/local',
        handlerRefs: [{ file: 'src/server.ts', exportName: 'localHandler' }],
      },
    ]);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

check('T4. Hono route fact collection is gated by package-scoped Hono dependency', () => {
  assert.equal(shouldCollectHonoRouteFactsForPackages([
    { relRoot: '.', packageJson: { dependencies: { next: '^15.0.0' } } },
  ]), false);
  assert.equal(shouldCollectHonoRouteFactsForPackages([
    { relRoot: '.', packageJson: { dependencies: { hono: '^4.0.0' } } },
  ]), true);
  assert.equal(shouldCollectHonoRouteFactsForPackages([
    { relRoot: '.', packageJson: { devDependencies: { hono: '^4.0.0' } } },
  ]), true);
});

if (failed > 0) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, 0 failed`);
