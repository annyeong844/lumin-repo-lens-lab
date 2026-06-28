// Tests for _lib/post-write-delta.mjs — P2-1 step 1.
//
// Pinning rules from docs/history/phases/p2/p2-1.md v3 §5.1:
//   - computeDelta is pure: same inputs → byte-identical output.
//   - 6-label classification (planned / planned-not-observed / silent-new /
//     pre-existing / removed / observed-unbaselined).
//   - Ambiguous remainder routes through baseline comparison, NOT hard-labeled silent-new.
//   - Absent-from-before preference fires only when baseline available AND scan ok.
//   - inventoryCompleteness populated; carries side-discriminated parse errors.
//   - requiredAcknowledgements returns EXACTLY silent-new.
//   - normalizeCodeShape is the same reference as extract-ts-escapes'.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  computeDelta,
  requiredAcknowledgements,
  inventoryUsable,
  CANONICAL_ESCAPE_KINDS,
} from '../_lib/post-write-delta.mjs';
import { normalizeCodeShape as sharedNormalize } from '../_lib/extract-ts-escapes.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Fixture builders ─────────────────────────────────────────

const CANON = [
  'explicit-any', 'as-any', 'angle-any', 'as-unknown-as-T',
  'rest-any-args', 'index-sig-any', 'generic-default-any',
  'ts-ignore', 'ts-expect-error', 'no-explicit-any-disable',
  'jsdoc-any',
];

let keySeq = 0;
function freshKey() { return `sha256:${String(++keySeq).padStart(64, '0')}`; }

function makeEscape({
  file = 'src/a.ts',
  line = 1,
  escapeKind = 'as-any',
  codeShape = 'x as any',
  normalizedCodeShape,
  insideExportedIdentity = null,
  occurrenceKey,
} = {}) {
  const norm = normalizedCodeShape ?? codeShape;
  return {
    file, line, escapeKind,
    codeShape,
    normalizedCodeShape: norm,
    insideExportedIdentity,
    occurrenceKey: occurrenceKey ?? freshKey(),
  };
}

function makeInventory({
  complete = true,
  scope = 'TS/JS production files',
  includeTests = true,
  exclude = [],
  filesWithParseErrors = [],
  escapeKinds = CANON,
  typeEscapeSupport = true,
  typeEscapes = [],
} = {}) {
  return {
    meta: {
      tool: 'any-inventory.mjs',
      generated: '2026-04-20T00:00:00Z',
      root: '/fake',
      complete,
      scope,
      includeTests,
      exclude,
      fileCount: 1,
      filesWithParseErrors,
      supports: { typeEscapes: typeEscapeSupport, escapeKinds },
    },
    typeEscapes,
  };
}

function makeAdvisory({
  invocationId = 'pre-INV-1',
  intentHash = 'abc123',
  plannedTypeEscapes = [],
  capabilities = null,
  anyInventoryPath = 'any-inventory.pre.pre-INV-1.json',
} = {}) {
  const preWrite = anyInventoryPath ? { anyInventoryPath } : {};
  return {
    invocationId,
    intentHash,
    intent: {
      names: [], shapes: [], files: [], dependencies: [],
      plannedTypeEscapes,
    },
    lookups: [],
    boundaryChecks: [],
    drift: [],
    capabilities,
    capabilityFailures: [],
    preWrite,
  };
}

function run(inputs) {
  return computeDelta({
    preWriteAdvisory: inputs.advisory,
    beforeInventory: inputs.before ?? null,
    afterInventory: inputs.after ?? null,
    deltaInvocationId: inputs.deltaInvocationId ?? 'DELTA-1',
  });
}

// ═══ T_PURE. Purity (reviewer P0-1) ═══

{
  const e = makeEscape({ occurrenceKey: 'sha256:AAA' });
  const advisory = makeAdvisory();
  const before = makeInventory({ typeEscapes: [e] });
  const after = makeInventory({ typeEscapes: [e] });

  const a = run({ advisory, before, after, deltaInvocationId: 'DELTA-X' });
  const b = run({ advisory, before, after, deltaInvocationId: 'DELTA-X' });
  assert('T_PURE_1. Same inputs (including deltaInvocationId) → byte-identical output',
    JSON.stringify(a) === JSON.stringify(b));

  const c = run({ advisory, before, after, deltaInvocationId: 'DELTA-Y' });
  assert('T_PURE_2a. Different deltaInvocationId → output differs in that field',
    a.deltaInvocationId === 'DELTA-X' && c.deltaInvocationId === 'DELTA-Y');
  const cNormalized = { ...c, deltaInvocationId: 'DELTA-X' };
  assert('T_PURE_2b. Different deltaInvocationId → ONLY that field differs',
    JSON.stringify(a) === JSON.stringify(cNormalized));

  // Source-grep pinning: no time / random / fs APIs inside computeDelta module.
  const src = readFileSync(path.join(DIR, '_lib', 'post-write-delta.mjs'), 'utf8');
  const forbidden = ['Date.now', 'new Date', 'Math.random', 'crypto.randomBytes', 'readFileSync', 'writeFileSync', 'existsSync'];
  for (const pat of forbidden) {
    assert(`T_PURE_3. post-write-delta.mjs does not use ${pat}`,
      !src.includes(pat),
      `pattern found in module`);
  }
}

// ═══ T_BASE. Baseline states ═══

{
  const e = makeEscape({ occurrenceKey: 'sha256:B1' });
  const advisory = makeAdvisory();
  const before = makeInventory({ typeEscapes: [e] });
  const after = makeInventory({ typeEscapes: [e] });
  const d = run({ advisory, before, after });
  assert('T_BASE_1a. baseline available, unchanged → all pre-existing',
    d.entries.length === 1 && d.entries[0].label === 'pre-existing');
  assert('T_BASE_1b. baseline.status === available',
    d.baseline.status === 'available');
  assert('T_BASE_1c. summary.preExisting === 1, silentNew === 0',
    d.summary.preExisting === 1 && d.summary.silentNew === 0);
}

{
  const advisory = makeAdvisory();
  const before = makeInventory({ typeEscapes: [] });
  const after = makeInventory({ typeEscapes: [makeEscape({ occurrenceKey: 'sha256:NEW' })] });
  const d = run({ advisory, before, after });
  assert('T_BASE_2. baseline available, new as-any → silent-new',
    d.entries.length === 1 && d.entries[0].label === 'silent-new');
}

{
  const dupKey = 'sha256:DUPLICATE-ANY';
  const advisory = makeAdvisory();
  const before = makeInventory({ typeEscapes: [
    makeEscape({
      line: 1,
      escapeKind: 'explicit-any',
      codeShape: 'y: any',
      normalizedCodeShape: 'y: any',
      insideExportedIdentity: 'src/a.ts::unused',
      occurrenceKey: dupKey,
    }),
  ] });
  const after = makeInventory({ typeEscapes: [
    makeEscape({
      line: 1,
      escapeKind: 'explicit-any',
      codeShape: 'y: any',
      normalizedCodeShape: 'y: any',
      insideExportedIdentity: 'src/a.ts::unused',
      occurrenceKey: dupKey,
    }),
    makeEscape({
      line: 2,
      escapeKind: 'explicit-any',
      codeShape: 'z: any',
      normalizedCodeShape: 'y: any',
      insideExportedIdentity: 'src/a.ts::unused',
      occurrenceKey: dupKey,
    }),
  ] });
  const d = run({ advisory, before, after });
  assert('T_BASE_2b. duplicate occurrenceKey is compared as a multiset: one pre-existing and one silent-new',
    d.summary.preExisting === 1 && d.summary.silentNew === 1,
    JSON.stringify(d.summary));
  assert('T_BASE_2c. requiredAcknowledgements includes the duplicate-key silent-new',
    requiredAcknowledgements(d).length === 1 &&
    requiredAcknowledgements(d)[0].occurrenceKey === dupKey);
  assert('T_BASE_2d. duplicate-key silent-new carries localization ambiguity diagnostic',
    (requiredAcknowledgements(d)[0].diagnostics ?? []).includes('ambiguous-duplicate-occurrence-key'),
    JSON.stringify(requiredAcknowledgements(d)[0]));
}

{
  const advisory = makeAdvisory();
  const before = makeInventory({ typeEscapes: [makeEscape({ occurrenceKey: 'sha256:OLD' })] });
  const after = makeInventory({ typeEscapes: [] });
  const d = run({ advisory, before, after });
  const removed = d.entries.filter(e => e.label === 'removed');
  assert('T_BASE_3. baseline available, gone from after → removed',
    removed.length === 1);
}

{
  const advisory = makeAdvisory({ anyInventoryPath: null });
  const after = makeInventory({ typeEscapes: [makeEscape({ occurrenceKey: 'sha256:O1' })] });
  const d = run({ advisory, before: null, after });
  assert('T_BASE_4a. baseline missing → all observed-unbaselined',
    d.entries.every(e => e.label === 'observed-unbaselined'));
  assert('T_BASE_4b. baseline missing → no silent-new emitted',
    d.summary.silentNew === 0);
  assert('T_BASE_4c. baseline missing → no removed emitted',
    d.summary.removed === 0);
}

{
  const advisory = makeAdvisory();
  const before = makeInventory({ typeEscapeSupport: false, typeEscapes: [] });
  const after = makeInventory({ typeEscapes: [] });
  const d = run({ advisory, before, after });
  assert('T_BASE_5. baseline unusable (supports.typeEscapes !== true) → baseline.status=missing',
    d.baseline.status === 'missing' && /unusable/.test(d.baseline.reason ?? ''));
}

// ═══ T_BASE_INCOMPLETE. Before incomplete due to parse errors (P0-2 fix) ═══
//
// If beforeInventory didn't parse `src/foo.ts`, we can't know whether
// its after occurrences are new. A naive `silent-new` claim would be a
// false Stage 3 acknowledgement request. Fix routes those files'
// after-occurrences to observed-unbaselined with a carry diagnostic.

{
  const afterEsc = makeEscape({
    file: 'src/foo.ts',
    line: 5,
    occurrenceKey: 'sha256:FOO-AFTER',
  });
  const advisory = makeAdvisory();
  const before = makeInventory({
    complete: false,
    filesWithParseErrors: [{ file: 'src/foo.ts', message: 'Unexpected token', line: 1 }],
    typeEscapes: [],  // empty because src/foo.ts didn't parse; no other files have escapes
  });
  const after = makeInventory({
    complete: true,
    typeEscapes: [afterEsc],
  });
  const d = run({ advisory, before, after });

  const match = d.entries.find((e) => e.occurrenceKey === 'sha256:FOO-AFTER');
  assert('T_BASE_INCOMPLETE_1a. after-occ in before-parse-error file → observed-unbaselined (NOT silent-new)',
    match?.label === 'observed-unbaselined',
    `got label: ${match?.label}`);
  assert('T_BASE_INCOMPLETE_1b. carry diagnostic "before-file-parse-error" attached',
    (match?.diagnostics ?? []).includes('before-file-parse-error'));
  assert('T_BASE_INCOMPLETE_1c. summary.silentNew === 0 (the potential false silent-new was downgraded)',
    d.summary.silentNew === 0);
  assert('T_BASE_INCOMPLETE_1d. requiredAcknowledgements empty (no silent-new emission)',
    requiredAcknowledgements(d).length === 0);
  assert('T_BASE_INCOMPLETE_1e. baseline.status still "available" — a partial before is still a baseline',
    d.baseline.status === 'available');
}

{
  // Control: files NOT in beforeInventory.filesWithParseErrors still
  // silent-new normally. Pins that the downgrade is narrowly scoped.
  const fooEsc = makeEscape({
    file: 'src/foo.ts',
    line: 5,
    occurrenceKey: 'sha256:FOO-IN-ERR',
  });
  const barEsc = makeEscape({
    file: 'src/bar.ts',
    line: 5,
    occurrenceKey: 'sha256:BAR-CLEAN',
  });
  const advisory = makeAdvisory();
  const before = makeInventory({
    complete: false,
    filesWithParseErrors: [{ file: 'src/foo.ts', message: 'err', line: 1 }],
    typeEscapes: [],
  });
  const after = makeInventory({
    complete: true,
    typeEscapes: [fooEsc, barEsc],
  });
  const d = run({ advisory, before, after });

  const fooEntry = d.entries.find((e) => e.occurrenceKey === 'sha256:FOO-IN-ERR');
  const barEntry = d.entries.find((e) => e.occurrenceKey === 'sha256:BAR-CLEAN');
  assert('T_BASE_INCOMPLETE_2a. parse-error file → observed-unbaselined',
    fooEntry?.label === 'observed-unbaselined');
  assert('T_BASE_INCOMPLETE_2b. clean-parse file → silent-new (narrowly scoped downgrade)',
    barEntry?.label === 'silent-new');
  assert('T_BASE_INCOMPLETE_2c. requiredAcknowledgements has only the clean-file silent-new',
    requiredAcknowledgements(d).length === 1 &&
    requiredAcknowledgements(d)[0].occurrenceKey === 'sha256:BAR-CLEAN');
}

// ═══ T_PLAN. Planned matching ═══

{
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'src/a.ts::foo', reason: 'upstream' }],
  });
  const after = makeInventory({ typeEscapes: [
    makeEscape({ insideExportedIdentity: 'src/a.ts::foo', occurrenceKey: 'sha256:P1' }),
  ]});
  const d = run({ advisory, before: makeInventory({ typeEscapes: [] }), after });
  assert('T_PLAN_1. planned at insideExportedIdentity matches → planned',
    d.entries.length === 1 && d.entries[0].label === 'planned');
}

{
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  const after = makeInventory({ typeEscapes: [makeEscape({ occurrenceKey: 'sha256:P2' })] });
  const d = run({ advisory, before: makeInventory({ typeEscapes: [] }), after });
  assert('T_PLAN_2. planned with "unknown" + 1 candidate → planned',
    d.entries[0].label === 'planned');
}

{
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'src/vendor/', reason: 'r' }],
  });
  const after = makeInventory({ typeEscapes: [
    makeEscape({ file: 'src/vendor/wrapper.ts', occurrenceKey: 'sha256:P3' }),
  ]});
  const d = run({ advisory, before: makeInventory({ typeEscapes: [] }), after });
  assert('T_PLAN_3. planned at file-prefix "src/vendor/" + file in that dir → planned',
    d.entries[0].label === 'planned');
}

{
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'src/foo', reason: 'r' }],
  });
  const after = makeInventory({ typeEscapes: [
    makeEscape({ file: 'src/foobar.ts', occurrenceKey: 'sha256:P4' }),
  ]});
  const d = run({ advisory, before: makeInventory({ typeEscapes: [] }), after });
  assert('T_PLAN_4a. planned at "src/foo" (no trailing /) does NOT match src/foobar.ts',
    !d.entries.some(e => e.label === 'planned'));
  assert('T_PLAN_4b. unmatched after escape falls to silent-new (baseline present, absent from before)',
    d.entries.some(e => e.label === 'silent-new'));
}

{
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'ts-ignore', locationHint: 'src/missing.ts', reason: 'r' }],
  });
  const after = makeInventory({ typeEscapes: [] });
  const d = run({ advisory, before: makeInventory({ typeEscapes: [] }), after });
  assert('T_PLAN_5. planned with no candidate → planned-not-observed',
    d.entries[0].label === 'planned-not-observed' && d.entries[0].occurrenceKey === null);
}

// ═══ T_ABS. Absent-from-before preference (reviewer P1-1) ═══

{
  // 2 candidates; one in before, one absent. Planned should pick the absent one.
  const preExisting = makeEscape({ line: 10, occurrenceKey: 'sha256:PRE' });
  const newOne      = makeEscape({ line: 20, occurrenceKey: 'sha256:NEW' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  const before = makeInventory({ typeEscapes: [preExisting] });
  const after = makeInventory({ typeEscapes: [preExisting, newOne] });
  const d = run({ advisory, before, after });
  const planned = d.entries.find(e => e.label === 'planned');
  assert('T_ABS_1a. baseline ok + unknown + 2 candidates → planned picks absent-from-before',
    planned?.occurrenceKey === 'sha256:NEW',
    `got: ${planned?.occurrenceKey}`);
  assert('T_ABS_1b. the pre-existing candidate classifies as pre-existing',
    d.entries.some(e => e.label === 'pre-existing' && e.occurrenceKey === 'sha256:PRE'));
}

{
  const dupKey = 'sha256:PLANNED-DUPLICATE';
  const oldOne = makeEscape({
    line: 10,
    occurrenceKey: dupKey,
    escapeKind: 'explicit-any',
    codeShape: 'y: any',
    normalizedCodeShape: 'y: any',
    insideExportedIdentity: 'src/a.ts::unused',
  });
  const newOne = makeEscape({
    line: 20,
    occurrenceKey: dupKey,
    escapeKind: 'explicit-any',
    codeShape: 'z: any',
    normalizedCodeShape: 'y: any',
    insideExportedIdentity: 'src/a.ts::unused',
  });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'explicit-any', locationHint: 'unknown', reason: 'intentional temporary shim' }],
  });
  const before = makeInventory({ typeEscapes: [oldOne] });
  const after = makeInventory({ typeEscapes: [oldOne, newOne] });
  const d = run({ advisory, before, after });
  const planned = d.entries.find(e => e.label === 'planned');
  assert('T_ABS_1c. planned matching treats duplicate occurrenceKey as separate instances',
    planned?.line === 20 && d.summary.preExisting === 1 && d.summary.silentNew === 0,
    JSON.stringify(d.entries));
}

{
  // scan-range mismatch → preference SKIPPED.
  const preExisting = makeEscape({ line: 10, occurrenceKey: 'sha256:PRE2' });
  const newOne      = makeEscape({ line: 20, occurrenceKey: 'sha256:NEW2' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  const before = makeInventory({ includeTests: false, typeEscapes: [preExisting] });
  const after  = makeInventory({ includeTests: true,  typeEscapes: [preExisting, newOne] });
  const d = run({ advisory, before, after });
  assert('T_ABS_2a. scan-range mismatch → scanRangeParity: mismatch',
    d.scanRangeParity.status === 'mismatch');
  // Under mismatch remainder → observed-unbaselined (not silent-new).
  assert('T_ABS_2b. under mismatch: unmatched remainders degrade to observed-unbaselined',
    d.entries.filter(e => e.label === 'observed-unbaselined').length >= 1);
  assert('T_ABS_2c. under mismatch: removed NOT computed',
    d.summary.removed === 0);
  assert('T_ABS_2d. under mismatch: requiredAcknowledgements empty',
    requiredAcknowledgements(d).length === 0);
}

{
  // All candidates pre-existing — planned picks first deterministic.
  const pre1 = makeEscape({ line: 5, occurrenceKey: 'sha256:PE1' });
  const pre2 = makeEscape({ line: 10, occurrenceKey: 'sha256:PE2' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  const before = makeInventory({ typeEscapes: [pre1, pre2] });
  const after  = makeInventory({ typeEscapes: [pre1, pre2] });
  const d = run({ advisory, before, after });
  const plannedEntry = d.entries.find(e => e.label === 'planned');
  assert('T_ABS_3a. all candidates pre-existing → planned picks first (deterministic)',
    plannedEntry?.occurrenceKey === 'sha256:PE1');
  assert('T_ABS_3b. the other pre-existing remains pre-existing',
    d.entries.some(e => e.label === 'pre-existing' && e.occurrenceKey === 'sha256:PE2'));
}

// ═══ T_AMB. Ambiguous-planned-match (routed through baseline) ═══

{
  // Tiebreak picks one; remainder IN baseline → pre-existing + diagnostic.
  const keptBoth1 = makeEscape({ line: 10, occurrenceKey: 'sha256:AMB1-A' });
  const keptBoth2 = makeEscape({ line: 20, occurrenceKey: 'sha256:AMB1-B' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  // Both in before AND after. No codeShape so tiebreak is deterministic order.
  const before = makeInventory({ typeEscapes: [keptBoth1, keptBoth2] });
  const after  = makeInventory({ typeEscapes: [keptBoth1, keptBoth2] });
  const d = run({ advisory, before, after });
  // Both candidates pre-existing → planned picks first, remainder passes to baseline.
  const planned = d.entries.find(e => e.label === 'planned');
  const preExist = d.entries.filter(e => e.label === 'pre-existing');
  assert('T_AMB_1a. remainder in baseline → pre-existing (NOT silent-new)',
    preExist.length === 1);
  assert('T_AMB_1b. remainder carries ambiguous-planned-match diagnostic',
    preExist[0].diagnostics.includes('ambiguous-planned-match'));
  assert('T_AMB_1c. requiredAcknowledgements empty (remainder was pre-existing)',
    requiredAcknowledgements(d).length === 0);
}

{
  // Tiebreak picks one; remainder NOT in baseline → silent-new + diagnostic.
  const exist = makeEscape({ line: 10, occurrenceKey: 'sha256:AMB2-EXISTS' });
  const fresh = makeEscape({ line: 20, occurrenceKey: 'sha256:AMB2-FRESH' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  const before = makeInventory({ typeEscapes: [exist] });
  const after  = makeInventory({ typeEscapes: [exist, fresh] });
  const d = run({ advisory, before, after });
  // Absent-from-before preference → planned picks `fresh`.
  // `exist` passes through to baseline → pre-existing.
  // No one is both-ambiguous-and-silent-new in this setup. Adjust: use scenario
  // where the tiebreak actually has multiple absent-from-before candidates.
  const freshA = makeEscape({ line: 20, occurrenceKey: 'sha256:AMB2B-A' });
  const freshB = makeEscape({ line: 30, occurrenceKey: 'sha256:AMB2B-B' });
  const before2 = makeInventory({ typeEscapes: [] });
  const after2  = makeInventory({ typeEscapes: [freshA, freshB] });
  const advisory2 = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
  });
  const d2 = run({ advisory: advisory2, before: before2, after: after2 });
  const silentNew = d2.entries.find(e => e.label === 'silent-new');
  assert('T_AMB_2a. remainder NOT in before → silent-new',
    silentNew !== undefined);
  assert('T_AMB_2b. silent-new remainder carries ambiguous-planned-match diagnostic',
    silentNew?.diagnostics.includes('ambiguous-planned-match'));
  assert('T_AMB_2c. requiredAcknowledgements includes silent-new entry',
    requiredAcknowledgements(d2).length === 1);
}

{
  // Baseline missing → remainder → observed-unbaselined + diagnostic.
  const a = makeEscape({ line: 10, occurrenceKey: 'sha256:AMB3-A' });
  const b = makeEscape({ line: 20, occurrenceKey: 'sha256:AMB3-B' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', reason: 'r' }],
    anyInventoryPath: null,
  });
  const after = makeInventory({ typeEscapes: [a, b] });
  const d = run({ advisory, before: null, after });
  const obs = d.entries.find(e => e.label === 'observed-unbaselined');
  assert('T_AMB_3a. baseline missing → remainder → observed-unbaselined',
    obs !== undefined);
  assert('T_AMB_3b. observed-unbaselined remainder carries ambiguous-planned-match diagnostic',
    obs?.diagnostics.includes('ambiguous-planned-match'));
  assert('T_AMB_3c. requiredAcknowledgements empty (baseline-missing)',
    requiredAcknowledgements(d).length === 0);
}

// ═══ T_SHAPE. CodeShape tiebreak reuses P2-0 normalizer ═══

{
  const plannedShape = `foo as "a   b" as any`;
  // Two candidates with same escapeKind — tiebreak by normalizedCodeShape.
  const exact = makeEscape({
    line: 5,
    codeShape: `foo as "a   b" as any`,
    normalizedCodeShape: sharedNormalize(`foo as "a   b" as any`),
    occurrenceKey: 'sha256:SHAPE-EXACT',
  });
  const distractor = makeEscape({
    line: 10,
    codeShape: `bar as any`,
    normalizedCodeShape: sharedNormalize(`bar as any`),
    occurrenceKey: 'sha256:SHAPE-DISTRACTOR',
  });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'unknown', codeShape: plannedShape, reason: 'r' }],
  });
  const before = makeInventory({ typeEscapes: [] });
  const after = makeInventory({ typeEscapes: [distractor, exact] });
  const d = run({ advisory, before, after });
  const planned = d.entries.find(e => e.label === 'planned');
  assert('T_SHAPE_1. codeShape tiebreak picks the string-literal-preserving match',
    planned?.occurrenceKey === 'sha256:SHAPE-EXACT',
    `got: ${planned?.occurrenceKey}`);
}

// ═══ T_CAP. Capability parity ═══

{
  const advisory = makeAdvisory();
  const d = run({ advisory, before: null, after: null });
  assert('T_CAP_1a. afterInventory null → capabilityParity.status: missing',
    d.capabilityParity.status === 'missing');
  assert('T_CAP_1b. entries: []',
    d.entries.length === 0);
  assert('T_CAP_1c. capabilityFailures has after-inventory-missing',
    d.capabilityFailures.some(f => f.kind === 'after-inventory-missing'));
  assert('T_CAP_1d. requiredAcknowledgements empty',
    requiredAcknowledgements(d).length === 0);
}

{
  const advisory = makeAdvisory();
  const after = makeInventory({ typeEscapeSupport: false });
  const d = run({ advisory, before: null, after });
  assert('T_CAP_2a. afterInventory supports.typeEscapes !== true → mismatch',
    d.capabilityParity.status === 'mismatch');
  assert('T_CAP_2b. entries: []',
    d.entries.length === 0);
}

{
  const advisory = makeAdvisory();
  // Reorder one kind to simulate drift.
  const drifted = [...CANON];
  [drifted[0], drifted[1]] = [drifted[1], drifted[0]];
  const after = makeInventory({ escapeKinds: drifted });
  const d = run({ advisory, before: null, after });
  assert('T_CAP_3. escapeKinds drift → mismatch',
    d.capabilityParity.status === 'mismatch');
}

{
  // advisory.capabilities variants — classification still proceeds when after is usable.
  for (const caps of [undefined, null, {}, { typeEscapes: false }]) {
    const advisory = makeAdvisory({ capabilities: caps });
    const after = makeInventory({ typeEscapes: [makeEscape({ occurrenceKey: 'sha256:CAP4-' + String(caps) })] });
    const d = run({ advisory, before: makeInventory({ typeEscapes: [] }), after });
    assert(`T_CAP_4. advisory.capabilities=${JSON.stringify(caps)} does NOT block classification`,
      d.capabilityParity.status === 'ok' && d.entries.length === 1);
  }
}

// ═══ T_SR. Scan-range parity ═══

{
  const planned = makeEscape({ insideExportedIdentity: 'src/a.ts::foo', occurrenceKey: 'sha256:SR1-P' });
  const unplanned = makeEscape({ line: 99, occurrenceKey: 'sha256:SR1-U' });
  const advisory = makeAdvisory({
    plannedTypeEscapes: [{ escapeKind: 'as-any', locationHint: 'src/a.ts::foo', reason: 'r' }],
  });
  const before = makeInventory({ includeTests: false, typeEscapes: [] });
  const after  = makeInventory({ includeTests: true,  typeEscapes: [planned, unplanned] });
  const d = run({ advisory, before, after });
  assert('T_SR_1a. mismatch: planned STILL emitted',
    d.entries.some(e => e.label === 'planned' && e.occurrenceKey === 'sha256:SR1-P'));
  assert('T_SR_1b. mismatch: unplanned → observed-unbaselined',
    d.entries.some(e => e.label === 'observed-unbaselined' && e.occurrenceKey === 'sha256:SR1-U'));
  assert('T_SR_1c. mismatch: removed NOT computed',
    d.summary.removed === 0);
  assert('T_SR_1d. mismatch: requiredAcknowledgements empty',
    requiredAcknowledgements(d).length === 0);
}

{
  const advisory = makeAdvisory();
  const before = makeInventory({ exclude: ['b', 'a'], typeEscapes: [] });
  const after  = makeInventory({ exclude: ['a', 'b'], typeEscapes: [] });
  const d = run({ advisory, before, after });
  assert('T_SR_2. exclude order differs but sorted equal → scanRangeParity: ok',
    d.scanRangeParity.status === 'ok');
}

// ═══ T_COMP. inventoryCompleteness ═══

{
  const advisory = makeAdvisory();
  const after = makeInventory({ complete: true, filesWithParseErrors: [] });
  const d = run({ advisory, before: null, after });
  assert('T_COMP_1a. afterComplete === true',
    d.inventoryCompleteness.afterComplete === true);
  assert('T_COMP_1b. filesWithParseErrors empty',
    d.inventoryCompleteness.filesWithParseErrors.length === 0);
}

{
  const advisory = makeAdvisory();
  const after = makeInventory({
    complete: false,
    filesWithParseErrors: [{ file: 'src/bad.ts', message: 'Unexpected token', line: 12 }],
  });
  const d = run({ advisory, before: null, after });
  assert('T_COMP_2a. afterComplete === false',
    d.inventoryCompleteness.afterComplete === false);
  assert('T_COMP_2b. filesWithParseErrors has entry with side: "after"',
    d.inventoryCompleteness.filesWithParseErrors.some(e => e.side === 'after' && e.file === 'src/bad.ts'));
}

{
  const advisory = makeAdvisory();
  const before = makeInventory({
    complete: false,
    filesWithParseErrors: [{ file: 'src/old-bad.ts', message: 'err', line: 1 }],
  });
  const after = makeInventory({ complete: true });
  const d = run({ advisory, before, after });
  assert('T_COMP_3a. beforeComplete === false (baseline available)',
    d.inventoryCompleteness.beforeComplete === false);
  assert('T_COMP_3b. filesWithParseErrors has entry with side: "before"',
    d.inventoryCompleteness.filesWithParseErrors.some(e => e.side === 'before'));
}

{
  const advisory = makeAdvisory({ anyInventoryPath: null });
  const after = makeInventory();
  const d = run({ advisory, before: null, after });
  assert('T_COMP_4a. baseline missing → beforeComplete: null',
    d.inventoryCompleteness.beforeComplete === null);
  assert('T_COMP_4b. no before entries in filesWithParseErrors',
    !d.inventoryCompleteness.filesWithParseErrors.some(e => e.side === 'before'));
}

// ═══ T_REQ. requiredAcknowledgements ═══

{
  // Build a delta with one entry of each of 6 labels manually — we only need the
  // filter function to pass through.
  const fakeDelta = {
    entries: [
      { label: 'planned',              occurrenceKey: 'k1', diagnostics: [] },
      { label: 'planned-not-observed', occurrenceKey: null, diagnostics: [] },
      { label: 'silent-new',           occurrenceKey: 'k2', diagnostics: [] },
      { label: 'pre-existing',         occurrenceKey: 'k3', diagnostics: [] },
      { label: 'removed',              occurrenceKey: 'k4', diagnostics: [] },
      { label: 'observed-unbaselined', occurrenceKey: 'k5', diagnostics: [] },
    ],
  };
  const req = requiredAcknowledgements(fakeDelta);
  assert('T_REQ_1a. returns exactly 1 entry from a 6-label delta',
    req.length === 1);
  assert('T_REQ_1b. returned entry is the silent-new one',
    req[0].label === 'silent-new');
  assert('T_REQ_2. observed-unbaselined NEVER in output',
    !req.some(e => e.label === 'observed-unbaselined'));
  assert('T_REQ_3. planned-not-observed NEVER in output',
    !req.some(e => e.label === 'planned-not-observed'));

  const ambiguousDelta = {
    entries: [
      { label: 'silent-new', occurrenceKey: 'k1', diagnostics: ['ambiguous-planned-match'] },
    ],
  };
  assert('T_REQ_4. silent-new + ambiguous-planned-match IS included',
    requiredAcknowledgements(ambiguousDelta).length === 1);
}

// ═══ T_EXPORT. Module exports ═══

{
  assert('T_EXPORT_1. CANONICAL_ESCAPE_KINDS matches canonical list',
    Array.isArray(CANONICAL_ESCAPE_KINDS) && CANONICAL_ESCAPE_KINDS.length === 11 &&
    CANONICAL_ESCAPE_KINDS.every((k, i) => k === CANON[i]));
  assert('T_EXPORT_2. inventoryUsable exported',
    typeof inventoryUsable === 'function');
  assert('T_EXPORT_3. inventoryUsable(null) === false',
    inventoryUsable(null) === false);
  assert('T_EXPORT_4. inventoryUsable(valid) === true',
    inventoryUsable(makeInventory()) === true);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
