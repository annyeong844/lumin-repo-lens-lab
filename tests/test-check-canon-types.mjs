// tests/test-check-canon-types.mjs
//
// P5-1 Step 0 — RED test for `_lib/check-canon-types.mjs` (drift engine).
//
// Pins the identity-diff + 1:1 owner-change upgrade + label-preserving
// renderer per canon-drift.md §3.1 type-drift categories + reviewer
// P0-5 (owner-changed Canon/Fresh label columns).

import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';

import { detectTypeOwnershipDrift } from '../_lib/check-canon-types.mjs';

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

const TYPE_LABEL_SET = new Set([
  'zero-internal-fan-in', 'low-signal-type-name', 'DUPLICATE_STRONG',
  'DUPLICATE_REVIEW', 'LOCAL_COMMON_NAME', 'single-owner-strong',
  'single-owner-weak', 'severely-any-contaminated', 'ANY_COLLISION',
]);

const workdir = mkdtempSync(path.join(tmpdir(), 'p5-1-types-'));

function writeCanon(canonPath, rows) {
  let md = '| Name | Identity | Owner | Fan-in | Status | Tags |\n';
  md += '|------|----------|-------|-------:|--------|------|\n';
  for (const r of rows) {
    md += `| \`${r.name}\` | \`${r.identity}\` | \`${r.owner}\` | ${r.fanIn} | ${r.label} ✅ | |\n`;
  }
  writeFileSync(canonPath, md, 'utf8');
}

function writeCanonWithFanInSpace(canonPath, rows) {
  let md = '| Name | Identity | Owner | Fan-in | Fan-in space | Status | Tags |\n';
  md += '|------|----------|-------|-------:|--------------|--------|------|\n';
  for (const r of rows) {
    const fanInSpace = r.fanInSpace ?? 'value 0, type 0, broad 0';
    md += `| \`${r.name}\` | \`${r.identity}\` | \`${r.owner}\` | ${r.fanIn} | ${fanInSpace} | ${r.label} ✅ | |\n`;
  }
  writeFileSync(canonPath, md, 'utf8');
}

// Minimal symbols.json shape for collectTypeIdentities — mirrors
// build-symbol-graph.mjs output (defIndex + fanInByIdentity).
function makeSymbols(typeDefs) {
  const defIndex = {};
  const fanInByIdentity = {};
  for (const d of typeDefs) {
    if (!defIndex[d.ownerFile]) defIndex[d.ownerFile] = {};
    defIndex[d.ownerFile][d.name] = {
      kind: d.kind ?? 'TSInterfaceDeclaration',
      line: d.line,
      anyContamination: d.anyContamination ?? null,
    };
    fanInByIdentity[`${d.ownerFile}::${d.name}`] = d.fanIn;
  }
  return {
    meta: { scope: 'fixture', supports: { identityFanIn: true } },
    defIndex,
    fanInByIdentity,
    reExportsByFile: {},
  };
}

function makeHash(ch) {
  return `sha256:${ch.repeat(64)}`;
}

function makeShapeIndex(facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    if (!groupsByHash[fact.hash]) groupsByHash[fact.hash] = [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  return {
    schemaVersion: 'shape-index.v1',
    meta: { complete },
    facts: facts.map((fact) => ({ ...fact })),
    groupsByHash,
  };
}

// ── skipped-missing-canon propagation ──────────────────────────

{
  const r = detectTypeOwnershipDrift({
    canonPath: path.join(workdir, 'nope.md'),
    symbols: makeSymbols([]),
    canonLabelSet: TYPE_LABEL_SET,
  });
  assert('T-1. missing canon → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon',
    `status=${r.status}`);
  assert('T-2. missing canon → drifts empty + reportMarkdown null',
    r.drifts.length === 0 && r.reportMarkdown === null,
    `drifts=${r.drifts.length}, md=${r.reportMarkdown === null}`);
}

// ── identity-added ─────────────────────────────────────────────

{
  const canonPath = path.join(workdir, 'canon-added.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/foo.ts::Foo', owner: 'src/foo.ts:10', fanIn: 3, label: 'single-owner-strong' },
  ]);
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 },
    { name: 'Bar', ownerFile: 'src/bar.ts', line: 5, fanIn: 1 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  const added = r.drifts.filter((d) => d.category === 'identity-added');
  assert('T-3. identity-added count = 1 when fresh has +1 identity',
    added.length === 1, `got ${added.length}: ${JSON.stringify(r.drifts)}`);
  assert('T-4. identity-added.identity is the fresh-only one',
    added[0] && added[0].identity === 'src/bar.ts::Bar',
    `identity=${added[0]?.identity}`);
  assert('T-5. identity-added family = added',
    added[0] && added[0].family === 'added',
    `family=${added[0]?.family}`);
}

// ── identity-removed ───────────────────────────────────────────

{
  const canonPath = path.join(workdir, 'canon-removed.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/foo.ts::Foo', owner: 'src/foo.ts:10', fanIn: 3, label: 'single-owner-strong' },
    { name: 'Gone', identity: 'src/gone.ts::Gone', owner: 'src/gone.ts:2', fanIn: 1, label: 'single-owner-weak' },
  ]);
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  const removed = r.drifts.filter((d) => d.category === 'identity-removed');
  assert('T-6. identity-removed count = 1 when canon has an extra identity',
    removed.length === 1, `got ${removed.length}`);
  assert('T-7. identity-removed.identity is the canon-only one',
    removed[0] && removed[0].identity === 'src/gone.ts::Gone',
    `identity=${removed[0]?.identity}`);
}

// ── label-changed ──────────────────────────────────────────────

{
  const canonPath = path.join(workdir, 'canon-label.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/foo.ts::Foo', owner: 'src/foo.ts:10', fanIn: 0, label: 'zero-internal-fan-in' },
  ]);
  // Fresh fanIn=3 will classify as single-owner-strong
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  const labelChanged = r.drifts.filter((d) => d.category === 'label-changed');
  assert('T-8. label-changed count = 1 when labels differ at same identity',
    labelChanged.length === 1, `drifts=${JSON.stringify(r.drifts)}`);
  assert('T-9. label-changed record carries both canon.label and fresh.label',
    labelChanged[0] &&
    labelChanged[0].canon.label === 'zero-internal-fan-in' &&
    labelChanged[0].fresh.label === 'single-owner-strong',
    `rec=${JSON.stringify(labelChanged[0])}`);
  assert('T-10. label-changed family = label-changed',
    labelChanged[0] && labelChanged[0].family === 'label-changed',
    `family=${labelChanged[0]?.family}`);
}

// ── owner-changed (1:1 upgrade) ────────────────────────────────

{
  const canonPath = path.join(workdir, 'canon-owner.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/a.ts::Foo', owner: 'src/a.ts:10', fanIn: 3, label: 'single-owner-strong' },
  ]);
  // Fresh: same exportedName 'Foo' but at src/b.ts
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/b.ts', line: 4, fanIn: 3 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  const ownerChanged = r.drifts.filter((d) => d.category === 'owner-changed');
  assert('T-11. owner-changed count = 1 when same exportedName moves file (1:1)',
    ownerChanged.length === 1,
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('T-12. owner-changed consumes the add + remove (no double emission)',
    r.drifts.filter((d) => d.category === 'identity-added').length === 0 &&
    r.drifts.filter((d) => d.category === 'identity-removed').length === 0,
    `all=${JSON.stringify(r.drifts.map((d) => d.category))}`);
  assert('T-13. owner-changed carries canon.owner + fresh.owner (different files)',
    ownerChanged[0] && ownerChanged[0].canon.owner.startsWith('src/a.ts') &&
    ownerChanged[0].fresh.owner.startsWith('src/b.ts'),
    `rec=${JSON.stringify(ownerChanged[0])}`);
  assert('T-14. owner-changed carries canon.label AND fresh.label (P0-5)',
    ownerChanged[0] &&
    typeof ownerChanged[0].canon.label === 'string' &&
    typeof ownerChanged[0].fresh.label === 'string',
    `labels=${JSON.stringify({ c: ownerChanged[0]?.canon?.label, f: ownerChanged[0]?.fresh?.label })}`);
  assert('T-15. owner-changed family = structural-status-changed',
    ownerChanged[0] && ownerChanged[0].family === 'structural-status-changed',
    `family=${ownerChanged[0]?.family}`);
  // Reviewer Finding #1 — top-level identity MUST follow canon-drift.md §4:
  // `ownerFile::exportedName`. Old/new live inside canon.identity + fresh.identity.
  assert('T-15b. owner-changed top-level identity is ownerFile::exportedName (no compound/arrow)',
    ownerChanged[0] && /^[^:]+::[^:]+$/.test(ownerChanged[0].identity) &&
    !ownerChanged[0].identity.includes('→') && !ownerChanged[0].identity.includes('->'),
    `identity=${ownerChanged[0]?.identity}`);
  assert('T-15c. owner-changed top-level identity anchors to canon (= canon.identity = remId)',
    ownerChanged[0] && ownerChanged[0].identity === 'src/a.ts::Foo' &&
    ownerChanged[0].canon.identity === 'src/a.ts::Foo' &&
    ownerChanged[0].fresh.identity === 'src/b.ts::Foo',
    `top=${ownerChanged[0]?.identity}, canon=${ownerChanged[0]?.canon?.identity}, fresh=${ownerChanged[0]?.fresh?.identity}`);
}

// ── ambiguous 2:1 → stays as added/removed with low confidence ─

{
  const canonPath = path.join(workdir, 'canon-ambig.md');
  writeCanon(canonPath, [
    { name: 'X', identity: 'src/a.ts::X', owner: 'src/a.ts:1', fanIn: 1, label: 'single-owner-weak' },
  ]);
  const sym = makeSymbols([
    { name: 'X', ownerFile: 'src/b.ts', line: 1, fanIn: 1 },
    { name: 'X', ownerFile: 'src/c.ts', line: 1, fanIn: 1 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  const ownerChanged = r.drifts.filter((d) => d.category === 'owner-changed');
  assert('T-16. 1-remove + 2-add same name → no owner-changed (ambiguous)',
    ownerChanged.length === 0,
    `drifts=${JSON.stringify(r.drifts)}`);
  const added = r.drifts.filter((d) => d.category === 'identity-added');
  const removed = r.drifts.filter((d) => d.category === 'identity-removed');
  assert('T-17. ambiguous case stays as 1 removed + 2 added',
    removed.length === 1 && added.length === 2,
    `removed=${removed.length}, added=${added.length}`);
  assert('T-18. ambiguous records flagged with confidence=low',
    [...added, ...removed].every((d) => d.confidence === 'low'),
    `confidences=${JSON.stringify([...added, ...removed].map((d) => d.confidence))}`);
}

// ── ambiguous rename resolved by unique shape pair ──────────────

{
  const canonPath = path.join(workdir, 'canon-shape-upgrade.md');
  writeCanon(canonPath, [
    { name: 'X', identity: 'src/a.ts::X', owner: 'src/a.ts:1', fanIn: 1, label: 'single-owner-weak' },
  ]);
  const sym = makeSymbols([
    { name: 'X', ownerFile: 'src/b.ts', line: 1, fanIn: 1 },
    { name: 'X', ownerFile: 'src/c.ts', line: 1, fanIn: 1 },
  ]);
  const r = detectTypeOwnershipDrift({
    canonPath,
    symbols: sym,
    canonLabelSet: TYPE_LABEL_SET,
    shapeIndex: makeShapeIndex([
      { identity: 'src/a.ts::X', hash: makeHash('a') },
      { identity: 'src/b.ts::X', hash: makeHash('a') },
      { identity: 'src/c.ts::X', hash: makeHash('b') },
    ]),
  });
  const ownerChanged = r.drifts.filter((d) => d.category === 'owner-changed');
  const added = r.drifts.filter((d) => d.category === 'identity-added');
  const removed = r.drifts.filter((d) => d.category === 'identity-removed');
  assert('T-18b. shape-index unique hash pair upgrades ambiguous rename to owner-changed',
    ownerChanged.length === 1 &&
    ownerChanged[0].canon.identity === 'src/a.ts::X' &&
    ownerChanged[0].fresh.identity === 'src/b.ts::X',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('T-18c. uniquely resolved leftover added record stays as the true extra identity',
    added.length === 1 && added[0].identity === 'src/c.ts::X' && removed.length === 0,
    `added=${JSON.stringify(added)}, removed=${JSON.stringify(removed)}`);
  assert('T-18d. uniquely resolved leftover add is high confidence',
    added.length === 1 && added[0].confidence === 'high',
    `confidence=${added[0]?.confidence}`);
}

// ── partial shape pairing leaves unresolved remainder low ───────

{
  const canonPath = path.join(workdir, 'canon-shape-partial.md');
  writeCanon(canonPath, [
    { name: 'X', identity: 'src/a.ts::X', owner: 'src/a.ts:1', fanIn: 1, label: 'single-owner-weak' },
    { name: 'X', identity: 'src/b.ts::X', owner: 'src/b.ts:1', fanIn: 1, label: 'single-owner-weak' },
  ]);
  const sym = makeSymbols([
    { name: 'X', ownerFile: 'src/c.ts', line: 1, fanIn: 1 },
    { name: 'X', ownerFile: 'src/d.ts', line: 1, fanIn: 1 },
  ]);
  const r = detectTypeOwnershipDrift({
    canonPath,
    symbols: sym,
    canonLabelSet: TYPE_LABEL_SET,
    shapeIndex: makeShapeIndex([
      { identity: 'src/a.ts::X', hash: makeHash('a') },
      { identity: 'src/c.ts::X', hash: makeHash('a') },
    ]),
  });
  const ownerChanged = r.drifts.filter((d) => d.category === 'owner-changed');
  const added = r.drifts.filter((d) => d.category === 'identity-added');
  const removed = r.drifts.filter((d) => d.category === 'identity-removed');
  assert('T-18e. unique shape pair upgrades only the grounded subset',
    ownerChanged.length === 1 &&
    ownerChanged[0].canon.identity === 'src/a.ts::X' &&
    ownerChanged[0].fresh.identity === 'src/c.ts::X',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('T-18f. unresolved ambiguous remainder stays as one added + one removed',
    added.length === 1 && added[0].identity === 'src/d.ts::X' &&
    removed.length === 1 && removed[0].identity === 'src/b.ts::X',
    `added=${JSON.stringify(added)}, removed=${JSON.stringify(removed)}`);
  assert('T-18g. unresolved ambiguous remainder stays low confidence',
    added[0]?.confidence === 'low' && removed[0]?.confidence === 'low',
    `confidences=${JSON.stringify({ add: added[0]?.confidence, rem: removed[0]?.confidence })}`);
}

// ── malformed shape index fails closed ──────────────────────────

{
  const canonPath = path.join(workdir, 'canon-shape-invalid.md');
  writeCanon(canonPath, [
    { name: 'X', identity: 'src/a.ts::X', owner: 'src/a.ts:1', fanIn: 1, label: 'single-owner-weak' },
  ]);
  const sym = makeSymbols([
    { name: 'X', ownerFile: 'src/b.ts', line: 1, fanIn: 1 },
    { name: 'X', ownerFile: 'src/c.ts', line: 1, fanIn: 1 },
  ]);
  const r = detectTypeOwnershipDrift({
    canonPath,
    symbols: sym,
    canonLabelSet: TYPE_LABEL_SET,
    shapeIndex: { schemaVersion: 'wrong' },
  });
  assert('T-18h. malformed shape-index does not force an owner-changed upgrade',
    !r.drifts.some((d) => d.category === 'owner-changed'),
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('T-18i. malformed shape-index falls closed to low-confidence add/remove ambiguity',
    r.drifts.filter((d) => d.category === 'identity-added').length === 2 &&
    r.drifts.filter((d) => d.category === 'identity-removed').length === 1 &&
    r.drifts.every((d) => d.category === 'owner-changed' || d.confidence === 'low'),
    `drifts=${JSON.stringify(r.drifts)}`);
}

// ── clean: no drift ────────────────────────────────────────────

{
  const canonPath = path.join(workdir, 'canon-clean.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/foo.ts::Foo', owner: 'src/foo.ts:10', fanIn: 3, label: 'single-owner-strong' },
  ]);
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  assert('T-19. clean run: status=clean + 0 drifts',
    r.status === 'clean' && r.drifts.length === 0,
    `status=${r.status}, drifts=${r.drifts.length}`);
  assert('T-20. clean run: reportMarkdown is a string (summary only)',
    typeof r.reportMarkdown === 'string' && r.reportMarkdown.length > 0,
    `md-length=${r.reportMarkdown?.length}`);
}

{
  const canonPath = path.join(workdir, 'canon-clean-fanin-space.md');
  writeCanonWithFanInSpace(canonPath, [
    {
      name: 'Foo',
      identity: 'src/foo.ts::Foo',
      owner: 'src/foo.ts:10',
      fanIn: 3,
      fanInSpace: 'value 2, type 1, broad 0',
      label: 'single-owner-strong',
    },
  ]);
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 },
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  assert('T-20b. type drift accepts promoted draft with Fan-in space column',
    r.status === 'clean' && r.drifts.length === 0,
    `status=${r.status}, drifts=${JSON.stringify(r.drifts)}, diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// ── renderer: owner-changed MD has Canon label + Fresh label columns ─

{
  const canonPath = path.join(workdir, 'canon-render.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/a.ts::Foo', owner: 'src/a.ts:10', fanIn: 3, label: 'single-owner-strong' },
  ]);
  const sym = makeSymbols([
    { name: 'Foo', ownerFile: 'src/b.ts', line: 4, fanIn: 0 },  // different label too
  ]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  assert('T-21. owner-changed MD table includes "Canon label" and "Fresh label" columns',
    r.reportMarkdown.includes('Canon label') && r.reportMarkdown.includes('Fresh label'),
    `md=${r.reportMarkdown.slice(0, 500)}`);
  assert('T-22. MD summary row counts match drift records',
    r.reportMarkdown.includes('owner-changed') && r.reportMarkdown.includes('| 1 |'),
    `md=${r.reportMarkdown.slice(0, 500)}`);
}

// ── renderer: sections omitted when zero ───────────────────────

{
  const canonPath = path.join(workdir, 'canon-clean2.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/foo.ts::Foo', owner: 'src/foo.ts:10', fanIn: 3, label: 'single-owner-strong' },
  ]);
  const sym = makeSymbols([{ name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 }]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  assert('T-23. zero-drift MD does NOT include individual category sections',
    !r.reportMarkdown.includes('## 2. identity-added') &&
    !r.reportMarkdown.includes('## 3. identity-removed'),
    `md=${r.reportMarkdown.slice(0, 500)}`);
  assert('T-24. zero-drift MD always includes §1 Summary',
    r.reportMarkdown.includes('## 1. Summary'),
    `md=${r.reportMarkdown.slice(0, 500)}`);
}

// ── drifts array structure (type-drift kind) ───────────────────

{
  const canonPath = path.join(workdir, 'canon-kind.md');
  writeCanon(canonPath, [
    { name: 'Foo', identity: 'src/foo.ts::Foo', owner: 'src/foo.ts:10', fanIn: 3, label: 'single-owner-strong' },
  ]);
  const sym = makeSymbols([]);
  const r = detectTypeOwnershipDrift({ canonPath, symbols: sym, canonLabelSet: TYPE_LABEL_SET });
  assert('T-25. every drift record has kind=type-drift',
    r.drifts.length > 0 && r.drifts.every((d) => d.kind === 'type-drift'),
    `kinds=${JSON.stringify(r.drifts.map((d) => d.kind))}`);
}

rmSync(workdir, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
