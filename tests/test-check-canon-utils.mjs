// tests/test-check-canon-utils.mjs
//
// P5-1 Step 0 — RED test for `_lib/check-canon-utils.mjs` (PURE layer).
//
// Pins the 3-tier parser strictness policy from canon-drift.md §5.e +
// reviewer P0-1 (v2 absorption): unrecognized-schema / parse-error /
// per-row canon-parse-error are distinct status+diagnostic shapes.
//
// Also pins `makeDriftRecord` family auto-derive + `buildCanonDriftJsonObject`
// shape match against canon-drift.md §6.

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  parseTypeOwnershipCanonText,
  parseHelperRegistryCanonText,
  parseTopologyCanonText,
  parseNamingCanonText,
  makeDriftRecord,
  buildCanonDriftJsonObject,
  CATEGORY_TO_FAMILY,
  HELPER_LABEL_SET,
  TOPOLOGY_LABEL_SET,
  NAMING_LABEL_SET,
} from '../_lib/check-canon-utils.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

// canonical type label set per classification-gates.md §9
const TYPE_LABEL_SET = new Set([
  'zero-internal-fan-in',
  'low-signal-type-name',
  'DUPLICATE_STRONG',
  'DUPLICATE_REVIEW',
  'LOCAL_COMMON_NAME',
  'single-owner-strong',
  'single-owner-weak',
  'severely-any-contaminated',
  'ANY_COLLISION',
]);

const CANONICAL_HEADER =
  '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
  '|------|----------|-------|-------:|--------|------|';
const CANONICAL_SEP = '|------|----------|-------|-------:|--------|------|';

// ── Tier 1: unrecognized schema ────────────────────────────────

{
  const r = parseTypeOwnershipCanonText({ text: '', canonLabelSet: TYPE_LABEL_SET });
  assert('U-1. empty text → skipped-unrecognized-schema',
    r.status === 'skipped-unrecognized-schema' && r.records.size === 0,
    `status=${r.status}, records=${r.records.size}`);
}

{
  const r = parseTypeOwnershipCanonText({
    text: '# Some markdown\n\nNo table here, just prose.\n',
    canonLabelSet: TYPE_LABEL_SET,
  });
  assert('U-2. prose-only markdown → skipped-unrecognized-schema',
    r.status === 'skipped-unrecognized-schema',
    `status=${r.status}`);
}

// ── Tier 2: recognized schema, malformed header ────────────────

{
  // Missing Identity column
  const text = CANONICAL_HEADER.replace('| Identity ', '').replace('|----------', '') + '\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-3. missing Identity column → parse-error',
    r.status === 'parse-error',
    `status=${r.status}`);
  assert('U-4. diagnostic for missing column names the column',
    r.diagnostics.some((d) => d.reason === 'missing-required-column' && d.column === 'Identity'),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

{
  // Renamed column: Owner → OwnerFile
  const text = CANONICAL_HEADER.replace('| Owner ', '| OwnerFile ') + '\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-5. renamed Owner column → parse-error',
    r.status === 'parse-error',
    `status=${r.status}`);
  assert('U-6. diagnostic names expected and observed column',
    r.diagnostics.some((d) =>
      (d.reason === 'renamed-column' || d.reason === 'missing-required-column') &&
      (d.expected === 'Owner' || d.column === 'Owner')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

{
  // Extra unknown column inserted
  const text =
    '| Name | Identity | Owner | Fan-in | Status | Tags | Extra |\n' +
    '|------|----------|-------|-------:|--------|------|-------|\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-7. extra unknown column → parse-error',
    r.status === 'parse-error',
    `status=${r.status}`);
  assert('U-8. diagnostic names the unknown column',
    r.diagnostics.some((d) => d.reason === 'unknown-column' && d.column === 'Extra'),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// ── Tier 3: valid header, per-row status error ─────────────────

{
  const text =
    CANONICAL_HEADER + '\n' +
    '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | bogus-label-xyz | |\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-9. unknown Status token → parse-error (whole source)',
    r.status === 'parse-error',
    `status=${r.status}`);
  assert('U-10. per-row canon-parse-error diagnostic emitted',
    r.diagnostics.some((d) => d.reason === 'canon-parse-error' || d.reason === 'unknown-status-label'),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// ── Clean parse ────────────────────────────────────────────────

{
  const text =
    CANONICAL_HEADER + '\n' +
    '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | single-owner-strong ✅ | |\n' +
    '| `Bar` | `src/bar.ts::Bar` | `src/bar.ts:5` | 0 | zero-internal-fan-in ⚠ | |\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-11. clean 2-row fixture → status=clean',
    r.status === 'clean',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('U-12. records.size === 2',
    r.records.size === 2,
    `size=${r.records.size}`);
  assert('U-13. identity cell backticks stripped',
    r.records.has('src/foo.ts::Foo') && r.records.has('src/bar.ts::Bar'),
    `keys=${[...r.records.keys()].join(', ')}`);
  const foo = r.records.get('src/foo.ts::Foo');
  assert('U-14. record fields parsed (exportedName/ownerFile/label/fanIn)',
    foo && foo.exportedName === 'Foo' && foo.ownerFile === 'src/foo.ts' &&
    foo.label === 'single-owner-strong' && foo.fanIn === 3,
    `foo=${JSON.stringify(foo)}`);
}

{
  const text =
    '| Name | Identity | Owner | Fan-in | Fan-in space | Status | Tags |\n' +
    '|------|----------|-------|-------:|--------------|--------|------|\n' +
    '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | value 2, type 1, broad 0 | single-owner-strong ✅ | |\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  const foo = r.records.get('src/foo.ts::Foo');
  assert('U-14b. type parser accepts optional Fan-in space column',
    r.status === 'clean' && r.records.size === 1,
    `status=${r.status}, records=${r.records.size}, diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('U-14c. Fan-in space column is ignored for drift semantics',
    foo && foo.fanIn === 3 && foo.label === 'single-owner-strong',
    `foo=${JSON.stringify(foo)}`);
}

// ── makeDriftRecord ────────────────────────────────────────────

{
  const rec = makeDriftRecord({
    kind: 'type-drift',
    category: 'owner-changed',
    identity: 'src/foo.ts::Foo',
    canon: { label: 'single-owner-strong', owner: 'src/foo.ts:12' },
    fresh: { label: 'single-owner-strong', owner: 'src/bar.ts:4' },
    confidence: 'high',
  });
  assert('U-15. makeDriftRecord auto-attaches family (structural-status-changed)',
    rec.family === 'structural-status-changed',
    `family=${rec.family}`);
  assert('U-16. record carries canon.label + fresh.label (P0-5)',
    rec.canon.label === 'single-owner-strong' && rec.fresh.label === 'single-owner-strong',
    `rec=${JSON.stringify(rec)}`);
}

// CATEGORY_TO_FAMILY exhaustive check
assert('U-17. CATEGORY_TO_FAMILY has exactly 20 entries (canon-drift §3.1)',
  Object.keys(CATEGORY_TO_FAMILY).length === 20,
  `count=${Object.keys(CATEGORY_TO_FAMILY).length}`);

// ── Whole-file scan: prefix memo table must not poison real table ─

{
  // Prior behavior locked onto the first `|` row it saw. A 2-col memo
  // with "Name" header incidentally shares 1 column with type-ownership
  // and used to trigger parse-error even when a valid table followed.
  const text =
    'Intro memo\n\n' +
    '| Name | Note |\n' +
    '|------|------|\n' +
    '| stuff | info |\n' +
    '\n\n' +
    '## Types\n\n' +
    '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
    '|------|----------|-------|-------:|--------|------|\n' +
    '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | single-owner-strong ✅ | |\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-19. prefix memo table with 1 matching col does NOT poison the parse',
    r.status === 'clean' && r.records.size === 1 && r.records.has('src/foo.ts::Foo'),
    `status=${r.status}, records=${r.records.size}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

{
  // Only a memo table exists — no type-ownership header anywhere.
  // Must return unrecognized-schema, not parse-error.
  const text =
    '| Name | Note |\n' +
    '|------|------|\n' +
    '| stuff | info |\n';
  const r = parseTypeOwnershipCanonText({ text, canonLabelSet: TYPE_LABEL_SET });
  assert('U-20. memo-only markdown (< 3 matching cols) → unrecognized-schema',
    r.status === 'skipped-unrecognized-schema',
    `status=${r.status}`);
}

// ── buildCanonDriftJsonObject ──────────────────────────────────

{
  const obj = buildCanonDriftJsonObject({
    meta: {
      tool: 'check-canon.mjs',
      generated: '2026-04-21T00:00:00Z',
      root: '/tmp/fake',
      canonDir: '/tmp/fake/canonical',
      scope: 'fixture',
      strict: false,
    },
    perSource: {
      'type-ownership': {
        status: 'clean',
        driftCount: 0,
        reportPath: '/tmp/fake/audit-output/canon-drift.type-ownership.md',
        diagnostics: [],
      },
    },
    drifts: [],
  });
  assert('U-18. buildCanonDriftJsonObject matches canon-drift §6 top-level keys',
    obj && obj.meta && obj.summary && obj.perSource && Array.isArray(obj.drifts) &&
    obj.summary.sourcesRequested === 1 && obj.summary.sourcesChecked === 1 &&
    obj.summary.sourcesSkipped === 0 && obj.summary.driftCount === 0,
    `obj=${JSON.stringify(obj)}`);
}

// ── P5-2: parseHelperRegistryCanonText (pure) ──────────────────

const HELPER_HEADER =
  '| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n' +
  '|------|----------|-------|-----------|-------:|--------|------|----------------------|';

// Tier 1 — unrecognized schema
{
  const r = parseHelperRegistryCanonText({ text: '', canonLabelSet: HELPER_LABEL_SET });
  assert('UH-1. empty text → skipped-unrecognized-schema',
    r.status === 'skipped-unrecognized-schema', `status=${r.status}`);
}

// Tier 2 — recognized schema, malformed header (renamed column)
{
  const text = HELPER_HEADER.replace('| Owner ', '| OwnerFile ') + '\n';
  const r = parseHelperRegistryCanonText({ text, canonLabelSet: HELPER_LABEL_SET });
  assert('UH-2. renamed Owner → parse-error for helper parser',
    r.status === 'parse-error',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// Tier 2 — missing Signature column
{
  const text = HELPER_HEADER.replace('| Signature ', '') + '\n';
  const r = parseHelperRegistryCanonText({ text, canonLabelSet: HELPER_LABEL_SET });
  assert('UH-3. missing Signature column → parse-error',
    r.status === 'parse-error' &&
    r.diagnostics.some((d) => d.reason === 'missing-required-column' && d.column === 'Signature'),
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// Tier 3 — unknown Status token
{
  const text =
    HELPER_HEADER + '\n' +
    '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | bogus-helper-label | | |\n';
  const r = parseHelperRegistryCanonText({ text, canonLabelSet: HELPER_LABEL_SET });
  assert('UH-4. unknown helper Status token → parse-error (whole source)',
    r.status === 'parse-error',
    `status=${r.status}`);
}

// Clean parse
{
  const text =
    HELPER_HEADER + '\n' +
    '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | central-helper ✅ | | |\n' +
    '| `doBar` | `src/bar.ts::doBar` | `src/bar.ts:2` | () => T | 1 | shared-helper ⚠ | | any-contaminated |\n';
  const r = parseHelperRegistryCanonText({ text, canonLabelSet: HELPER_LABEL_SET });
  assert('UH-5. clean 2-row helper fixture → status=clean',
    r.status === 'clean', `status=${r.status}`);
  assert('UH-6. 2 records keyed by identity verbatim',
    r.records.size === 2 && r.records.has('src/foo.ts::doFoo') && r.records.has('src/bar.ts::doBar'),
    `keys=${[...r.records.keys()]}`);
  const foo = r.records.get('src/foo.ts::doFoo');
  assert('UH-7. record fields parsed (exportedName/ownerFile/label/fanIn/signature)',
    foo && foo.exportedName === 'doFoo' && foo.ownerFile === 'src/foo.ts' &&
    foo.label === 'central-helper' && foo.fanIn === 3 &&
    typeof foo.signature === 'string' && foo.signature.length > 0,
    `foo=${JSON.stringify(foo)}`);
  const bar = r.records.get('src/bar.ts::doBar');
  assert('UH-8. anyUnknownSignal cell captured',
    bar && typeof bar.anyUnknownSignal === 'string' && bar.anyUnknownSignal.length > 0,
    `bar=${JSON.stringify(bar)}`);
}

// Whole-file scan: helper parser must skip a prefix memo table
{
  const text =
    '| Name | Note |\n' +
    '|------|------|\n' +
    '| foo | bar |\n\n' +
    HELPER_HEADER + '\n' +
    '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | central-helper ✅ | | |\n';
  const r = parseHelperRegistryCanonText({ text, canonLabelSet: HELPER_LABEL_SET });
  assert('UH-9. prefix memo table does NOT poison helper parse',
    r.status === 'clean' && r.records.size === 1,
    `status=${r.status}, records=${r.records.size}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// HELPER_LABEL_SET present and has 9 entries
assert('UH-10. HELPER_LABEL_SET exported with 9 entries',
  HELPER_LABEL_SET instanceof Set && HELPER_LABEL_SET.size === 9,
  `size=${HELPER_LABEL_SET?.size}`);

// Finding #3 — pipe in Signature column (TS union) must not break parser.
// GFM rules: pipes inside backtick code spans are literal; backslash-
// escaped pipes `\|` are literal too. Parser must handle both.
{
  const text =
    HELPER_HEADER + '\n' +
    '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | `(x: string | number) => void` | 3 | central-helper ✅ | | |\n' +
    '| `doBar` | `src/bar.ts::doBar` | `src/bar.ts:2` | (x: A \\| B) => void | 1 | shared-helper ⚠ | | |\n';
  const r = parseHelperRegistryCanonText({ text, canonLabelSet: HELPER_LABEL_SET });
  assert('UH-12. pipe inside backticked Signature does NOT break parser',
    r.status === 'clean' && r.records.has('src/foo.ts::doFoo'),
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('UH-13. backslash-escaped pipe in Signature round-trips',
    r.records.has('src/bar.ts::doBar') &&
    r.records.get('src/bar.ts::doBar').signature.includes('A | B'),
    `bar=${JSON.stringify(r.records.get('src/bar.ts::doBar'))}`);
}

// CATEGORY_TO_FAMILY includes 5 helper-drift entries
{
  const helperEntries = Object.keys(CATEGORY_TO_FAMILY).filter((k) => k.startsWith('helper-drift::'));
  assert('UH-11. CATEGORY_TO_FAMILY has exactly 5 helper-drift entries',
    helperEntries.length === 5, `count=${helperEntries.length}`);
}

// ── P5-3: parseTopologyCanonText (multi-section) ───────────────

function buildTopoCanon({ submodules = [], acyclic = true, cycles = [], crossEdges = [], oversize = [], workspaces = null } = {}) {
  const lines = [
    '## 1. Submodule inventory',
    '',
    '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
    '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
  ];
  for (const s of submodules) {
    lines.push(`| \`${s.name}\` | ${s.files} | ${s.loc} | ${s.inEdges} | ${s.outEdges} | ${s.sccMember ? '●' : '—'} | ${s.label} ✅ | |`);
  }
  lines.push('');
  lines.push('## 2. Cross-submodule edges (top 30)');
  lines.push('');
  if (crossEdges.length > 0) {
    lines.push('| From | To | Count |');
    lines.push('|------|----|------:|');
    for (const e of crossEdges) lines.push(`| \`${e.from}\` | \`${e.to}\` | ${e.count} |`);
    lines.push('');
  }
  lines.push('## 3. Cycles (SCCs)');
  lines.push('');
  if (acyclic) {
    lines.push('✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.', '');
  } else {
    lines.push('❌ Cycles observed — canon invariant violation:', '');
    for (let i = 0; i < cycles.length; i += 1) {
      lines.push(`### Cycle ${i + 1} (size ${cycles[i].members.length}) — forbidden-cycle ❌`, '');
      for (const m of cycles[i].members) lines.push(`- \`${m}\``);
      lines.push('');
    }
  }
  lines.push('## 4. Oversize files (≥ 400 LOC)', '');
  if (oversize.length > 0) {
    lines.push('| File | LOC | Status |', '|------|----:|--------|');
    for (const o of oversize) lines.push(`| \`${o.file}\` | ${o.loc} | ${o.label} ⚠ |`);
    lines.push('');
  }
  if (workspaces) {
    lines.push('## 5. Workspace boundaries', '', '| Package | Path | Files | LOC |', '|---------|------|------:|----:|');
    for (const w of workspaces) lines.push(`| \`${w.name}\` | \`${w.path}\` | ${w.files} | ${w.loc} |`);
    lines.push('');
  }
  return lines.join('\n');
}

// UY-1: clean multi-section parse
{
  const text = buildTopoCanon({
    submodules: [
      { name: 'src', files: 3, loc: 100, inEdges: 2, outEdges: 1, sccMember: false, label: 'shared-submodule' },
    ],
    acyclic: true,
    crossEdges: [{ from: 'src', to: 'lib', count: 5 }],
    oversize: [{ file: 'src/giant.ts', loc: 500, label: 'oversize' }],
  });
  const r = parseTopologyCanonText({ text, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('UY-1. clean topology canon → status=clean',
    r.status === 'clean', `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('UY-2. inventory has 1 submodule keyed by path',
    r.inventory instanceof Map && r.inventory.size === 1 && r.inventory.has('src'),
    `keys=${[...r.inventory.keys()]}`);
  assert('UY-3. inventory row carries sccMember + label + inEdges',
    r.inventory.get('src')?.sccMember === false &&
    r.inventory.get('src')?.label === 'shared-submodule' &&
    r.inventory.get('src')?.inEdges === 2,
    `row=${JSON.stringify(r.inventory.get('src'))}`);
  assert('UY-4. cycles = { acyclic: true, cycles: [] }',
    r.cycles?.acyclic === true && Array.isArray(r.cycles.cycles) && r.cycles.cycles.length === 0, '');
  assert('UY-5. crossEdges Map keyed by "<from> → <to>" literal',
    r.crossEdges instanceof Map && r.crossEdges.has('src → lib'),
    `keys=${[...(r.crossEdges?.keys() ?? [])]}`);
  assert('UY-6. oversize Map keyed by file path',
    r.oversize instanceof Map && r.oversize.has('src/giant.ts'), '');
}

// UY-7: §3 cycle parse
{
  const text = buildTopoCanon({
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 1, outEdges: 1, sccMember: true, label: 'cyclic-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 1, outEdges: 1, sccMember: true, label: 'cyclic-submodule' },
    ],
    acyclic: false,
    cycles: [{ members: ['src', 'lib'] }],
  });
  const r = parseTopologyCanonText({ text, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('UY-7. cycle listing parsed: acyclic=false + 1 cycle with 2 members',
    r.status === 'clean' && r.cycles.acyclic === false &&
    r.cycles.cycles.length === 1 && r.cycles.cycles[0].members.length === 2,
    `cycles=${JSON.stringify(r.cycles)}`);
}

// UY-8: §1/§3 disagreement → parse-error
{
  // §1 has src as NON-SCC but §3 lists src as cycle member
  const md = [
    '## 1. Submodule inventory',
    '',
    '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
    '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
    '| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |',
    '',
    '## 3. Cycles (SCCs)',
    '',
    '❌ Cycles observed — canon invariant violation:',
    '',
    '### Cycle 1 (size 2) — forbidden-cycle ❌',
    '',
    '- `src`',
    '- `lib`',
    '',
  ].join('\n');
  const r = parseTopologyCanonText({ text: md, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('UY-8. §1/§3 SCC disagreement → parse-error',
    r.status === 'parse-error',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// UY-8b: §2 cross-edges missing Count → parse-error (Finding #2)
{
  const md = [
    '## 1. Submodule inventory',
    '',
    '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
    '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
    '| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |',
    '',
    '## 2. Cross-submodule edges (top 30)',
    '',
    '| From | To |',       // MISSING Count column
    '|------|----|',
    '| `src` | `lib` |',
    '',
    '## 3. Cycles (SCCs)',
    '',
    '✅ No submodule-level cycles observed.',
    '',
  ].join('\n');
  const r = parseTopologyCanonText({ text: md, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('UY-8b. §2 cross-edges missing Count column → parse-error (not silent skip)',
    r.status === 'parse-error',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// UY-8c: §4 oversize missing Status → parse-error (Finding #2)
{
  const md = [
    '## 1. Submodule inventory',
    '',
    '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
    '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
    '| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |',
    '',
    '## 3. Cycles (SCCs)',
    '',
    '✅ No submodule-level cycles observed.',
    '',
    '## 4. Oversize files (≥ 400 LOC)',
    '',
    '| File | LOC |',       // MISSING Status column
    '|------|----:|',
    '| `src/big.ts` | 500 |',
    '',
  ].join('\n');
  const r = parseTopologyCanonText({ text: md, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('UY-8c. §4 oversize missing Status column → parse-error (not silent skip)',
    r.status === 'parse-error',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// UY-9: TOPOLOGY_LABEL_SET present + 8 entries
assert('UY-9. TOPOLOGY_LABEL_SET exported with 8 entries',
  TOPOLOGY_LABEL_SET instanceof Set && TOPOLOGY_LABEL_SET.size === 8,
  `size=${TOPOLOGY_LABEL_SET?.size}`);

// UY-10: CATEGORY_TO_FAMILY includes 6 topology-drift entries
{
  const topoEntries = Object.keys(CATEGORY_TO_FAMILY).filter((k) => k.startsWith('topology-drift::'));
  assert('UY-10. CATEGORY_TO_FAMILY has exactly 6 topology-drift entries',
    topoEntries.length === 6, `count=${topoEntries.length}`);
}

// ── P5-4: parseNamingCanonText (multi-section) ────────────────

function buildNamingCanon({ fileCohorts = [], symbolCohorts = [], outliers = null }) {
  const lines = [];
  lines.push('## 1. File-naming cohorts', '');
  lines.push('| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |');
  lines.push('|--------------------|------:|--------------------|----------------:|--------------:|--------|');
  for (const c of fileCohorts) {
    lines.push(`| \`${c.cohort}\` | ${c.files} | \`${c.convention}\` | ${c.rate}% | ${c.outliers ?? 0} | ${c.label} ✅ |`);
  }
  lines.push('');
  lines.push('## 2. Symbol-naming cohorts', '');
  lines.push('| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |');
  lines.push('|--------------------------|------:|--------------------|----------------:|--------------:|--------|');
  for (const c of symbolCohorts) {
    lines.push(`| \`${c.cohort}\` | ${c.items} | \`${c.convention}\` | ${c.rate}% | ${c.outliers ?? 0} | ${c.label} ✅ |`);
  }
  lines.push('');
  if (outliers !== null) {
    lines.push('## 3. Outliers', '');
    lines.push('| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |');
    lines.push('|----------|--------|------|--------------------|--------------------|--------|');
    for (const o of outliers) {
      lines.push(`| \`${o.identity}\` | \`${o.cohort}\` | \`${o.name}\` | \`${o.observed}\` | \`${o.dominant}\` | ${o.label} ⚠ |`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

// UN-1..UN-4: clean parse
{
  const text = buildNamingCanon({
    fileCohorts: [{ cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
    symbolCohorts: [{ cohort: 'src::helper-export', items: 2, convention: 'camelCase', rate: 100, label: 'camelCase-dominant' }],
  });
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-1. clean naming canon → status=clean',
    r.status === 'clean', `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('UN-2. fileCohorts Map keyed by submodule',
    r.fileCohorts instanceof Map && r.fileCohorts.has('src'), '');
  assert('UN-3. symbolCohorts Map keyed by submodule::kind',
    r.symbolCohorts instanceof Map && r.symbolCohorts.has('src::helper-export'), '');
  assert('UN-4. outliers Map empty when §3 absent',
    r.outliers instanceof Map && r.outliers.size === 0, '');
}

// UN-5: §3 outliers present + clean
{
  const text = buildNamingCanon({
    fileCohorts: [{ cohort: 'src', files: 4, convention: 'kebab-case', rate: 75, outliers: 1, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
    outliers: [{ identity: 'src/OLD.ts', cohort: 'src', name: 'OLD.ts', observed: 'UPPER_SNAKE', dominant: 'kebab-case', label: 'convention-outlier' }],
  });
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-5. §3 outliers parsed into outliers Map (clean)',
    r.status === 'clean' && r.outliers.size === 1 && r.outliers.has('src/OLD.ts'),
    `status=${r.status}, keys=${[...(r.outliers?.keys() ?? [])]}`);
}

// UN-6: P1-7 — §3 missing Status column → parse-error (not silent)
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 1 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------------|------:|--------------------|----------------:|--------------:|--------|',
    '',
    '## 3. Outliers', '',
    '| Identity | Cohort | Name | ObservedConvention | DominantConvention |',  // missing Status
    '|----------|--------|------|--------------------|--------------------|',
    '| `src/OLD.ts` | `src` | `OLD.ts` | `UPPER_SNAKE` | `kebab-case` |',
    '',
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-6. §3 outliers malformed (missing Status) → parse-error (NOT silent zero outliers)',
    r.status === 'parse-error',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// UN-7: §1 malformed header → parse-error
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files |',
    '|--------------------|------:|',
    '| `src` | 1 |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------------|------:|--------------------|----------------:|--------------:|--------|',
    '',
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-7. §1 malformed header → parse-error',
    r.status === 'parse-error', `status=${r.status}`);
}

// UN-8: unknown Status token in §1 → parse-error
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 1 | `kebab-case` | 100% | 0 | bogus-label |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------------|------:|--------------------|----------------:|--------------:|--------|',
    '',
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-8. unknown Status token in §1 row → parse-error',
    r.status === 'parse-error', `status=${r.status}`);
}

// UN-9: NAMING_LABEL_SET size = 10
assert('UN-9. NAMING_LABEL_SET size = 10 (§12.3)',
  NAMING_LABEL_SET.size === 10, `size=${NAMING_LABEL_SET?.size}`);

// UN-10: CATEGORY_TO_FAMILY has 5 naming-drift entries
{
  const entries = Object.keys(CATEGORY_TO_FAMILY).filter((k) => k.startsWith('naming-drift::'));
  assert('UN-10. CATEGORY_TO_FAMILY has exactly 5 naming-drift entries',
    entries.length === 5, `count=${entries.length}`);
}

// UN-11: prefix memo does not poison
{
  const text =
    '| Name | Note |\n|------|------|\n| x | y |\n\n' +
    buildNamingCanon({
      fileCohorts: [{ cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
      symbolCohorts: [],
    });
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-11. prefix memo table does NOT poison naming parse',
    r.status === 'clean' && r.fileCohorts.has('src'), `status=${r.status}`);
}

// UN-12: §3 absent (no section at all) → clean + empty outliers
{
  const text = buildNamingCanon({
    fileCohorts: [{ cohort: 'src', files: 3, convention: 'kebab-case', rate: 100, label: 'kebab-case-dominant' }],
    symbolCohorts: [],
    // outliers: null → section omitted
  });
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-12. §3 section absent → clean + empty outliers Map (P1-7 absent vs malformed distinction)',
    r.status === 'clean' && r.outliers.size === 0, '');
}

// UN-13: §2 Symbol cohorts section ABSENT → parse-error (Finding #2 post-landing)
// §2 is REQUIRED per canon-drift.md §5.d; only §3 is optional. Missing §2
// was previously silent-clean which is a canonical shape violation.
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 3 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |',
    '',
    // §2 section entirely missing. §3 also missing (OK for §3).
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-13. §2 Symbol cohorts section ABSENT → parse-error (REQUIRED section)',
    r.status === 'parse-error' &&
    r.diagnostics.some((d) => d.reason === 'missing-required-section' && /Symbol/.test(d.section ?? '')),
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// UN-14: §2 present but empty (no table, just heading) → clean + empty symbolCohorts
// When the section exists but contains a "_No symbol-naming cohorts observed._"
// banner or simply no table rows, that's a legitimate empty cohort set — not
// a canonical shape violation. Distinct from UN-13 (section entirely absent).
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 3 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '_No symbol-naming cohorts observed._',
    '',
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-14. §2 present-but-no-table → clean + empty symbolCohorts (legitimate empty set)',
    r.status === 'clean' && r.symbolCohorts.size === 0,
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// UN-15: naming parser normalizes display placeholders to null.
// P3 renders absent dominant conventions as `—`; P5 must treat that as the
// same value as the fresh collector's internal null to avoid instant false
// drift after promoting a draft to canonical.
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 5 | — | 40% | 0 | mixed-convention ⚠ |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src::helper-export` | 2 | `—` | — | — | insufficient-evidence ℹ |',
    '',
    '## 3. Outliers', '',
    '| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |',
    '|----------|--------|------|--------------------|--------------------|--------|',
    '| `src/OLD.ts` | `src` | `OLD.ts` | `UPPER_SNAKE` | — | convention-outlier ⚠ |',
    '',
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-15a. §1 DominantConvention "—" parses as null',
    r.status === 'clean' && r.fileCohorts.get('src')?.dominantConvention === null,
    `status=${r.status}, value=${r.fileCohorts.get('src')?.dominantConvention}`);
  assert('UN-15b. §2 backticked "—" parses as null',
    r.symbolCohorts.get('src::helper-export')?.dominantConvention === null,
    `value=${r.symbolCohorts.get('src::helper-export')?.dominantConvention}`);
  assert('UN-15c. §3 outlier DominantConvention "—" parses as null',
    r.outliers.get('src/OLD.ts')?.dominantConvention === null,
    `value=${r.outliers.get('src/OLD.ts')?.dominantConvention}`);
}

// UN-16: low-info-excluded rows in §3 validate but do not enter outlier diff set.
// P3 renders low-info rows in the informational Outliers section; P5 must not
// treat them as canonical outliers or a fresh P3→P5 round-trip emits false
// outlier-resolved drift on names like build.ts.
{
  const text = [
    '## 1. File-naming cohorts', '',
    '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
    '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
    '| `src` | 3 | `camelCase` | 100% | 0 | camelCase-dominant ✅ |',
    '',
    '## 2. Symbol-naming cohorts', '',
    '_No symbol-naming cohorts observed._',
    '',
    '## 3. Outliers', '',
    '| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |',
    '|----------|--------|------|--------------------|--------------------|--------|',
    '| `src/build.ts` | `src` | `build` | `camelCase` | `camelCase` | low-info-excluded ℹ |',
    '| `src/vite.config.ts` | `src` | `vite.config` | `mixed` | `camelCase` | convention-outlier ⚠ |',
    '',
  ].join('\n');
  const r = parseNamingCanonText({ text, canonLabelSet: NAMING_LABEL_SET });
  assert('UN-16a. low-info-excluded §3 row is accepted by schema',
    r.status === 'clean',
    `status=${r.status}, diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('UN-16b. low-info-excluded §3 row is excluded from drift outlier Map',
    !r.outliers.has('src/build.ts') && r.outliers.has('src/vite.config.ts') && r.outliers.size === 1,
    `outliers=${JSON.stringify([...r.outliers.keys()])}`);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
