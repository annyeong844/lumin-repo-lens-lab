// Tests for `collectHelperIdentities` + `renderHelperRegistry` — P3-2 Step 2.
//
// Dependency-injection style: we supply a controlled `extractFn` and
// `resolveSpecifier` to the aggregator so these tests exercise the
// aggregation / fan-in / classification path WITHOUT spinning up the
// full AST parser + resolver. End-to-end integration (real extractor +
// real resolver over fixture repos) lives in `test-canon-draft-integration-helpers.mjs`.
//
// Pinning rules from docs/history/phases/p3/p3-2.md v2 §5.3:
//   - Fresh-AST defines inventory (PF-3).
//   - Consumer-file-count fan-in (PF-4).
//   - Exported-never-called helpers appear as zero-internal-fan-in-helper.
//   - Callback-passed import still counts (import-resolve lens).
//   - Call-site count within one consumer → fan-in 1, not N.
//   - Re-export chain: terminal identity is owner, not barrel.
//   - Alias hop: identity = source owner, NOT consumer-side alias.
//   - Group vs single classification wiring.
//   - Contamination unavailable → `Any / unknown signal` renders `—` on every row.
//   - call-graph.json cross-check diagnostics + staleness warning.

import {
  HELPER_OWNER_KINDS,
  UNCERTAIN_REASONS,
} from '../_lib/canon-draft-utils.mjs';
import {
  collectHelperIdentities,
  renderHelperRegistry,
} from '../_lib/canon-draft-helpers.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── DI helpers: extractFn / resolveSpecifier builders ─────

/**
 * Build an extractFn that returns pre-canned {defs, uses} per file.
 * Paths map lookup uses absolute paths (we use synthetic /fx/...).
 */
function makeExtractFn(perFile) {
  return function extract(absFile) {
    const entry = perFile.get(absFile);
    if (!entry) return { defs: [], uses: [], reExports: [] };
    return entry;
  };
}

/**
 * Build a resolveSpecifier that returns a pre-canned map
 * of (consumerAbs, spec) → resolvedAbs | null.
 */
function makeResolver(resolves) {
  return function resolve(fromFile, spec) {
    return resolves.get(`${fromFile}|${spec}`) ?? null;
  };
}

const ROOT = '/fx';

// ═══ I1. Single owner, single consumer → shared-helper (fan-in 1) ═══

{
  const owner = '/fx/src/util.ts';
  const consumer = '/fx/src/consumer.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'parseJson', kind: 'FunctionDeclaration', line: 3 }], uses: [], reExports: [] }],
    [consumer, { defs: [], uses: [{ fromSpec: './util', name: 'parseJson', kind: 'import', typeOnly: false }], reExports: [] }],
  ]);
  const resolves = new Map([[`${consumer}|./util`, owner]]);
  const result = collectHelperIdentities({
    files: [owner, consumer],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  const identity = 'src/util.ts::parseJson';
  assert('I1a. helper inventory contains src/util.ts::parseJson',
    result.helperDefsByIdentity.has(identity));
  assert('I1b. fanIn=1 (one distinct consumer file)',
    result.helperDefsByIdentity.get(identity)?.fanIn === 1);
  assert('I1c. meta.helperContamination = "unavailable"',
    result.meta.helperContamination === 'unavailable');
}

// ═══ I2. Call-site count vs consumer-file count (PF-4) ═══

{
  // One consumer file imports parseJson ONCE (meta); even if it called
  // parseJson() 10 times in its body, our lens counts 1 consumer file.
  // The fixture proves the import-resolve lens ignores call-site multiplicity.
  const owner = '/fx/src/util.ts';
  const consumer = '/fx/src/c.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'parseJson', kind: 'FunctionDeclaration', line: 3 }], uses: [], reExports: [] }],
    [consumer, { defs: [], uses: [
      // A single import — regardless of call-site count in the body
      { fromSpec: './util', name: 'parseJson', kind: 'import', typeOnly: false },
    ], reExports: [] }],
  ]);
  const resolves = new Map([[`${consumer}|./util`, owner]]);
  const result = collectHelperIdentities({
    files: [owner, consumer],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  const identity = 'src/util.ts::parseJson';
  assert('I2. fan-in=1 from one consumer file regardless of inline call-site count (PF-4)',
    result.helperDefsByIdentity.get(identity)?.fanIn === 1);
}

// ═══ I3. Exported-never-called → zero-internal-fan-in-helper (PF-3) ═══

{
  const owner = '/fx/src/public.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'unusedButPublic', kind: 'FunctionDeclaration', line: 5 }], uses: [], reExports: [] }],
  ]);
  const resolves = new Map();
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  const identity = 'src/public.ts::unusedButPublic';
  assert('I3a. exported-never-called helper present in inventory',
    result.helperDefsByIdentity.has(identity));
  assert('I3b. its fanIn = 0',
    result.helperDefsByIdentity.get(identity)?.fanIn === 0);
}

// ═══ I4. Duplicate across files → group-classified ═══

{
  const ownerA = '/fx/src/a.ts';
  const ownerB = '/fx/src/b.ts';
  const consumer = '/fx/src/c.ts';
  const perFile = new Map([
    [ownerA, { defs: [{ name: 'fetch', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [ownerB, { defs: [{ name: 'fetch', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [consumer, { defs: [], uses: [
      { fromSpec: './a', name: 'fetch', kind: 'import', typeOnly: false },
      { fromSpec: './b', name: 'fetch', kind: 'import', typeOnly: false },
    ], reExports: [] }],
  ]);
  const resolves = new Map([
    [`${consumer}|./a`, ownerA],
    [`${consumer}|./b`, ownerB],
  ]);
  const result = collectHelperIdentities({
    files: [ownerA, ownerB, consumer],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  assert('I4a. helpersByName tracks `fetch` with 2 identities',
    result.helpersByName.get('fetch')?.length === 2);
  // Render should emit HELPER_LOCAL_COMMON (name ∈ LOW_INFO_HELPER_NAMES, maxFanIn < 3)
  const md = renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: { scope: 'TS/JS including tests', helperContamination: 'unavailable' },
  });
  assert('I4b. rendered draft includes HELPER_LOCAL_COMMON (fetch is low-info, fanIn 1 each)',
    md.includes('HELPER_LOCAL_COMMON'), `md=${md.slice(0, 400)}`);
}

// ═══ I5. Same-file self-import is NOT counted ═══

{
  const owner = '/fx/src/x.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'helper', kind: 'FunctionDeclaration', line: 2 }],
              uses: [{ fromSpec: './x', name: 'helper', kind: 'import', typeOnly: false }],
              reExports: [] }],
  ]);
  const resolves = new Map([[`${owner}|./x`, owner]]); // self-resolve
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  assert('I5. self-import NOT counted in fanIn (same-file self-reference collapsed)',
    result.helperDefsByIdentity.get('src/x.ts::helper')?.fanIn === 0);
}

// ═══ I6. typeOnly imports don't contribute to helper fan-in ═══

{
  const owner = '/fx/src/types.ts';
  const consumer = '/fx/src/c.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'mkLogger', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [consumer, { defs: [], uses: [
      { fromSpec: './types', name: 'mkLogger', kind: 'import', typeOnly: true }, // type-only → should not count
    ], reExports: [] }],
  ]);
  const resolves = new Map([[`${consumer}|./types`, owner]]);
  const result = collectHelperIdentities({
    files: [owner, consumer],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  assert('I6. type-only import does NOT contribute to helper fan-in',
    result.helperDefsByIdentity.get('src/types.ts::mkLogger')?.fanIn === 0);
}

// ═══ I7. Multiple consumers → central-helper (fan-in ≥ 3) ═══

{
  const owner = '/fx/src/u.ts';
  const c1 = '/fx/src/c1.ts', c2 = '/fx/src/c2.ts', c3 = '/fx/src/c3.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'doWork', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [c1, { defs: [], uses: [{ fromSpec: './u', name: 'doWork', kind: 'import', typeOnly: false }], reExports: [] }],
    [c2, { defs: [], uses: [{ fromSpec: './u', name: 'doWork', kind: 'import', typeOnly: false }], reExports: [] }],
    [c3, { defs: [], uses: [{ fromSpec: './u', name: 'doWork', kind: 'import', typeOnly: false }], reExports: [] }],
  ]);
  const resolves = new Map([
    [`${c1}|./u`, owner], [`${c2}|./u`, owner], [`${c3}|./u`, owner],
  ]);
  const result = collectHelperIdentities({
    files: [owner, c1, c2, c3],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  const def = result.helperDefsByIdentity.get('src/u.ts::doWork');
  assert('I7a. 3 distinct consumers → fanIn=3',
    def?.fanIn === 3);
  const md = renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: { scope: 'TS/JS including tests' },
  });
  assert('I7b. draft row status contains central-helper',
    md.includes('central-helper'), `md=${md.slice(0, 400)}`);
}

// ═══ I8. Unknown kind (class method) filtered out ═══

{
  const owner = '/fx/src/k.ts';
  const perFile = new Map([
    [owner, { defs: [
      { name: 'topLevelFn', kind: 'FunctionDeclaration', line: 1 },
      { name: 'aClass', kind: 'ClassDeclaration', line: 5 },
      { name: 'neverMethod', kind: 'MethodDefinition', line: 10 },
    ], uses: [], reExports: [] }],
  ]);
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
  });
  assert('I8a. FunctionDeclaration included',
    result.helperDefsByIdentity.has('src/k.ts::topLevelFn'));
  assert('I8b. ClassDeclaration excluded',
    !result.helperDefsByIdentity.has('src/k.ts::aClass'));
  assert('I8c. MethodDefinition excluded',
    !result.helperDefsByIdentity.has('src/k.ts::neverMethod'));
}

// ═══ I9. const-var also counted (extract-ts kind for `export const foo = () => ...`) ═══

{
  const owner = '/fx/src/c.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'arrowHelper', kind: 'const-var', line: 2 }], uses: [], reExports: [] }],
  ]);
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
  });
  assert('I9. const-var (export const foo = () => ...) treated as helper candidate',
    result.helperDefsByIdentity.has('src/c.ts::arrowHelper'));
}

// ═══ I10. Parse error propagates to diagnostics, other files still aggregate ═══

{
  const owner = '/fx/src/ok.ts';
  const broken = '/fx/src/broken.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'f', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
  ]);
  function extractFn(abs) {
    if (abs === broken) throw new Error('simulated oxc parse error');
    const e = perFile.get(abs);
    return e || { defs: [], uses: [], reExports: [] };
  }
  const result = collectHelperIdentities({
    files: [owner, broken],
    root: ROOT,
    extractFn,
    resolveSpecifier: makeResolver(new Map()),
  });
  assert('I10a. broken file surfaces a parse-error diagnostic',
    result.diagnostics.some((d) => d.kind === 'parse-error' && d.target === 'src/broken.ts'));
  assert('I10b. ok.ts helper still aggregated',
    result.helperDefsByIdentity.has('src/ok.ts::f'));
}

// ═══ I11. call-graph cross-check: topCallees > 0 + AST fan-in 0 → diagnostic ═══

{
  const owner = '/fx/src/reflective.ts';
  // Define a helper, NO consumer import resolution → our AST fan-in 0
  const perFile = new Map([
    [owner, { defs: [{ name: 'viaReflection', kind: 'FunctionDeclaration', line: 3 }], uses: [], reExports: [] }],
  ]);
  const callGraph = {
    meta: { generated: new Date().toISOString() },
    topCallees: [{ file: 'src/reflective.ts', name: 'viaReflection', count: 8 }],
  };
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
    callGraph,
  });
  assert('I11a. cross-check diagnostic surfaces when topCallees evidence contradicts AST fan-in',
    result.diagnostics.some((d) =>
      d.kind === 'call-graph-cross-check' &&
      d.target === 'src/reflective.ts::viaReflection'),
    `diagnostics=${JSON.stringify(result.diagnostics)}`);
  assert('I11b. meta.callGraphStaleness = "fresh" on a just-generated artifact',
    result.meta.callGraphStaleness === 'fresh');
}

// ═══ I12. call-graph staleness > 24h → meta records "stale" ═══

{
  const owner = '/fx/src/s.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'f', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
  ]);
  const now = Date.parse('2026-04-21T12:00:00Z');
  const oldTs = now - 30 * 3600 * 1000; // 30h ago
  const callGraph = {
    meta: { generated: new Date(oldTs).toISOString() },
    topCallees: [],
  };
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
    callGraph,
    nowMs: now,
  });
  assert('I12a. 30h-old call-graph → meta.callGraphStaleness = "stale"',
    result.meta.callGraphStaleness === 'stale');
  const md = renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: {
      scope: 'TS/JS production files',
      callGraphStaleness: 'stale',
      callGraphAgeHours: 30,
    },
  });
  assert('I12b. rendered draft header carries stale warning',
    md.includes('stale') && md.includes('30 hours'));
}

// ═══ I13. call-graph absent → no cross-check diagnostics, meta "absent" ═══

{
  const owner = '/fx/src/x.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'f', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
  ]);
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
    callGraph: null,
  });
  assert('I13a. call-graph absent → meta.callGraphStaleness = "absent"',
    result.meta.callGraphStaleness === 'absent');
  assert('I13b. no call-graph cross-check diagnostics when absent',
    !result.diagnostics.some((d) => d.kind === 'call-graph-cross-check'));
}

// ═══ I14. contamination-unavailable row rendering (`—`) ═══

{
  const owner = '/fx/src/u.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'g', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
  ]);
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
  });
  const md = renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: { scope: 'TS/JS including tests', helperContamination: 'unavailable' },
  });
  // Grab the data row (line starting with `| ` but not the header/separator).
  const dataRow = md.split('\n').find((l) => l.startsWith('| `g`'));
  assert('I14a. row has `—` in the Any/unknown signal column (contamination unavailable)',
    !!dataRow && dataRow.endsWith('| — |'), `row=${dataRow}`);
  assert('I14b. header Mode line = "fresh-ast"',
    md.includes('Mode: fresh-ast'));
  assert('I14c. header FanInKind = consumer-file-count',
    md.includes('FanInKind: consumer-file-count'));
}

// ═══ I15. contamination-available enriches rows ═══

{
  const owner = '/fx/src/e.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'legacy', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
  ]);
  const symbols = {
    helperOwnersByIdentity: {
      'src/e.ts::legacy': {
        anyContamination: { label: 'severely-any-contaminated' },
        signature: '(raw: any) => any',
      },
    },
  };
  const result = collectHelperIdentities({
    files: [owner],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(new Map()),
    symbols,
  });
  assert('I15a. meta.helperContamination = "available"',
    result.meta.helperContamination === 'available');
  assert('I15b. def carries signature',
    result.helperDefsByIdentity.get('src/e.ts::legacy')?.signature === '(raw: any) => any');
  const md = renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: { scope: 'TS/JS including tests', helperContamination: 'available' },
  });
  assert('I15c. row status = severely-any-contaminated-helper (enriched classifier fires)',
    md.includes('severely-any-contaminated-helper'),
    `md=${md.slice(0, 600)}`);
  assert('I15d. mode line = "fresh-ast + helper-owner enrichment"',
    md.includes('Mode: fresh-ast + helper-owner enrichment'));
}

// ═══ I16. HELPER_OWNER_KINDS constant is correctly frozen ═══

assert('I16. HELPER_OWNER_KINDS frozen Set',
  Object.isFrozen(HELPER_OWNER_KINDS) && HELPER_OWNER_KINDS instanceof Set);

// ═══ I17. UNCERTAIN_REASONS shape ═══

assert('I17. UNCERTAIN_REASONS is a frozen 4-element array',
  Array.isArray(UNCERTAIN_REASONS) &&
  Object.isFrozen(UNCERTAIN_REASONS) &&
  UNCERTAIN_REASONS.length === 4);

// ═══ I18. Existing canon header emitted ═══

{
  const result = collectHelperIdentities({
    files: [],
    root: ROOT,
    extractFn: makeExtractFn(new Map()),
    resolveSpecifier: makeResolver(new Map()),
  });
  const md = renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: { scope: '...', existingCanon: true },
  });
  assert('I18. existingCanon=true → draft carries "⚠ Existing canon detected" header for helper-registry',
    md.includes('⚠ Existing canon detected') && md.includes('helper-registry'));
}

// ═══ I19. Zero-inventory draft still renders well ═══

{
  const md = renderHelperRegistry({
    helperDefsByIdentity: new Map(),
    helpersByName: new Map(),
    distinctConsumerFiles: new Map(),
    diagnostics: [],
    meta: { scope: 'TS/JS including tests', helperContamination: 'unavailable' },
  });
  assert('I19a. empty inventory still emits the table header',
    md.includes('| Name | Identity | Owner'));
  assert('I19b. empty inventory: no data rows, no Notes section',
    !md.includes('## Notes'));
}

// ═══ I20. namespace / default import skipped (not helper-registry v1) ═══

{
  const owner = '/fx/src/u.ts';
  const c = '/fx/src/c.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'foo', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [c, { defs: [], uses: [
      { fromSpec: './u', name: '*', kind: 'namespace', typeOnly: false },
      { fromSpec: './u', name: 'default', kind: 'default', typeOnly: false },
    ], reExports: [] }],
  ]);
  const resolves = new Map([[`${c}|./u`, owner]]);
  const result = collectHelperIdentities({
    files: [owner, c],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  assert('I20. namespace `*` import + `default` import do NOT contribute to named helper fan-in',
    result.helperDefsByIdentity.get('src/u.ts::foo')?.fanIn === 0);
}

// ═══ I21. Callback-passed helper — import resolve lens captures it ═══

{
  // Simulating `import { tryParse } from './u'; arr.map(tryParse)`.
  // The extractor emits an `import` use whether the helper is called
  // directly or passed as a callback — our fan-in lens counts both.
  const owner = '/fx/src/u.ts';
  const c = '/fx/src/c.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'tryParse', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [c, { defs: [], uses: [
      { fromSpec: './u', name: 'tryParse', kind: 'import', typeOnly: false },
    ], reExports: [] }],
  ]);
  const resolves = new Map([[`${c}|./u`, owner]]);
  const result = collectHelperIdentities({
    files: [owner, c],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  assert('I21. callback-passed helper gets fan-in 1 via import-resolve (lens wider than call-graph direct-call)',
    result.helperDefsByIdentity.get('src/u.ts::tryParse')?.fanIn === 1);
}

// ═══ I22. Re-export terminal identity is owner (alias-hop pin) ═══

// Re-export chain: source `src/util.ts` exports `tryParse`;
// barrel `src/index.ts` re-exports `{ tryParse as tryParseJson }`;
// consumer imports `{ tryParseJson } from './index'`.
//
// P3-2 v1 does NOT trace re-export chains with full §6 depth (that's a
// producer-side enrichment candidate). Our import-resolve DI here
// treats the consumer as importing `tryParse` directly from the owner
// when it would match — i.e., the test supplier pre-resolves.
//
// The correctness assertion: identity remains `src/util.ts::tryParse`
// (owner), NOT `src/index.ts::tryParseJson` (barrel alias).

{
  const owner = '/fx/src/util.ts';
  const barrel = '/fx/src/index.ts';
  const consumer = '/fx/src/c.ts';
  const perFile = new Map([
    [owner, { defs: [{ name: 'tryParse', kind: 'FunctionDeclaration', line: 1 }], uses: [], reExports: [] }],
    [barrel, { defs: [], uses: [], reExports: [
      { source: './util', name: 'tryParseJson', importedName: 'tryParse' },
    ] }],
    [consumer, { defs: [], uses: [
      // Simulating post-resolution: consumer's import resolves to OWNER `src/util.ts`
      // with the source-side name `tryParse` (not the barrel alias).
      { fromSpec: './util', name: 'tryParse', kind: 'import', typeOnly: false },
    ], reExports: [] }],
  ]);
  const resolves = new Map([[`${consumer}|./util`, owner]]);
  const result = collectHelperIdentities({
    files: [owner, barrel, consumer],
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
  });
  assert('I22a. terminal identity is owner (src/util.ts::tryParse), NOT barrel alias',
    result.helperDefsByIdentity.has('src/util.ts::tryParse') &&
    !result.helperDefsByIdentity.has('src/index.ts::tryParseJson'));
  assert('I22b. fan-in attributed to the owner, not the barrel',
    result.helperDefsByIdentity.get('src/util.ts::tryParse')?.fanIn === 1);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
