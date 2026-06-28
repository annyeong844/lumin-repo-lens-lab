// Integration tests for P3-3 topology draft — P3-3 Step 4.
//
// End-to-end: fixture repo → triage-repo.mjs + measure-topology.mjs →
// generate-canon-draft.mjs --source topology → parse emitted Markdown.
//
// Parser scope limit inherited from P3-1/P3-2 integration tests: fixture-
// controlled only. NOT a general-purpose Markdown parser.
//
// Pinning rules from docs/history/phases/p3/p3-3.md v3 §5.5:
//   - Row count === distinct submodule count (P0-4 correctness).
//   - Sum(Files column) === topology.summary.files (P0-4 correctness).
//   - SCC fixture → forbidden-cycle + cyclic-submodule.
//   - Oversize fixture → extreme-oversize for ≥1000 LOC.
//   - Monorepo fixture → §5 populated.
//   - Acyclic fixture → ✅ banner.
//   - topology.json absent → exit 2.
//   - triage.json absent → exit 0, §5 omitted.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CANON_CLI = path.join(DIR, 'generate-canon-draft.mjs');
const TOPO_CLI = path.join(DIR, 'measure-topology.mjs');
const TRIAGE_CLI = path.join(DIR, 'triage-repo.mjs');

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

// Fixture-controlled §1 submodule-inventory table parser.
function parseInventoryRows(md) {
  const lines = md.split('\n');
  const start = lines.findIndex((l) => l.startsWith('| Submodule | Files'));
  if (start < 0) return [];
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith('|')) break;
    const cells = line.split('|').slice(1, -1).map((c) => c.trim());
    if (cells.length < 7) continue;
    rows.push({
      submodule: cells[0],
      files: Number(cells[1]),
      loc: Number(cells[2]),
      inEdges: Number(cells[3]),
      outEdges: Number(cells[4]),
      scc: cells[5],
      status: cells[6],
      tags: cells[7] ?? '',
    });
  }
  return rows;
}

function runProducersAndCanon(fx, out, extraCanonFlags = []) {
  execFileSync(NODE, [TRIAGE_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
  execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
  execFileSync(NODE, [CANON_CLI,
    '--root', fx, '--output', out, '--source', 'topology', ...extraCanonFlags,
  ], { stdio: 'ignore' });
}

// ═══ F1. 3-submodule fixture — inventory row correctness (P0-4 pins) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-3sub-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-3sub-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: '3sub-fx', type: 'module' }));
    write(fx, '_lib/a.mjs', `export const a = 1;\n`);
    write(fx, '_lib/b.mjs', `export const b = 2;\n`);
    write(fx, 'src/main.mjs', `import { a } from '../_lib/a.mjs'; export const x = a;\n`);
    write(fx, 'src/util.mjs', `import { b } from '../_lib/b.mjs'; export const y = b;\n`);
    write(fx, 'tests/smoke.mjs', `import { a } from '../_lib/a.mjs'; export const z = a;\n`);
    runProducersAndCanon(fx, out);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    const topology = JSON.parse(readFileSync(path.join(out, 'topology.json'), 'utf8'));
    const rows = parseInventoryRows(md);

    // F1a. Row count === distinct submodule count (P0-4 correctness)
    assert('F1a. inventory row count === distinct submodule count (3 submodules)',
      rows.length === 3, `got rows=${rows.length}, names=${rows.map(r => r.submodule).join(',')}`);

    // F1b. Sum(Files column) === topology.summary.files (P0-4 correctness)
    const filesSum = rows.reduce((s, r) => s + r.files, 0);
    assert('F1b. sum(Files column) === topology.summary.files',
      filesSum === topology.summary.files,
      `sum=${filesSum} vs summary.files=${topology.summary.files}`);

    // F1c. Sum === Object.keys(topology.nodes).length (alternative equivalence)
    assert('F1c. sum(Files column) === Object.keys(topology.nodes).length',
      filesSum === Object.keys(topology.nodes).length);

    // F1d. All 3 expected submodules present
    const names = new Set(rows.map((r) => r.submodule));
    assert('F1d. all 3 submodules (_lib, src, tests) present',
      names.has('`_lib`') && names.has('`src`') && names.has('`tests`'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F2. SCC fixture — 3 files in a cycle ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-scc-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-scc-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'scc-fx', type: 'module' }));
    // Intentional cycle: a → b → c → a
    write(fx, 'core/a.mjs',
      `import { b } from './b.mjs';\n` +
      `export function a() { return b() + 1 }\n`);
    write(fx, 'core/b.mjs',
      `import { c } from './c.mjs';\n` +
      `export function b() { return c() + 1 }\n`);
    write(fx, 'core/c.mjs',
      `import { a } from './a.mjs';\n` +
      `export function c() { return 1 + (Math.random() > 2 ? a() : 0) }\n`);
    runProducersAndCanon(fx, out);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('F2a. §3 shows cycle with forbidden-cycle label',
      md.includes('forbidden-cycle'));
    assert('F2b. SCC members listed',
      md.includes('`core/a.mjs`') &&
      md.includes('`core/b.mjs`') &&
      md.includes('`core/c.mjs`'));
    assert('F2c. § header shows "❌ Cycles observed"',
      md.includes('❌ Cycles observed'));
    // Parent submodule `core` should carry cyclic-submodule
    const rows = parseInventoryRows(md);
    const core = rows.find((r) => r.submodule === '`core`');
    assert('F2d. core submodule row carries cyclic-submodule',
      core && core.status.includes('cyclic-submodule'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F3. Oversize fixture — 1200-LOC file → extreme-oversize + 500-LOC file → oversize ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-over-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-over-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'over-fx', type: 'module' }));
    // Generate a 1200-line file and a 500-line file with real statements
    // (not comments — parser strips some comment lines from LOC in some tools).
    const bigLines = Array.from({ length: 1200 }, (_, i) => `export const x${i} = ${i};`).join('\n') + '\n';
    const midLines = Array.from({ length: 500 }, (_, i) => `export const y${i} = ${i};`).join('\n') + '\n';
    write(fx, 'huge/h.mjs', bigLines);
    write(fx, 'mid/m.mjs', midLines);
    runProducersAndCanon(fx, out);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('F3a. §4 oversize section present',
      md.includes('## 4. Oversize files'));
    assert('F3b. huge/h.mjs → extreme-oversize',
      md.includes('`huge/h.mjs`') && md.includes('extreme-oversize'));
    assert('F3c. mid/m.mjs → oversize (not extreme)',
      md.includes('`mid/m.mjs`') &&
      md.match(/`mid\/m\.mjs`[^\n]*\soversize\s/));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F4. Acyclic fixture — ✅ banner rendered explicitly ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-acyc-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-acyc-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'acyc-fx', type: 'module' }));
    write(fx, '_lib/util.mjs', `export const x = 1;\n`);
    write(fx, 'src/main.mjs', `import { x } from '../_lib/util.mjs'; export const y = x;\n`);
    runProducersAndCanon(fx, out);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('F4. acyclic fixture → ✅ banner explicitly rendered (not silently empty)',
      md.includes('✅ No submodule-level cycles observed'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F5. topology.json absent → exit 2 (hard dependency) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-notopo-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-notopo-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'notopo-fx', type: 'module' }));
    write(fx, 'src/x.mjs', `export const x = 1;\n`);
    // Intentionally no measure-topology run
    const res = spawnSync(NODE, [CANON_CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('F5a. topology.json absent → exit 2',
      res.status === 2, `got=${res.status}`);
    assert('F5b. stderr points at measure-topology.mjs',
      /measure-topology\.mjs/.test(res.stderr));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F6. triage.json absent → §5 omitted, exit 0 ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-notriage-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-notriage-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'notriage-fx', type: 'module' }));
    write(fx, 'src/x.mjs', `export const x = 1;\n`);
    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    // Intentionally NO triage run
    const res = spawnSync(NODE, [CANON_CLI,
      '--root', fx, '--output', out, '--source', 'topology',
    ], { encoding: 'utf8' });
    assert('F6a. exit 0 with triage.json absent',
      res.status === 0, `stderr=${res.stderr.slice(0, 300)}`);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    assert('F6b. §5 Workspace boundaries section NOT present',
      !md.includes('## 5. Workspace boundaries'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F7. Fixture-controlled round-trip — every emitted status is in canonical §11 set ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdt-i-rt-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdt-i-rt-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'rt-fx', type: 'module' }));
    write(fx, 'hub/a.mjs', `export const a = 1;\n`);
    write(fx, 'con1/c.mjs', `import { a } from '../hub/a.mjs'; export const c = a;\n`);
    write(fx, 'con2/c.mjs', `import { a } from '../hub/a.mjs'; export const c = a;\n`);
    write(fx, 'con3/c.mjs', `import { a } from '../hub/a.mjs'; export const c = a;\n`);
    write(fx, 'con4/c.mjs', `import { a } from '../hub/a.mjs'; export const c = a;\n`);
    write(fx, 'con5/c.mjs', `import { a } from '../hub/a.mjs'; export const c = a;\n`);
    runProducersAndCanon(fx, out);
    const md = readFileSync(path.join(fx, 'canonical-draft', 'topology.md'), 'utf8');
    const rows = parseInventoryRows(md);

    const CANONICAL_TOPOLOGY_LABELS = new Set([
      'cyclic-submodule', 'isolated-submodule', 'shared-submodule',
      'leaf-submodule', 'scoped-submodule',
    ]);
    const allCanonical = rows.every((r) => {
      const firstToken = r.status.split(' ')[0];
      return CANONICAL_TOPOLOGY_LABELS.has(firstToken);
    });
    assert('F7a. every submodule row status is a canonical §11.1 label',
      allCanonical, `rows=${JSON.stringify(rows.map((r) => r.status))}`);

    // hub has fanIn=5 → shared-submodule
    const hub = rows.find((r) => r.submodule === '`hub`');
    assert('F7b. hub (inDegree=5) → shared-submodule ✅',
      hub && hub.status.includes('shared-submodule'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F8. Path with spaces + $ — end-to-end shell safety ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'cdt-i-shell-'));
  const fx = path.join(parent, 'my $root');
  const out = path.join(parent, 'my $out');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shell-fx', type: 'module' }));
    write(fx, 'src/a.mjs', `export const a = 1;\n`);
    runProducersAndCanon(fx, out);
    assert('F8. fixture under `my $root/` produces draft via full pipeline',
      existsSync(path.join(fx, 'canonical-draft', 'topology.md')));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
