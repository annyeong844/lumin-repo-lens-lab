// Tests for P2-0 step 4: pre-write snapshot hook.
//
// Pins from docs/history/phases/p2/p2-0.md §5.5:
//   - Default (no --no-fresh-audit): any-inventory.pre.<invocationId>.json
//     written; preWrite.anyInventoryPath stamped into BOTH latest.json AND
//     <invocationId>.json; same pointer.
//   - --no-fresh-audit: no snapshot file; preWrite.anyInventoryPath ABSENT
//     (not null).
//   - Hook failure: CLI exits 0; failures[] has any-inventory-hook-failed;
//     preWrite.anyInventoryPath absent; no partial file left behind.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const PREWRITE = path.join(DIR, 'pre-write.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildFixture(fx) {
  write(fx, 'package.json', JSON.stringify({ name: 'fx', type: 'module' }));
  write(fx, 'src/a.ts', `export const foo = (x as any).y;\n`);
}

function writeIntent(out) {
  const intent = { names: ['foo'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] };
  const p = path.join(out, 'intent.json');
  writeFileSync(p, JSON.stringify(intent));
  return p;
}

function invocationFiles(out) {
  return readdirSync(out).filter((n) =>
    n.startsWith('pre-write-advisory.') && !n.endsWith('.latest.json')
  );
}

// ═══ T1. Default happy path: hook runs, both advisory files stamped ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-hook-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-hook-out-'));
  try {
    buildFixture(fx);
    const intentPath = writeIntent(out);
    execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    // Snapshot file exists.
    const snapshotFiles = readdirSync(out).filter((n) => n.startsWith('any-inventory.pre.'));
    assert('T1. any-inventory.pre.<invocationId>.json written',
      snapshotFiles.length === 1,
      `files: ${readdirSync(out).join(', ')}`);

    const snapshotPath = path.join(out, snapshotFiles[0]);
    const snapshot = JSON.parse(readFileSync(snapshotPath, 'utf8'));
    assert('T1b. snapshot carries typeEscapes array',
      Array.isArray(snapshot.typeEscapes));

    // Both advisory files have preWrite.anyInventoryPath.
    const latest = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const invs = invocationFiles(out);
    assert('T1c. exactly one invocation-specific advisory exists', invs.length === 1);
    const inv = JSON.parse(readFileSync(path.join(out, invs[0]), 'utf8'));

    assert('T1d. latest.json has preWrite.anyInventoryPath',
      latest.preWrite?.anyInventoryPath && latest.preWrite.anyInventoryPath.startsWith('any-inventory.pre.'));
    assert('T1e. invocation.json has preWrite.anyInventoryPath',
      inv.preWrite?.anyInventoryPath && inv.preWrite.anyInventoryPath.startsWith('any-inventory.pre.'));
    assert('T1f. both advisory files have IDENTICAL anyInventoryPath',
      latest.preWrite.anyInventoryPath === inv.preWrite.anyInventoryPath);

    // Pointer actually points at the snapshot file.
    const referenced = path.join(out, latest.preWrite.anyInventoryPath);
    assert('T1g. anyInventoryPath pointer resolves to an existing file',
      existsSync(referenced));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. --no-fresh-audit: no snapshot + anyInventoryPath ABSENT (not null) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-hook-nofresh-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-hook-nofresh-out-'));
  try {
    buildFixture(fx);
    const intentPath = writeIntent(out);
    execFileSync(NODE, [PREWRITE,
      '--root', fx, '--output', out, '--intent', intentPath, '--no-fresh-audit',
    ], { stdio: ['ignore', 'pipe', 'pipe'] });

    const snapshotFiles = readdirSync(out).filter((n) => n.startsWith('any-inventory.pre.'));
    assert('T2. --no-fresh-audit: NO snapshot file written',
      snapshotFiles.length === 0);

    const latest = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const invs = invocationFiles(out);
    const inv = JSON.parse(readFileSync(path.join(out, invs[0]), 'utf8'));

    // ABSENT (not null, not empty string).
    assert('T2b. latest.json preWrite does NOT contain anyInventoryPath',
      !latest.preWrite || !('anyInventoryPath' in latest.preWrite),
      `preWrite=${JSON.stringify(latest.preWrite)}`);
    assert('T2c. invocation.json preWrite does NOT contain anyInventoryPath',
      !inv.preWrite || !('anyInventoryPath' in inv.preWrite));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. Existing P1 advisory fields unchanged ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-hook-p1-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-hook-p1-out-'));
  try {
    buildFixture(fx);
    const intentPath = writeIntent(out);
    execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('T3. invocationId still present',
      typeof parsed.invocationId === 'string' && parsed.invocationId.length > 0);
    assert('T3b. intentHash still 64-char sha256 hex',
      /^[a-f0-9]{64}$/.test(parsed.intentHash));
    assert('T3c. lookups array preserved',
      Array.isArray(parsed.lookups));
    assert('T3d. drift array preserved',
      Array.isArray(parsed.drift));
    assert('T3e. capabilities preserved',
      parsed.capabilities !== undefined);
    assert('T3f. failures array preserved',
      Array.isArray(parsed.failures));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. Snapshot carries meta.supports.typeEscapes capability ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-hook-caps-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-hook-caps-out-'));
  try {
    buildFixture(fx);
    const intentPath = writeIntent(out);
    execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    const snapshotFiles = readdirSync(out).filter((n) => n.startsWith('any-inventory.pre.'));
    const snapshot = JSON.parse(readFileSync(path.join(out, snapshotFiles[0]), 'utf8'));
    assert('T4. snapshot meta.supports.typeEscapes === true',
      snapshot.meta?.supports?.typeEscapes === true);
    assert('T4b. snapshot meta.complete === true on clean fixture',
      snapshot.meta?.complete === true);
    assert('T4c. snapshot meta.supports.escapeKinds has 11 entries',
      Array.isArray(snapshot.meta?.supports?.escapeKinds) &&
      snapshot.meta.supports.escapeKinds.length === 11);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Hook writes directly to invocation-specific path ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-hook-no-clobber-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-hook-no-clobber-out-'));
  try {
    buildFixture(fx);
    const intentPath = writeIntent(out);
    const sentinelPath = path.join(out, 'any-inventory.json');
    const sentinel = JSON.stringify({ sentinel: true }) + '\n';
    writeFileSync(sentinelPath, sentinel);

    execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    const latest = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const snapshotPath = path.join(out, latest.preWrite.anyInventoryPath);

    assert('T5. pre-write hook leaves existing shared any-inventory.json untouched',
      existsSync(sentinelPath) && readFileSync(sentinelPath, 'utf8') === sentinel,
      `files: ${readdirSync(out).join(', ')}`);
    assert('T5b. pre-write hook still writes invocation-specific snapshot',
      existsSync(snapshotPath) &&
      latest.preWrite.anyInventoryPath.startsWith('any-inventory.pre.'),
      JSON.stringify(latest.preWrite));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
