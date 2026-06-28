// Tests for `generate-canon-draft.mjs --source topology` — P3-3 Step 3.
//
// Pinning rules from docs/history/phases/p3/p3-3.md v3 §5.4:
//   - --source topology accepted; P3-1 / P3-2 routes unchanged.
//   - topology.json ABSENT → exit 2 (hard dependency, distinct from exit 1).
//   - --source invalid value → exit 1; stderr lists all three accepted sources.
//   - Non-overwrite versioning: second run writes topology.v2.md.
//   - Existing canonical/topology.md → ⚠ Existing canon detected header.
//   - --canon-output override respected.
//   - Shell safety: fixture under `my $root/` passes end-to-end.
//   - triage.json absent → §5 omitted; CLI exits 0.
//   - topology.meta.complete === false → header warning + exit 0.
//   - Single-package vs monorepo — draft Mode: line reflects triage.json.mode.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'generate-canon-draft.mjs');
const TOPO_CLI = path.join(DIR, 'measure-topology.mjs');

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

function buildTopologyFixture(fx) {
  write(fx, 'package.json', JSON.stringify({ name: 'topo-fx', type: 'module' }));
  write(fx, '_lib/util.mjs', `export function helper() { return 1 }\n`);
  write(fx, 'src/app.mjs',
    `import { helper } from '../_lib/util.mjs';\n` +
    `export const x = helper();\n`);
}

// ═══ T1. Happy path — topology draft emitted, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-happy-out-'));
  try {
    buildTopologyFixture(fx);
    // Run measure-topology to produce topology.json
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });

    assert('T1a. exit 0 on happy path', res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);

    const draftPath = path.join(fx, 'canonical-draft', 'topology.md');
    assert('T1b. default canon-output writes to <root>/canonical-draft/topology.md',
      existsSync(draftPath));

    const md = readFileSync(draftPath, 'utf8');
    assert('T1c. draft contains "# Topology draft" header',
      md.includes('# Topology draft'));
    assert('T1d. §1 Submodule inventory section present',
      md.includes('## 1. Submodule inventory'));
    assert('T1e. §3 Cycles section present (✅ acyclic on this fixture)',
      md.includes('## 3. Cycles') && md.includes('✅'));
    assert('T1f. CrossEdgeSource: full-list (producer emits crossSubmoduleEdges)',
      md.includes('CrossEdgeSource: full-list'));
    assert('T1g. ClassificationConfidence: high',
      md.includes('ClassificationConfidence: high'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. --source topology accepted; unknown value rejected with all 3 listed ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-src-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-src-out-'));
  try {
    buildTopologyFixture(fx);
    const resBad = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'foobar',
    ], { encoding: 'utf8' });
    assert('T2a. --source foobar → exit 1 (unknown source)',
      resBad.status === 1);
    assert('T2b. stderr lists all 4 accepted sources (P3 closed)',
      /type-ownership/.test(resBad.stderr) &&
      /helper-registry/.test(resBad.stderr) &&
      /topology/.test(resBad.stderr) &&
      /naming/.test(resBad.stderr));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. topology.json ABSENT → exit 2 (hard dependency) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-notopo-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-notopo-out-'));
  try {
    buildTopologyFixture(fx);
    // Intentionally do NOT run measure-topology.mjs — topology.json absent
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T3a. topology.json absent → exit 2 (NOT exit 1 — distinct code)',
      res.status === 2, `got status=${res.status}; stderr=${res.stderr.slice(0, 300)}`);
    assert('T3b. stderr mentions measure-topology.mjs (recovery hint)',
      /measure-topology\.mjs/.test(res.stderr));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. P3-1 / P3-2 regression — existing sources unchanged ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-regr-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-regr-out-'));
  try {
    buildTopologyFixture(fx);
    write(fx, 'src/types.ts', `export type User = { id: string };\n`);
    // Plant minimal symbols.json for type-ownership + helper-registry paths
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      meta: { tool: 'build-symbol-graph.mjs', generated: '2026-04-21T00:00:00Z', root: fx, supports: { identityFanIn: true } },
      defIndex: { 'src/types.ts': { User: { name: 'User', kind: 'TSTypeAliasDeclaration', line: 1 } } },
      fanInByIdentity: {},
      reExportsByFile: {},
    }));
    const res1 = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'type-ownership',
    ], { encoding: 'utf8' });
    assert('T4a. --source type-ownership exits 0 (P3-1 regression)',
      res1.status === 0, `stderr=${res1.stderr.slice(0, 200)}`);
    assert('T4b. type-ownership.md emitted',
      existsSync(path.join(fx, 'canonical-draft', 'type-ownership.md')));

    const res2 = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'helper-registry',
    ], { encoding: 'utf8' });
    assert('T4c. --source helper-registry exits 0 (P3-2 regression)',
      res2.status === 0, `stderr=${res2.stderr.slice(0, 200)}`);
    assert('T4d. helper-registry.md emitted',
      existsSync(path.join(fx, 'canonical-draft', 'helper-registry.md')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Non-overwrite versioning ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-ver-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-ver-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'topology'], { stdio: 'ignore' });
    const first = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    // Refresh topology.json with a new file
    write(fx, 'new/extra.mjs', `export const extra = 1;\n`);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'topology'], { stdio: 'ignore' });
    const files = readdirSync(path.join(fx, 'canonical-draft'));
    assert('T5a. second run produces topology.v2.md',
      files.includes('topology.v2.md'));
    assert('T5b. first topology.md preserved byte-for-byte',
      readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8') === first);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T6. Existing canonical/topology.md → observational header ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-canon-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-canon-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    write(fx, 'canonical/topology.md', '# Existing canon (prior content)\n');
    execFileSync(NODE, [CLI, '--root', fx, '--output', out, '--source', 'topology'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('T6. existing canon → draft carries "⚠ Existing canon detected" for topology.md',
      md.includes('⚠ Existing canon detected') && md.includes('topology.md'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T7. --canon-output override ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-cout-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-cout-out-'));
  const cout = mkdtempSync(path.join(tmpdir(), 'cdt-cout-custom-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--canon-output', cout, '--source', 'topology',
    ], { stdio: 'ignore' });
    assert('T7a. --canon-output custom dir receives topology.md',
      existsSync(path.join(cout, 'topology.md')));
    assert('T7b. default <root>/canonical-draft/ NOT created',
      !existsSync(path.join(fx, 'canonical-draft', 'topology.md')));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(cout, { recursive: true, force: true });
  }
}

// ═══ T8. Shell safety: path with spaces + $ ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'cdt-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T8. path with spaces + $ survives end-to-end',
      res.status === 0 && existsSync(path.join(fx, 'canonical-draft', 'topology.md')),
      `stderr=${res.stderr.slice(0, 300)}`);
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T9. triage.json absent → §5 omitted, CLI exits 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-notriage-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-notriage-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    // Intentionally NO triage.json in out/
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T9a. exit 0 when triage.json absent',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('T9b. draft has no §5 Workspace boundaries section',
      !md.includes('## 5. Workspace boundaries'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T10. topology.meta.complete=false → header warning, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-incom-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-incom-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    // Manually patch topology.json's meta.complete to false
    const topoPath = path.join(out, 'topology.json');
    const topology = JSON.parse(readFileSync(topoPath, 'utf8'));
    topology.meta.complete = false;
    writeFileSync(topoPath, JSON.stringify(topology, null, 2));
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T10a. exit 0 when complete=false',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('T10b. draft header carries incomplete warning',
      md.includes('topology.json incomplete'));
    assert('T10c. TopologyComplete: false in meta lines',
      md.includes('TopologyComplete: false'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T11. --production → scope reflects "TS/JS production files" ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-prod-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-prod-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology', '--production',
    ], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('T11. --production → Scope: "TS/JS production files"',
      md.includes('Scope: TS/JS production files'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T12. crossSubmoduleEdges absent (pre-P3-3-pre producer simulation) → degraded mode ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-degraded-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-degraded-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    // Simulate pre-P3-3-pre producer: strip crossSubmoduleEdges
    const topoPath = path.join(out, 'topology.json');
    const topology = JSON.parse(readFileSync(topoPath, 'utf8'));
    delete topology.crossSubmoduleEdges;
    writeFileSync(topoPath, JSON.stringify(topology, null, 2));
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T12a. exit 0 in degraded mode (graceful)',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('T12b. CrossEdgeSource: top-30-only + ClassificationConfidence: medium',
      md.includes('CrossEdgeSource: top-30-only') &&
      md.includes('ClassificationConfidence: medium'));
    assert('T12c. Header warning about top-30 cross-edge lens',
      md.includes('top-30 cross-edge lens'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T13. stderr summary line with counts ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-summary-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-summary-out-'));
  try {
    buildTopologyFixture(fx);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    const res = spawnSync(NODE, [CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T13. stderr summary mentions submodule count + crossEdgeSource + confidence',
      /\d+ submodules/.test(res.stderr) &&
      /crossEdgeSource=/.test(res.stderr) &&
      /confidence=/.test(res.stderr),
      `stderr=${res.stderr}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T14. Missing --root → exit 1 ═══

{
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-noroot-'));
  try {
    const res = spawnSync(NODE, [CLI,
      '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('T14. missing --root → exit 1 (distinct from exit 2)',
      res.status === 1, `got status=${res.status}`);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
