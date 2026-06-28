// Tests for audit-repo.mjs --post-write integration — P2-2 step 1.
//
// Exit-code contract (docs/history/phases/p2/p2-2.md v2 §4.2):
//   0 — audit succeeded; post-write ran or was not requested.
//   2 — --post-write without --pre-write-advisory; OR --pre-write + --post-write together.
//
// Additional pinning:
//   - manifest.postWrite carries summary fields (silentNew, requiredAcknowledgementCount,
//     baselineStatus, scanRangeParity, afterComplete) when ran === true.
//   - Verbatim flag forwarding including --no-include-tests.
//   - --delta-out relocates deltaPath.
//   - Stdout ordering + stderr segregation.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const AUDIT_REPO = path.join(DIR, 'audit-repo.mjs');

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
  write(fx, 'package.json', JSON.stringify({ name: 'aupost-fx', type: 'module' }));
  write(fx, 'src/a.ts', `export const formatDate = (d) => d.toString();\n`);
}

function writeIntent(out, plannedTypeEscapes = [], files = []) {
  const intent = { names: [], shapes: [], files, dependencies: [], plannedTypeEscapes };
  const p = path.join(out, 'intent.json');
  writeFileSync(p, JSON.stringify(intent));
  return p;
}

function runPreWrite(fx, out, extraArgs = []) {
  const intentPath = writeIntent(out);
  return spawnSync(NODE, [AUDIT_REPO,
    '--root', fx, '--output', out, '--profile', 'quick', '--pre-write', '--intent', intentPath,
    ...extraArgs,
  ], { encoding: 'utf8' });
}

function runPostWrite(fx, out, advisoryPath, extraArgs = []) {
  return spawnSync(NODE, [AUDIT_REPO,
    '--root', fx, '--output', out, '--profile', 'quick',
    '--post-write', '--pre-write-advisory', advisoryPath, ...extraArgs,
  ], { encoding: 'utf8' });
}

function runPreWriteWithFiles(fx, out, files, extraArgs = []) {
  const intentPath = writeIntent(out, [], files);
  return spawnSync(NODE, [AUDIT_REPO,
    '--root', fx, '--output', out, '--profile', 'quick', '--pre-write', '--intent', intentPath,
    ...extraArgs,
  ], { encoding: 'utf8' });
}

function readManifest(out) {
  return JSON.parse(readFileSync(path.join(out, 'manifest.json'), 'utf8'));
}

// ═══ T1. Happy path + manifest summary fields (reviewer P0-1) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-happy-out-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');

    const res = runPostWrite(fx, out, advisory);
    assert('T1. exit 0', res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    assert('T1b. stdout contains "## post-write delta"',
      res.stdout.includes('## post-write delta'));

    const m = readManifest(out);
    assert('T1c. manifest.postWrite.requested === true',
      m.postWrite?.requested === true);
    assert('T1d. manifest.postWrite.ran === true',
      m.postWrite?.ran === true);
    assert('T1e. manifest.postWrite.deltaPath is absolute path to an existing file',
      typeof m.postWrite?.deltaPath === 'string' &&
      path.isAbsolute(m.postWrite.deltaPath) &&
      existsSync(m.postWrite.deltaPath));

    // Summary fields populated.
    assert('T1f. manifest.postWrite.silentNew is a number',
      typeof m.postWrite?.silentNew === 'number');
    assert('T1g. manifest.postWrite.requiredAcknowledgementCount is a number',
      typeof m.postWrite?.requiredAcknowledgementCount === 'number');
    assert('T1h. manifest.postWrite.baselineStatus is available|missing',
      ['available', 'missing'].includes(m.postWrite?.baselineStatus));
    assert('T1i. manifest.postWrite.scanRangeParity is ok|mismatch|baseline-missing',
      ['ok', 'mismatch', 'baseline-missing'].includes(m.postWrite?.scanRangeParity));
    assert('T1j. manifest.postWrite.afterComplete is boolean',
      typeof m.postWrite?.afterComplete === 'boolean');

    // Summary fields match delta JSON (P0-1 pinning).
    const delta = JSON.parse(readFileSync(m.postWrite.deltaPath, 'utf8'));
    assert('T1k. silentNew matches delta.summary.silentNew',
      m.postWrite.silentNew === delta.summary.silentNew);
    assert('T1l. requiredAcknowledgementCount matches silent-new entry count',
      m.postWrite.requiredAcknowledgementCount ===
      (delta.entries ?? []).filter((e) => e.label === 'silent-new').length);
    assert('T1m. baselineStatus matches delta.baseline.status',
      m.postWrite.baselineStatus === delta.baseline.status);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. --post-write without --pre-write-advisory → exit 2 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-noadv-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-noadv-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [AUDIT_REPO,
      '--root', fx, '--output', out, '--profile', 'quick', '--post-write',
    ], { encoding: 'utf8' });

    assert('T2. exit 2', res.status === 2,
      `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`);
    assert('T2b. stderr mentions --pre-write-advisory',
      /pre-write-advisory/.test(res.stderr));

    const m = readManifest(out);
    assert('T2c. manifest.postWrite.requested === true',
      m.postWrite?.requested === true);
    assert('T2d. manifest.postWrite.ran === false',
      m.postWrite?.ran === false);
    assert('T2e. reason mentions --pre-write-advisory missing',
      /pre-write-advisory/.test(m.postWrite?.reason ?? ''));
    assert('T2f. no summary fields when ran===false',
      m.postWrite?.silentNew === undefined &&
      m.postWrite?.deltaPath === undefined);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. --pre-write + --post-write mutually exclusive ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-excl-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-excl-out-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    const res = spawnSync(NODE, [AUDIT_REPO,
      '--root', fx, '--output', out, '--profile', 'quick',
      '--pre-write', '--intent', intent,
      '--post-write', '--pre-write-advisory', path.join(out, 'some.json'),
    ], { encoding: 'utf8' });

    assert('T3. both flags → exit 2',
      res.status === 2, `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`);
    assert('T3b. stderr mentions mutually exclusive',
      /mutually exclusive/.test(res.stderr));

    const m = readManifest(out);
    assert('T3c. manifest.preWrite recorded the conflict',
      m.preWrite?.ran === false &&
      /mutually exclusive/.test(m.preWrite?.reason ?? ''));
    assert('T3d. manifest.postWrite recorded the conflict',
      m.postWrite?.ran === false &&
      /mutually exclusive/.test(m.postWrite?.reason ?? ''));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. --post-write with non-existent advisory → ran=false, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-bogus-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-bogus-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [AUDIT_REPO,
      '--root', fx, '--output', out, '--profile', 'quick',
      '--post-write', '--pre-write-advisory', path.join(out, 'does-not-exist.json'),
    ], { encoding: 'utf8' });

    assert('T4. exit 0 (graceful)',
      res.status === 0, `status=${res.status}`);

    const m = readManifest(out);
    assert('T4b. manifest.postWrite.ran === false',
      m.postWrite?.ran === false);
    assert('T4c. reason starts with "post-write.mjs exited non-zero:"',
      (m.postWrite?.reason ?? '').startsWith('post-write.mjs exited non-zero:'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. --post-write NOT requested → manifest.postWrite undefined ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-unset-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-unset-out-'));
  try {
    buildFixture(fx);
    spawnSync(NODE, [AUDIT_REPO,
      '--root', fx, '--output', out, '--profile', 'quick',
    ], { encoding: 'utf8' });

    const m = readManifest(out);
    assert('T5. manifest.postWrite undefined (field omitted)',
      m.postWrite === undefined);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. Forwarding — --include-tests ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-inc-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-inc-out-'));
  try {
    buildFixture(fx);
    write(fx, 'tests/sample.test.ts', `const x = 1;\n`);
    runPreWrite(fx, out, ['--include-tests']);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    runPostWrite(fx, out, advisory, ['--include-tests']);
    const inv = JSON.parse(readFileSync(path.join(out, 'any-inventory.json'), 'utf8'));
    assert('T6. --include-tests forwarded → meta.includeTests === true',
      inv.meta.includeTests === true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T7. Forwarding — --no-include-tests (reviewer P0-2) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-noinc-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-noinc-out-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    runPostWrite(fx, out, advisory, ['--no-include-tests']);
    const inv = JSON.parse(readFileSync(path.join(out, 'any-inventory.json'), 'utf8'));
    assert('T7. --no-include-tests forwarded → meta.includeTests === false',
      inv.meta.includeTests === false);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T8. Forwarding — --production ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-prod-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-prod-out-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    runPostWrite(fx, out, advisory, ['--production']);
    const inv = JSON.parse(readFileSync(path.join(out, 'any-inventory.json'), 'utf8'));
    assert('T8. --production forwarded → meta.includeTests === false',
      inv.meta.includeTests === false);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T9. --delta-out relocates deltaPath (reviewer P0-3) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-dout-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-dout-out-'));
  const deltaOut = mkdtempSync(path.join(tmpdir(), 'aup-dout-delta-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    runPostWrite(fx, out, advisory, ['--delta-out', deltaOut]);

    const m = readManifest(out);
    const deltaFile = path.join(deltaOut, 'post-write-delta.latest.json');
    assert('T9a. manifest.postWrite.deltaPath starts with --delta-out dir',
      m.postWrite?.deltaPath?.startsWith(deltaOut),
      `deltaPath=${m.postWrite?.deltaPath}, deltaOut=${deltaOut}`);
    assert('T9b. delta file exists at --delta-out location',
      existsSync(deltaFile));
    assert('T9c. default output dir does NOT contain a post-write-delta.latest.json',
      !existsSync(path.join(out, 'post-write-delta.latest.json')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(deltaOut, { recursive: true, force: true });
  }
}

// ═══ T10. Path with spaces + $ ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'aup-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    const res = runPostWrite(fx, out, advisory);
    assert('T10. path with spaces + $ survives end-to-end via audit-repo',
      res.status === 0 &&
      res.stdout.includes('## post-write delta') &&
      existsSync(path.join(out, 'post-write-delta.latest.json')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T11. Stdout ordering + stderr segregation ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-sep-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-sep-out-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    const res = runPostWrite(fx, out, advisory);

    // Stdout flow: pipeline step logs ([audit-repo] ok NAME) → post-write
    // spawn's delta Markdown (## post-write delta …) → orchestrator's final
    // [audit-repo] wrote manifest.json line.
    const stepIdx = res.stdout.indexOf('[audit-repo] ok ');
    const deltaIdx = res.stdout.indexOf('## post-write delta');
    const wroteIdx = res.stdout.indexOf('[audit-repo] wrote');
    assert('T11a. stdout has step log BEFORE post-write delta BEFORE wrote-line',
      stepIdx >= 0 && deltaIdx >= 0 && wroteIdx >= 0 &&
      stepIdx < deltaIdx && deltaIdx < wroteIdx,
      `stepIdx=${stepIdx}, deltaIdx=${deltaIdx}, wroteIdx=${wroteIdx}`);
    assert('T11b. stdout has NO [post-write] diagnostic prefix',
      !/\[post-write\]/.test(res.stdout));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T12. --strict-post-write: ran=false with bogus advisory → exit 2 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-strict-bogus-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-strict-bogus-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [AUDIT_REPO,
      '--root', fx, '--output', out, '--profile', 'quick',
      '--post-write', '--pre-write-advisory', path.join(out, 'does-not-exist.json'),
      '--strict-post-write',
    ], { encoding: 'utf8' });

    assert('T12. --strict-post-write + bogus advisory → exit 2',
      res.status === 2, `status=${res.status}, stderr=${res.stderr.slice(0, 200)}`);

    const m = readManifest(out);
    assert('T12b. manifest.postWrite.ran === false (unchanged from non-strict)',
      m.postWrite?.ran === false);
    assert('T12c. reason still starts with post-write.mjs exited non-zero:',
      (m.postWrite?.reason ?? '').startsWith('post-write.mjs exited non-zero:'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T12a. Absolute preWrite.anyInventoryPath works via audit-repo ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-abs-before-'));
  const preOut = mkdtempSync(path.join(tmpdir(), 'aup-abs-before-pre-'));
  const postOut = mkdtempSync(path.join(tmpdir(), 'aup-abs-before-post-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, preOut);
    const advisory = path.join(preOut, 'pre-write-advisory.latest.json');
    const parsed = JSON.parse(readFileSync(advisory, 'utf8'));
    const beforeRel = parsed.preWrite?.anyInventoryPath;
    parsed.preWrite.anyInventoryPath = path.join(preOut, beforeRel);
    writeFileSync(advisory, JSON.stringify(parsed, null, 2) + '\n');

    const res = runPostWrite(fx, postOut, advisory);
    assert('T12a. audit-repo --post-write accepts absolute anyInventoryPath',
      res.status === 0, `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`);
    const m = readManifest(postOut);
    assert('T12b. manifest baselineStatus === available for absolute before path',
      m.postWrite?.ran === true && m.postWrite?.baselineStatus === 'available',
      JSON.stringify(m.postWrite));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(preOut, { recursive: true, force: true });
    rmSync(postOut, { recursive: true, force: true });
  }
}

// ═══ T13. --strict-post-write: happy path → exit 0 (no change when ran=true) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-strict-ok-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-strict-ok-out-'));
  try {
    buildFixture(fx);
    runPreWrite(fx, out);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    const res = runPostWrite(fx, out, advisory, ['--strict-post-write']);
    assert('T13. --strict-post-write + happy path → exit 0',
      res.status === 0, `status=${res.status}`);
    const m = readManifest(out);
    assert('T13b. manifest.postWrite.ran === true',
      m.postWrite?.ran === true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T14. --strict-post-write without --post-write → inert (no-op) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-strict-noop-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-strict-noop-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [AUDIT_REPO,
      '--root', fx, '--output', out, '--profile', 'quick',
      '--strict-post-write',
    ], { encoding: 'utf8' });
    assert('T14. --strict-post-write without --post-write → exit 0 (no effect)',
      res.status === 0, `status=${res.status}`);
    const m = readManifest(out);
    assert('T14b. manifest.postWrite undefined',
      m.postWrite === undefined);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T15. --strict-post-write: baseline-missing (ran=true, silent-new=0) → exit 0 ═══
//
// Pinning that strict mode does NOT conflate baseline-missing with failure.
// Baseline-missing is a LEGITIMATE ran=true outcome (delta computed, all
// entries observed-unbaselined, no silent-new, requiredAcknowledgements empty).
// Strict mode targets the ran=false failure mode only.

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-strict-bm-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-strict-bm-out-'));
  try {
    buildFixture(fx);
    // Pre-write with --no-fresh-audit → advisory has no anyInventoryPath.
    runPreWrite(fx, out, ['--no-fresh-audit']);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    // Add an escape so the delta is non-empty.
    write(fx, 'src/a.ts', `export const formatDate = (d) => d as any;\n`);
    const res = runPostWrite(fx, out, advisory, ['--strict-post-write']);
    assert('T15. --strict-post-write + baseline-missing → exit 0 (ran=true)',
      res.status === 0, `status=${res.status}`);
    const m = readManifest(out);
    assert('T15b. manifest.postWrite.ran === true AND baselineStatus === missing',
      m.postWrite?.ran === true && m.postWrite?.baselineStatus === 'missing');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T16. --strict-post-write-confidence: caveated ran=true delta → exit 2 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-strict-conf-bm-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-strict-conf-bm-out-'));
  try {
    buildFixture(fx);
    // Pre-write with --no-fresh-audit → advisory has no anyInventoryPath,
    // so post-write computes a legitimate but confidence-limited delta.
    runPreWrite(fx, out, ['--no-fresh-audit']);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');
    write(fx, 'src/a.ts', `export const formatDate = (d) => d as any;\n`);
    const res = runPostWrite(fx, out, advisory, ['--strict-post-write-confidence']);
    assert('T16. --strict-post-write-confidence + baseline-missing → exit 2',
      res.status === 2, `status=${res.status}, stderr=${res.stderr.slice(0, 300)}`);
    assert('T16b. stderr names confidence-limited post-write delta',
      /strict-post-write-confidence/.test(res.stderr ?? '') &&
      /baseline=missing/.test(res.stderr ?? ''),
      res.stderr);
    const m = readManifest(out);
    assert('T16c. manifest is still written with ran=true baselineStatus=missing',
      m.postWrite?.ran === true && m.postWrite?.baselineStatus === 'missing',
      JSON.stringify(m.postWrite));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T17. post-write detects unexpected new files outside intent.files ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'aup-file-delta-'));
  const out = mkdtempSync(path.join(tmpdir(), 'aup-file-delta-out-'));
  try {
    buildFixture(fx);
    runPreWriteWithFiles(fx, out, ['src/planned.ts']);
    const advisory = path.join(out, 'pre-write-advisory.latest.json');

    write(fx, 'src/planned.ts', `export const planned = 1;\n`);
    write(fx, 'src/unplanned.ts', `export const unplanned = 1;\n`);

    const res = runPostWrite(fx, out, advisory);
    assert('T17. post-write with planned + unexpected files → exit 0',
      res.status === 0, `status=${res.status}; stderr=${res.stderr.slice(0, 300)}`);

    const m = readManifest(out);
    const delta = JSON.parse(readFileSync(m.postWrite.deltaPath, 'utf8'));
    assert('T17b. fileDelta.status computed',
      delta.fileDelta?.status === 'computed', JSON.stringify(delta.fileDelta));
    assert('T17c. planned file appears under plannedNew',
      delta.fileDelta?.plannedNew?.length === 1 &&
      delta.fileDelta.plannedNew[0] === 'src/planned.ts',
      JSON.stringify(delta.fileDelta));
    assert('T17d. unplanned file appears under unexpectedNew',
      delta.fileDelta?.unexpectedNew?.length === 1 &&
      delta.fileDelta.unexpectedNew[0] === 'src/unplanned.ts',
      JSON.stringify(delta.fileDelta));
    assert('T17e. manifest exposes unexpected new file count',
      m.postWrite?.unexpectedNewFileCount === 1 &&
      m.postWrite?.plannedMissingFileCount === 0,
      JSON.stringify(m.postWrite));
    const summary = readFileSync(path.join(out, 'audit-summary.latest.md'), 'utf8');
    assert('T17f. summary tells reader to review file delta',
      summary.includes('Post-write file delta needs review') &&
      summary.includes('1 unexpected new file'),
      summary);
    assert('T17g. stdout renders File delta section',
      res.stdout.includes('File delta:') &&
      res.stdout.includes('Unexpected new files:') &&
      res.stdout.includes('src/unplanned.ts'),
      res.stdout.slice(-1000));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
