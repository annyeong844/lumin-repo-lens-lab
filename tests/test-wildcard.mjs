// Tests for package.json "exports" wildcard subpath resolution.
// Covers:
//  - simple "./*" wildcard (was broken too)
//  - subpath "./features/*" (user's main report)
//  - multiple wildcards with most-specific-match tie-break
//  - dist→src remapping (mapOutputToSource equivalent)
//  - negative: specs that shouldn't match must fall through to EXTERNAL
import path from 'node:path';
import { tmpdir } from 'node:os';
import { writeFileSync, mkdirSync, rmSync } from 'node:fs';
import { detectRepoMode } from '../_lib/repo-mode.mjs';
import { buildAliasMap } from '../_lib/alias-map.mjs';
import { makeResolver } from '../_lib/resolver-core.mjs';

// Use the OS tmpdir — `/tmp` is Linux-only; on Windows Node's path resolver
// adds the current drive prefix which breaks fixture/assertion comparisons.
const ROOT_BASE = path.join(tmpdir(), 'wildcard-fx');
rmSync(ROOT_BASE, { recursive: true, force: true });

function mkFx(name, pkgJson, filesMap) {
  const dir = path.join(ROOT_BASE, name);
  mkdirSync(dir, { recursive: true });
  writeFileSync(path.join(dir, 'package.json'), JSON.stringify(pkgJson, null, 2));
  for (const [rel, content] of Object.entries(filesMap)) {
    const full = path.join(dir, rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, content);
  }
  return dir;
}

function resolveIn(dir, fromRelFile, spec) {
  const repoMode = detectRepoMode(dir);
  const aliasMap = buildAliasMap(dir, repoMode);
  const resolve = makeResolver(dir, aliasMap);
  const r = resolve(path.join(dir, fromRelFile), spec);
  return r;
}

let passed = 0, failed = 0;
function eq(label, actual, expected) {
  if (actual === expected) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    console.log(`        got:      ${JSON.stringify(actual)}`);
    console.log(`        expected: ${JSON.stringify(expected)}`);
  }
}

// ── Case A: simple "./*" wildcard to src/ ─────────────────────────
{
  const dir = mkFx('a-star', {
    name: '@scope/a',
    type: 'module',
    exports: { './*': './src/*.ts' },
  }, {
    'src/leaf.ts': 'export const L = 1;',
    'src/consumer.ts': `import { L } from '@scope/a/leaf'; export const x = L;`,
  });
  eq('A1. "./*" resolves @scope/a/leaf to src/leaf.ts',
    resolveIn(dir, 'src/consumer.ts', '@scope/a/leaf'),
    path.join(dir, 'src/leaf.ts'));
}

// ── Case B: subpath "./features/*" ────────────────────────────────
{
  const dir = mkFx('b-sub', {
    name: '@scope/b',
    type: 'module',
    exports: { './features/*': './src/features/*.ts' },
  }, {
    'src/features/alpha.ts': 'export const ALPHA = 1;',
    'src/consumer.ts': `import { ALPHA } from '@scope/b/features/alpha'; export const x = ALPHA;`,
  });
  eq('B1. "./features/*" resolves @scope/b/features/alpha',
    resolveIn(dir, 'src/consumer.ts', '@scope/b/features/alpha'),
    path.join(dir, 'src/features/alpha.ts'));
}

// ── Case C: multiple wildcards, deepest prefix wins ───────────────
{
  const dir = mkFx('c-multi', {
    name: '@scope/c',
    type: 'module',
    exports: {
      './*': './src/*.ts',
      './features/*': './src/features/*.ts',
    },
  }, {
    'src/root-thing.ts': 'export const R = 1;',
    'src/features/specific.ts': 'export const S = 2;',
    'src/consumer.ts': 'export const x = 0;',
  });
  eq('C1. bare spec resolves via "./*"',
    resolveIn(dir, 'src/consumer.ts', '@scope/c/root-thing'),
    path.join(dir, 'src/root-thing.ts'));
  eq('C2. features spec prefers "./features/*" over "./*"',
    resolveIn(dir, 'src/consumer.ts', '@scope/c/features/specific'),
    path.join(dir, 'src/features/specific.ts'));
}

// ── Case D: dist/ target remaps to src/ and .js → .ts ─────────────
{
  const dir = mkFx('d-dist', {
    name: '@scope/d',
    type: 'module',
    exports: { './*': './dist/*.js' },
  }, {
    'src/worker.ts': 'export const W = 1;',
    'src/consumer.ts': 'export const x = 0;',
  });
  eq('D1. "./*" → "./dist/*.js" remaps to src/*.ts',
    resolveIn(dir, 'src/consumer.ts', '@scope/d/worker'),
    path.join(dir, 'src/worker.ts'));
}

// ── Case E: nested subpath "./ui/components/*" ────────────────────
{
  const dir = mkFx('e-nested', {
    name: '@scope/e',
    type: 'module',
    exports: { './ui/components/*': './src/ui/components/*.ts' },
  }, {
    'src/ui/components/button.ts': 'export const B = 1;',
    'src/consumer.ts': 'export const x = 0;',
  });
  eq('E1. deeply nested subpath wildcard works',
    resolveIn(dir, 'src/consumer.ts', '@scope/e/ui/components/button'),
    path.join(dir, 'src/ui/components/button.ts'));
}

// ── Case F: negative - unmatched spec must remain EXTERNAL ────────
{
  const dir = mkFx('f-neg', {
    name: '@scope/f',
    type: 'module',
    exports: { './features/*': './src/features/*.ts' },
  }, {
    'src/features/a.ts': 'export const A = 1;',
    'src/consumer.ts': 'export const x = 0;',
  });
  eq('F1. unrelated scoped package stays EXTERNAL',
    resolveIn(dir, 'src/consumer.ts', 'lodash'),
    'EXTERNAL');
  eq('F2. different pkg name stays EXTERNAL',
    resolveIn(dir, 'src/consumer.ts', '@other/pkg/features/x'),
    'EXTERNAL');
  eq('F3. matched exports wildcard with missing target → UNRESOLVED_INTERNAL',
    resolveIn(dir, 'src/consumer.ts', '@scope/f/features/missing'),
    'UNRESOLVED_INTERNAL');
}

// ── Case G: exact exports still work (regression) ────────────────
{
  const dir = mkFx('g-exact', {
    name: '@scope/g',
    type: 'module',
    exports: {
      '.': './src/index.ts',
      './specific': './src/specific.ts',
    },
  }, {
    'src/index.ts': 'export const I = 1;',
    'src/specific.ts': 'export const S = 1;',
    'src/consumer.ts': 'export const x = 0;',
  });
  eq('G1. exact "." still resolves',
    resolveIn(dir, 'src/consumer.ts', '@scope/g'),
    path.join(dir, 'src/index.ts'));
  eq('G2. exact "./specific" still resolves',
    resolveIn(dir, 'src/consumer.ts', '@scope/g/specific'),
    path.join(dir, 'src/specific.ts'));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
