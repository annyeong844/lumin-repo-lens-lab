// Tests for `generate-canon-draft.mjs` CLI — P3-1 Step 3.
//
// Pinning rules from docs/history/phases/p3/p3-1.md v2 §5.4:
//   - --source type-ownership is the only accepted value in P3-1.
//   - Non-overwrite versioning: v{N}.md on rerun.
//   - Existing-canon header block when canonical/type-ownership.md present.
//   - --canon-output override respected.
//   - Shell safety: path with spaces + $.
//   - Scan-range flags forwarded; scope string reflects actual.
//   - symbols.json absent: fresh AST pass flag + barrels-opaque:true meta.

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
  write(fx, 'package.json', JSON.stringify({ name: 'cd-fx', type: 'module' }));
  write(fx, 'src/types.ts', `export type User = { id: string; name: string };\n`);
}

function writeSymbols(out, override = {}) {
  const base = {
    meta: {
      tool: 'build-symbol-graph.mjs',
      generated: '2026-04-21T00:00:00Z',
      root: '/fake',
      supports: { identityFanIn: true },
    },
    defIndex: {
      'src/types.ts': {
        User: { name: 'User', kind: 'TSTypeAliasDeclaration', line: 1 },
      },
    },
    fanInByIdentity: { 'src/types.ts::User': 2 },
    reExportsByFile: {},
  };
  const merged = { ...base, ...override };
  writeFileSync(path.join(out, 'symbols.json'), JSON.stringify(merged));
  return merged;
}

function writeShapeIndex(out, facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    if (!groupsByHash[fact.hash]) groupsByHash[fact.hash] = [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  writeFileSync(path.join(out, 'shape-index.json'), JSON.stringify({
    schemaVersion: 'shape-index.v1',
    meta: { complete },
    facts,
    groupsByHash,
    diagnostics: [],
  }));
}

// ═══ T1. Happy path — draft emitted, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-happy-out-'));
  try {
    buildFixture(fx);
    writeSymbols(out);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });

    assert('T1a. exit 0 on happy path', res.status === 0, `stderr=${res.stderr.slice(0, 200)}`);

    const draftPath = path.join(fx, 'canonical-draft', 'type-ownership.md');
    assert('T1b. default canon-output writes to <root>/canonical-draft/',
      existsSync(draftPath));

    const md = readFileSync(draftPath, 'utf8');
    assert('T1c. draft contains "# Type ownership draft" header',
      md.includes('# Type ownership draft'));
    assert('T1d. draft includes the User identity', md.includes('src/types.ts::User'));
    assert('T1e. draft status column has a canonical label marker',
      /single-owner-(weak|strong)|zero-internal-fan-in/.test(md));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. --source other values rejected ═══
//
// (v2 refresh — post-P3-2 helper-registry is now accepted, so the
// rejection test uses `topology` which lands in P3-3.)

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-src-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-src-out-'));
  try {
    buildFixture(fx);
    writeSymbols(out);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'foobar',
    ], { encoding: 'utf8' });
    assert('T2. --source foobar → exit 1 (unknown source)',
      res.status === 1);
    assert('T2b. stderr lists all 4 canonical sources (P3 now closed)',
      /type-ownership/.test(res.stderr) &&
      /helper-registry/.test(res.stderr) &&
      /topology/.test(res.stderr) &&
      /naming/.test(res.stderr));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. Missing --root → exit 1 ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'cd-noroot-out-'));
  try {
    const res = spawnSync(NODE, [CLI,
      '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });
    assert('T3. missing --root → exit 1',
      res.status === 1 || res.status !== 0);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. Non-overwrite versioning ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-rerun-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-rerun-out-'));
  try {
    buildFixture(fx);
    writeSymbols(out);
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });
    const firstContent = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    // Change fixture slightly so second run differs (add a type)
    write(fx, 'src/more.ts', `export type Extra = { x: number };\n`);
    writeSymbols(out, {
      defIndex: {
        'src/types.ts': { User: { name: 'User', kind: 'TSTypeAliasDeclaration', line: 1 } },
        'src/more.ts':  { Extra: { name: 'Extra', kind: 'TSTypeAliasDeclaration', line: 1 } },
      },
      fanInByIdentity: { 'src/types.ts::User': 2, 'src/more.ts::Extra': 1 },
    });
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });

    const files = readdirSync(path.join(fx, 'canonical-draft'));
    assert('T4a. second run produces type-ownership.v2.md',
      files.includes('type-ownership.v2.md'));
    assert('T4b. first type-ownership.md preserved byte-for-byte',
      readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8') === firstContent);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Existing canon header block ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-canon-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-canon-out-'));
  try {
    buildFixture(fx);
    // Plant an existing canonical/type-ownership.md.
    write(fx, 'canonical/type-ownership.md', '# Existing canon (prior content)\n');
    writeSymbols(out);
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    assert('T5. existing canon → draft carries "⚠ Existing canon detected" header',
      md.includes('⚠ Existing canon detected'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. --canon-output override ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-cout-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-cout-out-'));
  const cout = mkdtempSync(path.join(tmpdir(), 'cd-cout-custom-'));
  try {
    buildFixture(fx);
    writeSymbols(out);
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--canon-output', cout, '--source', 'type-ownership',
    ], { stdio: 'ignore' });
    assert('T6a. --canon-output custom dir receives the draft',
      existsSync(path.join(cout, 'type-ownership.md')));
    assert('T6b. default <root>/canonical-draft/ NOT created',
      !existsSync(path.join(fx, 'canonical-draft', 'type-ownership.md')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(cout, { recursive: true, force: true });
  }
}

// ═══ T7. Path with spaces + $ ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'cd-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildFixture(fx);
    writeSymbols(out);
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });
    assert('T7. path with spaces + $ survives end-to-end',
      res.status === 0 && existsSync(path.join(fx, 'canonical-draft', 'type-ownership.md')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T8. Scope reflects --production / --include-tests ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-scope-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-scope-out-'));
  try {
    buildFixture(fx);
    writeSymbols(out);
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership', '--production',
    ], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    assert('T8a. --production → scope "TS/JS production files"',
      md.includes('TS/JS production files'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T9. symbols.json absent → fresh AST pass diagnostic ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-nosym-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-nosym-out-'));
  try {
    buildFixture(fx);
    // No writeSymbols() here — symbols.json absent.
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });
    assert('T9a. exit 0 when symbols.json absent (graceful)',
      res.status === 0);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    assert('T9b. draft meta notes fresh-ast-pass or barrels-opaque',
      md.includes('fresh-ast-pass') || md.includes('barrels-opaque') || md.includes('opaque'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T10. shape-index.json enriches type-ownership draft evidence ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cd-shape-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cd-shape-out-'));
  const hash = 'sha256:' + 'd'.repeat(64);
  try {
    buildFixture(fx);
    writeSymbols(out, {
      defIndex: {
        'src/a.ts': { Result: { name: 'Result', kind: 'TSTypeAliasDeclaration', line: 1 } },
        'src/b.ts': { Result: { name: 'Result', kind: 'TSTypeAliasDeclaration', line: 1 } },
      },
      fanInByIdentity: {
        'src/a.ts::Result': 18,
        'src/b.ts::Result': 3,
      },
    });
    writeShapeIndex(out, [
      { identity: 'src/a.ts::Result', hash },
      { identity: 'src/b.ts::Result', hash },
    ]);

    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });

    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    assert('T10a. exit 0 with optional shape-index.json present',
      res.status === 0, `stderr=${res.stderr.slice(0, 200)}`);
    assert('T10b. draft preserves DUPLICATE_STRONG label',
      md.includes('DUPLICATE_STRONG'));
    assert('T10c. draft includes shape evidence from shape-index.json',
      md.includes('## Shape evidence') &&
      md.includes('same-shape evidence') &&
      md.includes(hash));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
