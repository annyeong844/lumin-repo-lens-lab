// Tests for `generate-canon-draft.mjs --source naming` — P3-4 Step 3.
//
// Pinning rules from docs/history/phases/p3/p3-4.md v2 §5.4:
//   - --source naming accepted.
//   - P3-1/2/3 regression — other sources still work.
//   - Non-overwrite versioning (shared helper).
//   - Existing canonical/naming.md → observational header.
//   - --canon-output override respected.
//   - Shell safety: my $root/.
//   - Scan-range flags forwarded.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'generate-canon-draft.mjs');

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
  write(fx, 'package.json', JSON.stringify({ name: 'nm-fx', type: 'module' }));
  // _lib with mixed-case files so multiple cohort states emerge.
  write(fx, '_lib/canon-util.mjs', `export function doWork() {}\n`);
  write(fx, '_lib/alias-helper.mjs', `export function loadAliases() {}\n`);
  write(fx, '_lib/resolver-core.mjs', `export function makeResolver() {}\n`);
  write(fx, 'src/app.mjs', `import { doWork } from '../_lib/canon-util.mjs';\nexport const x = doWork();\n`);
}

// ═══ T1. Happy path — naming draft emitted, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-happy-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'naming',
    ], { encoding: 'utf8' });
    assert('T1a. exit 0 on happy path', res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const draftPath = path.join(fx, 'canonical-draft', 'naming.md');
    assert('T1b. draft emitted at <root>/canonical-draft/naming.md',
      existsSync(draftPath));
    const md = readFileSync(draftPath, 'utf8');
    assert('T1c. header + sections present',
      md.includes('# Naming conventions draft') &&
      md.includes('## 1. File-naming cohorts') &&
      md.includes('## 2. Symbol-naming cohorts'));
    assert('T1d. CohortIdentityShape meta line',
      md.includes('CohortIdentityShape: submodule | submodule::kind'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. Invalid --source rejected with full 4-value list ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-src-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-src-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'foobar',
    ], { encoding: 'utf8' });
    assert('T2a. unknown source → exit 1',
      res.status === 1);
    assert('T2b. stderr lists all 4 sources (CANON_DRAFT_SOURCES)',
      /type-ownership/.test(res.stderr) &&
      /helper-registry/.test(res.stderr) &&
      /topology/.test(res.stderr) &&
      /naming/.test(res.stderr),
      `stderr=${res.stderr}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. Non-overwrite versioning ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-ver-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-ver-out-'));
  try {
    buildFixture(fx);
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'naming'], { stdio: 'ignore' });
    const first = readFileSync(path.join(fx, 'canonical-draft', 'naming.md'), 'utf8');
    // Add new file to change output
    write(fx, '_lib/extra-helper.mjs', `export function extra() {}\n`);
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'naming'], { stdio: 'ignore' });
    const files = readdirSync(path.join(fx, 'canonical-draft'));
    assert('T3a. second run writes naming.v2.md',
      files.includes('naming.v2.md'));
    assert('T3b. first naming.md byte-preserved',
      readFileSync(path.join(fx, 'canonical-draft', 'naming.md'), 'utf8') === first);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. Existing canon observational header ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-canon-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-canon-out-'));
  try {
    buildFixture(fx);
    write(fx, 'canonical/naming.md', '# Existing canon\n');
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'naming'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'naming.md'), 'utf8');
    assert('T4. existing canon → ⚠ Existing canon detected header',
      md.includes('⚠ Existing canon detected') && md.includes('naming.md'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. --canon-output override ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-cout-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-cout-out-'));
  const cout = mkdtempSync(path.join(tmpdir(), 'cdn-cout-custom-'));
  try {
    buildFixture(fx);
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--canon-output', cout, '--source', 'naming',
    ], { stdio: 'ignore' });
    assert('T5a. --canon-output dir receives draft',
      existsSync(path.join(cout, 'naming.md')));
    assert('T5b. default <root>/canonical-draft/ NOT populated',
      !existsSync(path.join(fx, 'canonical-draft', 'naming.md')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(cout, { recursive: true, force: true });
  }
}

// ═══ T6. Shell safety: my $root/ ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'cdn-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'naming',
    ], { encoding: 'utf8' });
    assert('T6. path with spaces + $ survives',
      res.status === 0 && existsSync(path.join(fx, 'canonical-draft', 'naming.md')),
      `stderr=${res.stderr.slice(0, 300)}`);
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T7. P3-1/2/3 regression — other sources still work after naming added ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-reg-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-reg-out-'));
  try {
    buildFixture(fx);
    write(fx, 'src/types.ts', `export type User = { id: string };\n`);
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      meta: { tool: 'build-symbol-graph.mjs', generated: '2026-04-21T00:00:00Z', root: fx, supports: { identityFanIn: true } },
      defIndex: { 'src/types.ts': { User: { name: 'User', kind: 'TSTypeAliasDeclaration', line: 1 } } },
      fanInByIdentity: {}, reExportsByFile: {},
    }));
    const t1 = spawnSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { encoding: 'utf8' });
    assert('T7a. type-ownership regression green', t1.status === 0);
    const t2 = spawnSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { encoding: 'utf8' });
    assert('T7b. helper-registry regression green', t2.status === 0);
    // topology needs topology.json; not including in this regression check.
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T8. stderr summary line present ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-sum-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-sum-out-'));
  try {
    buildFixture(fx);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'naming',
    ], { encoding: 'utf8' });
    assert('T8. stderr summary includes file/symbol cohort counts',
      /\d+ file cohorts/.test(res.stderr) && /\d+ symbol cohorts/.test(res.stderr),
      `stderr=${res.stderr}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T9. Missing --root → exit 1 ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-noroot-'));
  try {
    const res = spawnSync(NODE, [CLI,
      '--output', out, '--source', 'naming',
    ], { encoding: 'utf8' });
    assert('T9. missing --root → exit 1',
      res.status === 1);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T10. --production changes scope string ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdn-prod-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdn-prod-out-'));
  try {
    buildFixture(fx);
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'naming', '--production',
    ], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'naming.md'), 'utf8');
    assert('T10. --production → Scope: "TS/JS production files"',
      md.includes('Scope: TS/JS production files'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
