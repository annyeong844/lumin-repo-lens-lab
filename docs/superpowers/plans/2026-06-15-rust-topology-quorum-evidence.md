# Rust Topology Quorum Evidence Implementation Plan

> **For agentic workers:** Implement task-by-task. Keep tests behavior-focused: one minimum guaranteed happy path, realistic edge cases, and hard stops. Scaffolding-only checks are not test value.

**Goal:** Build the M4 lab-only quorum evidence collector that records Rust topology compare evidence without enabling `prefer`.

**Architecture:** Keep JS topology output authoritative. Update the M3 gate so clean-run eligibility uses the M4 quorum field names and dirty-source diagnostics. Add `_lib/rust-topology-quorum.mjs` for quorum validation, run-record construction, append-only updates, summary rendering, and injected runner orchestration. Add `scripts/record-rust-topology-quorum.mjs` as a thin lab-only CLI.

**Tech Stack:** Node.js ESM, Vitest, existing `atomicWrite`, existing artifact JSON helpers, existing Rust topology compare metadata.

---

## Review Amendments Locked Into This Plan

- First quorum creation must work: missing `baselines/rust-topology-prefer-quorum.json` returns `null`, then `defaultEvidence()` creates the file.
- `--all-required` is batch orchestration, not a hidden mode inside a single-corpus recorder.
- The CLI must pass a real `measure-topology.mjs` runner, not rely on an injected test runner.
- Source cleanliness is never assumed clean. Missing source diagnostics are non-clean.
- Dirty-source guard is strict: no `collector` object means the run is not clean evidence.
- Markdown summaries use a text writer, not JSON stringification.
- M3 gate verification must run for real before the summary claims `eligible`.
- Run records must match top-level quorum `rustSidecarSourceCommit` and `policyVersion` before append.

## Rules For This Plan

- Use behavior-first tests only.
- Create importable code before tests that import it; do not test for mere file, function, or module existence.
- Tests must prove product behavior:
  - minimum happy path
  - realistic edge case
  - hard stop / no-evidence path
- `prefer` stays disabled.
- Private CI stays unused.

## File Structure

- Modify: `_lib/rust-topology-prefer-gate.mjs`
  - Replace `wrapperElapsedMs` required field with `commandWallElapsedMs` and `scannerBridgeElapsedMs`.
  - Reject dirty-source diagnostic runs in `cleanRunMatches()`.
- Modify: `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer-gate.mjs`
  - Mirror the root gate module.
- Modify: `tests/rust-topology-prefer-gate.test.mjs`
  - Update realistic clean-run fixture.
  - Add dirty-source non-clean evidence coverage.
- Create: `_lib/rust-topology-quorum.mjs`
  - Core collector functions.
- Create: `scripts/record-rust-topology-quorum.mjs`
  - Lab-only CLI wrapper.
- Create: `tests/rust-topology-quorum.test.mjs`
  - Behavior checks for quorum collector core.
- Modify: `tests/README.md`
  - Regenerate after adding the test.

## Task 1: Harden The M3 Gate For M4 Quorum Fields

**Files:**
- `_lib/rust-topology-prefer-gate.mjs`
- `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer-gate.mjs`
- `tests/rust-topology-prefer-gate.test.mjs`

- [ ] **Step 1: Update the root gate required fields**

In `_lib/rust-topology-prefer-gate.mjs`, change `REQUIRED_QUORUM_RUN_FIELDS` so it uses the M4 elapsed fields:

```js
const REQUIRED_QUORUM_RUN_FIELDS = [
  'labSourceCommit',
  'rustSidecarSourceCommit',
  'rustSidecarBinary',
  'command',
  'corpusRoot',
  'cacheMode',
  'fileCount',
  'filesCompared',
  'mismatches',
  'commandWallElapsedMs',
  'scannerBridgeElapsedMs',
  'sidecarElapsedMs',
  'sidecarStatus',
  'policyVersion',
  'machineOs',
];
```

- [ ] **Step 2: Add dirty-source clean evidence guard**

Add this helper above `cleanRunMatches()`:

```js
function hasCleanSourceDiagnostics(run) {
  const collector = run?.collector;
  if (!collector || typeof collector !== 'object') return false;
  return (
    collector.sourceDirty === false &&
    collector.workingTreeClean === true &&
    collector.labWorkingTreeClean === true &&
    collector.rustSidecarWorkingTreeClean === true
  );
}
```

Update `cleanRunMatches()`:

```js
function cleanRunMatches(run, rustSidecarSourceCommit, policyVersion) {
  return (
    hasRequiredRunFields(run) &&
    hasCleanSourceDiagnostics(run) &&
    run?.rustSidecarSourceCommit === rustSidecarSourceCommit &&
    run?.cacheMode === 'no-incremental' &&
    run?.mismatches === 0 &&
    run?.sidecarStatus === 'matched' &&
    run?.policyVersion === policyVersion
  );
}
```

- [ ] **Step 3: Mirror the same gate changes**

Apply the same functional changes to:

```text
skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer-gate.mjs
```

The root and skill gate modules should stay byte-identical.

- [ ] **Step 4: Update the existing gate test fixture**

In `tests/rust-topology-prefer-gate.test.mjs`, update `cleanRun()`:

```js
function cleanRun(corpus, index = 0) {
  return {
    labSourceCommit: `lab-${index}`,
    rustSidecarSourceCommit: '87116819c23d1e1adfbfca5def44552856e4f464',
    rustSidecarBinary: 'experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe',
    command: `node measure-topology.mjs --rust-topology-prefer-gate-corpus ${corpus}`,
    corpusRoot: `C:/corpora/${corpus}`,
    cacheMode: 'no-incremental',
    fileCount: 10 + index,
    filesCompared: 10 + index,
    mismatches: 0,
    commandWallElapsedMs: 1000 + index,
    scannerBridgeElapsedMs: 100 + index,
    sidecarElapsedMs: 10 + index,
    sidecarStatus: 'matched',
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    machineOs: 'Microsoft Windows NT 10.0.26200.0',
    collector: {
      workingTreeClean: true,
      sourceDirty: false,
      labWorkingTreeClean: true,
      rustSidecarWorkingTreeClean: true,
    },
  };
}
```

- [ ] **Step 5: Add the dirty-source behavior check**

Add this test:

```js
it('does not count dirty-source quorum runs as clean evidence', () => {
  const quorum = cleanQuorum();
  quorum.runs['nuxt-main'][2] = {
    ...quorum.runs['nuxt-main'][2],
    collector: {
      ...quorum.runs['nuxt-main'][2].collector,
      sourceDirty: true,
      workingTreeClean: false,
    },
  };

  const gate = evaluateRustTopologyPreferGate({
    mode: 'compare',
    currentCorpus: 'lab-self',
    rustTopologyScanner: matchedScanner(),
    quorumEvidence: quorum,
  });

  expect(gate.status).toBe('blocked-corpus-quorum');
  expect(gate.reason).toBe('required-corpus-history-incomplete');
  expect(gate.incompleteCorpora).toEqual(['nuxt-main']);
});
```

- [ ] **Step 6: Verify gate behavior**

Run:

```powershell
C:\nvm4w\nodejs\node.exe .\node_modules\vitest\vitest.mjs run tests\rust-topology-prefer-gate.test.mjs
```

Expected: all gate tests pass.

- [ ] **Step 7: Commit**

```powershell
git add _lib/rust-topology-prefer-gate.mjs skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer-gate.mjs tests/rust-topology-prefer-gate.test.mjs
git commit -m "Harden rust topology quorum clean-run gate"
```

## Task 2: Add Quorum Collector Core

**Files:**
- `_lib/rust-topology-quorum.mjs`
- `tests/rust-topology-quorum.test.mjs`

- [ ] **Step 1: Create the collector core module with real behavior**

Create `_lib/rust-topology-quorum.mjs`:

```js
import { existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';

import { atomicWrite } from './atomic-write.mjs';
import { readJsonFile } from './artifacts.mjs';
import { MODULE_EDGE_SCANNER_POLICY_VERSION } from './js-module-edge-scanner.mjs';
import {
  REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA,
  RUST_TOPOLOGY_PREFER_QUORUM_PATH,
} from './rust-topology-prefer-gate.mjs';

export const QUORUM_SCHEMA_VERSION = 1;
export const DEFAULT_M4_QUORUM_OUTPUT_ROOT =
  'C:/Users/endof/Downloads/lumin-perf-lab/baselines/m4-rust-topology-quorum';

const REQUIRED_RUN_FIELDS = [
  'labSourceCommit',
  'rustSidecarSourceCommit',
  'rustSidecarBinary',
  'command',
  'corpusRoot',
  'cacheMode',
  'fileCount',
  'filesCompared',
  'mismatches',
  'commandWallElapsedMs',
  'scannerBridgeElapsedMs',
  'sidecarElapsedMs',
  'sidecarStatus',
  'policyVersion',
  'machineOs',
  'recordedAt',
  'outputDir',
  'topologyJson',
  'collector',
];

function assertRequiredCorpus(corpus) {
  if (!REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.includes(corpus)) {
    throw new Error(`unknown required corpus: ${corpus}`);
  }
}

function slashPath(value) {
  return String(value ?? '').replaceAll('\\', '/');
}

export function parseCorpusRootEntry(entry) {
  const text = String(entry ?? '');
  const index = text.indexOf('=');
  if (index <= 0 || index === text.length - 1) {
    throw new Error(`--corpus-root must use name=path: ${text}`);
  }
  const corpus = text.slice(0, index);
  const root = text.slice(index + 1);
  assertRequiredCorpus(corpus);
  return [corpus, root];
}

function readRootsJson(filePath) {
  if (!filePath) return {};
  const parsed = readJsonFile(filePath, {
    tag: 'rust-topology-quorum-roots',
    strict: true,
  });
  const roots = parsed?.roots ?? parsed;
  if (!roots || typeof roots !== 'object' || Array.isArray(roots)) {
    throw new Error(`roots json must contain an object root map: ${filePath}`);
  }
  return roots;
}

export function normalizeRootMap({
  allRequired = false,
  corpus,
  root,
  corpusRoots = [],
  rootsJson,
} = {}) {
  const entries = new Map(Object.entries(readRootsJson(rootsJson)));
  for (const entry of corpusRoots) {
    const [name, value] = parseCorpusRootEntry(entry);
    entries.set(name, value);
  }
  if (corpus || root) {
    if (!corpus || !root) throw new Error('--corpus and --root must be provided together');
    assertRequiredCorpus(corpus);
    entries.set(corpus, root);
  }
  const wanted = allRequired ? REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA : [corpus].filter(Boolean);
  const result = {};
  for (const name of wanted) {
    const value = entries.get(name);
    if (!value) throw new Error(`missing required corpus roots: ${name}`);
    result[name] = value;
  }
  return result;
}

export function validateRunRecord(record) {
  for (const field of REQUIRED_RUN_FIELDS) {
    const value = record?.[field];
    if (value === undefined || value === null || value === '') {
      throw new Error(`run record missing required field: ${field}`);
    }
  }
  if (record.cacheMode !== 'no-incremental') {
    throw new Error(`quorum evidence requires no-incremental cache mode: ${record.cacheMode}`);
  }
  if (!record.collector || typeof record.collector !== 'object') {
    throw new Error('run record missing collector source diagnostics');
  }
  return record;
}

export function validateQuorumEvidence(evidence) {
  if (!evidence || typeof evidence !== 'object' || Array.isArray(evidence)) {
    throw new Error('quorum evidence must be an object');
  }
  if (evidence.schemaVersion !== QUORUM_SCHEMA_VERSION) {
    throw new Error(`unsupported quorum schemaVersion: ${evidence.schemaVersion}`);
  }
  const declared = new Set(evidence.requiredCorpora ?? []);
  const missing = REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.filter((corpus) => !declared.has(corpus));
  if (missing.length > 0) {
    throw new Error(`quorum evidence missing required corpora: ${missing.join(', ')}`);
  }
  if (!evidence.runs || typeof evidence.runs !== 'object' || Array.isArray(evidence.runs)) {
    throw new Error('quorum evidence must contain runs object');
  }
  return evidence;
}

export function appendRunRecord(evidence, corpus, record) {
  assertRequiredCorpus(corpus);
  const base = validateQuorumEvidence({
    ...evidence,
    runs: evidence?.runs ?? {},
  });
  const checked = validateRunRecord(record);
  if (checked.rustSidecarSourceCommit !== base.rustSidecarSourceCommit) {
    throw new Error('run record rustSidecarSourceCommit differs from quorum evidence');
  }
  if (checked.policyVersion !== base.policyVersion) {
    throw new Error('run record policyVersion differs from quorum evidence');
  }
  return {
    ...base,
    runs: {
      ...base.runs,
      [corpus]: [...(Array.isArray(base.runs[corpus]) ? base.runs[corpus] : []), checked],
    },
  };
}

export function buildRunRecordFromTopology({
  corpus,
  corpusRoot,
  outputDir,
  topology,
  command,
  commandWallElapsedMs,
  labSourceCommit,
  rustSidecarSourceCommit,
  rustSidecarBinary,
  machineOs,
  recordedAt,
  collector,
}) {
  assertRequiredCorpus(corpus);
  const scanner = topology?.meta?.rustTopologyScanner;
  if (!scanner || typeof scanner !== 'object') {
    throw new Error('topology.json missing meta.rustTopologyScanner');
  }
  return validateRunRecord({
    labSourceCommit,
    rustSidecarSourceCommit,
    rustSidecarBinary,
    command,
    corpusRoot: slashPath(corpusRoot),
    cacheMode: 'no-incremental',
    fileCount: topology?.summary?.files ?? topology?.summary?.fileCount ?? 0,
    filesCompared: scanner.filesCompared,
    mismatches: scanner.mismatches,
    commandWallElapsedMs,
    scannerBridgeElapsedMs: scanner.elapsedMs,
    sidecarElapsedMs: scanner.sidecarTiming?.elapsedMs ?? 0,
    sidecarStatus: scanner.status,
    policyVersion: scanner.policyVersion,
    machineOs,
    recordedAt,
    outputDir: slashPath(outputDir),
    topologyJson: slashPath(path.join(outputDir, 'topology.json')),
    collector,
  });
}

export function readQuorumEvidence(filePath = RUST_TOPOLOGY_PREFER_QUORUM_PATH) {
  try {
    return readJsonFile(filePath, {
      tag: 'rust-topology-quorum',
      strict: true,
    });
  } catch (error) {
    if (error?.code === 'ENOENT') return null;
    throw error;
  }
}

export function readOrCreateQuorumEvidence(filePath, rustSidecarSourceCommit) {
  return readQuorumEvidence(filePath) ?? defaultEvidence(rustSidecarSourceCommit);
}

export function writeJsonAtomic(filePath, value) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  atomicWrite(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

export function writeTextAtomic(filePath, text) {
  mkdirSync(path.dirname(filePath), { recursive: true });
  atomicWrite(filePath, `${String(text)}\n`);
}

export function pathExists(filePath) {
  return existsSync(filePath);
}

export function defaultEvidence(rustSidecarSourceCommit) {
  return {
    schemaVersion: QUORUM_SCHEMA_VERSION,
    requiredCorpora: [...REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA],
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    rustSidecarSourceCommit,
    runs: {},
  };
}
```

- [ ] **Step 2: Add behavior tests for the minimum happy path and realistic input validation**

Create `tests/rust-topology-quorum.test.mjs`:

```js
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  appendRunRecord,
  buildRunRecordFromTopology,
  defaultEvidence,
  normalizeRootMap,
  parseCorpusRootEntry,
  pathExists,
  readOrCreateQuorumEvidence,
  validateRunRecord,
} from '../_lib/rust-topology-quorum.mjs';
import {
  REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA,
} from '../_lib/rust-topology-prefer-gate.mjs';
import { MODULE_EDGE_SCANNER_POLICY_VERSION } from '../_lib/js-module-edge-scanner.mjs';

function tempDir(name) {
  return mkdtempSync(path.join(tmpdir(), `${name}-`));
}

function matchedTopology() {
  return {
    summary: { files: 11 },
    meta: {
      rustTopologyScanner: {
        status: 'matched',
        policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
        filesCompared: 11,
        mismatches: 0,
        elapsedMs: 100,
        sidecarTiming: { elapsedMs: 5 },
      },
    },
  };
}

function completeRun(overrides = {}) {
  return {
    labSourceCommit: 'lab-commit',
    rustSidecarSourceCommit: 'rust-commit',
    rustSidecarBinary: 'target/release/lumin-topology-scanner.exe',
    command: 'node measure-topology.mjs --no-incremental --clear-incremental-cache --rust-topology-scanner compare',
    corpusRoot: 'C:/corpora/geulbat-phase1',
    cacheMode: 'no-incremental',
    fileCount: 11,
    filesCompared: 11,
    mismatches: 0,
    commandWallElapsedMs: 1200,
    scannerBridgeElapsedMs: 100,
    sidecarElapsedMs: 5,
    sidecarStatus: 'matched',
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    machineOs: 'Microsoft Windows NT 10.0.26200.0',
    recordedAt: '2026-06-15T18:48:28+09:00',
    outputDir: 'C:/outputs/geulbat-phase1/run-001',
    topologyJson: 'C:/outputs/geulbat-phase1/run-001/topology.json',
    collector: {
      workingTreeClean: true,
      sourceDirty: false,
      labWorkingTreeClean: true,
      rustSidecarWorkingTreeClean: true,
    },
    ...overrides,
  };
}

describe('Rust topology quorum collector core', () => {
  it('builds and appends a matched no-incremental run record without reordering history', () => {
    const first = completeRun({ recordedAt: '2026-06-15T18:48:28+09:00' });
    const second = buildRunRecordFromTopology({
      corpus: 'geulbat-phase1',
      corpusRoot: 'C:/corpora/geulbat-phase1',
      outputDir: 'C:/outputs/geulbat-phase1/run-002',
      topology: matchedTopology(),
      command: first.command,
      commandWallElapsedMs: 1300,
      labSourceCommit: 'lab-commit',
      rustSidecarSourceCommit: 'rust-commit',
      rustSidecarBinary: 'target/release/lumin-topology-scanner.exe',
      machineOs: 'Microsoft Windows NT 10.0.26200.0',
      recordedAt: '2026-06-15T18:49:28+09:00',
      collector: first.collector,
    });

    const updated = appendRunRecord({
      ...defaultEvidence('rust-commit'),
      runs: { 'geulbat-phase1': [first] },
    }, 'geulbat-phase1', second);

    expect(updated.runs['geulbat-phase1'].map((run) => run.recordedAt)).toEqual([
      '2026-06-15T18:48:28+09:00',
      '2026-06-15T18:49:28+09:00',
    ]);
    expect(second).toMatchObject({
      fileCount: 11,
      filesCompared: 11,
      mismatches: 0,
      commandWallElapsedMs: 1300,
      scannerBridgeElapsedMs: 100,
      sidecarElapsedMs: 5,
      topologyJson: 'C:/outputs/geulbat-phase1/run-002/topology.json',
    });
  });

  it('requires explicit roots for all required corpora', () => {
    const roots = REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.map(
      (corpus) => `${corpus}=C:/corpora/${corpus}`,
    );

    expect(parseCorpusRootEntry('lab-self=C:/repo/lab')).toEqual([
      'lab-self',
      'C:/repo/lab',
    ]);
    expect(normalizeRootMap({ allRequired: true, corpusRoots: roots })).toEqual(
      Object.fromEntries(
        REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.map((corpus) => [
          corpus,
          `C:/corpora/${corpus}`,
        ]),
      ),
    );
    expect(() => normalizeRootMap({
      allRequired: true,
      corpusRoots: roots.slice(1),
    })).toThrow(/missing required corpus roots/);
  });

  it('allows roots-json as a path map but not as corpus policy', () => {
    const dir = tempDir('lumin-quorum-roots');
    const rootsPath = path.join(dir, 'roots.json');
    writeFileSync(rootsPath, JSON.stringify({
      roots: Object.fromEntries(
        REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.map((corpus) => [
          corpus,
          `C:/corpora/${corpus}`,
        ]),
      ),
      requiredCorpora: ['lab-self'],
    }));

    const rootMap = normalizeRootMap({ allRequired: true, rootsJson: rootsPath });
    expect(Object.keys(rootMap).sort()).toEqual([...REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA].sort());
  });

  it('rejects cached runs and missing audit fields before they enter quorum evidence', () => {
    expect(() => validateRunRecord(completeRun({ cacheMode: 'incremental' })))
      .toThrow(/no-incremental/);

    const missingField = completeRun();
    delete missingField.commandWallElapsedMs;
    expect(() => validateRunRecord(missingField)).toThrow(/commandWallElapsedMs/);

    const missingCollector = completeRun();
    delete missingCollector.collector;
    expect(() => validateRunRecord(missingCollector)).toThrow(/collector source diagnostics/);
  });

  it('creates default evidence for a missing quorum file and rejects mixed source commits', () => {
    const dir = tempDir('lumin-quorum-first-run');
    const quorumPath = path.join(dir, 'missing-quorum.json');

    expect(readOrCreateQuorumEvidence(quorumPath, 'rust-commit')).toMatchObject({
      rustSidecarSourceCommit: 'rust-commit',
      runs: {},
    });

    expect(() => appendRunRecord(
      defaultEvidence('different-rust-commit'),
      'geulbat-phase1',
      completeRun(),
    )).toThrow(/rustSidecarSourceCommit differs/);
  });
});
```

- [ ] **Step 3: Verify collector core behavior**

Run:

```powershell
C:\nvm4w\nodejs\node.exe .\node_modules\vitest\vitest.mjs run tests\rust-topology-quorum.test.mjs
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```powershell
git add _lib/rust-topology-quorum.mjs tests/rust-topology-quorum.test.mjs
git commit -m "Add rust topology quorum core"
```

## Task 3: Add Runner Orchestration, Hard Stop Behavior, And Summary

**Files:**
- `_lib/rust-topology-quorum.mjs`
- `tests/rust-topology-quorum.test.mjs`

- [ ] **Step 1: Extend `_lib/rust-topology-quorum.mjs` with orchestration and summary functions**

Add:

```js
function nextRunOutputDir(outputRoot, corpus, runIndex) {
  return path.join(outputRoot, corpus, `run-${String(runIndex).padStart(3, '0')}`);
}

function isSummaryCleanRun(run) {
  const collector = run?.collector;
  return (
    run?.sidecarStatus === 'matched' &&
    run?.mismatches === 0 &&
    run?.cacheMode === 'no-incremental' &&
    collector?.sourceDirty === false &&
    collector?.workingTreeClean === true &&
    collector?.labWorkingTreeClean === true &&
    collector?.rustSidecarWorkingTreeClean === true
  );
}

function latestThreeStatus(runs = []) {
  const recent = runs.slice(-3);
  const clean = recent.length === 3 && recent.every(isSummaryCleanRun);
  return clean ? 'clean' : 'incomplete';
}

export async function recordRustTopologyQuorum({
  corpus,
  root,
  quorumPath = RUST_TOPOLOGY_PREFER_QUORUM_PATH,
  outputRoot = DEFAULT_M4_QUORUM_OUTPUT_ROOT,
  rustSidecarBinary,
  rustSidecarSourceCommit,
  labSourceCommit,
  machineOs,
  timeoutMs = 60000,
  now = () => new Date().toISOString(),
  runner,
  sourceState,
} = {}) {
  if (!runner) throw new Error('runner is required for quorum recording');
  if (!sourceState) throw new Error('sourceState probe is required for quorum evidence');
  const rootMap = normalizeRootMap({ corpus, root });
  if (!rootMap[corpus]) throw new Error(`missing root for corpus: ${corpus}`);
  const evidence = readOrCreateQuorumEvidence(quorumPath, rustSidecarSourceCommit);
  validateQuorumEvidence(evidence);
  const existingRuns = evidence.runs?.[corpus] ?? [];
  const outputDir = nextRunOutputDir(outputRoot, corpus, existingRuns.length + 1);
  mkdirSync(outputDir, { recursive: true });
  const args = [
    'measure-topology.mjs',
    '--root',
    rootMap[corpus],
    '--output',
    outputDir,
    '--no-incremental',
    '--clear-incremental-cache',
    '--rust-topology-scanner',
    'compare',
    '--rust-topology-scanner-bin',
    rustSidecarBinary,
  ];
  const command = ['node', ...args].join(' ');
  const run = await runner({ corpus, root: rootMap[corpus], outputDir, command, args, timeoutMs });
  if (run.exitCode !== 0 && !run.topology?.meta?.rustTopologyScanner) {
    throw new Error('hard measure-topology failure: no scanner metadata');
  }
  const record = buildRunRecordFromTopology({
    corpus,
    corpusRoot: rootMap[corpus],
    outputDir,
    topology: run.topology,
    command: run.command ?? command,
    commandWallElapsedMs: run.commandWallElapsedMs,
    labSourceCommit,
    rustSidecarSourceCommit,
    rustSidecarBinary,
    machineOs,
    recordedAt: now(),
    collector: sourceState(),
  });
  const updated = appendRunRecord(evidence, corpus, record);
  writeJsonAtomic(quorumPath, updated);
  return { evidence: updated, record, commands: [run.command ?? command] };
}

export async function recordRustTopologyQuorumBatch({
  allRequired = false,
  corpus,
  root,
  corpusRoots = [],
  rootsJson,
  repeat = 1,
  ...rest
} = {}) {
  const rootMap = normalizeRootMap({
    allRequired,
    corpus,
    root,
    corpusRoots,
    rootsJson,
  });
  if (Object.keys(rootMap).length === 0) {
    throw new Error('no quorum corpora selected; pass --corpus/--root or --all-required roots');
  }
  const commands = [];
  let lastResult = null;
  for (let i = 0; i < repeat; i++) {
    for (const [name, corpusRoot] of Object.entries(rootMap)) {
      lastResult = await recordRustTopologyQuorum({
        corpus: name,
        root: corpusRoot,
        ...rest,
      });
      commands.push(...(lastResult.commands ?? []));
    }
  }
  return { ...lastResult, commands, rootMap };
}

export function renderQuorumSummary({ evidence, gateCheck, commands = [] } = {}) {
  validateQuorumEvidence(evidence);
  const lines = [
    '# M4 Rust Topology Quorum Evidence',
    '',
    `Date: ${new Date().toISOString().slice(0, 10)}`,
    '',
    '## Decision',
    '',
    'This records quorum evidence for the Rust topology scanner. `prefer` remains disabled and JS remains authoritative.',
    '',
    '## Commands',
    '',
    ...commands.map((command) => `- \`${command}\``),
    '',
    '## Corpus Runs',
    '',
    '| Corpus | Runs | Latest Three | Files Compared | Mismatches | Command Wall ms | Scanner Bridge ms | Sidecar ms |',
    '| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |',
  ];
  for (const corpus of REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA) {
    const runs = evidence.runs?.[corpus] ?? [];
    const last = runs.at(-1) ?? {};
    lines.push(`| \`${corpus}\` | ${runs.length} | ${latestThreeStatus(runs)} | ${last.filesCompared ?? 0} | ${last.mismatches ?? 0} | ${last.commandWallElapsedMs ?? 0} | ${last.scannerBridgeElapsedMs ?? 0} | ${last.sidecarElapsedMs ?? 0} |`);
  }
  lines.push(
    '',
    '## M3 Gate Verification',
    '',
    'Command:',
    '',
    '```bash',
    gateCheck?.command ?? '',
    '```',
    '',
    `- \`status\`: \`${gateCheck?.status ?? 'unknown'}\``,
    `- \`preferEnabled\`: \`${String(gateCheck?.preferEnabled)}\``,
    `- \`jsRemainsOracle\`: \`${String(gateCheck?.jsRemainsOracle)}\``,
    '',
    'Private CI was not used.',
    '',
  );
  return lines.join('\n');
}
```

- [ ] **Step 2: Add tests for completed compare failure, hard stop, and summary**

Append to `tests/rust-topology-quorum.test.mjs`:

```js
import {
  recordRustTopologyQuorum,
  recordRustTopologyQuorumBatch,
  renderQuorumSummary,
} from '../_lib/rust-topology-quorum.mjs';

it('records every required corpus in all-required batch mode', async () => {
  const dir = tempDir('lumin-quorum-all-required');
  const roots = REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.map(
    (corpus) => `${corpus}=C:/corpora/${corpus}`,
  );

  const result = await recordRustTopologyQuorumBatch({
    allRequired: true,
    corpusRoots: roots,
    quorumPath: path.join(dir, 'quorum.json'),
    outputRoot: path.join(dir, 'outputs'),
    rustSidecarBinary: 'target/release/lumin-topology-scanner.exe',
    rustSidecarSourceCommit: 'rust-commit',
    labSourceCommit: 'lab-commit',
    machineOs: 'Microsoft Windows NT 10.0.26200.0',
    now: () => '2026-06-15T18:48:28+09:00',
    sourceState: () => completeRun().collector,
    runner: async ({ command }) => ({
      exitCode: 0,
      command,
      commandWallElapsedMs: 50,
      topology: matchedTopology(),
    }),
  });

  expect(Object.keys(result.evidence.runs).sort()).toEqual(
    [...REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA].sort(),
  );
  expect(result.commands).toHaveLength(REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.length);
});

it('records completed compare failures when scanner metadata exists', async () => {
  const dir = tempDir('lumin-quorum-completed-failure');
  const failedTopology = matchedTopology();
  failedTopology.meta.rustTopologyScanner = {
    ...failedTopology.meta.rustTopologyScanner,
    status: 'risk-mismatch',
    mismatches: 1,
  };

  const result = await recordRustTopologyQuorum({
    corpus: 'geulbat-phase1',
    root: 'C:/corpora/geulbat-phase1',
    quorumPath: path.join(dir, 'quorum.json'),
    outputRoot: path.join(dir, 'outputs'),
    rustSidecarBinary: 'target/release/lumin-topology-scanner.exe',
    rustSidecarSourceCommit: 'rust-commit',
    labSourceCommit: 'lab-commit',
    machineOs: 'Microsoft Windows NT 10.0.26200.0',
    now: () => '2026-06-15T18:48:28+09:00',
    sourceState: () => completeRun().collector,
    runner: async () => ({
      exitCode: 0,
      command: 'node measure-topology.mjs --no-incremental --clear-incremental-cache --rust-topology-scanner compare',
      commandWallElapsedMs: 50,
      topology: failedTopology,
    }),
  });

  expect(result.record).toMatchObject({
    sidecarStatus: 'risk-mismatch',
    mismatches: 1,
  });
});

it('does not append quorum evidence when the runner hard-fails before scanner metadata exists', async () => {
  const dir = tempDir('lumin-quorum-hard-failure');
  const quorumPath = path.join(dir, 'quorum.json');

  await expect(recordRustTopologyQuorum({
    corpus: 'geulbat-phase1',
    root: 'C:/corpora/geulbat-phase1',
    quorumPath,
    outputRoot: path.join(dir, 'outputs'),
    rustSidecarBinary: 'target/release/lumin-topology-scanner.exe',
    rustSidecarSourceCommit: 'rust-commit',
    labSourceCommit: 'lab-commit',
    machineOs: 'Microsoft Windows NT 10.0.26200.0',
    sourceState: () => completeRun().collector,
    runner: async () => ({
      exitCode: 1,
      command: 'node measure-topology.mjs --no-incremental --clear-incremental-cache --rust-topology-scanner compare',
      commandWallElapsedMs: 10,
      topology: null,
    }),
  })).rejects.toThrow(/hard measure-topology failure/);

  expect(pathExists(quorumPath)).toBe(false);
});

it('renders a summary with M3 gate verification and oracle status', () => {
  const evidence = {
    ...defaultEvidence('rust-commit'),
    runs: Object.fromEntries(
      REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.map((corpus) => [
        corpus,
        [completeRun(), completeRun(), completeRun()],
      ]),
    ),
  };

  const summary = renderQuorumSummary({
    evidence,
    gateCheck: {
      command: 'node measure-topology.mjs --rust-topology-prefer-gate --rust-topology-prefer-quorum baselines/rust-topology-prefer-quorum.json',
      status: 'eligible',
      preferEnabled: false,
      jsRemainsOracle: true,
    },
    commands: ['node scripts/record-rust-topology-quorum.mjs --all-required --repeat 3'],
  });

  expect(summary).toContain('# M4 Rust Topology Quorum Evidence');
  expect(summary).toContain('node measure-topology.mjs --rust-topology-prefer-gate');
  expect(summary).toContain('`status`: `eligible`');
  expect(summary).toContain('`preferEnabled`: `false`');
  expect(summary).toContain('`jsRemainsOracle`: `true`');
});
```

If adding a second import block causes duplicate imports, merge these names into the existing import from `_lib/rust-topology-quorum.mjs`.

- [ ] **Step 3: Verify behavior**

Run:

```powershell
C:\nvm4w\nodejs\node.exe .\node_modules\vitest\vitest.mjs run tests\rust-topology-quorum.test.mjs
```

Expected: all quorum tests pass.

- [ ] **Step 4: Commit**

```powershell
git add _lib/rust-topology-quorum.mjs tests/rust-topology-quorum.test.mjs
git commit -m "Add rust topology quorum runner and summary"
```

## Task 4: Add The Lab-Only CLI Wrapper

**Files:**
- `scripts/record-rust-topology-quorum.mjs`
- `tests/rust-topology-quorum.test.mjs`

- [ ] **Step 1: Create the CLI wrapper**

Create `scripts/record-rust-topology-quorum.mjs`:

```js
#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { platform, release } from 'node:os';
import path from 'node:path';
import { parseArgs } from 'node:util';
import { fileURLToPath } from 'node:url';

import {
  DEFAULT_M4_QUORUM_OUTPUT_ROOT,
  recordRustTopologyQuorumBatch,
  renderQuorumSummary,
  writeTextAtomic,
} from '../_lib/rust-topology-quorum.mjs';
import {
  RUST_TOPOLOGY_PREFER_QUORUM_PATH,
} from '../_lib/rust-topology-prefer-gate.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const { values } = parseArgs({
  options: {
    corpus: { type: 'string' },
    root: { type: 'string' },
    'all-required': { type: 'boolean', default: false },
    'corpus-root': { type: 'string', multiple: true, default: [] },
    'roots-json': { type: 'string' },
    repeat: { type: 'string', default: '1' },
    'output-root': { type: 'string', default: DEFAULT_M4_QUORUM_OUTPUT_ROOT },
    quorum: { type: 'string', default: RUST_TOPOLOGY_PREFER_QUORUM_PATH },
    'rust-topology-scanner-bin': { type: 'string' },
    'rust-sidecar-source-commit': { type: 'string' },
    'lab-source-commit': { type: 'string' },
    'gate-check-corpus': { type: 'string', default: 'lab-self' },
    'timeout-ms': { type: 'string', default: '60000' },
  },
});

if (!values['rust-topology-scanner-bin']) {
  throw new Error('--rust-topology-scanner-bin is required');
}
if (!values['rust-sidecar-source-commit']) {
  throw new Error('--rust-sidecar-source-commit is required');
}

const repeat = Number(values.repeat);
if (!Number.isInteger(repeat) || repeat < 1) {
  throw new Error('--repeat must be a positive integer');
}

function runGit(args, cwd = repoRoot) {
  const child = spawnSync('git', args, {
    cwd,
    encoding: 'utf8',
    windowsHide: true,
  });
  if (child.status !== 0) {
    throw new Error(`git ${args.join(' ')} failed: ${child.stderr || child.stdout}`);
  }
  return child.stdout.trim();
}

function sourceState() {
  const labWorkingTreeClean = runGit(['status', '--porcelain']) === '';
  const rustSidecarRoot = path.join(repoRoot, 'experiments', 'rust-sidecar', 'topology-scanner');
  const rustSidecarWorkingTreeClean = runGit(['status', '--porcelain'], rustSidecarRoot) === '';
  return {
    workingTreeClean: labWorkingTreeClean && rustSidecarWorkingTreeClean,
    sourceDirty: !(labWorkingTreeClean && rustSidecarWorkingTreeClean),
    labWorkingTreeClean,
    rustSidecarWorkingTreeClean,
    labSourceCommit,
    rustSidecarSourceCommit: values['rust-sidecar-source-commit'],
  };
}

function readTopologyJson(outputDir) {
  return JSON.parse(readFileSync(path.join(outputDir, 'topology.json'), 'utf8'));
}

async function measureTopologyRunner({ args, command, outputDir, timeoutMs }) {
  const started = Date.now();
  const child = spawnSync(process.execPath, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: timeoutMs,
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64,
  });
  let topology = null;
  try {
    topology = readTopologyJson(outputDir);
  } catch (error) {
    if (child.status === 0) throw error;
  }
  return {
    exitCode: child.status ?? 1,
    command,
    commandWallElapsedMs: Date.now() - started,
    topology,
    stdout: child.stdout,
    stderr: child.stderr,
  };
}

function runGateCheck({ root, outputRoot, rustSidecarBinary, quorumPath, corpus, timeoutMs }) {
  const outputDir = path.join(outputRoot, 'm3-gate-check');
  const args = [
    'measure-topology.mjs',
    '--root',
    root,
    '--output',
    outputDir,
    '--no-incremental',
    '--clear-incremental-cache',
    '--rust-topology-scanner',
    'compare',
    '--rust-topology-scanner-bin',
    rustSidecarBinary,
    '--rust-topology-prefer-gate',
    '--rust-topology-prefer-gate-corpus',
    corpus,
    '--rust-topology-prefer-quorum',
    quorumPath,
  ];
  const command = ['node', ...args].join(' ');
  const child = spawnSync(process.execPath, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: timeoutMs,
    windowsHide: true,
    maxBuffer: 1024 * 1024 * 64,
  });
  if (child.status !== 0) {
    throw new Error(`M3 gate verification failed: ${child.stderr || child.stdout}`);
  }
  const gate = readTopologyJson(outputDir)?.meta?.rustTopologyPreferGate;
  return {
    command,
    status: gate?.status ?? 'unknown',
    preferEnabled: gate?.preferEnabled === true,
    jsRemainsOracle: gate?.jsRemainsOracle === true,
  };
}

const timeoutMs = Number(values['timeout-ms']);
const labSourceCommit = values['lab-source-commit'] ?? runGit(['rev-parse', 'HEAD']);
const result = await recordRustTopologyQuorumBatch({
  corpus: values.corpus,
  root: values.root,
  allRequired: values['all-required'],
  corpusRoots: values['corpus-root'],
  rootsJson: values['roots-json'],
  repeat,
  quorumPath: values.quorum,
  outputRoot: values['output-root'],
  rustSidecarBinary: values['rust-topology-scanner-bin'],
  rustSidecarSourceCommit: values['rust-sidecar-source-commit'],
  labSourceCommit,
  machineOs: `${platform()} ${release()}`,
  timeoutMs,
  runner: measureTopologyRunner,
  sourceState,
});

const gateCheckRoot = result?.rootMap?.[values['gate-check-corpus']];
if (!gateCheckRoot) {
  throw new Error(`M3 gate verification needs a root for ${values['gate-check-corpus']}`);
}
const gateCheck = runGateCheck({
  root: gateCheckRoot,
  outputRoot: values['output-root'],
  rustSidecarBinary: values['rust-topology-scanner-bin'],
  quorumPath: values.quorum,
  corpus: values['gate-check-corpus'],
  timeoutMs,
});

const summary = renderQuorumSummary({
  evidence: result?.evidence,
  gateCheck,
  commands: result?.commands ?? [],
});
writeTextAtomic('baselines/m4-rust-topology-quorum-2026-06-15.md', summary);
console.log(`[rust-topology-quorum] updated ${values.quorum}`);
```

- [ ] **Step 2: Verify syntax and targeted behavior**

Run:

```powershell
C:\nvm4w\nodejs\node.exe --check scripts\record-rust-topology-quorum.mjs
C:\nvm4w\nodejs\node.exe .\node_modules\vitest\vitest.mjs run tests\rust-topology-quorum.test.mjs
```

Expected: syntax check exits 0 and quorum tests pass.

- [ ] **Step 3: Commit**

```powershell
git add scripts/record-rust-topology-quorum.mjs tests/rust-topology-quorum.test.mjs _lib/rust-topology-quorum.mjs
git commit -m "Add rust topology quorum collector CLI"
```

## Task 5: Final Verification And Documentation

**Files:**
- `tests/README.md`
- maybe runtime artifacts if intentionally recording real evidence:
  - `baselines/rust-topology-prefer-quorum.json`
  - `baselines/m4-rust-topology-quorum-2026-06-15.md`

- [ ] **Step 1: Regenerate test docs**

Run:

```powershell
C:\nvm4w\nodejs\node.exe scripts\update-test-doc.mjs
```

Expected: `tests/README.md` includes `tests/rust-topology-quorum.test.mjs`.

- [ ] **Step 2: Run targeted tests**

Run:

```powershell
C:\nvm4w\nodejs\node.exe .\node_modules\vitest\vitest.mjs run tests\rust-topology-prefer-gate.test.mjs tests\rust-topology-quorum.test.mjs
```

Expected: both test files pass.

- [ ] **Step 3: Run syntax checks for changed JS/MJS**

Run:

```powershell
C:\nvm4w\nodejs\node.exe --check _lib\rust-topology-prefer-gate.mjs
C:\nvm4w\nodejs\node.exe --check skills\lumin-repo-lens-lab\_engine\lib\rust-topology-prefer-gate.mjs
C:\nvm4w\nodejs\node.exe --check _lib\rust-topology-quorum.mjs
C:\nvm4w\nodejs\node.exe --check scripts\record-rust-topology-quorum.mjs
C:\nvm4w\nodejs\node.exe --check tests\rust-topology-prefer-gate.test.mjs
C:\nvm4w\nodejs\node.exe --check tests\rust-topology-quorum.test.mjs
```

Expected: all commands exit 0.

- [ ] **Step 4: Run drift, test-doc, and diff checks**

Run:

```powershell
C:\nvm4w\nodejs\node.exe scripts\check-drift.mjs
C:\nvm4w\nodejs\node.exe scripts\update-test-doc.mjs --check
git diff --check
```

Expected:

- `check-drift.mjs` exits 0.
- `update-test-doc.mjs --check` exits 0.
- `git diff --check` exits 0, allowing only Git CRLF normalization warnings if they appear.

- [ ] **Step 5: Confirm private CI policy is untouched**

Run:

```powershell
Select-String -Path .github\workflows\ci.yml -Pattern "workflow_dispatch|push:|pull_request:"
```

Expected: `workflow_dispatch` present; `push:` and `pull_request:` absent.

- [ ] **Step 6: Commit test docs if changed**

```powershell
git add tests/README.md
git commit -m "Document rust topology quorum tests"
```

Skip this commit if `tests/README.md` did not change.

## Final Verification

Run:

```powershell
git status -sb
git log --oneline -5
```

Expected:

- Working tree clean.
- Recent commits show the M4 quorum implementation commits.
- No private CI was triggered.

## Self-Review Checklist

- The plan implements every M4 design requirement.
- The plan does not enable `prefer`.
- The plan does not add a public prefer command.
- The plan does not touch stable `/lumin-repo-lens:*`.
- The plan uses behavior-focused tests: happy path, realistic edge cases, and hard stops.
- The plan does not rely on scaffolding presence as test value.
- The plan uses `commandWallElapsedMs`, `scannerBridgeElapsedMs`, and `sidecarElapsedMs`; it does not reintroduce `wrapperElapsedMs`.
