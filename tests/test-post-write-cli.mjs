// CLI smoke test for post-write.mjs — P2-1 step 4.
//
// Pinning rules from docs/history/phases/p2/p2-1.md v3 §5.4.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const PREWRITE = path.join(DIR, 'pre-write.mjs');
const POSTWRITE = path.join(DIR, 'post-write.mjs');

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
  write(fx, 'src/a.ts', `export const formatDate = (d) => d.toString();\n`);
}

function writeIntent(outDir, plannedTypeEscapes = []) {
  const intent = { names: [], shapes: [], files: [], dependencies: [], plannedTypeEscapes };
  const p = path.join(outDir, 'intent.json');
  writeFileSync(p, JSON.stringify(intent));
  return p;
}

function preWrite(fx, out, intentPath, extraArgs = []) {
  return execFileSync(NODE, [PREWRITE, '--root', fx, '--output', out, '--intent', intentPath, ...extraArgs], {
    stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8',
  });
}

function postWrite(fx, out, advisoryPath, extraArgs = []) {
  return execFileSync(NODE, [POSTWRITE, '--root', fx, '--output', out, '--pre-write-advisory', advisoryPath, ...extraArgs], {
    stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8',
  });
}

function findAdvisory(out) {
  return path.join(out, 'pre-write-advisory.latest.json');
}

// ═══ T1. Happy path — planned + unplanned edit ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-happy-out-'));
  try {
    buildFixture(fx);
    // Pre-write with a planned escape declaration.
    const intent = writeIntent(out, [
      { escapeKind: 'as-any', locationHint: 'src/a.ts', reason: 'upstream bug', codeShape: 'x as any' },
    ]);
    preWrite(fx, out, intent);
    // Edit: add one planned escape + one unplanned.
    write(fx, 'src/a.ts',
      `export const formatDate = (d) => d.toString();\n` +
      `export const planned = (x) => x as any;\n` +
      `export const unplanned = (y) => y as any;\n`
    );

    const advisory = findAdvisory(out);
    const stdout = postWrite(fx, out, advisory);

    assert('T1. exit 0 (implicit — execFileSync would throw)', true);
    assert('T1b. stdout contains "post-write delta" header',
      stdout.includes('## post-write delta'));
    assert('T1c. delta JSON latest written',
      existsSync(path.join(out, 'post-write-delta.latest.json')));
    const deltaFiles = readdirSync(out).filter((n) =>
      n.startsWith('post-write-delta.') && !n.endsWith('.latest.json'));
    assert('T1d. specific delta file exists',
      deltaFiles.length === 1);
    const delta = JSON.parse(readFileSync(path.join(out, 'post-write-delta.latest.json'), 'utf8'));
    assert('T1e. preWriteInvocationId matches advisory',
      delta.preWriteInvocationId === JSON.parse(readFileSync(advisory, 'utf8')).invocationId);
    assert('T1f. deltaInvocationId populated and well-formed',
      typeof delta.deltaInvocationId === 'string' && delta.deltaInvocationId.length > 0);
    assert('T1g. deltaInvocationId !== preWriteInvocationId (freshly generated)',
      delta.deltaInvocationId !== delta.preWriteInvocationId);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. Source-grep pinning: CLI generates deltaInvocationId before computeDelta ═══

{
  const src = readFileSync(POSTWRITE, 'utf8');
  assert('T2a. post-write.mjs imports generateDeltaInvocationId',
    src.includes('generateDeltaInvocationId'));
  assert('T2b. post-write.mjs calls computeDelta with deltaInvocationId in arg object',
    /computeDelta\s*\(\s*\{[^}]*deltaInvocationId/s.test(src));
}

// ═══ T3. Re-run preserves prior specific file ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-rerun-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-rerun-out-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    preWrite(fx, out, intent);
    const advisory = findAdvisory(out);
    postWrite(fx, out, advisory);
    const after1 = readdirSync(out).filter((n) =>
      n.startsWith('post-write-delta.') && !n.endsWith('.latest.json'));
    postWrite(fx, out, advisory);
    const after2 = readdirSync(out).filter((n) =>
      n.startsWith('post-write-delta.') && !n.endsWith('.latest.json'));
    assert('T3. second post-write run produces a NEW specific file (prior preserved)',
      after2.length === 2 && after1.every((n) => after2.includes(n)),
      `after1=${after1}, after2=${after2}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. --no-fresh-audit + missing after-inventory ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-nofresh-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-nofresh-out-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    preWrite(fx, out, intent);
    const advisory = findAdvisory(out);
    // Delete any-inventory.json that was produced during pre-write's hook
    // to simulate "no after-inventory available".
    const invPath = path.join(out, 'any-inventory.json');
    if (existsSync(invPath)) rmSync(invPath);

    const stdout = postWrite(fx, out, advisory, ['--no-fresh-audit']);
    const delta = JSON.parse(readFileSync(path.join(out, 'post-write-delta.latest.json'), 'utf8'));
    assert('T4a. capabilityParity.status === "missing"',
      delta.capabilityParity.status === 'missing');
    assert('T4b. entries: []',
      delta.entries.length === 0);
    assert('T4c. capabilityFailures carries after-inventory-missing',
      (delta.capabilityFailures ?? []).some((f) => f.kind === 'after-inventory-missing'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Scan-range flag forwarding ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-flags-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-flags-out-'));
  try {
    buildFixture(fx);
    write(fx, 'tests/sample.test.ts', `const x = 1;\n`);
    const intent = writeIntent(out);
    // Pre-write with --include-tests so advisory/before-inventory scope is "tests included".
    preWrite(fx, out, intent, ['--include-tests']);
    const advisory = findAdvisory(out);

    // Post-write explicit --include-tests → after-inventory meta.includeTests === true.
    postWrite(fx, out, advisory, ['--include-tests']);
    const invInc = JSON.parse(readFileSync(path.join(out, 'any-inventory.json'), 'utf8'));
    assert('T5a. --include-tests: after-inventory meta.includeTests === true',
      invInc.meta.includeTests === true);

    // Post-write --production → after-inventory meta.includeTests === false.
    postWrite(fx, out, advisory, ['--production']);
    const invProd = JSON.parse(readFileSync(path.join(out, 'any-inventory.json'), 'utf8'));
    assert('T5b. --production: after-inventory meta.includeTests === false',
      invProd.meta.includeTests === false);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. Path with spaces + $ ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'pw2-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    preWrite(fx, out, intent);
    const advisory = findAdvisory(out);
    const stdout = postWrite(fx, out, advisory);
    assert('T6. path with spaces + $ survives end-to-end',
      stdout.includes('## post-write delta') &&
      existsSync(path.join(out, 'post-write-delta.latest.json')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T7. Missing --pre-write-advisory ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-noadv-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-noadv-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [POSTWRITE, '--root', fx, '--output', out], {
      stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8',
    });
    assert('T7a. missing --pre-write-advisory → exit 1',
      res.status === 1);
    assert('T7b. stderr mentions advisory',
      /pre-write-advisory/i.test(res.stderr ?? ''));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T8. Advisory file does not exist ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-bogus-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-bogus-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [POSTWRITE,
      '--root', fx, '--output', out, '--pre-write-advisory', path.join(out, 'does-not-exist.json'),
    ], { stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8' });
    assert('T8a. non-existent advisory → exit 1',
      res.status === 1);
    assert('T8b. stderr mentions not found',
      /not found/i.test(res.stderr ?? ''));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T9. Advisory without preWrite.anyInventoryPath ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-nopath-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-nopath-out-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    // Pre-write with --no-fresh-audit so the hook skips → advisory lacks anyInventoryPath.
    preWrite(fx, out, intent, ['--no-fresh-audit']);
    const advisory = findAdvisory(out);
    const advContent = JSON.parse(readFileSync(advisory, 'utf8'));
    assert('T9pre. advisory has no preWrite.anyInventoryPath (pre-condition)',
      !advContent.preWrite || !('anyInventoryPath' in (advContent.preWrite ?? {})));

    const stdout = postWrite(fx, out, advisory);
    const delta = JSON.parse(readFileSync(path.join(out, 'post-write-delta.latest.json'), 'utf8'));
    assert('T9a. baseline.status === "missing"',
      delta.baseline.status === 'missing');
    assert('T9b. exit 0 (execFileSync didn’t throw)', true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T10. Parse error in after-inventory → caveated summary ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-pe-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-pe-out-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    preWrite(fx, out, intent);
    const advisory = findAdvisory(out);
    // Introduce a parse-error file so after-inventory has complete=false.
    write(fx, 'src/bad.ts', `const x = ;;;broken\n`);
    const stdout = postWrite(fx, out, advisory);
    assert('T10a. stdout mentions after-inventory incomplete',
      stdout.includes('after-inventory incomplete'));
    assert('T10b. stdout does NOT contain "No silent new any in the scan range."',
      !stdout.includes('No silent new any in the scan range.'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T11. Stdout markdown only, stderr diagnostics only ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-sep-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw2-sep-out-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(out);
    preWrite(fx, out, intent);
    const advisory = findAdvisory(out);
    const res = spawnSync(NODE, [POSTWRITE,
      '--root', fx, '--output', out, '--pre-write-advisory', advisory,
    ], { stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8' });
    assert('T11a. exit 0', res.status === 0);
    assert('T11b. stdout starts with "## post-write delta"',
      res.stdout.startsWith('## post-write delta'));
    assert('T11c. stderr contains [post-write] diagnostic (or empty)',
      res.stderr.length === 0 || /\[post-write\]/.test(res.stderr));
    assert('T11d. stdout has NO [post-write] diagnostic prefix',
      !/\[post-write\]/.test(res.stdout));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T12. Advisory outside --output still finds before-inventory ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-cross-out-'));
  const preOut = mkdtempSync(path.join(tmpdir(), 'pw2-cross-pre-'));
  const postOut = mkdtempSync(path.join(tmpdir(), 'pw2-cross-post-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(preOut);
    preWrite(fx, preOut, intent);
    const advisory = findAdvisory(preOut);

    postWrite(fx, postOut, advisory);
    const delta = JSON.parse(readFileSync(path.join(postOut, 'post-write-delta.latest.json'), 'utf8'));
    assert('T12a. before-inventory found via advisory dir or scanRange.output',
      delta.baseline.status === 'available',
      `baseline=${JSON.stringify(delta.baseline)}`);
    assert('T12b. scan range parity can be evaluated',
      delta.scanRangeParity.status !== 'baseline-missing',
      `scanRangeParity=${JSON.stringify(delta.scanRangeParity)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(preOut, { recursive: true, force: true });
    rmSync(postOut, { recursive: true, force: true });
  }
}

// ═══ T13. Absolute preWrite.anyInventoryPath is loaded as-is ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw2-abs-before-'));
  const preOut = mkdtempSync(path.join(tmpdir(), 'pw2-abs-before-pre-'));
  const postOut = mkdtempSync(path.join(tmpdir(), 'pw2-abs-before-post-'));
  try {
    buildFixture(fx);
    const intent = writeIntent(preOut);
    preWrite(fx, preOut, intent);
    const advisory = findAdvisory(preOut);
    const parsed = JSON.parse(readFileSync(advisory, 'utf8'));
    const beforeRel = parsed.preWrite?.anyInventoryPath;
    const beforeAbs = path.join(preOut, beforeRel);
    parsed.preWrite.anyInventoryPath = beforeAbs;
    writeFileSync(advisory, JSON.stringify(parsed, null, 2) + '\n');

    postWrite(fx, postOut, advisory);
    const delta = JSON.parse(readFileSync(path.join(postOut, 'post-write-delta.latest.json'), 'utf8'));
    assert('T13. absolute preWrite.anyInventoryPath loads before-inventory',
      delta.baseline.status === 'available',
      `baseline=${JSON.stringify(delta.baseline)}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(preOut, { recursive: true, force: true });
    rmSync(postOut, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
