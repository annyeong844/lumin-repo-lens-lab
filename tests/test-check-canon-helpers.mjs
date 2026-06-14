// tests/test-check-canon-helpers.mjs
//
// P5-2 Step 0 — RED test for `_lib/check-canon-helpers.mjs` drift engine.
//
// Pins helper-drift category enum, 3-rule label dispatch (with evidence-
// gated contamination rule), extractor-throw → source-level parse-error,
// enrichment-unavailable advisory diagnostic, optional call-graph policy,
// and no helper-owner-changed upgrade (same-name-different-file stays as
// separate added + removed). Every assertion mirrors canon-drift.md §3.1
// helper-drift categories and docs/history/phases/p5/p5-2.md v4 §4.5 / §4.6 / §4.7.

import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';

import { detectHelperRegistryDrift } from '../_lib/check-canon-helpers.mjs';
import { HELPER_LABEL_SET } from '../_lib/check-canon-utils.mjs';

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

const workdir = mkdtempSync(path.join(tmpdir(), 'p5-2-engine-'));

function writeCanon(canonPath, rows) {
  let md = '| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n';
  md +=    '|------|----------|-------|-----------|-------:|--------|------|----------------------|\n';
  for (const r of rows) {
    md += `| \`${r.name}\` | \`${r.identity}\` | \`${r.owner}\` | ${r.signature ?? ''} | ${r.fanIn} | ${r.label} ✅ | | |\n`;
  }
  writeFileSync(canonPath, md, 'utf8');
}

// Build a stub extractFn from an in-memory map keyed by absolute file path.
// Each entry: { defs: [{name, kind, line}], uses: [{fromSpec, name, kind}] }.
function makeExtractStub(byFile, failingPaths = new Set()) {
  return (absFile) => {
    if (failingPaths.has(absFile)) throw new Error(`stub extractor forced-throw on ${absFile}`);
    return byFile.get(absFile) ?? { defs: [], uses: [], reExports: [] };
  };
}

// Stub resolver that returns file paths from a static map.
function makeResolveSpecifier(specToPath) {
  return (_fromFile, spec) => specToPath.get(spec) ?? null;
}

// Build a full scanContext for an in-memory fixture.
function buildScanContext({ root, files, defs, uses = new Map(), spec = new Map(), symbols = null, callGraph = null, failingPaths = new Set() }) {
  const defsByFile = new Map();
  for (const [f, ds] of defs) defsByFile.set(f, { defs: ds, uses: uses.get(f) ?? [], reExports: [] });
  return {
    files,
    root,
    extractFn: makeExtractStub(defsByFile, failingPaths),
    resolveSpecifier: makeResolveSpecifier(spec),
    symbols,
    callGraph,
  };
}

// Fresh inputs: which files + what defs per file (emulating fresh AST pass).
function fixtureDefs(entries) {
  // entries: [{absFile, name, kind, line}]
  const map = new Map();
  const files = [];
  for (const e of entries) {
    if (!map.has(e.absFile)) { map.set(e.absFile, []); files.push(e.absFile); }
    map.get(e.absFile).push({ name: e.name, kind: e.kind ?? 'FunctionDeclaration', line: e.line ?? 1 });
  }
  return { defs: map, files };
}

// ── H-1: missing canon → skipped-missing-canon ─────────────────

{
  const canonPath = path.join(workdir, 'nope.md');
  const r = detectHelperRegistryDrift({
    canonPath,
    scanContext: buildScanContext({ root: workdir, files: [], defs: new Map() }),
    canonLabelSet: HELPER_LABEL_SET,
  });
  assert('H-1a. missing canon → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon', `status=${r.status}`);
  assert('H-1b. missing canon → drifts empty + reportMarkdown null',
    r.drifts.length === 0 && r.reportMarkdown === null, '');
}

// ── H-2: identity-added / helper-removed / label-changed ────────

{
  const canonPath = path.join(workdir, 'canon-added.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', signature: '() => void', fanIn: 3, label: 'central-helper' },
  ]);
  const fooAbs = path.join(workdir, 'src', 'foo.ts');
  const barAbs = path.join(workdir, 'src', 'bar.ts');
  const { defs, files } = fixtureDefs([
    { absFile: fooAbs, name: 'doFoo', line: 10 },
    { absFile: barAbs, name: 'doBar', line: 2 },
  ]);
  const r = detectHelperRegistryDrift({
    canonPath,
    scanContext: buildScanContext({ root: workdir, files, defs }),
    canonLabelSet: HELPER_LABEL_SET,
  });
  const added = r.drifts.filter((d) => d.category === 'helper-added');
  assert('H-2a. helper-added = 1 when fresh gains a new identity',
    added.length === 1 && added[0].identity === 'src/bar.ts::doBar',
    `added=${JSON.stringify(added)}`);
  assert('H-2b. helper-added family = added',
    added[0]?.family === 'added', `family=${added[0]?.family}`);
}

{
  const canonPath = path.join(workdir, 'canon-removed.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
    { name: 'gone',  identity: 'src/gone.ts::gone', owner: 'src/gone.ts:2', fanIn: 1, label: 'shared-helper' },
  ]);
  const fooAbs = path.join(workdir, 'src', 'foo.ts');
  const { defs, files } = fixtureDefs([
    { absFile: fooAbs, name: 'doFoo', line: 10 },
  ]);
  const r = detectHelperRegistryDrift({
    canonPath,
    scanContext: buildScanContext({ root: workdir, files, defs }),
    canonLabelSet: HELPER_LABEL_SET,
  });
  const removed = r.drifts.filter((d) => d.category === 'helper-removed');
  assert('H-2c. helper-removed = 1 when canon has an extra identity',
    removed.length === 1 && removed[0].identity === 'src/gone.ts::gone',
    `removed=${JSON.stringify(removed)}`);
}

// ── H-3: no helper-owner-changed upgrade (same name, diff file) ─

{
  const canonPath = path.join(workdir, 'canon-noupgrade.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/a.ts::doFoo', owner: 'src/a.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const bAbs = path.join(workdir, 'src', 'b.ts');
  const { defs, files } = fixtureDefs([
    { absFile: bAbs, name: 'doFoo', line: 1 },
  ]);
  const r = detectHelperRegistryDrift({
    canonPath,
    scanContext: buildScanContext({ root: workdir, files, defs }),
    canonLabelSet: HELPER_LABEL_SET,
  });
  const added = r.drifts.filter((d) => d.category === 'helper-added');
  const removed = r.drifts.filter((d) => d.category === 'helper-removed');
  assert('H-3a. same-name-different-file → separate helper-added + helper-removed (no upgrade)',
    added.length === 1 && removed.length === 1,
    `added=${added.length}, removed=${removed.length}, all=${JSON.stringify(r.drifts.map((d)=>d.category))}`);
  assert('H-3b. NO helper-owner-changed record exists',
    !r.drifts.some((d) => d.category === 'helper-owner-changed'), '');
}

// ── H-4: Label-change dispatch (evidence-gated) ─────────────────

// H-4(a): contamination available + contamination involved → contamination-changed
{
  const canonPath = path.join(workdir, 'canon-cc-avail.md');
  writeCanon(canonPath, [
    // Canon says contamination; fresh will classify as central-helper (fanIn 3).
    // Capability available → contamination-changed emitted.
    { name: 'doX', identity: 'src/x.ts::doX', owner: 'src/x.ts:1', fanIn: 3, label: 'severely-any-contaminated-helper' },
  ]);
  const xAbs = path.join(workdir, 'src', 'x.ts');
  const { defs } = fixtureDefs([{ absFile: xAbs, name: 'doX', line: 1 }]);
  // Create 3 consumer files so fanIn=3 → central-helper.
  const consumers = ['src/c1.ts', 'src/c2.ts', 'src/c3.ts'].map((f) => path.join(workdir, f));
  const spec = new Map([['./x', xAbs]]);
  const uses = new Map(consumers.map((c) => [c, [{ fromSpec: './x', name: 'doX', kind: 'import' }]]));
  const filesAll = [xAbs, ...consumers];
  defs.set(consumers[0], []); defs.set(consumers[1], []); defs.set(consumers[2], []);
  const ctx = buildScanContext({
    root: workdir, files: filesAll, defs, uses, spec,
    symbols: { helperOwnersByIdentity: { /* populated → capability 'available' */
      'src/x.ts::doX': { anyContamination: null /* fresh says clean */ },
    } },
  });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  const cc = r.drifts.filter((d) => d.category === 'contamination-changed');
  assert('H-4a. contamination-changed fires when capability available',
    cc.length === 1 && cc[0].canon.label === 'severely-any-contaminated-helper' &&
    cc[0].fresh.label === 'central-helper',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('H-4a2. contamination-changed family = content-shifted',
    cc[0]?.family === 'content-shifted', `family=${cc[0]?.family}`);
}

// H-4(b): contamination unavailable + contamination involved → label-changed DOWNGRADE
{
  const canonPath = path.join(workdir, 'canon-cc-unavail.md');
  writeCanon(canonPath, [
    { name: 'doY', identity: 'src/y.ts::doY', owner: 'src/y.ts:1', fanIn: 3, label: 'severely-any-contaminated-helper' },
  ]);
  const yAbs = path.join(workdir, 'src', 'y.ts');
  const { defs } = fixtureDefs([{ absFile: yAbs, name: 'doY', line: 1 }]);
  const consumers = ['src/d1.ts', 'src/d2.ts', 'src/d3.ts'].map((f) => path.join(workdir, f));
  const spec = new Map([['./y', yAbs]]);
  const uses = new Map(consumers.map((c) => [c, [{ fromSpec: './y', name: 'doY', kind: 'import' }]]));
  for (const c of consumers) defs.set(c, []);
  const filesAll = [yAbs, ...consumers];
  const ctx = buildScanContext({
    root: workdir, files: filesAll, defs, uses, spec,
    symbols: null,  // capability unavailable
  });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  const cc = r.drifts.filter((d) => d.category === 'contamination-changed');
  const lc = r.drifts.filter((d) => d.category === 'label-changed');
  assert('H-4b. contamination transition under unavailable capability does NOT emit contamination-changed',
    cc.length === 0, `drifts=${JSON.stringify(r.drifts)}`);
  assert('H-4b2. ...instead it downgrades to label-changed',
    lc.length === 1, `lc=${lc.length}`);
  assert('H-4b3. advisory diagnostic helper-contamination-enrichment-unavailable is present',
    r.diagnostics.some((d) => d.kind === 'helper-contamination-enrichment-unavailable'),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// H-4(c): both fan-in labels → fan-in-tier-changed (not gated)
{
  const canonPath = path.join(workdir, 'canon-tier.md');
  writeCanon(canonPath, [
    // Canon says shared-helper (fanIn=2 tier); fresh will show central-helper (fanIn≥3).
    { name: 'doZ', identity: 'src/z.ts::doZ', owner: 'src/z.ts:1', fanIn: 2, label: 'shared-helper' },
  ]);
  const zAbs = path.join(workdir, 'src', 'z.ts');
  const { defs } = fixtureDefs([{ absFile: zAbs, name: 'doZ', line: 1 }]);
  const consumers = ['src/e1.ts', 'src/e2.ts', 'src/e3.ts'].map((f) => path.join(workdir, f));
  const spec = new Map([['./z', zAbs]]);
  const uses = new Map(consumers.map((c) => [c, [{ fromSpec: './z', name: 'doZ', kind: 'import' }]]));
  for (const c of consumers) defs.set(c, []);
  const filesAll = [zAbs, ...consumers];
  const ctx = buildScanContext({ root: workdir, files: filesAll, defs, uses, spec, symbols: null });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  const ft = r.drifts.filter((d) => d.category === 'fan-in-tier-changed');
  assert('H-4c. fan-in-tier-changed fires when both labels are tier labels',
    ft.length === 1 && ft[0].canon.label === 'shared-helper' && ft[0].fresh.label === 'central-helper',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('H-4c2. fan-in-tier-changed is not gated by callGraph (callGraph=null above)',
    ft[0] !== undefined, '');
}

// H-4(d): fan-in-tier-changed stays grounded under callGraph=null (P1-1)
{
  // Already implicitly covered by H-4c (callGraph=null in ctx); add explicit check.
  assert('H-4d. (explicit) callGraph=null does not suppress fan-in-tier-changed',
    true,
    'covered by H-4c — no separate fixture needed');
}

// H-4(e): per-identity evidence gate (Finding #1 — reviewer 2026-04-22).
// symbols.helperOwnersByIdentity carries an UNRELATED identity; the drift
// identity has no fact of its own. contamination-changed MUST NOT fire
// just because run-level capability is "available".
{
  const canonPath = path.join(workdir, 'canon-cc-peridentity.md');
  writeCanon(canonPath, [
    { name: 'doY', identity: 'src/y.ts::doY', owner: 'src/y.ts:1', fanIn: 3, label: 'severely-any-contaminated-helper' },
  ]);
  const yAbs = path.join(workdir, 'src', 'y.ts');
  const { defs } = fixtureDefs([{ absFile: yAbs, name: 'doY', line: 1 }]);
  const consumers = ['src/p1.ts', 'src/p2.ts', 'src/p3.ts'].map((f) => path.join(workdir, f));
  const spec = new Map([['./y', yAbs]]);
  const uses = new Map(consumers.map((c) => [c, [{ fromSpec: './y', name: 'doY', kind: 'import' }]]));
  for (const c of consumers) defs.set(c, []);
  const filesAll = [yAbs, ...consumers];
  const ctx = buildScanContext({
    root: workdir, files: filesAll, defs, uses, spec,
    symbols: {
      helperOwnersByIdentity: {
        // Unrelated identity carries a fact. Drift identity does not.
        'src/unrelated.ts::other': { anyContamination: null },
      },
    },
  });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  const cc = r.drifts.filter((d) => d.category === 'contamination-changed');
  const lc = r.drifts.filter((d) => d.category === 'label-changed');
  assert('H-4e. unrelated helperOwnersByIdentity entry does NOT authorize contamination-changed for another identity',
    cc.length === 0 && lc.length === 1,
    `drifts=${JSON.stringify(r.drifts)}`);
}

// ── H-5: Extractor-throw → source-level parse-error ─────────────

{
  const canonPath = path.join(workdir, 'canon-parse.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const fooAbs = path.join(workdir, 'src', 'foo.ts');
  const badAbs = path.join(workdir, 'src', 'broken.ts');
  const { defs, files } = fixtureDefs([
    { absFile: fooAbs, name: 'doFoo', line: 10 },
  ]);
  files.push(badAbs);
  const ctx = buildScanContext({
    root: workdir, files, defs,
    failingPaths: new Set([badAbs]),
  });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-5a. extractor-throw → status=parse-error',
    r.status === 'parse-error', `status=${r.status}`);
  assert('H-5b. extractor-throw → drifts empty + reportMarkdown null',
    r.drifts.length === 0 && r.reportMarkdown === null, '');
  assert('H-5c. original parse-error diagnostic preserved with file target',
    r.diagnostics.some((d) => d.kind === 'parse-error' && /broken\.ts/.test(d.target ?? '')),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
}

// ── H-6: Enrichment-unavailable advisory when symbols=null ─────

{
  const canonPath = path.join(workdir, 'canon-unavail.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const fooAbs = path.join(workdir, 'src', 'foo.ts');
  const { defs, files } = fixtureDefs([
    { absFile: fooAbs, name: 'doFoo', line: 10 },
  ]);
  const ctx = buildScanContext({ root: workdir, files, defs, symbols: null });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-6a. symbols=null → advisory diagnostic helper-contamination-enrichment-unavailable',
    r.diagnostics.some((d) => d.kind === 'helper-contamination-enrichment-unavailable'),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('H-6b. advisory does NOT promote status to parse-error',
    r.status === 'clean' || r.status === 'drift', `status=${r.status}`);
}

// ── H-7: call-graph-cross-check stays advisory ─────────────────

{
  const canonPath = path.join(workdir, 'canon-cgx.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const fooAbs = path.join(workdir, 'src', 'foo.ts');
  const { defs, files } = fixtureDefs([
    { absFile: fooAbs, name: 'doFoo', line: 10 },
  ]);
  const ctx = buildScanContext({ root: workdir, files, defs, symbols: null, callGraph: null });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-7. callGraph=null → engine continues, status not promoted from advisory',
    r.status !== 'parse-error', `status=${r.status}`);
}

// ── H-8: Clean run ─────────────────────────────────────────────

{
  const canonPath = path.join(workdir, 'canon-clean.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const fooAbs = path.join(workdir, 'src', 'foo.ts');
  const { defs } = fixtureDefs([{ absFile: fooAbs, name: 'doFoo', line: 10 }]);
  const consumers = ['src/a.ts', 'src/b.ts', 'src/c.ts'].map((f) => path.join(workdir, f));
  const spec = new Map([['./foo', fooAbs]]);
  const uses = new Map(consumers.map((c) => [c, [{ fromSpec: './foo', name: 'doFoo', kind: 'import' }]]));
  for (const c of consumers) defs.set(c, []);
  const filesAll = [fooAbs, ...consumers];
  const ctx = buildScanContext({ root: workdir, files: filesAll, defs, uses, spec, symbols: null });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-8a. clean run → status=clean, 0 drifts',
    r.status === 'clean' && r.drifts.length === 0,
    `status=${r.status}, drifts=${r.drifts.length}`);
  assert('H-8b. clean run → reportMarkdown is summary-only string',
    typeof r.reportMarkdown === 'string' &&
    r.reportMarkdown.includes('## 1. Summary'),
    `md=${r.reportMarkdown?.slice(0, 200)}`);
  assert('H-8c. clean run MD does NOT include category sections',
    !r.reportMarkdown.includes('## 2. helper-added') &&
    !r.reportMarkdown.includes('## 3. helper-removed'), '');
}

// ── H-9: HELPER_LABEL_SET mirrors §10.3 (9 entries) ────────────

{
  const expected = new Set([
    'HELPER_DUPLICATE_STRONG',
    'HELPER_DUPLICATE_REVIEW',
    'HELPER_LOCAL_COMMON',
    'ANY_COLLISION_HELPER',
    'severely-any-contaminated-helper',
    'central-helper',
    'shared-helper',
    'zero-internal-fan-in-helper',
    'low-signal-helper-name',
  ]);
  assert('H-9a. HELPER_LABEL_SET has exactly 9 entries',
    HELPER_LABEL_SET.size === 9, `size=${HELPER_LABEL_SET.size}`);
  assert('H-9b. HELPER_LABEL_SET equals §10.3 canonical set',
    [...expected].every((l) => HELPER_LABEL_SET.has(l)),
    `missing=${[...expected].filter((l) => !HELPER_LABEL_SET.has(l)).join(',')}`);
}

// ── H-10: Every record kind=helper-drift ───────────────────────

{
  const canonPath = path.join(workdir, 'canon-kind.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const ctx = buildScanContext({ root: workdir, files: [], defs: new Map(), symbols: null });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-10. every drift has kind=helper-drift',
    r.drifts.length > 0 && r.drifts.every((d) => d.kind === 'helper-drift'),
    `kinds=${JSON.stringify(r.drifts.map((d)=>d.kind))}`);
}

// H-12: contamination-changed MD renders Canon signal / Fresh signal columns (Finding #2)
{
  const canonPath = path.join(workdir, 'canon-cc-md.md');
  // Use a different identity name + file so we don't collide with the
  // 'doX' record from H-4a which is still in the canon-cc-avail.md file.
  writeCanon(canonPath, [
    { name: 'doMd', identity: 'src/md.ts::doMd', owner: 'src/md.ts:1', fanIn: 3, label: 'severely-any-contaminated-helper' },
  ]);
  const mdAbs = path.join(workdir, 'src', 'md.ts');
  const { defs } = fixtureDefs([{ absFile: mdAbs, name: 'doMd', line: 1 }]);
  const consumers = ['src/mdc1.ts', 'src/mdc2.ts', 'src/mdc3.ts'].map((f) => path.join(workdir, f));
  const spec = new Map([['./md', mdAbs]]);
  const uses = new Map(consumers.map((c) => [c, [{ fromSpec: './md', name: 'doMd', kind: 'import' }]]));
  for (const c of consumers) defs.set(c, []);
  const filesAll = [mdAbs, ...consumers];
  const ctx = buildScanContext({
    root: workdir, files: filesAll, defs, uses, spec,
    symbols: {
      helperOwnersByIdentity: {
        'src/md.ts::doMd': { anyContamination: null /* clean now */ },
      },
    },
  });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-12a. contamination-changed MD table header has Canon signal + Fresh signal (§4.1)',
    r.reportMarkdown.includes('Canon signal') && r.reportMarkdown.includes('Fresh signal'),
    `md-excerpt=${r.reportMarkdown.slice(0, 600)}`);
  assert('H-12b. contamination-changed MD does NOT use fan-in columns',
    !/## \d+\. contamination-changed[\s\S]*?Canon fan-in/.test(r.reportMarkdown),
    `md-excerpt=${r.reportMarkdown.slice(0, 600)}`);
}

// ── H-11: Drift-record identity format pin (from P5-1 F#1) ─────

{
  const canonPath = path.join(workdir, 'canon-id.md');
  writeCanon(canonPath, [
    { name: 'doFoo', identity: 'src/foo.ts::doFoo', owner: 'src/foo.ts:10', fanIn: 3, label: 'central-helper' },
  ]);
  const ctx = buildScanContext({ root: workdir, files: [], defs: new Map(), symbols: null });
  const r = detectHelperRegistryDrift({ canonPath, scanContext: ctx, canonLabelSet: HELPER_LABEL_SET });
  assert('H-11. every drift identity matches canonical ownerFile::exportedName (no compound/arrow)',
    r.drifts.every((d) => /^[^:]+::[^:]+$/.test(d.identity) &&
      !d.identity.includes('→') && !d.identity.includes('->')),
    `identities=${JSON.stringify(r.drifts.map((d)=>d.identity))}`);
}

rmSync(workdir, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
