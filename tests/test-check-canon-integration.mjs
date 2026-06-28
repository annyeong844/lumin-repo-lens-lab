// tests/test-check-canon-integration.mjs
//
// P5-1 Step 0 — RED test for end-to-end drift detection.
//
// Uses 5 fixtures under tests/fixtures/canon-drift-types-*/ with
// precomputed audit-output/symbols.json. Exercises the full pipeline
// (CLI → loader → engine → writer) and pins two hard safety properties:
//
//   1. canonical/type-ownership.md is byte-identical pre/post run
//      (check-canon NEVER writes to canonical).
//   2. A stale canonical-draft/type-ownership.md does NOT affect the
//      drift result (current observation is always symbols.json-based,
//      never prior draft md files — session v2 P1-7).

import { spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, existsSync, mkdirSync, mkdtempSync, cpSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const CLI = path.join(DIR, 'check-canon.mjs');
const FIXTURE_ROOT = path.join(DIR, 'tests', 'fixtures');

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

function sha256(p) {
  return createHash('sha256').update(readFileSync(p)).digest('hex');
}

function runCli(args, { cwd } = {}) {
  const res = spawnSync(process.execPath, [CLI, ...args], {
    cwd: cwd ?? DIR,
    encoding: 'utf8',
  });
  return { exit: res.status ?? -1, stdout: res.stdout ?? '', stderr: res.stderr ?? '' };
}

function runFixture(srcName) {
  const srcDir = path.join(FIXTURE_ROOT, `canon-drift-types-${srcName}`);
  if (!existsSync(srcDir)) return null;
  const workDir = mkdtempSync(path.join(tmpdir(), `p5-1-intg-${srcName}-`));
  cpSync(srcDir, workDir, { recursive: true });
  const canonPath = path.join(workDir, 'canonical', 'type-ownership.md');
  const canonShaBefore = existsSync(canonPath) ? sha256(canonPath) : null;
  const outDir = path.join(workDir, 'audit-output');
  const r = runCli(['--source', 'type-ownership', '--root', workDir, '--output', outDir]);
  const canonShaAfter = existsSync(canonPath) ? sha256(canonPath) : null;
  const jsonPath = path.join(outDir, 'canon-drift.json');
  const mdPath = path.join(outDir, 'canon-drift.type-ownership.md');
  const json = existsSync(jsonPath) ? JSON.parse(readFileSync(jsonPath, 'utf8')) : null;
  return {
    ...r, workDir, canonShaBefore, canonShaAfter, jsonPath, mdPath, json,
  };
}

const cleanup = [];

// ── I-1: clean fixture ─────────────────────────────────────────

{
  const r = runFixture('clean');
  if (r) cleanup.push(r.workDir);
  assert('I-1a. clean fixture → exit 0',
    r && r.exit === 0, `exit=${r?.exit}, stderr=${r?.stderr.slice(0, 200)}`);
  assert('I-1b. clean fixture: canonical/type-ownership.md byte-identical pre/post (no canon writes)',
    r && r.canonShaBefore && r.canonShaBefore === r.canonShaAfter,
    `before=${r?.canonShaBefore}, after=${r?.canonShaAfter}`);
  assert('I-1c. clean fixture: JSON status = clean',
    r?.json?.perSource?.['type-ownership']?.status === 'clean',
    `status=${r?.json?.perSource?.['type-ownership']?.status}`);
  assert('I-1d. clean fixture: driftCount = 0',
    r?.json?.perSource?.['type-ownership']?.driftCount === 0,
    `driftCount=${r?.json?.perSource?.['type-ownership']?.driftCount}`);
}

// ── I-2: identity-added ────────────────────────────────────────

{
  const r = runFixture('added');
  if (r) cleanup.push(r.workDir);
  assert('I-2a. added fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('I-2b. added fixture: drifts[] includes category=identity-added',
    r?.json?.drifts?.some((d) => d.category === 'identity-added'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('I-2c. added fixture: canon byte-identical',
    r && r.canonShaBefore === r.canonShaAfter, '');
}

// ── I-3: identity-removed ──────────────────────────────────────

{
  const r = runFixture('removed');
  if (r) cleanup.push(r.workDir);
  assert('I-3a. removed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('I-3b. removed fixture: drifts[] includes category=identity-removed',
    r?.json?.drifts?.some((d) => d.category === 'identity-removed'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('I-3c. removed fixture: canon byte-identical',
    r && r.canonShaBefore === r.canonShaAfter, '');
}

// ── I-4: label-changed ─────────────────────────────────────────

{
  const r = runFixture('label-changed');
  if (r) cleanup.push(r.workDir);
  assert('I-4a. label-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('I-4b. label-changed fixture: drifts[] includes category=label-changed',
    r?.json?.drifts?.some((d) => d.category === 'label-changed'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('I-4c. label-changed record has distinct canon.label / fresh.label',
    r?.json?.drifts?.some((d) =>
      d.category === 'label-changed' && d.canon?.label !== d.fresh?.label),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
}

// ── I-5: owner-changed ─────────────────────────────────────────

{
  const r = runFixture('owner-changed');
  if (r) cleanup.push(r.workDir);
  assert('I-5a. owner-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('I-5b. owner-changed fixture: drifts[] includes category=owner-changed',
    r?.json?.drifts?.some((d) => d.category === 'owner-changed'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('I-5c. owner-changed record carries canon.label AND fresh.label (P0-5)',
    r?.json?.drifts?.some((d) =>
      d.category === 'owner-changed' &&
      typeof d.canon?.label === 'string' && typeof d.fresh?.label === 'string'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('I-5d. owner-changed MD contains "Canon label" + "Fresh label" columns',
    r?.mdPath && existsSync(r.mdPath) &&
    (() => {
      const md = readFileSync(r.mdPath, 'utf8');
      return md.includes('Canon label') && md.includes('Fresh label');
    })(),
    `md exists=${r?.mdPath && existsSync(r.mdPath)}`);
  // Reviewer Finding #1 — end-to-end pin on owner-changed identity format.
  const ownerRec = r?.json?.drifts?.find((d) => d.category === 'owner-changed');
  assert('I-5e. owner-changed JSON identity is canonical ownerFile::exportedName (no arrow)',
    ownerRec && /^[^:]+::[^:]+$/.test(ownerRec.identity) && !ownerRec.identity.includes('→'),
    `identity=${ownerRec?.identity}`);
  assert('I-5f. owner-changed canon.identity + fresh.identity differ + follow canonical format',
    ownerRec && ownerRec.canon?.identity !== ownerRec.fresh?.identity &&
    /^[^:]+::[^:]+$/.test(ownerRec.canon?.identity ?? '') &&
    /^[^:]+::[^:]+$/.test(ownerRec.fresh?.identity ?? ''),
    `canon=${ownerRec?.canon?.identity}, fresh=${ownerRec?.fresh?.identity}`);
}

// ── I-6: stale canonical-draft/ is ignored ─────────────────────

{
  // Use the clean fixture but inject a BOGUS stale canonical-draft.
  // Result must still be clean (drift must not come from draft md files).
  const srcDir = path.join(FIXTURE_ROOT, 'canon-drift-types-clean');
  if (existsSync(srcDir)) {
    const work = mkdtempSync(path.join(tmpdir(), 'p5-1-intg-stale-'));
    cpSync(srcDir, work, { recursive: true });
    mkdirSync(path.join(work, 'canonical-draft'), { recursive: true });
    writeFileSync(path.join(work, 'canonical-draft', 'type-ownership.md'),
      // bogus "drift" content that should be ignored
      '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
      '|--|--|--|--:|--|--|\n' +
      '| `BOGUS` | `src/bogus.ts::BOGUS` | `src/bogus.ts:1` | 99 | severely-any-contaminated | |\n',
      'utf8');
    const r = runCli(['--source', 'type-ownership',
      '--root', work, '--output', path.join(work, 'audit-output')]);
    const json = JSON.parse(readFileSync(path.join(work, 'audit-output', 'canon-drift.json'), 'utf8'));
    assert('I-6a. stale canonical-draft present → result still clean (no draft reads)',
      r.exit === 0 && json.perSource?.['type-ownership']?.status === 'clean',
      `exit=${r.exit}, status=${json.perSource?.['type-ownership']?.status}`);
    assert('I-6b. drifts[] empty (did not ingest bogus BOGUS from draft)',
      json.drifts.length === 0, `drifts=${JSON.stringify(json.drifts)}`);
    cleanup.push(work);
  } else {
    assert('I-6. stale canonical-draft ignored (skipped — clean fixture missing)', false,
      'fixture canon-drift-types-clean not present');
  }
}

// ── P5-2: helper-registry fixtures end-to-end ─────────────────

function runHelperFixture(srcName) {
  const srcDir = path.join(FIXTURE_ROOT, `canon-drift-helpers-${srcName}`);
  if (!existsSync(srcDir)) return null;
  const workDir = mkdtempSync(path.join(tmpdir(), `p5-2-intg-${srcName}-`));
  cpSync(srcDir, workDir, { recursive: true });
  const canonPath = path.join(workDir, 'canonical', 'helper-registry.md');
  const canonShaBefore = existsSync(canonPath) ? sha256(canonPath) : null;
  const outDir = path.join(workDir, 'audit-output');
  const r = runCli(['--source', 'helper-registry', '--root', workDir, '--output', outDir]);
  const canonShaAfter = existsSync(canonPath) ? sha256(canonPath) : null;
  const jsonPath = path.join(outDir, 'canon-drift.json');
  const mdPath = path.join(outDir, 'canon-drift.helper-registry.md');
  const json = existsSync(jsonPath) ? JSON.parse(readFileSync(jsonPath, 'utf8')) : null;
  return { ...r, workDir, canonShaBefore, canonShaAfter, jsonPath, mdPath, json };
}

// IH-1 clean
{
  const r = runHelperFixture('clean');
  if (r) cleanup.push(r.workDir);
  assert('IH-1a. helper clean fixture → exit 0',
    r && r.exit === 0, `exit=${r?.exit}, stderr=${r?.stderr.slice(0, 200)}`);
  assert('IH-1b. helper canon byte-identical pre/post',
    r && r.canonShaBefore === r.canonShaAfter, '');
  assert('IH-1c. JSON perSource["helper-registry"].status = clean',
    r?.json?.perSource?.['helper-registry']?.status === 'clean',
    `status=${r?.json?.perSource?.['helper-registry']?.status}`);
}

// IH-2 helper-added
{
  const r = runHelperFixture('added');
  if (r) cleanup.push(r.workDir);
  assert('IH-2a. helper-added fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IH-2b. drifts include category=helper-added',
    r?.json?.drifts?.some((d) => d.category === 'helper-added'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('IH-2c. helper canon byte-identical',
    r && r.canonShaBefore === r.canonShaAfter, '');
}

// IH-3 helper-removed
{
  const r = runHelperFixture('removed');
  if (r) cleanup.push(r.workDir);
  assert('IH-3a. helper-removed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IH-3b. drifts include category=helper-removed',
    r?.json?.drifts?.some((d) => d.category === 'helper-removed'), '');
}

// IH-4 label-changed
{
  const r = runHelperFixture('label-changed');
  if (r) cleanup.push(r.workDir);
  assert('IH-4a. helper label-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IH-4b. drifts include category=label-changed',
    r?.json?.drifts?.some((d) => d.category === 'label-changed'), '');
}

// IH-5 contamination-changed (requires symbols.helperOwnersByIdentity)
{
  const r = runHelperFixture('contamination-changed');
  if (r) cleanup.push(r.workDir);
  assert('IH-5a. helper contamination-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IH-5b. drifts include category=contamination-changed (evidence-gated)',
    r?.json?.drifts?.some((d) => d.category === 'contamination-changed'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  // Finding #2: MD must render Canon signal + Fresh signal columns for contamination section.
  assert('IH-5c. contamination-changed MD includes Canon signal + Fresh signal columns',
    r?.mdPath && existsSync(r.mdPath) &&
    (() => {
      const md = readFileSync(r.mdPath, 'utf8');
      return md.includes('Canon signal') && md.includes('Fresh signal');
    })(),
    `md exists=${r?.mdPath && existsSync(r.mdPath)}`);
  assert('IH-5d. contamination-changed JSON records carry canon.anyUnknownSignal evidence',
    r?.json?.drifts?.some((d) =>
      d.category === 'contamination-changed' &&
      typeof d.canon?.anyUnknownSignal === 'string'),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
}

// IH-6 fan-in-tier-changed
{
  const r = runHelperFixture('fan-in-tier-changed');
  if (r) cleanup.push(r.workDir);
  assert('IH-6a. fan-in-tier-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IH-6b. drifts include category=fan-in-tier-changed',
    r?.json?.drifts?.some((d) => d.category === 'fan-in-tier-changed'), '');
}

// IH-7 identity format pin across all helper fixtures
{
  for (const fixName of ['added', 'removed', 'label-changed', 'contamination-changed', 'fan-in-tier-changed']) {
    const r = runHelperFixture(fixName);
    if (!r) continue;
    cleanup.push(r.workDir);
    const drifts = r.json?.drifts ?? [];
    const allCanonical = drifts.every((d) =>
      /^[^:]+::[^:]+$/.test(d.identity) && !d.identity.includes('→'));
    assert(`IH-7.${fixName}. every helper-drift identity is ownerFile::exportedName (no arrow)`,
      allCanonical, `ids=${JSON.stringify(drifts.map((d)=>d.identity))}`);
  }
}

// ── P5-3: topology fixtures end-to-end ────────────────────────

function runTopologyFixture(srcName) {
  const srcDir = path.join(FIXTURE_ROOT, `canon-drift-topology-${srcName}`);
  if (!existsSync(srcDir)) return null;
  const workDir = mkdtempSync(path.join(tmpdir(), `p5-3-intg-${srcName}-`));
  cpSync(srcDir, workDir, { recursive: true });
  const canonPath = path.join(workDir, 'canonical', 'topology.md');
  const canonShaBefore = existsSync(canonPath) ? sha256(canonPath) : null;
  const outDir = path.join(workDir, 'audit-output');
  const r = runCli(['--source', 'topology', '--root', workDir, '--output', outDir]);
  const canonShaAfter = existsSync(canonPath) ? sha256(canonPath) : null;
  const jsonPath = path.join(outDir, 'canon-drift.json');
  const mdPath = path.join(outDir, 'canon-drift.topology.md');
  const json = existsSync(jsonPath) ? JSON.parse(readFileSync(jsonPath, 'utf8')) : null;
  return { ...r, workDir, canonShaBefore, canonShaAfter, jsonPath, mdPath, json };
}

// IY-1 clean
{
  const r = runTopologyFixture('clean');
  if (r) cleanup.push(r.workDir);
  assert('IY-1a. topology clean fixture → exit 0',
    r && r.exit === 0, `exit=${r?.exit}`);
  assert('IY-1b. canon bytes identical pre/post',
    r && r.canonShaBefore === r.canonShaAfter, '');
  assert('IY-1c. JSON perSource["topology"].status = clean',
    r?.json?.perSource?.['topology']?.status === 'clean', '');
}

// IY-2 submodule-added
{
  const r = runTopologyFixture('submodule-added');
  if (r) cleanup.push(r.workDir);
  assert('IY-2a. submodule-added fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IY-2b. drifts include category=submodule-added',
    r?.json?.drifts?.some((d) => d.category === 'submodule-added'), '');
  assert('IY-2c. submodule-added identity has no ::, no →',
    r?.json?.drifts?.filter((d) => d.category === 'submodule-added')
      ?.every((d) => !d.identity.includes('::') && !d.identity.includes('→')), '');
}

// IY-3 submodule-removed
{
  const r = runTopologyFixture('submodule-removed');
  if (r) cleanup.push(r.workDir);
  assert('IY-3a. submodule-removed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IY-3b. drifts include category=submodule-removed',
    r?.json?.drifts?.some((d) => d.category === 'submodule-removed'), '');
}

// IY-4 scc-status-changed
{
  const r = runTopologyFixture('scc-status-changed');
  if (r) cleanup.push(r.workDir);
  assert('IY-4a. scc-status-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IY-4b. drifts include category=scc-status-changed',
    r?.json?.drifts?.some((d) => d.category === 'scc-status-changed'), '');
  assert('IY-4c. scc-status-changed carries canon.sccMember + fresh.sccMember (booleans)',
    r?.json?.drifts?.some((d) =>
      d.category === 'scc-status-changed' &&
      typeof d.canon?.sccMember === 'boolean' &&
      typeof d.fresh?.sccMember === 'boolean' &&
      d.canon.sccMember !== d.fresh.sccMember), '');
}

// IY-5 oversize-changed
{
  const r = runTopologyFixture('oversize-changed');
  if (r) cleanup.push(r.workDir);
  assert('IY-5a. oversize-changed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IY-5b. drifts include category=oversize-changed',
    r?.json?.drifts?.some((d) => d.category === 'oversize-changed'), '');
  assert('IY-5c. oversize-changed identity is a file path',
    r?.json?.drifts?.filter((d) => d.category === 'oversize-changed')
      ?.every((d) => /\.ts$/.test(d.identity) || d.identity.includes('/')), '');
}

// IY-6 cross-edge-added
{
  const r = runTopologyFixture('cross-edge-added');
  if (r) cleanup.push(r.workDir);
  assert('IY-6a. cross-edge-added fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IY-6b. drifts include category=cross-edge-added with "<from> → <to>" identity',
    r?.json?.drifts?.some((d) =>
      d.category === 'cross-edge-added' && / → /.test(d.identity)),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('IY-6c. cross-edge-added JSON carries fresh.count',
    r?.json?.drifts?.some((d) =>
      d.category === 'cross-edge-added' && typeof d.fresh?.count === 'number'), '');
  assert('IY-6d. MD contains "Display scope: top-30" text on cross-edge row',
    r?.mdPath && existsSync(r.mdPath) &&
    readFileSync(r.mdPath, 'utf8').includes('top-30'),
    `md exists=${r?.mdPath && existsSync(r.mdPath)}`);
}

// IY-7 cross-edge-removed (Finding #3 — separate fixture)
{
  const r = runTopologyFixture('cross-edge-removed');
  if (r) cleanup.push(r.workDir);
  assert('IY-7a. cross-edge-removed fixture → exit 1',
    r && r.exit === 1, `exit=${r?.exit}`);
  assert('IY-7b. drifts include category=cross-edge-removed with "<from> → <to>" identity',
    r?.json?.drifts?.some((d) =>
      d.category === 'cross-edge-removed' && / → /.test(d.identity)),
    `drifts=${JSON.stringify(r?.json?.drifts)}`);
  assert('IY-7c. cross-edge-removed JSON carries canon.count + canon.line',
    r?.json?.drifts?.some((d) =>
      d.category === 'cross-edge-removed' &&
      typeof d.canon?.count === 'number' &&
      typeof d.canon?.line === 'number'), '');
  assert('IY-7d. MD contains "Display scope: top-30" text on cross-edge-removed row',
    r?.mdPath && existsSync(r.mdPath) &&
    readFileSync(r.mdPath, 'utf8').includes('top-30'), '');
}

// IY-8 stale canonical-draft/topology.md ignored
{
  const srcDir = path.join(FIXTURE_ROOT, 'canon-drift-topology-clean');
  if (existsSync(srcDir)) {
    const work = mkdtempSync(path.join(tmpdir(), 'p5-3-intg-stale-'));
    cpSync(srcDir, work, { recursive: true });
    mkdirSync(path.join(work, 'canonical-draft'), { recursive: true });
    writeFileSync(path.join(work, 'canonical-draft', 'topology.md'),
      // bogus stale draft with fake submodule
      '## 1. Submodule inventory\n\n| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |\n' +
      '|-----------|------:|----:|---------:|----------:|-----|--------|------|\n' +
      '| `bogus` | 99 | 9999 | 0 | 0 | — | isolated-submodule ⚠ | |\n',
      'utf8');
    const r = runCli(['--source', 'topology', '--root', work, '--output', path.join(work, 'audit-output')]);
    const json = JSON.parse(readFileSync(path.join(work, 'audit-output', 'canon-drift.json'), 'utf8'));
    assert('IY-8a. stale canonical-draft/topology.md ignored → result still clean',
      r.exit === 0 && json.perSource?.['topology']?.status === 'clean',
      `exit=${r.exit}, status=${json.perSource?.['topology']?.status}`);
    assert('IY-8b. drifts empty (bogus draft not ingested)',
      json.drifts.length === 0, `drifts=${JSON.stringify(json.drifts)}`);
    cleanup.push(work);
  }
}

for (const d of cleanup) rmSync(d, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
