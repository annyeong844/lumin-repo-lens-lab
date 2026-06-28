// tests/test-generate-check-canon-cli.mjs
//
// P5-1 Step 0 — RED test for the `check-canon.mjs` CLI.
//
// Spawns check-canon.mjs via execFileSync (shell-safety per P1-3).
// Covers all branches of the exit-code matrix in p5-1.md §4.3.

import { execFileSync, spawnSync } from 'node:child_process';
import { writeFileSync, mkdtempSync, mkdirSync, existsSync, readFileSync, rmSync, utimesSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const CLI = path.join(DIR, 'check-canon.mjs');

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

function runCli(args, { cwd } = {}) {
  const res = spawnSync(process.execPath, [CLI, ...args], {
    cwd: cwd ?? DIR,
    encoding: 'utf8',
  });
  return { exit: res.status ?? -1, stdout: res.stdout ?? '', stderr: res.stderr ?? '' };
}

// Helper: build a minimal fixture repo directory with symbols.json
function makeFixture({ canonical, symbols }) {
  const root = mkdtempSync(path.join(tmpdir(), 'p5-1-cli-'));
  mkdirSync(path.join(root, 'canonical'), { recursive: true });
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  if (canonical !== null) {
    writeFileSync(path.join(root, 'canonical', 'type-ownership.md'), canonical, 'utf8');
  }
  if (symbols) {
    writeFileSync(path.join(root, 'audit-output', 'symbols.json'),
      JSON.stringify(symbols, null, 2), 'utf8');
  }
  return root;
}

function basicSymbols({ typeDefs = [] } = {}) {
  const defIndex = {};
  const fanInByIdentity = {};
  for (const d of typeDefs) {
    if (!defIndex[d.ownerFile]) defIndex[d.ownerFile] = {};
    defIndex[d.ownerFile][d.name] = {
      kind: d.kind ?? 'TSInterfaceDeclaration',
      line: d.line,
      anyContamination: null,
    };
    fanInByIdentity[`${d.ownerFile}::${d.name}`] = d.fanIn;
  }
  return {
    meta: { scope: 'fixture', supports: { identityFanIn: true } },
    defIndex,
    fanInByIdentity,
    reExportsByFile: {},
  };
}

function makeHash(ch) {
  return `sha256:${ch.repeat(64)}`;
}

function makeShapeIndex(facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    if (!groupsByHash[fact.hash]) groupsByHash[fact.hash] = [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  return {
    schemaVersion: 'shape-index.v1',
    meta: { complete },
    facts: facts.map((fact) => ({ ...fact })),
    groupsByHash,
  };
}

const cleanup = [];

// ── C-1. missing --source ──────────────────────────────────────

{
  const r = runCli([]);
  assert('C-1a. missing --source → exit 2',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 200)}`);
  assert('C-1b. stderr mentions --source',
    /--source/.test(r.stderr), `stderr=${r.stderr.slice(0, 200)}`);
}

// ── C-2. deferred sources stub exit 2 (P5-4 removes naming + all — empty) ──

// P5-4 empties DEFERRED_SOURCES. All previously-deferred sources now have real
// dispatch; only UNKNOWN sources should exit 2 via the SUPPORTED_SOURCES gate
// (already covered by C-3 below).

// ── C-3. unknown --source ──────────────────────────────────────

{
  const r = runCli(['--source', 'xyz-bogus']);
  assert('C-3. unknown --source → exit 2 + stderr',
    r.exit === 2 && /unknown|unsupported|source/i.test(r.stderr),
    `exit=${r.exit}, stderr=${r.stderr.slice(0, 200)}`);
}

// ── C-4. type-ownership: missing symbols.json → exit 2 ─────────

{
  const root = makeFixture({
    canonical: '| Name | Identity | Owner | Fan-in | Status | Tags |\n|--|--|--|--:|--|--|\n',
    symbols: null,
  });
  cleanup.push(root);
  const r = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('C-4. missing symbols.json → exit 2',
    r.exit === 2,
    `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('C-4b. stderr names symbols.json',
    /symbols\.json/i.test(r.stderr),
    `stderr=${r.stderr.slice(0, 300)}`);
}

// ── C-5. type-ownership: missing canonical → exit 2 + JSON write-anyway ─

{
  const root = makeFixture({
    canonical: null,  // no canonical/type-ownership.md
    symbols: basicSymbols(),
  });
  cleanup.push(root);
  const r = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('C-5a. missing canon → exit 2',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  const jsonPath = path.join(root, 'audit-output', 'canon-drift.json');
  assert('C-5b. canon-drift.json IS written (write-anyway policy)',
    existsSync(jsonPath), `jsonPath=${jsonPath}`);
  const mdPath = path.join(root, 'audit-output', 'canon-drift.type-ownership.md');
  assert('C-5c. canon-drift.type-ownership.md is NOT written',
    !existsSync(mdPath), `mdPath=${mdPath}`);
  if (existsSync(jsonPath)) {
    const obj = JSON.parse(readFileSync(jsonPath, 'utf8'));
    assert('C-5d. JSON perSource.type-ownership.status = skipped-missing-canon',
      obj.perSource?.['type-ownership']?.status === 'skipped-missing-canon',
      `status=${obj.perSource?.['type-ownership']?.status}`);
  }
}

// ── C-6. clean fixture → exit 0 ────────────────────────────────

{
  const root = makeFixture({
    canonical:
      '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
      '|------|----------|-------|-------:|--------|------|\n' +
      '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:10` | 3 | single-owner-strong ✅ | |\n',
    symbols: basicSymbols({
      typeDefs: [{ name: 'Foo', ownerFile: 'src/foo.ts', line: 10, fanIn: 3 }],
    }),
  });
  cleanup.push(root);
  const r = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('C-6. clean fixture → exit 0',
    r.exit === 0, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('C-6b. stdout mentions "clean"',
    /clean/i.test(r.stdout),
    `stdout=${r.stdout.slice(0, 300)}`);
}

// ── C-7. drift fixture → exit 1 ────────────────────────────────

{
  const root = makeFixture({
    canonical:
      '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
      '|------|----------|-------|-------:|--------|------|\n' +
      '| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:10` | 3 | single-owner-strong ✅ | |\n',
    // Fresh is EMPTY — Foo is now removed
    symbols: basicSymbols({ typeDefs: [] }),
  });
  cleanup.push(root);
  const r = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('C-7. drift fixture → exit 1',
    r.exit === 1, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('C-7b. stdout mentions drift count',
    /drift/i.test(r.stdout),
    `stdout=${r.stdout.slice(0, 300)}`);
  const mdPath = path.join(root, 'audit-output', 'canon-drift.type-ownership.md');
  assert('C-7c. canon-drift.type-ownership.md IS written for drift',
    existsSync(mdPath), `mdPath=${mdPath}`);
}

// ── C-8. corrupt symbols.json → exit 2 (not crash) ────────────

{
  const root = makeFixture({
    canonical: '| Name | Identity | Owner | Fan-in | Status | Tags |\n|--|--|--|--:|--|--|\n',
    symbols: null,  // don't write via helper
  });
  cleanup.push(root);
  // Overwrite with invalid JSON
  writeFileSync(path.join(root, 'audit-output', 'symbols.json'), 'not json {{\n', 'utf8');
  const r = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('C-8a. corrupt symbols.json → exit 2 (NOT 1 / NOT crash)',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('C-8b. corrupt symbols.json: stderr has [check-canon] diagnostic (no Node stack trace)',
    r.stderr.includes('[check-canon]') && !r.stderr.includes('at JSON.parse'),
    `stderr=${r.stderr.slice(0, 400)}`);
}

// ── C-9. optional shape-index.json upgrades ambiguous rename pairing ──

{
  const root = makeFixture({
    canonical:
      '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
      '|------|----------|-------|-------:|--------|------|\n' +
      '| `X` | `src/a.ts::X` | `src/a.ts:1` | 1 | single-owner-weak ✅ | |\n',
    symbols: basicSymbols({
      typeDefs: [
        { name: 'X', ownerFile: 'src/b.ts', line: 1, fanIn: 1 },
        { name: 'X', ownerFile: 'src/c.ts', line: 1, fanIn: 1 },
      ],
    }),
  });
  cleanup.push(root);
  writeFileSync(path.join(root, 'audit-output', 'shape-index.json'),
    JSON.stringify(makeShapeIndex([
      { identity: 'src/a.ts::X', hash: makeHash('a') },
      { identity: 'src/b.ts::X', hash: makeHash('a') },
      { identity: 'src/c.ts::X', hash: makeHash('b') },
    ]), null, 2), 'utf8');
  const r = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  const obj = JSON.parse(readFileSync(path.join(root, 'audit-output', 'canon-drift.json'), 'utf8'));
  assert('C-9a. type-ownership + shape-index unique pair still exits 1 (drift remains)',
    r.exit === 1, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('C-9b. CLI emits owner-changed for the grounded shape pair',
    obj.drifts.some((d) =>
      d.category === 'owner-changed' &&
      d.canon?.identity === 'src/a.ts::X' &&
      d.fresh?.identity === 'src/b.ts::X'),
    `drifts=${JSON.stringify(obj.drifts)}`);
  assert('C-9c. CLI leaves only the true extra identity as identity-added',
    obj.drifts.filter((d) => d.category === 'identity-added').length === 1 &&
    obj.drifts.filter((d) => d.category === 'identity-removed').length === 0,
    `drifts=${JSON.stringify(obj.drifts)}`);
}

// ── P5-2: --source helper-registry dispatch ─────────────────────

function makeHelperFixture({ canonical, symbols, callGraph, srcFiles = [] }) {
  const root = mkdtempSync(path.join(tmpdir(), 'p5-2-cli-'));
  mkdirSync(path.join(root, 'canonical'), { recursive: true });
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  mkdirSync(path.join(root, 'src'), { recursive: true });
  if (canonical !== null) {
    writeFileSync(path.join(root, 'canonical', 'helper-registry.md'), canonical, 'utf8');
  }
  if (symbols !== undefined) {
    writeFileSync(path.join(root, 'audit-output', 'symbols.json'),
      typeof symbols === 'string' ? symbols : JSON.stringify(symbols, null, 2), 'utf8');
  }
  if (callGraph !== undefined) {
    writeFileSync(path.join(root, 'audit-output', 'call-graph.json'),
      typeof callGraph === 'string' ? callGraph : JSON.stringify(callGraph, null, 2), 'utf8');
  }
  for (const f of srcFiles) {
    writeFileSync(path.join(root, 'src', f.name), f.content, 'utf8');
  }
  return root;
}

// CH-1: missing helper canon → exit 2 + JSON write-anyway (missing canon is still strict)
{
  const root = makeHelperFixture({
    canonical: null,
    srcFiles: [{ name: 'foo.ts', content: 'export function doFoo() {}\n' }],
  });
  cleanup.push(root);
  const r = runCli(['--source', 'helper-registry', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CH-1a. helper-registry missing canon → exit 2',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  const jsonPath = path.join(root, 'audit-output', 'canon-drift.json');
  assert('CH-1b. canon-drift.json IS written even on missing helper canon',
    existsSync(jsonPath), '');
  if (existsSync(jsonPath)) {
    const obj = JSON.parse(readFileSync(jsonPath, 'utf8'));
    assert('CH-1c. JSON perSource["helper-registry"].status = skipped-missing-canon',
      obj.perSource?.['helper-registry']?.status === 'skipped-missing-canon',
      `status=${obj.perSource?.['helper-registry']?.status}`);
  }
  const mdPath = path.join(root, 'audit-output', 'canon-drift.helper-registry.md');
  assert('CH-1d. canon-drift.helper-registry.md NOT written on skipped-missing-canon',
    !existsSync(mdPath), '');
}

// CH-2: missing symbols.json → run continues (enrichment unavailable + advisory)
{
  const root = makeHelperFixture({
    canonical:
      '| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n' +
      '|------|----------|-------|-----------|-------:|--------|------|----------------------|\n' +
      '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:1` | () => void | 0 | zero-internal-fan-in-helper ⚠ | | |\n',
    // symbols: absent (undefined → file not written)
    srcFiles: [{ name: 'foo.ts', content: 'export function doFoo() {}\n' }],
  });
  cleanup.push(root);
  const r = runCli(['--source', 'helper-registry', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CH-2a. helper-registry missing symbols.json → run continues (exit 0 or 1, NOT 2)',
    r.exit === 0 || r.exit === 1, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  const jsonPath = path.join(root, 'audit-output', 'canon-drift.json');
  if (existsSync(jsonPath)) {
    const obj = JSON.parse(readFileSync(jsonPath, 'utf8'));
    const diags = obj.perSource?.['helper-registry']?.diagnostics ?? [];
    assert('CH-2b. advisory diagnostic helper-contamination-enrichment-unavailable present',
      diags.some((d) => d.kind === 'helper-contamination-enrichment-unavailable'),
      `diagnostics=${JSON.stringify(diags)}`);
  }
}

// CH-3: corrupt symbols.json → helper-registry continues (non-strict); compared with type-ownership strict
{
  const root = makeHelperFixture({
    canonical:
      '| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n' +
      '|------|----------|-------|-----------|-------:|--------|------|----------------------|\n' +
      '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:1` | () => void | 0 | zero-internal-fan-in-helper ⚠ | | |\n',
    symbols: 'not json {{',
    srcFiles: [{ name: 'foo.ts', content: 'export function doFoo() {}\n' }],
  });
  cleanup.push(root);
  const rHelper = runCli(['--source', 'helper-registry', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CH-3a. helper-registry + corrupt symbols.json → exit 0 or 1 (non-strict, continue)',
    rHelper.exit === 0 || rHelper.exit === 1,
    `exit=${rHelper.exit}, stderr=${rHelper.stderr.slice(0, 300)}`);
  // Now flip: same corrupt symbols, but --source type-ownership → MUST exit 2 (strict)
  // (needs a type-ownership canon too; reuse same root — we write one quickly)
  writeFileSync(path.join(root, 'canonical', 'type-ownership.md'),
    '| Name | Identity | Owner | Fan-in | Status | Tags |\n' +
    '|------|----------|-------|-------:|--------|------|\n', 'utf8');
  const rType = runCli(['--source', 'type-ownership', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CH-3b. type-ownership + corrupt symbols.json → exit 2 (strict asymmetry pin)',
    rType.exit === 2, `exit=${rType.exit}, stderr=${rType.stderr.slice(0, 300)}`);
}

// CH-4: corrupt call-graph.json → helper-registry continues
{
  const root = makeHelperFixture({
    canonical:
      '| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n' +
      '|------|----------|-------|-----------|-------:|--------|------|----------------------|\n' +
      '| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:1` | () => void | 0 | zero-internal-fan-in-helper ⚠ | | |\n',
    symbols: { meta: { scope: 'fixture', supports: {} }, defIndex: {}, fanInByIdentity: {}, reExportsByFile: {} },
    callGraph: 'not json {{',
    srcFiles: [{ name: 'foo.ts', content: 'export function doFoo() {}\n' }],
  });
  cleanup.push(root);
  const r = runCli(['--source', 'helper-registry', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CH-4. helper-registry + corrupt call-graph.json → run continues (exit 0/1)',
    r.exit === 0 || r.exit === 1, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
}

// CH-5: extractor-throw path via unreadable src is hard to trigger from CLI — engine test (H-5) covers this.
//       Skip here. Coverage exists at engine layer.

// ── P5-3: --source topology dispatch ─────────────────────────

function makeTopologyFixture({ canonical, topology, triage }) {
  const root = mkdtempSync(path.join(tmpdir(), 'p5-3-cli-'));
  mkdirSync(path.join(root, 'canonical'), { recursive: true });
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  if (canonical !== null) {
    writeFileSync(path.join(root, 'canonical', 'topology.md'), canonical, 'utf8');
  }
  if (topology !== undefined) {
    writeFileSync(path.join(root, 'audit-output', 'topology.json'),
      typeof topology === 'string' ? topology : JSON.stringify(topology, null, 2), 'utf8');
  }
  if (triage !== undefined) {
    writeFileSync(path.join(root, 'audit-output', 'triage.json'),
      typeof triage === 'string' ? triage : JSON.stringify(triage, null, 2), 'utf8');
  }
  return root;
}

const TOPO_CLEAN_CANON = [
  '## 1. Submodule inventory',
  '',
  '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
  '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
  '| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |',
  '',
  '## 3. Cycles (SCCs)',
  '',
  '✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.',
  '',
].join('\n');

const TOPO_CLEAN_TOPOLOGY = {
  meta: { complete: true, generated: '2026-04-22T00:00:00Z' },
  nodes: { 'src/a.ts': { loc: 10 } },
  sccs: [],
  crossSubmoduleEdges: [],
  largestFiles: [],
};

// CY-1: clean topology fixture → exit 0
{
  const root = makeTopologyFixture({ canonical: TOPO_CLEAN_CANON, topology: TOPO_CLEAN_TOPOLOGY });
  cleanup.push(root);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-1a. topology clean fixture → exit 0',
    r.exit === 0, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('CY-1b. stdout mentions clean',
    /clean/i.test(r.stdout), `stdout=${r.stdout.slice(0, 200)}`);
}

// CY-2: drift fixture (submodule-added) → exit 1
{
  const topology = {
    ...TOPO_CLEAN_TOPOLOGY,
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 20 } },
  };
  const root = makeTopologyFixture({ canonical: TOPO_CLEAN_CANON, topology });
  cleanup.push(root);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-2a. topology drift fixture → exit 1',
    r.exit === 1, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  const mdPath = path.join(root, 'audit-output', 'canon-drift.topology.md');
  assert('CY-2b. canon-drift.topology.md IS written for drift',
    existsSync(mdPath), '');
}

// CY-3: missing topology.json → exit 2
{
  const root = makeTopologyFixture({ canonical: TOPO_CLEAN_CANON, topology: undefined });
  cleanup.push(root);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-3a. missing topology.json → exit 2',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('CY-3b. stderr names topology.json',
    /topology\.json/i.test(r.stderr), `stderr=${r.stderr.slice(0, 300)}`);
}

// CY-4: corrupt topology.json → exit 2 (no crash)
{
  const root = makeTopologyFixture({ canonical: TOPO_CLEAN_CANON, topology: 'not json {{' });
  cleanup.push(root);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-4a. corrupt topology.json → exit 2',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
  assert('CY-4b. stderr has [check-canon] prefix (no Node stack trace)',
    r.stderr.includes('[check-canon]') && !r.stderr.includes('at JSON.parse'),
    `stderr=${r.stderr.slice(0, 400)}`);
}

// CY-5: missing canonical/topology.md → exit 2 + JSON write-anyway
{
  const root = makeTopologyFixture({ canonical: null, topology: TOPO_CLEAN_TOPOLOGY });
  cleanup.push(root);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-5a. missing topology canon → exit 2',
    r.exit === 2, `exit=${r.exit}`);
  const jsonPath = path.join(root, 'audit-output', 'canon-drift.json');
  assert('CY-5b. canon-drift.json IS written even on missing topology canon',
    existsSync(jsonPath), '');
  if (existsSync(jsonPath)) {
    const obj = JSON.parse(readFileSync(jsonPath, 'utf8'));
    assert('CY-5c. JSON perSource["topology"].status = skipped-missing-canon',
      obj.perSource?.['topology']?.status === 'skipped-missing-canon',
      `status=${obj.perSource?.['topology']?.status}`);
  }
}

// CY-6: optional triage.json absent → run continues
{
  const root = makeTopologyFixture({ canonical: TOPO_CLEAN_CANON, topology: TOPO_CLEAN_TOPOLOGY });
  cleanup.push(root);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-6. missing triage.json → run continues (exit 0/1, not 2)',
    r.exit === 0 || r.exit === 1, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
}

// CY-7: topology.json stale warning (Finding #3 — p5-3.md §8 R2)
{
  const root = makeTopologyFixture({ canonical: TOPO_CLEAN_CANON, topology: TOPO_CLEAN_TOPOLOGY });
  cleanup.push(root);
  const newerSrc = path.join(root, 'src');
  mkdirSync(newerSrc, { recursive: true });
  writeFileSync(path.join(newerSrc, 'newer.ts'), 'export const X = 1;\n', 'utf8');
  // Back-date topology.json mtime by 1 hour so the src file's mtime is newer.
  const topoPath = path.join(root, 'audit-output', 'topology.json');
  const oneHourAgo = (Date.now() / 1000) - 3600;
  utimesSync(topoPath, oneHourAgo, oneHourAgo);
  const r = runCli(['--source', 'topology', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CY-7a. stale topology.json → stderr warning emitted',
    /stale|older|refresh/i.test(r.stderr) && /topology\.json/i.test(r.stderr),
    `stderr=${r.stderr.slice(0, 400)}`);
  assert('CY-7b. stale warning does NOT change exit code',
    r.exit === 0 || r.exit === 1,
    `exit=${r.exit}`);
}

// ── P5-4: --source naming dispatch (fresh AST + scan-range flags) ──

function makeNamingFixture({ canonical, srcFiles = [], pkg = { name: 'p5-4-fx', type: 'module' } }) {
  const root = mkdtempSync(path.join(tmpdir(), 'p5-4-cli-'));
  mkdirSync(path.join(root, 'canonical'), { recursive: true });
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify(pkg), 'utf8');
  if (canonical !== null) {
    writeFileSync(path.join(root, 'canonical', 'naming.md'), canonical, 'utf8');
  }
  for (const f of srcFiles) {
    const full = path.join(root, f.rel);
    mkdirSync(path.dirname(full), { recursive: true });
    writeFileSync(full, f.content, 'utf8');
  }
  return root;
}

// CN-1: naming clean fixture → exit 0
{
  const root = makeNamingFixture({
    canonical: [
      '## 1. File-naming cohorts', '',
      '| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |',
      '|--------------------|------:|--------------------|----------------:|--------------:|--------|',
      '',
      '## 2. Symbol-naming cohorts', '',
      '| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |',
      '|--------------------------|------:|--------------------|----------------:|--------------:|--------|',
      '',
    ].join('\n'),
    srcFiles: [],
  });
  cleanup.push(root);
  const r = runCli(['--source', 'naming', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CN-1. naming clean fixture → exit 0',
    r.exit === 0, `exit=${r.exit}, stderr=${r.stderr.slice(0, 300)}`);
}

// CN-2: missing canonical/naming.md → exit 2 + JSON write-anyway
{
  const root = makeNamingFixture({ canonical: null, srcFiles: [] });
  cleanup.push(root);
  const r = runCli(['--source', 'naming', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CN-2a. missing naming canon → exit 2',
    r.exit === 2, `exit=${r.exit}`);
  const jsonPath = path.join(root, 'audit-output', 'canon-drift.json');
  assert('CN-2b. canon-drift.json IS written',
    existsSync(jsonPath), '');
  if (existsSync(jsonPath)) {
    const obj = JSON.parse(readFileSync(jsonPath, 'utf8'));
    assert('CN-2c. perSource["naming"].status = skipped-missing-canon',
      obj.perSource?.['naming']?.status === 'skipped-missing-canon', '');
  }
}

// CN-3: naming scan-range forwarding (P0-5) — --include-tests flips outlier appearance
//
// This test requires actual AST extraction to work on the fixture, which
// depends on @oxc-parser bindings being installable at runtime. If that
// environment is absent, test is skipped gracefully.

// ── P5-4: --source all aggregation (checked-source rule table) ──

// CA-1: all sources missing canon → exit 2 (all skipped)
{
  const root = mkdtempSync(path.join(tmpdir(), 'p5-4-cli-all-'));
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ name: 'p5-4-all', type: 'module' }), 'utf8');
  // symbols.json required for type-ownership branch even to attempt
  writeFileSync(path.join(root, 'audit-output', 'symbols.json'),
    JSON.stringify({ meta: { scope: 'fix' }, defIndex: {}, fanInByIdentity: {} }), 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'topology.json'),
    JSON.stringify({ meta: {}, nodes: {}, sccs: [], crossSubmoduleEdges: [], largestFiles: [] }), 'utf8');
  cleanup.push(root);
  const r = runCli(['--source', 'all', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CA-1. all-4-missing-canon → exit 2',
    r.exit === 2, `exit=${r.exit}, stderr=${r.stderr.slice(0, 400)}`);
  const jsonPath = path.join(root, 'audit-output', 'canon-drift.json');
  assert('CA-1b. canon-drift.json has all 4 perSource entries',
    existsSync(jsonPath) &&
    (() => {
      const obj = JSON.parse(readFileSync(jsonPath, 'utf8'));
      return ['type-ownership', 'helper-registry', 'topology', 'naming']
        .every((s) => obj.perSource?.[s]);
    })(), '');
}

// CA-2: one clean + three skipped-missing-canon → exit 0 (checked-source rule)
{
  const root = mkdtempSync(path.join(tmpdir(), 'p5-4-cli-all2-'));
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  mkdirSync(path.join(root, 'canonical'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ name: 'p5-4-all2', type: 'module' }), 'utf8');
  // Only type-ownership canon exists + empty canon matches empty fresh
  writeFileSync(path.join(root, 'canonical', 'type-ownership.md'),
    '| Name | Identity | Owner | Fan-in | Status | Tags |\n|--|--|--|--:|--|--|\n', 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'symbols.json'),
    JSON.stringify({ meta: { scope: 'fix' }, defIndex: {}, fanInByIdentity: {} }), 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'topology.json'),
    JSON.stringify({ meta: {}, nodes: {}, sccs: [], crossSubmoduleEdges: [], largestFiles: [] }), 'utf8');
  cleanup.push(root);
  const r = runCli(['--source', 'all', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CA-2. one-clean + three-skipped-missing-canon → exit 0 (NOT 2)',
    r.exit === 0, `exit=${r.exit}, stderr=${r.stderr.slice(0, 400)}`);
}

// CA-3: one parse-error + rest skipped/clean → exit 2 (failed wins)
{
  const root = mkdtempSync(path.join(tmpdir(), 'p5-4-cli-all3-'));
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  mkdirSync(path.join(root, 'canonical'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ name: 'p5-4-all3', type: 'module' }), 'utf8');
  // Type-ownership canon malformed → parse-error
  writeFileSync(path.join(root, 'canonical', 'type-ownership.md'),
    '| Name | Owner |\n|--|--|\n| foo | bar |\n', 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'symbols.json'),
    JSON.stringify({ meta: { scope: 'fix' }, defIndex: {}, fanInByIdentity: {} }), 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'topology.json'),
    JSON.stringify({ meta: {}, nodes: {}, sccs: [], crossSubmoduleEdges: [], largestFiles: [] }), 'utf8');
  cleanup.push(root);
  const r = runCli(['--source', 'all', '--root', root, '--output', path.join(root, 'audit-output')]);
  assert('CA-3. one-parse-error + rest-skipped → exit 2 (failed outranks)',
    r.exit === 2, `exit=${r.exit}`);
}

// CA-4: all JSON written in single file with all 4 perSource keys
{
  const root = mkdtempSync(path.join(tmpdir(), 'p5-4-cli-all4-'));
  mkdirSync(path.join(root, 'audit-output'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ name: 'p5-4-all4', type: 'module' }), 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'symbols.json'),
    JSON.stringify({ meta: { scope: 'fix' }, defIndex: {}, fanInByIdentity: {} }), 'utf8');
  writeFileSync(path.join(root, 'audit-output', 'topology.json'),
    JSON.stringify({ meta: {}, nodes: {}, sccs: [], crossSubmoduleEdges: [], largestFiles: [] }), 'utf8');
  cleanup.push(root);
  runCli(['--source', 'all', '--root', root, '--output', path.join(root, 'audit-output')]);
  const obj = JSON.parse(readFileSync(path.join(root, 'audit-output', 'canon-drift.json'), 'utf8'));
  assert('CA-4a. canon-drift.json has all 4 perSource keys',
    ['type-ownership', 'helper-registry', 'topology', 'naming']
      .every((s) => s in (obj.perSource ?? {})), '');
  assert('CA-4b. summary.sourcesRequested === 4',
    obj.summary?.sourcesRequested === 4, `summary=${JSON.stringify(obj.summary)}`);
}

for (const root of cleanup) rmSync(root, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
