// Tests for _lib/post-write-artifact.mjs — P2-1 step 3.
//
// Pinning rules from docs/history/phases/p2/p2-1.md v3 §5.3:
//   - generateDeltaInvocationId shape matches generateInvocationId.
//   - Dual-write: latest.json + <preWriteInvocationId>.<deltaInvocationId>.json.
//   - Byte-identical content.
//   - Atomic (no .tmp.* leftovers).
//   - Re-run with new deltaInvocationId → new specific file; prior preserved.
//   - delta JSON's deltaInvocationId must match filename component byte-for-byte.

import { mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import {
  writeDelta,
  generateDeltaInvocationId,
} from '../_lib/post-write-artifact.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function makeDelta(overrides = {}) {
  return {
    preWriteInvocationId: 'pre-INV-1',
    deltaInvocationId: 'DELTA-1',
    intentHash: 'abc',
    baseline: { status: 'available', source: null },
    capabilityParity: { status: 'ok' },
    scanRangeParity: { status: 'ok' },
    inventoryCompleteness: { afterComplete: true, beforeComplete: true, filesWithParseErrors: [] },
    entries: [],
    summary: { planned: 0, plannedNotObserved: 0, silentNew: 0, preExisting: 0, removed: 0, observedUnbaselined: 0 },
    failures: [],
    ...overrides,
  };
}

// ═══ T1. generateDeltaInvocationId shape ═══

{
  const id = generateDeltaInvocationId();
  assert('T1. generateDeltaInvocationId returns string',
    typeof id === 'string');
  // Shape: YYYY-MM-DDTHH-mm-ssZ-<6-char-hex>
  assert('T1b. matches generateInvocationId shape regex',
    /^\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}Z-[0-9a-f]{6}$/.test(id),
    `got: ${id}`);
  // Two calls → different ids (random suffix).
  const id2 = generateDeltaInvocationId();
  assert('T1c. two calls produce different ids',
    id !== id2);
}

// ═══ T2. Dual-write happy path ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'pw-art-'));
  try {
    const delta = makeDelta({ preWriteInvocationId: 'PRE-1', deltaInvocationId: 'DEL-1' });
    const { latestPath, specificPath } = writeDelta(out, delta);
    const files = readdirSync(out);

    assert('T2. returns latestPath + specificPath',
      typeof latestPath === 'string' && typeof specificPath === 'string');
    assert('T2b. post-write-delta.latest.json exists',
      files.includes('post-write-delta.latest.json'));
    assert('T2c. post-write-delta.<pre>.<delta>.json exists',
      files.includes('post-write-delta.PRE-1.DEL-1.json'));

    const latest = readFileSync(latestPath, 'utf8');
    const specific = readFileSync(specificPath, 'utf8');
    assert('T2d. latest + specific byte-identical',
      latest === specific);
    assert('T2e. specific contains deltaInvocationId',
      JSON.parse(specific).deltaInvocationId === 'DEL-1');
    assert('T2f. specific contains preWriteInvocationId',
      JSON.parse(specific).preWriteInvocationId === 'PRE-1');
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. Filename pattern regex ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'pw-art-pat-'));
  try {
    const delta = makeDelta({ preWriteInvocationId: 'PRE-X', deltaInvocationId: 'DEL-Y' });
    const { specificPath } = writeDelta(out, delta);
    const basename = path.basename(specificPath);
    assert('T3. filename matches post-write-delta.<pre>.<delta>.json pattern',
      /^post-write-delta\.[^.]+\.[^.]+\.json$/.test(basename),
      `got: ${basename}`);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. Reject missing ids ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'pw-art-err-'));
  try {
    let threw = false;
    try {
      writeDelta(out, makeDelta({ preWriteInvocationId: '', deltaInvocationId: 'X' }));
    } catch { threw = true; }
    assert('T4a. empty preWriteInvocationId rejected', threw);

    threw = false;
    try {
      writeDelta(out, makeDelta({ preWriteInvocationId: 'X', deltaInvocationId: '' }));
    } catch { threw = true; }
    assert('T4b. empty deltaInvocationId rejected', threw);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Atomic — no .tmp.* leftovers ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'pw-art-atom-'));
  try {
    writeDelta(out, makeDelta({ preWriteInvocationId: 'P', deltaInvocationId: 'D' }));
    const files = readdirSync(out);
    const tmpLeftovers = files.filter((f) => f.includes('.tmp.'));
    assert('T5. no .tmp.* leftovers after normal write',
      tmpLeftovers.length === 0,
      `found: ${tmpLeftovers.join(', ')}`);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. Re-run with new deltaInvocationId preserves prior specific file ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'pw-art-rerun-'));
  try {
    const first = makeDelta({ preWriteInvocationId: 'PRE', deltaInvocationId: 'DELTA-A' });
    writeDelta(out, first);
    const second = makeDelta({ preWriteInvocationId: 'PRE', deltaInvocationId: 'DELTA-B', entries: [] });
    writeDelta(out, second);

    const files = new Set(readdirSync(out));
    assert('T6a. prior specific file preserved',
      files.has('post-write-delta.PRE.DELTA-A.json'));
    assert('T6b. new specific file written',
      files.has('post-write-delta.PRE.DELTA-B.json'));
    assert('T6c. latest.json present (singular)',
      files.has('post-write-delta.latest.json'));

    // latest.json points to the newer run — byte-match against DELTA-B specific.
    const latest = readFileSync(path.join(out, 'post-write-delta.latest.json'), 'utf8');
    const specificB = readFileSync(path.join(out, 'post-write-delta.PRE.DELTA-B.json'), 'utf8');
    assert('T6d. latest.json overwritten to match newest specific',
      latest === specificB);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T7. deltaInvocationId in JSON matches filename ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'pw-art-match-'));
  try {
    const delta = makeDelta({ preWriteInvocationId: 'P1', deltaInvocationId: 'D1' });
    const { specificPath } = writeDelta(out, delta);
    const inJson = JSON.parse(readFileSync(specificPath, 'utf8')).deltaInvocationId;
    const filenameComponent = path.basename(specificPath).replace(/^post-write-delta\.P1\.(.+)\.json$/, '$1');
    assert('T7. delta.deltaInvocationId === filename component byte-match',
      inJson === filenameComponent && inJson === 'D1');
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
