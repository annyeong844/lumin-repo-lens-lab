# Rust Topology Prefer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Keep tests behavior-focused: minimum guaranteed happy path, realistic edge cases, and hard-stop paths. Do not add scaffolding tests that only prove files, functions, or modules exist.

**Goal:** Implement lab-only, explicit opt-in, run-level Rust topology `prefer` mode with visible blocked prefer diagnostic and strict artifact guard.

**Architecture:** Keep `measure-topology.mjs` as the orchestrator and JS as the diagnostic hard stop. Add a small pure decision module for prefer metadata, reuse the existing Rust scanner bridge and M3 quorum gate, and build a Rust candidate topology artifact only when the run is no-incremental, fixed-corpus, gate-eligible, policy-matched, and sidecar-matched. The first M5 implementation runs artifact guard on every prefer run.

**Tech Stack:** Node.js ESM, existing `measure-topology.mjs`, existing Rust sidecar bridge, existing M3 prefer gate, Vitest, PowerShell-friendly CLI commands.

---

## Design Commitments

- `prefer` is lab-only and explicit opt-in.
- `off` and `compare` behavior must remain unchanged.
- `prefer` is run-level only; no per-file Rust/JS mixing.
- Initial `prefer` is no-incremental/full-coverage only.
- Initial `prefer` is scoped to the fixed required corpus set:
  - `geulbat-phase1`
  - `lab-self`
  - `stable-source-clean`
  - `nuxt-main`
- M3 `rustTopologyPreferGate.status === "eligible"` is required.
- Rust output can be used only when artifact guard passes.
- Any unknown or unsupported state falls back to JS with explicit metadata.
- Private CI stays unused.

## File Structure

- Modify: `docs/lab/m5-rust-topology-prefer-design-2026-06-15.md`
  - Add exact reasons for cache-mode and corpus-scope blocks discovered while writing this plan.
- Create: `_lib/rust-topology-prefer.mjs`
  - Own M5 prefer status/reason constants.
  - Compute sidecar binary SHA-256.
  - Normalize topology artifacts for artifact guard.
  - Evaluate prefer decision metadata from scanner, gate, cache mode, corpus, binary identity, and guard result.
- Modify: `measure-topology.mjs`
  - Accept `--rust-topology-scanner prefer`.
  - Keep compare/off paths unchanged.
  - In prefer mode, run the existing JS path first, run Rust comparison, evaluate the gate, build a Rust candidate artifact, run artifact guard, then write either Rust artifact or blocked prefer diagnostic artifact with `meta.rustTopologyPrefer`.
- Modify: `tests/rust-topology-scanner-bridge.test.mjs`
  - Add behavior coverage for binary identity only if the bridge owns it; otherwise leave bridge tests unchanged.
- Create: `tests/rust-topology-prefer.test.mjs`
  - Pure behavior checks for prefer decision metadata.
- Modify: `tests/topology-producer-cross-edges.test.mjs`
  - Add end-to-end producer behavior for prefer happy path, blocked paths, artifact guard mismatch, and rollback.
- Modify: `tests/README.md`
  - Regenerate after adding the new test file.
- Modify mirror files under `skills/lumin-repo-lens-lab/_engine/` only after root implementation passes:
  - `skills/lumin-repo-lens-lab/_engine/producers/measure-topology.mjs`
  - `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer.mjs`

## Task 1: Patch The M5 Design Vocabulary Gap

**Files:**
- Modify: `docs/lab/m5-rust-topology-prefer-design-2026-06-15.md`

- [ ] **Step 1: Add exact blocked reasons for scope blocks**

In the `Prefer Status Vocabulary` section, add these reasons:

```md
- `blocked-cache-mode`
- `blocked-corpus-scope`
```

Use `blocked-cache-mode` when `prefer` is requested without no-incremental full coverage.

Use `blocked-corpus-scope` when the current corpus is not one of the fixed required corpora before or during gate evaluation.

- [ ] **Step 2: Verify vocabulary references**

Run:

```bash
rg -n "blocked-cache-mode|blocked-corpus-scope|scope path|corpus path" docs/lab/m5-rust-topology-prefer-design-2026-06-15.md
```

Expected:

- both new reasons appear in the exact vocabulary list;
- scope/corpus validation language points to exact blocked metadata, not vague wording.

## Task 2: Add The Prefer Decision Module

**Files:**
- Create: `_lib/rust-topology-prefer.mjs`
- Test: `tests/rust-topology-prefer.test.mjs`

- [ ] **Step 1: Create `_lib/rust-topology-prefer.mjs`**

Add the module with these responsibilities:

```js
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

import { MODULE_EDGE_SCANNER_POLICY_VERSION } from './js-module-edge-scanner.mjs';
import { REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA } from './rust-topology-prefer-gate.mjs';

export const RUST_TOPOLOGY_PREFER_STATUSES = Object.freeze([
  'not-requested',
  'used-rust',
  'blocked',
]);

export const RUST_TOPOLOGY_PREFER_REASONS = Object.freeze([
  'not-requested',
  'gate-eligible-artifact-guard-passed',
  'blocked-quorum-missing',
  'blocked-gate-missing',
  'blocked-gate-ineligible',
  'blocked-binary-not-found',
  'blocked-timeout',
  'blocked-non-zero-exit',
  'blocked-invalid-json-output',
  'blocked-policy-version',
  'blocked-sidecar-source-commit',
  'blocked-sidecar-binary-sha256',
  'blocked-count-mismatch',
  'blocked-edge-mismatch',
  'blocked-risk-mismatch',
  'blocked-artifact-contract',
  'blocked-cache-mode',
  'blocked-corpus-scope',
  'blocked-unknown-sidecar-status',
  'blocked-unknown-prefer-status',
]);

const SCANNER_TO_PREFER_REASON = new Map([
  ['binary-not-found', 'blocked-binary-not-found'],
  ['timeout', 'blocked-timeout'],
  ['non-zero-exit', 'blocked-non-zero-exit'],
  ['invalid-json-output', 'blocked-invalid-json-output'],
  ['count-mismatch', 'blocked-count-mismatch'],
  ['edge-mismatch', 'blocked-edge-mismatch'],
  ['risk-mismatch', 'blocked-risk-mismatch'],
]);

export function hashFileSha256(filePath) {
  const hash = createHash('sha256');
  hash.update(readFileSync(filePath));
  return `sha256:${hash.digest('hex')}`;
}

export function normalizeTopologyForRustPreferGuard(topology) {
  const normalized = structuredClone(topology);
  if (normalized?.meta) {
    normalized.meta.generated = '<generated>';
    delete normalized.meta.rustTopologyScanner;
    delete normalized.meta.rustTopologyPreferGate;
    delete normalized.meta.rustTopologyPrefer;
  }
  if (normalized?.summary?.performance) {
    normalized.summary.performance.scannerMs = '<scannerMs>';
  }
  return normalized;
}

export function compareTopologyArtifactContract(jsArtifact, rustArtifact) {
  const js = normalizeTopologyForRustPreferGuard(jsArtifact);
  const rust = normalizeTopologyForRustPreferGuard(rustArtifact);
  const passed = JSON.stringify(js) === JSON.stringify(rust);
  return {
    status: passed ? 'passed' : 'failed',
    passed,
  };
}

function blocked({ reason, base }) {
  return {
    ...base,
    status: 'blocked',
    usedRust: false,
    reason,
  };
}

export function evaluateRustTopologyPrefer({
  requested = false,
  mode = 'off',
  isIncremental = false,
  currentCorpus,
  rustTopologyScanner,
  rustTopologyPreferGate,
  quorumEvidencePath,
  rustSidecarBinary,
  rustSidecarBinarySha256,
  rustSidecarBuildProfile = 'release',
  artifactGuard,
} = {}) {
  const base = {
    schemaVersion: 1,
    requested,
    mode,
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    rustSidecarBinary: rustSidecarBinary ?? null,
    rustSidecarBinarySha256: rustSidecarBinarySha256 ?? null,
    rustSidecarBuildProfile,
    quorumEvidence: quorumEvidencePath ?? null,
    gateStatus: rustTopologyPreferGate?.status ?? null,
    filesCompared: rustTopologyScanner?.filesCompared ?? 0,
    mismatches: rustTopologyScanner?.mismatches ?? 0,
    sidecarTiming: rustTopologyScanner?.sidecarTiming ?? null,
    artifactGuard: artifactGuard ?? { status: 'not-run' },
  };

  if (!requested || mode !== 'prefer') {
    return {
      ...base,
      status: 'not-requested',
      usedRust: false,
      reason: 'not-requested',
    };
  }
  if (isIncremental) return blocked({ reason: 'blocked-cache-mode', base });
  if (!REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.includes(currentCorpus)) {
    return blocked({ reason: 'blocked-corpus-scope', base });
  }
  if (!rustTopologyPreferGate) return blocked({ reason: 'blocked-gate-missing', base });
  if (rustTopologyPreferGate.status !== 'eligible') {
    const reason = rustTopologyPreferGate.reason === 'quorum-evidence-missing'
      ? 'blocked-quorum-missing'
      : 'blocked-gate-ineligible';
    return blocked({ reason, base });
  }
  if (!rustTopologyScanner) return blocked({ reason: 'blocked-unknown-sidecar-status', base });
  if (
    rustTopologyScanner.policyVersion &&
    rustTopologyScanner.policyVersion !== MODULE_EDGE_SCANNER_POLICY_VERSION
  ) {
    return blocked({ reason: 'blocked-policy-version', base });
  }
  if (rustTopologyScanner.reason === 'policy-version-mismatch') {
    return blocked({ reason: 'blocked-policy-version', base });
  }
  if (rustTopologyScanner.status !== 'matched') {
    return blocked({
      reason: SCANNER_TO_PREFER_REASON.get(rustTopologyScanner.status) ??
        'blocked-unknown-sidecar-status',
      base,
    });
  }
  if ((rustTopologyScanner.mismatches ?? 0) !== 0) {
    return blocked({ reason: 'blocked-unknown-sidecar-status', base });
  }
  if (artifactGuard?.status !== 'passed') {
    return blocked({ reason: 'blocked-artifact-contract', base });
  }
  return {
    ...base,
    status: 'used-rust',
    usedRust: true,
    reason: 'gate-eligible-artifact-guard-passed',
  };
}
```

- [ ] **Step 2: Add behavior checks for decision metadata**

Create `tests/rust-topology-prefer.test.mjs` with these checks:

```js
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  compareTopologyArtifactContract,
  evaluateRustTopologyPrefer,
  hashFileSha256,
} from '../_lib/rust-topology-prefer.mjs';
import { MODULE_EDGE_SCANNER_POLICY_VERSION } from '../_lib/js-module-edge-scanner.mjs';

const matchedScanner = {
  status: 'matched',
  policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
  filesCompared: 1,
  mismatches: 0,
  sidecarTiming: { files: 1, elapsedMs: 1 },
};

const eligibleGate = {
  status: 'eligible',
  reason: 'all-required-corpora-matched',
  preferEnabled: false,
  jsRemainsOracle: true,
};

function base(overrides = {}) {
  return {
    requested: true,
    mode: 'prefer',
    isIncremental: false,
    currentCorpus: 'lab-self',
    rustTopologyScanner: matchedScanner,
    rustTopologyPreferGate: eligibleGate,
    rustSidecarBinary: 'C:/bin/lumin-topology-scanner.exe',
    rustSidecarBinarySha256: 'sha256:abc',
    artifactGuard: { status: 'passed', passed: true },
    ...overrides,
  };
}

describe('Rust topology prefer decision', () => {
  it('uses Rust only for explicit prefer with eligible gate and passing artifact guard', () => {
    expect(evaluateRustTopologyPrefer(base())).toMatchObject({
      status: 'used-rust',
      usedRust: true,
      reason: 'gate-eligible-artifact-guard-passed',
      gateStatus: 'eligible',
      policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    });
  });

  it('falls back visibly when prefer is requested with incremental cache coverage', () => {
    expect(evaluateRustTopologyPrefer(base({ isIncremental: true }))).toMatchObject({
      status: 'blocked',
      usedRust: false,
      reason: 'blocked-cache-mode',
    });
  });

  it('falls back visibly when current corpus is outside the fixed required set', () => {
    expect(evaluateRustTopologyPrefer(base({ currentCorpus: 'random-repo' }))).toMatchObject({
      status: 'blocked',
      reason: 'blocked-corpus-scope',
    });
  });

  it('falls back visibly when scanner comparison mismatches', () => {
    expect(evaluateRustTopologyPrefer(base({
      rustTopologyScanner: { ...matchedScanner, status: 'edge-mismatch', mismatches: 1 },
    }))).toMatchObject({
      status: 'blocked',
      reason: 'blocked-edge-mismatch',
    });
  });

  it('falls back visibly when artifact guard fails', () => {
    expect(evaluateRustTopologyPrefer(base({
      artifactGuard: { status: 'failed', passed: false },
    }))).toMatchObject({
      status: 'blocked',
      reason: 'blocked-artifact-contract',
    });
  });

  it('normalizes topology artifacts by removing only Rust prefer metadata', () => {
    const jsArtifact = {
      meta: { generated: 'a', rustTopologyScanner: {}, rustTopologyPreferGate: {} },
      summary: { files: 1, performance: { scannerMs: 12 } },
      nodes: { 'src/a.ts': { loc: 1 } },
      edges: [],
    };
    const rustArtifact = {
      meta: { generated: 'b', rustTopologyScanner: {}, rustTopologyPrefer: {} },
      summary: { files: 1, performance: { scannerMs: 99 } },
      nodes: { 'src/a.ts': { loc: 1 } },
      edges: [],
    };

    expect(compareTopologyArtifactContract(jsArtifact, rustArtifact)).toMatchObject({
      status: 'passed',
      passed: true,
    });
  });

  it('detects real topology contract drift after metadata normalization', () => {
    const jsArtifact = {
      meta: { generated: 'a' },
      summary: { files: 1, performance: { scannerMs: 12 } },
      nodes: { 'src/a.ts': { loc: 1 } },
      edges: [],
    };
    const rustArtifact = {
      meta: { generated: 'b' },
      summary: { files: 1, performance: { scannerMs: 99 } },
      nodes: { 'src/a.ts': { loc: 2 } },
      edges: [],
    };

    expect(compareTopologyArtifactContract(jsArtifact, rustArtifact)).toMatchObject({
      status: 'failed',
      passed: false,
    });
  });

  it('hashes the sidecar binary bytes for metadata', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'lumin-sidecar-sha-'));
    try {
      const file = path.join(dir, 'sidecar.bin');
      writeFileSync(file, 'sidecar-bytes');
      expect(hashFileSha256(file)).toMatch(/^sha256:[0-9a-f]{64}$/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
```

- [ ] **Step 3: Run the pure prefer tests**

Run:

```bash
npx vitest run tests/rust-topology-prefer.test.mjs
```

Expected:

- PASS.

## Task 3: Refactor Topology Artifact Assembly Without Behavior Change

**Files:**
- Modify: `measure-topology.mjs`
- Test: `tests/topology-producer-cross-edges.test.mjs`

This task makes Rust candidate artifact assembly possible without changing output.

- [ ] **Step 1: Extract local artifact assembly helper inside `measure-topology.mjs`**

Inside `measure-topology.mjs`, extract the aggregation block beginning at `// ─── aggregate` into a local helper:

```js
function assembleTopologyArtifactFromEntries({
  sourceEntries,
  files,
  edgesLabel = 'js',
  rustMetadata = {},
}) {
  // Move the existing nodes/edges/fanIn/fanOut/SCC/submodule/bigFiles logic
  // here without changing field names.
  // Return { artifact, nodes, edges, totalLoc, parseErrors }.
}
```

Keep this helper in the same file for M5. Do not create a broad topology framework yet. This file is already the owner of topology semantics.

- [ ] **Step 2: Replace the current assembly block with the helper**

The JS path should still:

- build `sourceEntries` from `nextCache.entries`;
- produce the same `artifact.summary`;
- produce the same `nodes`, `edges`, `topFanIn`, `topFanOut`, `sccs`, and submodule fields;
- write `topology.json` once.

- [ ] **Step 3: Run existing topology behavior checks**

Run:

```bash
npx vitest run tests/topology-producer-cross-edges.test.mjs
```

Expected:

- PASS.

Do not proceed if this fails. This refactor must be behavior-preserving.

## Task 4: Build Rust Candidate Entries In Prefer Mode

**Files:**
- Modify: `measure-topology.mjs`

- [ ] **Step 1: Allow `prefer` in CLI mode validation**

Change:

```js
if (!['off', 'compare'].includes(rustScannerMode)) {
  throw new Error(`unsupported --rust-topology-scanner mode: ${rustScannerMode}`);
}
```

to:

```js
if (!['off', 'compare', 'prefer'].includes(rustScannerMode)) {
  throw new Error(`unsupported --rust-topology-scanner mode: ${rustScannerMode}`);
}
```

Keep the existing compare+incremental hard stop for `compare`. For `prefer`, do not throw for incremental mode; the prefer decision should fall back to JS with `blocked-cache-mode` metadata so the run still produces useful topology output.

- [ ] **Step 2: Import the prefer helpers**

Add imports:

```js
import {
  compareTopologyArtifactContract,
  evaluateRustTopologyPrefer,
  hashFileSha256,
} from './_lib/rust-topology-prefer.mjs';
```

- [ ] **Step 3: Add Rust result to entry conversion**

Add a helper near `processFileTs`:

```js
function buildRustTopologyEntryFromScannerResult(f, rustResult) {
  const edgesOut = [];
  let externalCount = 0;
  let unresolvedCount = 0;
  if (!rustResult || rustResult.ok !== true) {
    return {
      loc: rustResult?.loc ?? 0,
      edges: [],
      externalCount: 0,
      unresolvedCount: 0,
      parseError: false,
      scannerMode: 'rust-module-edge-risk',
    };
  }
  for (const edge of rustResult.edges ?? []) {
    const outcome = resolveTopologyEdge(f, edge.source, edge, edgesOut);
    if (outcome === 'external') externalCount++;
    else if (outcome === 'unresolved') unresolvedCount++;
  }
  return {
    loc: rustResult.loc ?? 0,
    edges: edgesOut,
    externalCount,
    unresolvedCount,
    parseError: false,
    scannerMode: 'rust-module-edge',
  };
}
```

This intentionally does not try to recover replacement edges for `ok:false` Rust results. Artifact guard must catch any loss. If the guard fails, prefer is blocked.

- [ ] **Step 4: Build a Rust candidate entry map only after scanner comparison**

After `rustScannerComparison`, create a candidate map only when the scanner returned results:

```js
function buildRustCandidateEntries({ jsEntries, rustResults }) {
  const rustByFile = new Map(
    (rustResults ?? []).map((entry) => [String(entry.file).replaceAll('\\', '/'), entry]),
  );
  const entries = structuredClone(jsEntries);
  for (const file of rustComparableJsResults.map((entry) => entry.file)) {
    const key = file.replaceAll('\\', '/');
    const rustResult = rustByFile.get(key);
    entries[file] = buildRustTopologyEntryFromScannerResult(file, rustResult);
  }
  return entries;
}
```

Use the same absolute-file keys that `nextCache.entries` uses. Normalize only for lookup.

## Task 5: Evaluate Prefer And Write Either Rust Or JS Artifact

**Files:**
- Modify: `measure-topology.mjs`

- [ ] **Step 1: Compute binary SHA only when prefer is requested and binary exists**

Before evaluating prefer:

```js
const rustPreferRequested = rustScannerMode === 'prefer';
let rustSidecarBinarySha256 = null;
try {
  if (rustPreferRequested && cli.raw['rust-topology-scanner-bin']) {
    rustSidecarBinarySha256 = hashFileSha256(cli.raw['rust-topology-scanner-bin']);
  }
} catch {
  rustSidecarBinarySha256 = null;
}
```

If the binary is missing, the scanner metadata will already be `binary-not-found`; the prefer reason should become `blocked-binary-not-found`.

- [ ] **Step 2: Assemble JS artifact first**

Keep the JS artifact as the diagnostic artifact. It must include:

- `meta.rustTopologyScanner` when scanner metadata exists;
- `meta.rustTopologyPreferGate` when gate is enabled.

- [ ] **Step 3: Assemble Rust candidate artifact only for prefer mode**

When `rustPreferRequested` and `rustScannerComparison.rustResults.length > 0`:

```js
const rustCandidateEntries = buildRustCandidateEntries({
  jsEntries: sourceEntriesForAssembly,
  rustResults: rustScannerComparison.rustResults,
});
const rustCandidateArtifact = assembleTopologyArtifactFromEntries({
  sourceEntries: rustCandidateEntries,
  files,
  rustMetadata: {
    rustTopologyScanner: rustScannerComparison.metadata,
    ...(rustTopologyPreferGate ? { rustTopologyPreferGate } : {}),
  },
}).artifact;
```

- [ ] **Step 4: Run artifact guard**

```js
const artifactGuard = rustCandidateArtifact
  ? compareTopologyArtifactContract(jsArtifact, rustCandidateArtifact)
  : { status: 'not-run' };
```

- [ ] **Step 5: Evaluate prefer**

```js
const rustTopologyPrefer = evaluateRustTopologyPrefer({
  requested: rustPreferRequested,
  mode: rustScannerMode,
  isIncremental,
  currentCorpus: cli.raw['rust-topology-prefer-gate-corpus'],
  rustTopologyScanner: rustScannerComparison.metadata,
  rustTopologyPreferGate,
  quorumEvidencePath: rustPreferQuorumPath,
  rustSidecarBinary: cli.raw['rust-topology-scanner-bin'],
  rustSidecarBinarySha256,
  artifactGuard,
});
```

- [ ] **Step 6: Choose final artifact**

```js
const finalArtifact = rustTopologyPrefer.usedRust && rustCandidateArtifact
  ? rustCandidateArtifact
  : jsArtifact;

finalArtifact.meta.rustTopologyPrefer = rustTopologyPrefer;
```

Write `finalArtifact`.

Do not write two topology files in M5. The single output must be honest about whether Rust was used or blocked prefer diagnostic was used.

## Task 6: Add End-To-End Prefer Behavior Checks

**Files:**
- Modify: `tests/topology-producer-cross-edges.test.mjs`

Use the existing fixture helpers:

- `runTopologyWithStderr`
- `writeFakeRustTopologySidecar`
- `cleanQuorumEvidence`
- `normalizeTopologyForGateContract`

Do not add file/function existence tests.

- [ ] **Step 1: Add the minimum happy path**

Add a test:

```js
it("uses Rust for explicit prefer when gate is eligible and artifact guard passes", () => {
  const fixture = createTempRepoFixture({
    prefix: "vitest-topology-rust-prefer-happy-",
    packageJson: { name: "rust-prefer-happy-fx", type: "module" },
  });
  try {
    fixture.write("src/empty.mjs", "export const value = 1;\n");
    const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));
    const quorumPath = path.join(fixture.root, "rust-topology-prefer-quorum.json");
    writeFileSync(quorumPath, JSON.stringify(cleanQuorumEvidence(), null, 2));

    const topology = runTopologyWithStderr(fixture, {
      args: [
        "--no-incremental",
        "--clear-incremental-cache",
        "--rust-topology-scanner",
        "prefer",
        "--rust-topology-scanner-bin",
        sidecar,
        "--rust-topology-timeout-ms",
        "1000",
        "--rust-topology-prefer-gate",
        "--rust-topology-prefer-gate-corpus",
        "lab-self",
        "--rust-topology-prefer-quorum",
        quorumPath,
      ],
    }).topology;

    expect(topology.meta.rustTopologyPrefer).toMatchObject({
      requested: true,
      mode: "prefer",
      status: "used-rust",
      usedRust: true,
      reason: "gate-eligible-artifact-guard-passed",
    });
    expect(topology.summary.files).toBe(1);
    expect(topology.edges).toEqual([]);
  } finally {
    fixture.cleanup();
  }
}, 30000);
```

- [ ] **Step 2: Add missing-binary blocked path**

Use a real fixture with one import and pass a missing sidecar path. Expect:

```js
expect(topology.meta.rustTopologyPrefer).toMatchObject({
  status: "blocked",
  usedRust: false,
  reason: "blocked-binary-not-found",
});
expect(topology.edges.length).toBeGreaterThan(0);
```

This proves the user still gets JS topology output.

- [ ] **Step 3: Add ineligible-gate blocked path**

Use a quorum file with one required corpus missing. Expect:

```js
expect(topology.meta.rustTopologyPrefer).toMatchObject({
  status: "blocked",
  reason: "blocked-gate-ineligible",
});
expect(topology.meta.rustTopologyPreferGate.status).toBe("blocked-corpus-quorum");
```

- [ ] **Step 4: Add artifact-guard blocked path with a plausible sidecar bug**

Add a fake sidecar variant that returns the same edges and risk but wrong `loc`.
The existing scanner comparison ignores LOC, so scanner status can be `matched`,
but artifact guard must fail because node LOC differs.

Expected:

```js
expect(topology.meta.rustTopologyScanner.status).toBe("matched");
expect(topology.meta.rustTopologyPrefer).toMatchObject({
  status: "blocked",
  reason: "blocked-artifact-contract",
  usedRust: false,
});
```

This is a real edge case: sidecar edge parity can be true while artifact
contract is still wrong.

- [ ] **Step 5: Add cache-mode hard stop**

Run `prefer` without `--no-incremental`. Expect:

```js
expect(topology.meta.rustTopologyPrefer).toMatchObject({
  status: "blocked",
  reason: "blocked-cache-mode",
});
```

The topology output should still be valid JS output.

- [ ] **Step 6: Add corpus-scope hard stop**

Run `prefer` with `--rust-topology-prefer-gate-corpus random-repo`. Expect:

```js
expect(topology.meta.rustTopologyPrefer).toMatchObject({
  status: "blocked",
  reason: "blocked-corpus-scope",
});
```

- [ ] **Step 7: Add rollback checks for off and compare**

Use the existing compare/off tests as the baseline. Ensure:

- `off` does not emit `meta.rustTopologyPrefer`;
- `compare` does not emit `meta.rustTopologyPrefer` or emits `not-requested` only if the implementation chooses always-present metadata;
- neither mode uses Rust as topology owner.

Do not change existing expectations unless the new metadata is intentionally always present. Prefer absent metadata for `off` and `compare` to keep old artifacts quiet.

- [ ] **Step 8: Run targeted topology tests**

Run:

```bash
npx vitest run tests/topology-producer-cross-edges.test.mjs
```

Expected:

- PASS.

## Task 7: Mirror Root Changes Into The Skill Package

**Files:**
- Create: `skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/producers/measure-topology.mjs`

- [ ] **Step 1: Copy the prefer helper**

Copy `_lib/rust-topology-prefer.mjs` to:

```text
skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer.mjs
```

- [ ] **Step 2: Apply producer import path rewrite**

In the skills producer, imports should use `../lib/...` rather than `./_lib/...`.

Root:

```js
import { ... } from './_lib/rust-topology-prefer.mjs';
```

Skills:

```js
import { ... } from '../lib/rust-topology-prefer.mjs';
```

- [ ] **Step 3: Verify root/skills functional sync**

Run:

```bash
git diff --no-index _lib/rust-topology-prefer.mjs skills/lumin-repo-lens-lab/_engine/lib/rust-topology-prefer.mjs
```

Expected:

- no diff for the helper file.

For `measure-topology.mjs`, expect only path rewrites already normal for root vs skills.

## Task 8: Documentation And Review Packet

**Files:**
- Modify: `tests/README.md`
- Create review zip under `C:/Users/endof/Downloads/lumin-perf-lab/review/`

- [ ] **Step 1: Regenerate or update test docs**

Use the repo's existing test-doc workflow if available. If the workflow is not obvious, update `tests/README.md` manually with:

- `tests/rust-topology-prefer.test.mjs`
- the new prefer producer cases in `tests/topology-producer-cross-edges.test.mjs`

- [ ] **Step 2: Run final local validation**

Run:

```bash
npx vitest run tests/rust-topology-prefer.test.mjs tests/topology-producer-cross-edges.test.mjs tests/rust-topology-scanner-bridge.test.mjs tests/rust-topology-prefer-gate.test.mjs
node --check _lib/rust-topology-prefer.mjs
node --check measure-topology.mjs
node --check tests/rust-topology-prefer.test.mjs
node --check tests/topology-producer-cross-edges.test.mjs
git diff --check
```

Expected:

- all commands pass.

- [ ] **Step 3: Confirm private CI remains unused**

Before opening the PR:

```bash
git status -sb
```

After opening the draft PR:

```bash
gh pr view <number> --repo annyeong844/lumin_lab --json isDraft,statusCheckRollup
```

Expected:

- draft PR;
- `statusCheckRollup: []`.

- [ ] **Step 4: Build review packet**

Create a zip under:

```text
C:/Users/endof/Downloads/lumin-perf-lab/review/
```

Include:

- `docs/lab/m5-rust-topology-prefer-design-2026-06-15.md`
- `docs/superpowers/plans/2026-06-16-rust-topology-prefer.md`
- `_lib/rust-topology-prefer.mjs`
- `_lib/rust-topology-scanner.mjs`
- `_lib/rust-topology-prefer-gate.mjs`
- `measure-topology.mjs`
- `tests/rust-topology-prefer.test.mjs`
- `tests/topology-producer-cross-edges.test.mjs`
- `baselines/rust-topology-prefer-quorum.json`
- `baselines/m4-rust-topology-quorum-2026-06-15.md`

## Self-Review Checklist

- [ ] M5 keeps `prefer` explicit opt-in.
- [ ] `off` and `compare` behavior are unchanged.
- [ ] Initial `prefer` rejects incremental/cache-aware coverage.
- [ ] Initial `prefer` rejects non-required corpus scope.
- [ ] M3 gate `eligible` is required.
- [ ] Binary SHA-256 is recorded.
- [ ] Policy version uses `MODULE_EDGE_SCANNER_POLICY_VERSION`.
- [ ] Artifact guard runs on every prefer run.
- [ ] blocked prefer diagnostic metadata is visible for every blocked path.
- [ ] Tests are behavior-first and do not check mere file/function existence.
- [ ] Private CI is not triggered.

## Handoff

After this plan is accepted, implementation should be done in a draft PR. Do
not merge directly. The PR should stay private-source only unless package
surface changes require public lab package validation.
