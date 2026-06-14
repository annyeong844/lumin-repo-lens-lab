// Tests for extension-less relative import resolution.
// Guards a v1.3.0 → v1.3.1 regression: narrow extension table returned null
// for `./mod` → `mod.cjs`, `./view` → `view.jsx`, `./dir` → `dir/index.js`.
import {
  writeFileSync, mkdirSync, rmSync, mkdtempSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { detectRepoMode } from '../_lib/repo-mode.mjs';
import { buildAliasMap } from '../_lib/alias-map.mjs';
import {
  explainUnresolvedSpecifier,
  makeResolver,
  isGeneratedVirtualResolution,
  isNonSourceAssetResolution,
  isResolvedFile,
} from '../_lib/resolver-core.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// `/tmp/...` is Linux-only; use os.tmpdir() for Windows portability.
const ROOT = path.join(tmpdir(), 'fx-resolver-paths');

rmSync(ROOT, { recursive: true, force: true });
mkdirSync(path.join(ROOT, 'src'), { recursive: true });
mkdirSync(path.join(ROOT, 'dir'), { recursive: true });
mkdirSync(path.join(ROOT, 'cjs-dir'), { recursive: true });
mkdirSync(path.join(ROOT, 'decl-dir'), { recursive: true });
writeFileSync(path.join(ROOT, 'package.json'), JSON.stringify({
  name: 'fx',
  type: 'module',
  scripts: {
    tailwind: 'tailwindcss --input ./src/styles.css --output ./src/tailwind.generated.css',
  },
}));

// Target files with varied extensions
writeFileSync(path.join(ROOT, 'src/mod.cjs'), 'module.exports = 1;\n');
writeFileSync(path.join(ROOT, 'src/view.jsx'), 'export const V = () => 1;\n');
writeFileSync(path.join(ROOT, 'src/util.mts'), 'export const U = 1;\n');
writeFileSync(path.join(ROOT, 'src/conf.cts'), 'export const C = 1;\n');
writeFileSync(path.join(ROOT, 'src/types.d.ts'), 'export interface T {}\n');
writeFileSync(path.join(ROOT, 'src/embed.css'), '.embed { color: red; }\n');
writeFileSync(path.join(ROOT, 'src/embed-cache.css'), '.embed-cache { color: blue; }\n');
writeFileSync(path.join(ROOT, 'dir/index.js'), 'export const I = 1;\n');
writeFileSync(path.join(ROOT, 'cjs-dir/index.cjs'), 'module.exports = 1;\n');
writeFileSync(path.join(ROOT, 'decl-dir/index.d.ts'), 'export interface D {}\n');
// Consumer
writeFileSync(path.join(ROOT, 'src/consumer.ts'), 'export const x = 1;\n');

const mode = detectRepoMode(ROOT);
const resolve = makeResolver(ROOT, buildAliasMap(ROOT, mode));
const from = path.join(ROOT, 'src/consumer.ts');

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

// ── Extension-less file matches — regression guards ───────────
eq('T1. ./mod resolves to mod.cjs',
  resolve(from, './mod'), path.join(ROOT, 'src/mod.cjs'));
eq('T2. ./view resolves to view.jsx',
  resolve(from, './view'), path.join(ROOT, 'src/view.jsx'));
eq('T3. ./util resolves to util.mts',
  resolve(from, './util'), path.join(ROOT, 'src/util.mts'));
eq('T4. ./conf resolves to conf.cts',
  resolve(from, './conf'), path.join(ROOT, 'src/conf.cts'));
eq('T4b. ./types resolves to types.d.ts',
  resolve(from, './types'), path.join(ROOT, 'src/types.d.ts'));

// ── Directory with non-.ts index ──────────────────────────────
eq('T5. ../dir resolves to dir/index.js',
  resolve(from, '../dir'), path.join(ROOT, 'dir/index.js'));
eq('T6. ../cjs-dir resolves to cjs-dir/index.cjs',
  resolve(from, '../cjs-dir'), path.join(ROOT, 'cjs-dir/index.cjs'));
eq('T6b. ../decl-dir resolves to decl-dir/index.d.ts',
  resolve(from, '../decl-dir'), path.join(ROOT, 'decl-dir/index.d.ts'));

// ── Explicit extension still works (no regression) ────────────
eq('T7. ./mod.cjs explicit still works',
  resolve(from, './mod.cjs'), path.join(ROOT, 'src/mod.cjs'));
eq('T8. ./view.jsx explicit still works',
  resolve(from, './view.jsx'), path.join(ROOT, 'src/view.jsx'));

// ── Non-existent spec still returns null (not EXTERNAL) for relative ─
eq('T9. missing relative spec returns null',
  resolve(from, './nonexistent'), null);

// ── Bundler resource-query asset imports ────────────────────
eq('T9b. ./embed.css?inline resolves as non-source asset, not a JS file',
  isNonSourceAssetResolution(resolve(from, './embed.css?inline')), true);
eq('T9c. ./embed.css?inline is not a resolved source file',
  isResolvedFile(resolve(from, './embed.css?inline')), false);
const generatedAssetExplanation = explainUnresolvedSpecifier(
  ROOT,
  buildAliasMap(ROOT, mode),
  from,
  './tailwind.generated.css?inline',
);
eq('T9d. missing relative generated asset reports generated artifact reason',
  generatedAssetExplanation?.reason, 'workspace-generated-artifact-missing');
eq('T9e. missing relative generated asset preserves stripped target candidate',
  generatedAssetExplanation?.targetCandidates?.[0], 'src/tailwind.generated.css');
eq('T9f. missing relative generated asset cites package script output path',
  generatedAssetExplanation?.generatedArtifact?.evidence?.some((item) =>
    item.kind === 'script-output-path' &&
    item.field === 'scripts.tailwind' &&
    item.matched === 'src/tailwind.generated.css'), true);

// ── T10. isResolvedFile predicate — 4-way sentinel discrimination ─
// Callers of resolve() need to tell "real file path" from EXTERNAL /
// UNRESOLVED_INTERNAL / null. Without this predicate, classify-dead-exports
// and measure-topology passed UNRESOLVED_INTERNAL through as a path string
// (C-1/D-1 review finding, fixed 2026-04-20).
eq('T10a. isResolvedFile(absolute path) === true',
  isResolvedFile('/abs/path/file.ts'), true);
eq('T10b. isResolvedFile("EXTERNAL") === false',
  isResolvedFile('EXTERNAL'), false);
eq('T10c. isResolvedFile("UNRESOLVED_INTERNAL") === false',
  isResolvedFile('UNRESOLVED_INTERNAL'), false);
eq('T10d. isResolvedFile(null) === false',
  isResolvedFile(null), false);
eq('T10e. isResolvedFile(undefined) === false',
  isResolvedFile(undefined), false);
eq('T10f. isResolvedFile(123) === false (non-string)',
  isResolvedFile(123), false);

// ── T11. run-local memoization preserves every resolver result class ─────
function memoStatsOrFail(label, resolver) {
  const ok = typeof resolver.memoStats === 'function';
  eq(label, ok, true);
  return ok ? resolver.memoStats() : null;
}

function stageStatsOrFail(label, resolver) {
  const ok = typeof resolver.stageStats === 'function';
  eq(label, ok, true);
  return ok ? resolver.stageStats() : null;
}

const memoStart = memoStatsOrFail('T11a. resolver exposes run-local memo stats', resolve);
const stageStart = stageStatsOrFail('T11a2. resolver exposes run-local stage stats', resolve);
if (memoStart) {
  eq('T11b. repeated null relative miss returns null first',
    resolve(from, './memoized-missing'), null);
  eq('T11c. repeated null relative miss returns null from cache',
    resolve(from, './memoized-missing'), null);
  const memoAfterNull = resolve.memoStats();
  eq('T11d. repeated null result records one memo hit',
    memoAfterNull.hits - memoStart.hits, 1);
  eq('T11e. repeated null result records one memo miss',
    memoAfterNull.misses - memoStart.misses, 1);
  eq('T11f. repeated null result grows memo by one entry',
    memoAfterNull.size - memoStart.size, 1);

  const memoBeforeAsset = resolve.memoStats();
  eq('T11g. first non-source asset result remains asset sentinel',
    isNonSourceAssetResolution(resolve(from, './embed-cache.css?inline')), true);
  eq('T11h. cached non-source asset result remains asset sentinel',
    isNonSourceAssetResolution(resolve(from, './embed-cache.css?inline')), true);
  const memoAfterAsset = resolve.memoStats();
  eq('T11i. repeated asset sentinel records one memo hit',
    memoAfterAsset.hits - memoBeforeAsset.hits, 1);
  eq('T11j. repeated asset sentinel records one memo miss',
    memoAfterAsset.misses - memoBeforeAsset.misses, 1);
}
if (stageStart) {
  const stageAfter = resolve.stageStats();
  eq('T11u. stage stats include relative attempts',
    stageAfter.relative.attempts > stageStart.relative.attempts, true);
  eq('T11v. stage stats include relative terminal results',
    stageAfter.relative.terminalResults > stageStart.relative.terminalResults, true);
  eq('T11w. stage stats include relative wall time',
    typeof stageAfter.relative.wallMs === 'number', true);
  eq('T11x. stage stats include memo hit count',
    stageAfter.memoHit.count >= stageStart.memoHit.count, true);
}

// ── T11y-T11ad. scoped baseUrl probe cache crosses importer files ─────
{
  const BASEURL_CACHE_ROOT = mkdtempSync(path.join(tmpdir(), 'fx-baseurl-probe-cache-'));
  try {
    mkdirSync(path.join(BASEURL_CACHE_ROOT, 'app'), { recursive: true });
    writeFileSync(path.join(BASEURL_CACHE_ROOT, 'package.json'),
      JSON.stringify({ name: 'fx-baseurl-probe-cache', type: 'module' }));
    writeFileSync(path.join(BASEURL_CACHE_ROOT, 'tsconfig.json'), JSON.stringify({
      compilerOptions: { baseUrl: '.' },
      include: ['app/**/*.ts'],
    }));
    writeFileSync(path.join(BASEURL_CACHE_ROOT, 'app/_types.ts'),
      'export interface PageProps { slug: string }\n');
    writeFileSync(path.join(BASEURL_CACHE_ROOT, 'app/a.ts'),
      "import type { PageProps } from 'app/_types';\nexport type A = PageProps;\n");
    writeFileSync(path.join(BASEURL_CACHE_ROOT, 'app/b.ts'),
      "import type { PageProps } from 'app/_types';\nexport type B = PageProps;\n");

    const baseUrlMode = detectRepoMode(BASEURL_CACHE_ROOT);
    const baseUrlResolve = makeResolver(BASEURL_CACHE_ROOT, buildAliasMap(BASEURL_CACHE_ROOT, baseUrlMode));
    const aFile = path.join(BASEURL_CACHE_ROOT, 'app/a.ts');
    const bFile = path.join(BASEURL_CACHE_ROOT, 'app/b.ts');
    const baseUrlStageStart = baseUrlResolve.stageStats();

    eq('T11y. first baseUrl local import resolves',
      baseUrlResolve(aFile, 'app/_types'),
      path.join(BASEURL_CACHE_ROOT, 'app/_types.ts'));
    eq('T11z. second baseUrl local import from another file resolves',
      baseUrlResolve(bFile, 'app/_types'),
      path.join(BASEURL_CACHE_ROOT, 'app/_types.ts'));
    eq('T11aa. scoped baseUrl caches resolved probes across importer files',
      (baseUrlResolve.stageStats().scopedBaseUrl.cacheHits ?? 0) -
        (baseUrlStageStart.scopedBaseUrl.cacheHits ?? 0) >= 1,
      true);
    eq('T11aa2. scoped baseUrl records first resolved probe cache miss',
      (baseUrlResolve.stageStats().scopedBaseUrl.cacheMisses ?? 0) -
        (baseUrlStageStart.scopedBaseUrl.cacheMisses ?? 0),
      1);

    const externalStageStart = baseUrlResolve.stageStats();
    eq('T11ab. first repeated external package remains external under baseUrl',
      baseUrlResolve(aFile, 'react'), 'EXTERNAL');
    eq('T11ac. second repeated external package remains external under baseUrl',
      baseUrlResolve(bFile, 'react'), 'EXTERNAL');
    eq('T11ad. scoped baseUrl caches no-match probes across importer files',
      (baseUrlResolve.stageStats().scopedBaseUrl.cacheHits ?? 0) -
        (externalStageStart.scopedBaseUrl.cacheHits ?? 0) >= 1,
      true);
    eq('T11ae. scoped baseUrl records first no-match probe cache miss',
      (baseUrlResolve.stageStats().scopedBaseUrl.cacheMisses ?? 0) -
        (externalStageStart.scopedBaseUrl.cacheMisses ?? 0),
      1);
  } finally {
    rmSync(BASEURL_CACHE_ROOT, { recursive: true, force: true });
  }
}

const unresolvedAliasMap = new Map([
  ['@missing/internal', {
    type: 'exact',
    path: path.join(ROOT, 'src/missing-internal.ts'),
    source: 'test',
  }],
]);
const resolveUnresolvedAlias = makeResolver(ROOT, unresolvedAliasMap);
const unresolvedMemoStart = memoStatsOrFail('T11k. exact-alias resolver exposes memo stats', resolveUnresolvedAlias);
if (unresolvedMemoStart) {
  eq('T11l. first unresolved internal alias preserves sentinel',
    resolveUnresolvedAlias(from, '@missing/internal'), 'UNRESOLVED_INTERNAL');
  eq('T11m. cached unresolved internal alias preserves sentinel',
    resolveUnresolvedAlias(from, '@missing/internal'), 'UNRESOLVED_INTERNAL');
  const unresolvedMemoAfter = resolveUnresolvedAlias.memoStats();
  eq('T11n. unresolved internal alias records one memo hit',
    unresolvedMemoAfter.hits - unresolvedMemoStart.hits, 1);
  eq('T11o. unresolved internal alias records one memo miss',
    unresolvedMemoAfter.misses - unresolvedMemoStart.misses, 1);
}

const virtualAliasMap = new Map([
  ['@virtual/*', {
    type: 'wildcard',
    matchPrefix: '@virtual/',
    matchSuffix: '',
    targetPattern: './generated/*',
    pkgDir: ROOT,
    pkgName: '@virtual',
    source: 'test',
    generatedVirtualSurfaces: [{
      id: 'generated-virtual:test:enums',
      source: 'generated-virtual',
      virtual: true,
      runtimeEquivalence: false,
      targetSubpath: 'enums',
      exports: [{ name: 'GeneratedEnum', spaces: ['value', 'type'] }],
    }],
  }],
]);
const resolveVirtual = makeResolver(ROOT, virtualAliasMap);
const virtualMemoStart = memoStatsOrFail('T11p. generated-virtual resolver exposes memo stats', resolveVirtual);
if (virtualMemoStart) {
  const virtualFirst = resolveVirtual(from, '@virtual/enums');
  const virtualSecond = resolveVirtual(from, '@virtual/enums');
  const virtualMemoAfter = resolveVirtual.memoStats();
  eq('T11q. first generated virtual result remains virtual',
    isGeneratedVirtualResolution(virtualFirst), true);
  eq('T11r. cached generated virtual result preserves object identity',
    virtualSecond === virtualFirst, true);
  eq('T11s. generated virtual result records one memo hit',
    virtualMemoAfter.hits - virtualMemoStart.hits, 1);
  eq('T11t. generated virtual result records one memo miss',
    virtualMemoAfter.misses - virtualMemoStart.misses, 1);
}

// ── T12. wildcard alias stage cache crosses importer files ─────
{
  const siblingFrom = path.join(ROOT, 'src/other-consumer.ts');
  writeFileSync(path.join(ROOT, 'src/wild-target.ts'), 'export const WildTarget = 1;\n');

  const wildcardAliasMap = new Map([
    ['@wild/*', {
      type: 'wildcard',
      matchPrefix: '@wild/',
      matchSuffix: '',
      targetPattern: './src/*',
      pkgDir: ROOT,
      pkgName: '@wild',
      source: 'test',
    }],
  ]);
  const resolveWildcardAlias = makeResolver(ROOT, wildcardAliasMap);

  const resolvedStageStart = resolveWildcardAlias.stageStats();
  eq('T12a. first wildcard alias resolved lookup succeeds',
    resolveWildcardAlias(from, '@wild/wild-target'), path.join(ROOT, 'src/wild-target.ts'));
  eq('T12b. second wildcard alias resolved lookup from sibling succeeds',
    resolveWildcardAlias(siblingFrom, '@wild/wild-target'), path.join(ROOT, 'src/wild-target.ts'));
  const resolvedStageAfter = resolveWildcardAlias.stageStats();
  eq('T12c. wildcard alias caches resolved probes across importer files',
    resolvedStageAfter.wildcardAlias.cacheHits - resolvedStageStart.wildcardAlias.cacheHits >= 1,
    true);
  eq('T12d. wildcard alias records first resolved probe cache miss',
    resolvedStageAfter.wildcardAlias.cacheMisses - resolvedStageStart.wildcardAlias.cacheMisses,
    1);

  const noMatchStageStart = resolveWildcardAlias.stageStats();
  eq('T12e. first wildcard no-match falls through to external',
    resolveWildcardAlias(from, 'react'), 'EXTERNAL');
  eq('T12f. second wildcard no-match from sibling still falls through to external',
    resolveWildcardAlias(siblingFrom, 'react'), 'EXTERNAL');
  const noMatchStageAfter = resolveWildcardAlias.stageStats();
  eq('T12g. wildcard alias caches no-match probes across importer files',
    noMatchStageAfter.wildcardAlias.cacheHits - noMatchStageStart.wildcardAlias.cacheHits >= 1,
    true);
  eq('T12h. wildcard alias records first no-match probe cache miss',
    noMatchStageAfter.wildcardAlias.cacheMisses - noMatchStageStart.wildcardAlias.cacheMisses,
    1);

  const unresolvedStageStart = resolveWildcardAlias.stageStats();
  eq('T12i. first missing wildcard alias remains unresolved internal',
    resolveWildcardAlias(from, '@wild/missing'), 'UNRESOLVED_INTERNAL');
  eq('T12j. cached missing wildcard alias remains unresolved internal',
    resolveWildcardAlias(siblingFrom, '@wild/missing'), 'UNRESOLVED_INTERNAL');
  const unresolvedStageAfter = resolveWildcardAlias.stageStats();
  eq('T12k. wildcard alias caches unresolved probes across importer files',
    unresolvedStageAfter.wildcardAlias.cacheHits - unresolvedStageStart.wildcardAlias.cacheHits >= 1,
    true);
}

{
  const siblingFrom = path.join(ROOT, 'src/virtual-consumer.ts');
  const resolveVirtualStage = makeResolver(ROOT, virtualAliasMap);
  const virtualStageStart = resolveVirtualStage.stageStats();
  const virtualFirst = resolveVirtualStage(from, '@virtual/enums');
  const virtualSecond = resolveVirtualStage(siblingFrom, '@virtual/enums');
  const virtualStageAfter = resolveVirtualStage.stageStats();
  eq('T12l. wildcard generated virtual lookup remains virtual',
    isGeneratedVirtualResolution(virtualFirst), true);
  eq('T12m. wildcard generated virtual stage cache preserves object identity across importer files',
    virtualSecond === virtualFirst, true);
  eq('T12n. wildcard generated virtual cached object is frozen',
    Object.isFrozen(virtualSecond), true);
  let mutationBlocked = false;
  try {
    virtualSecond.aliasSource = 'mutated';
  } catch {
    mutationBlocked = true;
  }
  eq('T12o. frozen wildcard generated virtual result rejects mutation',
    mutationBlocked, true);
  eq('T12p. wildcard alias caches generated virtual probes across importer files',
    virtualStageAfter.wildcardAlias.cacheHits - virtualStageStart.wildcardAlias.cacheHits >= 1,
    true);
  eq('T12q. wildcard alias records first generated virtual probe cache miss',
    virtualStageAfter.wildcardAlias.cacheMisses - virtualStageStart.wildcardAlias.cacheMisses,
    1);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
