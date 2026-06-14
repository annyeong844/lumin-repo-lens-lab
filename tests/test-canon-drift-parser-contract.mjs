// tests/test-canon-drift-parser-contract.mjs
//
// P5-0 Step 2 — round-trip smoke fixture (reviewer recommendation 2 from
// `docs/history/phases/p5/session.md` v2: round-trip test over source-grep pin).
//
// For each of the 4 P3 canon renderers, build a MINIMAL in-memory input,
// call the renderer, extract the first table header row, and assert its
// column list matches the contract documented in `canonical/canon-drift.md`
// §5. Also cross-asserts that §5 literal text mentions each column name.
//
// Guards: any renderer change that shifts column headers immediately fails
// this test — and its paired §5 spec edit is flagged in the same reviewer
// pass (the test fails either way until both are in sync).

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { renderTypeOwnership } from '../_lib/canon-draft-types.mjs';
import { renderHelperRegistry } from '../_lib/canon-draft-helpers.mjs';
import { renderTopology } from '../_lib/canon-draft-topology.mjs';
import { renderNaming } from '../_lib/canon-draft-naming.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) {
    passed += 1;
    console.log(`  PASS  ${label}`);
  } else {
    failed += 1;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

const driftPath = path.join(DIR, 'canonical', 'canon-drift.md');
const driftText = readFileSync(driftPath, 'utf8');

// Extract a markdown table header line. The first `|`-delimited row whose
// cells match `/^[A-Z]/` (column label convention) is returned. Returns
// the raw cell list (trimmed, with leading/trailing empty cells dropped).
function extractHeaderColumns(markdown, startAfter = 0) {
  const lines = markdown.split(/\r?\n/);
  for (let i = startAfter; i < lines.length; i += 1) {
    const line = lines[i];
    if (!line.trimStart().startsWith('|')) continue;
    // Skip separator row like `|-----|---|:----|`
    if (/^\s*\|[\s:|-]+\|\s*$/.test(line)) continue;
    const cells = line.split('|').map((c) => c.trim()).filter(Boolean);
    if (cells.length >= 2 && cells.every((c) => /^[A-Z]/.test(c))) {
      return { cells, lineIndex: i };
    }
  }
  return null;
}

// Extract the header row that appears immediately after a section heading.
function extractHeaderUnder(markdown, headingText) {
  const idx = markdown.indexOf(headingText);
  if (idx < 0) return null;
  return extractHeaderColumns(markdown, markdown.slice(0, idx).split(/\r?\n/).length);
}

// ── 1. type-ownership round-trip ───────────────────────────────

{
  const ownerFile = 'src/foo.ts';
  const exportedName = 'Foo';
  const identity = `${ownerFile}::${exportedName}`;
  const typeDefsByIdentity = new Map([[identity, {
    name: exportedName,
    ownerFile,
    line: 10,
    fanIn: 1,
    kind: 'alias',
    anyContamination: undefined,
  }]]);
  const identitiesByName = new Map([[exportedName, [identity]]]);
  const typeUsesByIdentity = new Map([[identity, { fanIn: 1 }]]);

  const md = renderTypeOwnership({
    typeDefsByIdentity,
    identitiesByName,
    typeUsesByIdentity,
    diagnostics: [],
    meta: { scope: 'test-scope', source: 'fixture' },
  });

  const header = extractHeaderColumns(md);
  const expected = ['Name', 'Identity', 'Owner', 'Fan-in', 'Fan-in space', 'Status', 'Tags'];
  assert('RT-T1. renderTypeOwnership emits a table header',
    header !== null, 'no `| X | Y |` header row found');
  assert('RT-T2. renderTypeOwnership header has 7 columns',
    header && header.cells.length === 7, `got ${header?.cells.length ?? 0}`);
  assert('RT-T3. renderTypeOwnership header matches canon-drift §5.a',
    header && expected.every((c, i) => header.cells[i] === c),
    `expected ${expected.join(' | ')}; got ${header?.cells.join(' | ')}`);
  for (const col of expected) {
    assert(`RT-T4.${col}. §5.a lists column "${col}"`,
      driftText.includes('`' + col + '`'),
      `canon-drift.md §5.a missing backticked column \`${col}\``);
  }
}

// ── 2. helper-registry round-trip ──────────────────────────────

{
  const ownerFile = 'src/foo.ts';
  const exportedName = 'doFoo';
  const identity = `${ownerFile}::${exportedName}`;
  const helperDefsByIdentity = new Map([[identity, {
    name: exportedName,
    ownerFile,
    line: 20,
    fanIn: 1,
    kind: 'function',
    signature: '(x: number) => number',
    paramCount: 1,
    returnKind: 'primitive',
    anyContamination: undefined,
  }]]);
  const helpersByName = new Map([[exportedName, [identity]]]);

  const md = renderHelperRegistry({
    helperDefsByIdentity,
    helpersByName,
    distinctConsumerFiles: new Map(),
    diagnostics: [],
    meta: { scope: 'test-scope', source: 'fixture' },
  });

  const header = extractHeaderColumns(md);
  const expected = ['Name', 'Identity', 'Owner', 'Signature', 'Fan-in', 'Status', 'Tags', 'Any / unknown signal'];
  assert('RT-H1. renderHelperRegistry emits a table header',
    header !== null, 'no header row');
  assert('RT-H2. renderHelperRegistry header has 8 columns',
    header && header.cells.length === 8, `got ${header?.cells.length ?? 0}`);
  assert('RT-H3. renderHelperRegistry header matches canon-drift §5.b',
    header && expected.every((c, i) => header.cells[i] === c),
    `expected ${expected.join(' | ')}; got ${header?.cells.join(' | ')}`);
  for (const col of expected) {
    assert(`RT-H4.${col}. §5.b lists column "${col}"`,
      driftText.includes('`' + col + '`'),
      `canon-drift.md §5.b missing backticked column \`${col}\``);
  }
}

// ── 3. topology round-trip ─────────────────────────────────────

{
  const submodulesByPath = new Map([
    ['src/lib', {
      name: 'src/lib',
      files: 1,
      loc: 50,
      inDegree: 0,
      outDegree: 0,
      sccMember: false,
    }],
  ]);

  const md = renderTopology({
    submodulesByPath,
    crossEdgesForDisplay: [{ from: 'src/a', to: 'src/b', count: 2 }],
    sccs: [],
    oversizeFiles: [{ file: 'src/giant.ts', loc: 450, label: 'oversize', marker: '⚠' }],
    workspaces: null,
    diagnostics: [],
    meta: { scope: 'test-scope', source: 'fixture', crossEdgeSource: 'full-list' },
  });

  // §1. Submodule inventory — expect 8 columns.
  const inv = extractHeaderUnder(md, '## 1. Submodule inventory');
  const expectedInv = ['Submodule', 'Files', 'LOC', 'In-edges', 'Out-edges', 'SCC', 'Status', 'Tags'];
  assert('RT-Y1. topology §1 inventory header present',
    inv !== null, 'no header under "## 1. Submodule inventory"');
  assert('RT-Y2. topology §1 inventory has 8 columns',
    inv && inv.cells.length === 8, `got ${inv?.cells.length ?? 0}`);
  assert('RT-Y3. topology §1 header matches canon-drift §5.c.§1',
    inv && expectedInv.every((c, i) => inv.cells[i] === c),
    `expected ${expectedInv.join(' | ')}; got ${inv?.cells.join(' | ')}`);

  // §2. Cross-submodule edges — expect 3 columns (canon-drift §5.c §2, v1.1).
  const ce = extractHeaderUnder(md, '## 2. Cross-submodule edges');
  const expectedCe = ['From', 'To', 'Count'];
  assert('RT-Y4cross. topology §2 cross-edges header present',
    ce !== null, 'no header under "## 2. Cross-submodule edges"');
  assert('RT-Y5cross. topology §2 cross-edges has 3 columns',
    ce && ce.cells.length === 3, `got ${ce?.cells.length ?? 0}`);
  assert('RT-Y6cross. topology §2 header matches canon-drift §5.c.§2',
    ce && expectedCe.every((c, i) => ce.cells[i] === c),
    `expected ${expectedCe.join(' | ')}; got ${ce?.cells.join(' | ')}`);

  // §4. Oversize — expect 3 columns.
  const ov = extractHeaderUnder(md, '## 4. Oversize files');
  const expectedOv = ['File', 'LOC', 'Status'];
  assert('RT-Y4. topology §4 oversize header present',
    ov !== null, 'no header under "## 4. Oversize files"');
  assert('RT-Y5. topology §4 oversize has 3 columns',
    ov && ov.cells.length === 3, `got ${ov?.cells.length ?? 0}`);
  assert('RT-Y6. topology §4 header matches canon-drift §5.c.§4',
    ov && expectedOv.every((c, i) => ov.cells[i] === c),
    `expected ${expectedOv.join(' | ')}; got ${ov?.cells.join(' | ')}`);

  // §5 cross-ref. canon-drift §5.c names each column.
  for (const col of [...expectedInv, ...expectedCe, ...expectedOv]) {
    assert(`RT-Y7.${col}. §5.c lists column "${col}"`,
      driftText.includes('`' + col + '`'),
      `canon-drift.md §5.c missing \`${col}\``);
  }
}

// ── 4. naming round-trip ───────────────────────────────────────

{
  const fileCohort = {
    cohortId: 'src/lib',
    members: [{ file: 'src/lib/a.ts' }, { file: 'src/lib/b.ts' }],
    classification: {
      label: 'consistent-kebab-case',
      marker: '✅',
      consistencyRate: 1.0,
      dominantConvention: 'kebab-case',
    },
  };
  const symbolCohort = {
    cohortId: 'src/lib::helper-export',
    members: [{ name: 'doA' }, { name: 'doB' }],
    classification: {
      label: 'consistent-camelCase',
      marker: '✅',
      consistencyRate: 1.0,
      dominantConvention: 'camelCase',
    },
  };

  const md = renderNaming({
    fileCohorts: new Map([[fileCohort.cohortId, fileCohort]]),
    symbolCohorts: new Map([[symbolCohort.cohortId, symbolCohort]]),
    perItemRows: [{
      cohortId: 'src/lib',
      itemLabel: 'convention-outlier',
      identity: 'src/lib/WEIRD.ts',
      cohort: 'src/lib',
      name: 'WEIRD.ts',
      observedConvention: 'UPPERCASE',
      dominantConvention: 'kebab-case',
      status: 'outlier ❌',
    }],
    diagnostics: [],
    meta: { scope: 'test-scope', source: 'fixture' },
  });

  // §1. File cohorts — expect 6 columns.
  const fc = extractHeaderUnder(md, '## 1. File-naming cohorts');
  const expectedFc = ['Cohort (submodule)', 'Files', 'DominantConvention', 'ConsistencyRate', 'OutliersCount', 'Status'];
  assert('RT-N1. naming §1 file-cohort header present',
    fc !== null, 'no header under "## 1. File-naming cohorts"');
  assert('RT-N2. naming §1 file-cohort has 6 columns',
    fc && fc.cells.length === 6, `got ${fc?.cells.length ?? 0}`);
  assert('RT-N3. naming §1 header matches canon-drift §5.d.§1',
    fc && expectedFc.every((c, i) => fc.cells[i] === c),
    `expected ${expectedFc.join(' | ')}; got ${fc?.cells.join(' | ')}`);

  // §2. Symbol cohorts — expect 6 columns.
  const sc = extractHeaderUnder(md, '## 2. Symbol-naming cohorts');
  const expectedSc = ['Cohort (submodule::kind)', 'Items', 'DominantConvention', 'ConsistencyRate', 'OutliersCount', 'Status'];
  assert('RT-N4. naming §2 symbol-cohort header present',
    sc !== null, 'no header under "## 2. Symbol-naming cohorts"');
  assert('RT-N5. naming §2 symbol-cohort has 6 columns',
    sc && sc.cells.length === 6, `got ${sc?.cells.length ?? 0}`);
  assert('RT-N6. naming §2 header matches canon-drift §5.d.§2',
    sc && expectedSc.every((c, i) => sc.cells[i] === c),
    `expected ${expectedSc.join(' | ')}; got ${sc?.cells.join(' | ')}`);

  // §5 cross-ref — §5.d must mention the column names.
  const namingExpectedForCrossRef = [
    'Cohort (submodule)', 'Files', 'DominantConvention', 'ConsistencyRate',
    'OutliersCount', 'Status', 'Cohort (submodule::kind)', 'Items',
  ];
  for (const col of namingExpectedForCrossRef) {
    assert(`RT-N7.${col}. §5.d lists column "${col}"`,
      driftText.includes('`' + col + '`'),
      `canon-drift.md §5.d missing \`${col}\``);
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
