// Regression guard for Node.js `#imports` subpath resolution (FP-03).
//
// v1.10.1 drift fix: the `#imports` branch in `_lib/alias-map.mjs` used
// to roll its own narrow path mapping:
//   `.replace(/^\.\//, '').replace(/^dist\//, 'src/').replace(/\.js$/, '.ts')`
// The `exports` branch used the richer `mapOutputToSource` which
// swaps `.mjs/.cjs/.js → .ts`, `.jsx → .tsx`, and covers six source-
// dir conventions (src/ source/ lib/ build/ out/ es/ esm/). So
// `#foo` pointing at `./dist/foo.mjs` resolved correctly for the
// `exports` side but NOT for the `imports` side — same bug class as
// FP-40 on the exports side.
//
// This suite pins the fix: every `.mjs` / `.cjs` / `.jsx` suffix and
// every out-dir convention resolves on both shapes (exact key +
// wildcard key `/* */`).

import {
  writeFileSync, mkdirSync, rmSync, mkdtempSync, readFileSync,
} from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { detectRepoMode } from '../_lib/repo-mode.mjs';
import { buildAliasMap } from '../_lib/alias-map.mjs';
import { makeResolver } from '../_lib/resolver-core.mjs';

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

let passed = 0, failed = 0;
function eq(label, actual, expected) {
  if (actual === expected) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    console.log(`        got:      ${actual}`);
    console.log(`        expected: ${expected}`);
  }
}

function mkFixture(name, pkgJson, files) {
  const root = mkdtempSync(path.join(tmpdir(), `hash-imp-${name}-`));
  writeFileSync(path.join(root, 'package.json'), JSON.stringify(pkgJson, null, 2));
  for (const [rel, content] of Object.entries(files)) {
    const full = path.join(root, rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, content);
  }
  return root;
}

function resolveIn(root, consumer, spec) {
  const mode = detectRepoMode(root);
  const map = buildAliasMap(root, mode);
  const resolve = makeResolver(root, map);
  return resolve(path.join(root, consumer), spec);
}

// ── A. Exact `#foo` with `.mjs` output (the FP-40-class gap) ──
{
  const root = mkFixture('exact-mjs', {
    name: 'hash-exact-mjs',
    type: 'module',
    imports: { '#entry': './dist/entry.mjs' },
  }, {
    'src/entry.ts': 'export const E = 1;\n',
    'src/consumer.ts': `import { E } from '#entry'; export const c = E;\n`,
  });
  try {
    eq('A1. `#entry` → `./dist/entry.mjs` swaps to `src/entry.ts` via mapOutputToSource',
      resolveIn(root, 'src/consumer.ts', '#entry'),
      path.join(root, 'src/entry.ts'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── B. Exact `#foo` with `.cjs` ──
{
  const root = mkFixture('exact-cjs', {
    name: 'hash-exact-cjs',
    type: 'module',
    imports: { '#util': './dist/util.cjs' },
  }, {
    'src/util.ts': 'export const U = 1;\n',
    'src/consumer.ts': `import { U } from '#util'; export const c = U;\n`,
  });
  try {
    eq('B1. `#util` → `./dist/util.cjs` swaps to `src/util.ts`',
      resolveIn(root, 'src/consumer.ts', '#util'),
      path.join(root, 'src/util.ts'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── C. Exact `#foo` with non-dist source-dir convention (`lib/`) ──
{
  const root = mkFixture('exact-lib', {
    name: 'hash-exact-lib',
    type: 'module',
    imports: { '#helpers': './dist/helpers.js' },
  }, {
    'lib/helpers.ts': 'export const H = 1;\n',
    'lib/consumer.ts': `import { H } from '#helpers'; export const c = H;\n`,
  });
  try {
    eq('C1. `#helpers` → `./dist/helpers.js` resolves to `lib/helpers.ts` (OUT_SRC_PAIRS)',
      resolveIn(root, 'lib/consumer.ts', '#helpers'),
      path.join(root, 'lib/helpers.ts'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── D. Baseline — plain `.js` still works (no regression) ──
{
  const root = mkFixture('exact-js', {
    name: 'hash-exact-js',
    type: 'module',
    imports: { '#legacy': './dist/legacy.js' },
  }, {
    'src/legacy.ts': 'export const L = 1;\n',
    'src/consumer.ts': `import { L } from '#legacy'; export const c = L;\n`,
  });
  try {
    eq('D1. baseline `.js` target still resolves',
      resolveIn(root, 'src/consumer.ts', '#legacy'),
      path.join(root, 'src/legacy.ts'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── D2. Exact `#foo` with `.jsx → .tsx` ─────────────────
{
  const root = mkFixture('exact-jsx', {
    name: 'hash-exact-jsx',
    type: 'module',
    imports: { '#button': './dist/ui/Button.jsx' },
  }, {
    'src/ui/Button.tsx': 'export const Button = () => null;\n',
    'src/consumer.tsx': `import { Button } from '#button'; export const c = Button;\n`,
  });
  try {
    eq('D2. exact `.jsx` target swaps to authored `.tsx` source',
      resolveIn(root, 'src/consumer.tsx', '#button'),
      path.join(root, 'src/ui/Button.tsx'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── E. Wildcard `#foo/*` with `.mjs` pattern ──
{
  const root = mkFixture('wild-mjs', {
    name: 'hash-wild-mjs',
    type: 'module',
    imports: { '#feat/*': './dist/features/*.mjs' },
  }, {
    'src/features/alpha.ts': 'export const A = 1;\n',
    'src/features/beta.ts':  'export const B = 1;\n',
    'src/consumer.ts': `import { A } from '#feat/alpha'; export const c = A;\n`,
  });
  try {
    eq('E1. `#feat/alpha` → `./dist/features/*.mjs` pattern swaps to `src/features/alpha.ts`',
      resolveIn(root, 'src/consumer.ts', '#feat/alpha'),
      path.join(root, 'src/features/alpha.ts'));
    eq('E2. a different subpath from the same wildcard also resolves',
      resolveIn(root, 'src/consumer.ts', '#feat/beta'),
      path.join(root, 'src/features/beta.ts'));
    eq('E3. unsuffixed wildcard strips runtime `.js` tail before probing source `.ts`',
      resolveIn(root, 'src/consumer.ts', '#feat/alpha.js'),
      path.join(root, 'src/features/alpha.ts'));
    eq('E4. matched #imports wildcard with missing target → UNRESOLVED_INTERNAL',
      resolveIn(root, 'src/consumer.ts', '#feat/gamma'),
      'UNRESOLVED_INTERNAL');
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── F. Wildcard `#foo/*` with `.jsx → .tsx` ──
{
  const root = mkFixture('wild-jsx', {
    name: 'hash-wild-jsx',
    type: 'module',
    imports: { '#ui/*': './dist/ui/*.jsx' },
  }, {
    'src/ui/Button.tsx': 'export const B = 1;\n',
    'src/consumer.ts':  `import { B } from '#ui/Button'; export const c = B;\n`,
  });
  try {
    eq('F1. `.jsx` pattern swaps to `.tsx` (was dropped by old narrow regex)',
      resolveIn(root, 'src/consumer.ts', '#ui/Button'),
      path.join(root, 'src/ui/Button.tsx'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── G. Wildcard `#foo/*.js` with suffix-preserving key ──
{
  const root = mkFixture('wild-suffix-js', {
    name: 'hash-wild-suffix-js',
    type: 'module',
    imports: { '#web/request/*.js': './src/adapter/web/request/*.ts' },
  }, {
    'src/adapter/web/request/project-scope.ts':
      'export interface ProjectScopeRegistry { ok: boolean; }\\n' +
      'export function readProjectScope(): ProjectScopeRegistry { return { ok: true }; }\\n',
    'src/consumer.ts':
      "import { readProjectScope, type ProjectScopeRegistry } from '#web/request/project-scope.js';\\n" +
      'export const value = readProjectScope();\\n' +
      'export type ConsumerScope = ProjectScopeRegistry;\\n',
  });
  try {
    eq('G1. `#web/request/project-scope.js` suffix wildcard resolves to authored TS',
      resolveIn(root, 'src/consumer.ts', '#web/request/project-scope.js'),
      path.join(root, 'src/adapter/web/request/project-scope.ts'));

    const outDir = path.join(root, 'out');
    const run = spawnSync(process.execPath, [
      path.join(REPO_ROOT, 'build-symbol-graph.mjs'),
      '--root', root,
      '--output', outDir,
    ], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
    });
    eq('G2. suffix wildcard fixture graph build exits 0',
      run.status, 0);

    const symbols = JSON.parse(readFileSync(path.join(outDir, 'symbols.json'), 'utf8'));
    const dead = new Set((symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`));
    eq('G3. type-only consumer through suffix wildcard protects interface export',
      dead.has('src/adapter/web/request/project-scope.ts::ProjectScopeRegistry'),
      false);
    eq('G4. value consumer through suffix wildcard protects function export',
      dead.has('src/adapter/web/request/project-scope.ts::readProjectScope'),
      false);
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── H. mapOutputPatternToSource unit cases ──
{
  // Import directly to pin the helper's behavior — wildcard callers
  // rely on specific rewrites.
  const {
    mapOutputPatternToSource,
    mapOutputPatternToSourceCandidates,
  } = await import('../_lib/alias-map.mjs');

  eq('H1. dist/*.mjs → src/*.ts',
    mapOutputPatternToSource('./dist/*.mjs'), 'src/*.ts');
  eq('H2. dist/*.cjs → src/*.ts',
    mapOutputPatternToSource('./dist/*.cjs'), 'src/*.ts');
  eq('H3. dist/*.js → src/*.ts (backward compat)',
    mapOutputPatternToSource('./dist/*.js'), 'src/*.ts');
  eq('H4. dist/*.jsx → src/*.tsx',
    mapOutputPatternToSource('./dist/*.jsx'), 'src/*.tsx');
  eq('H5. lib/ convention unchanged (no OUT match → leave as-is)',
    mapOutputPatternToSource('./lib/*.mjs'), 'lib/*.ts');
  eq('H6. esm/ → src/',
    mapOutputPatternToSource('./esm/features/*.mjs'), 'src/features/*.ts');
  eq('H7. plain target without out-dir still strips `./` and swaps suffix',
    mapOutputPatternToSource('./pattern.mjs'), 'pattern.ts');
  eq('H8. candidate patterns retain authored JS sources before TS fallbacks',
    JSON.stringify(mapOutputPatternToSourceCandidates('./dist/features/*.js').slice(0, 3)),
    JSON.stringify(['src/features/*.js', 'src/features/*.ts', 'src/features/*.tsx']));
}

// ── H2. Hash wildcard with authored JS source ───────────────────
{
  const root = mkFixture('wild-js-source', {
    name: 'hash-wild-js-source',
    type: 'module',
    imports: { '#internal/*': './src/internal/*.js' },
  }, {
    'src/internal/util.js': 'export const used = 1;\n',
    'src/consumer.js': `import { used } from '#internal/util'; export const c = used;\n`,
  });
  try {
    eq('H2-1. #imports wildcard preserves authored JS source target',
      resolveIn(root, 'src/consumer.js', '#internal/util'),
      path.join(root, 'src/internal/util.js'));
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── H3. Hash wildcard directory target resolves to index file ─────
{
  const root = mkFixture('wild-dir-index', {
    name: 'hash-wild-dir-index',
    type: 'module',
    imports: { '#internal/*': './src/internal/*' },
  }, {
    'src/internal/util/index.js': 'export const used = 1;\n',
    'src/consumer.js': `import { used } from '#internal/util'; export const c = used;\n`,
  });
  try {
    eq('H3-1. #imports wildcard directory target resolves to index.js, not directory path',
      resolveIn(root, 'src/consumer.js', '#internal/util'),
      path.join(root, 'src/internal/util/index.js'));

    const outDir = path.join(root, 'out');
    const run = spawnSync(process.execPath, [
      path.join(REPO_ROOT, 'build-symbol-graph.mjs'),
      '--root', root,
      '--output', outDir,
    ], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
    });
    eq('H3-2. directory target fixture graph build exits 0',
      run.status, 0);

    const symbols = JSON.parse(readFileSync(path.join(outDir, 'symbols.json'), 'utf8'));
    const dead = new Set((symbols.deadProdList ?? []).map((d) => `${d.file}::${d.symbol}`));
    eq('H3-3. import through directory index protects exported symbol',
      dead.has('src/internal/util/index.js::used'),
      false);
  } finally { rmSync(root, { recursive: true, force: true }); }
}

// ── I. buildAliasMap survives a malformed workspace package.json (E1 fix) ──
//
// Monorepo where ONE workspace has a truncated / BOM-mangled /
// comment-containing pkg.json. Before 2026-04-21 this aborted the entire
// alias-map build, poisoning resolver output for every unrelated workspace.
// Fix guards the JSON.parse with try/catch and skips the offending package.
{
  const root = mkdtempSync(path.join(tmpdir(), 'aliasmap-malformed-'));
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({
    name: 'root', private: true,
    workspaces: ['pkgs/good', 'pkgs/bad'],
  }));
  mkdirSync(path.join(root, 'pkgs', 'good', 'src'), { recursive: true });
  writeFileSync(path.join(root, 'pkgs', 'good', 'package.json'), JSON.stringify({
    name: '@m/good', exports: { '.': './src/index.ts' },
  }));
  writeFileSync(path.join(root, 'pkgs', 'good', 'src', 'index.ts'), 'export const ok = 1;\n');
  mkdirSync(path.join(root, 'pkgs', 'bad'), { recursive: true });
  writeFileSync(path.join(root, 'pkgs', 'bad', 'package.json'),
    '{ this is not valid JSON — could be a half-written edit }');

  const mode = detectRepoMode(root);
  // Must not throw; the good workspace must still be registered.
  let map, threw = false;
  try { map = buildAliasMap(root, mode); } catch { threw = true; }
  eq('I1. buildAliasMap does NOT throw on malformed workspace pkg.json',
    threw, false);

  // Resolver built from the (partial) alias map still resolves the good pkg.
  const resolve = makeResolver(root, map);
  const fromFile = path.join(root, 'pkgs', 'good', 'src', 'consumer.ts');
  writeFileSync(fromFile, 'export const x = 1;\n');
  const r = resolve(fromFile, '@m/good');
  eq('I2. good workspace still resolves despite sibling malformed pkg.json',
    r, path.join(root, 'pkgs', 'good', 'src', 'index.ts'));

  rmSync(root, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
