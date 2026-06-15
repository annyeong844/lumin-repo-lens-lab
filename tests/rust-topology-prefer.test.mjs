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

const SIDE_CAR_COMMIT = '87116819c23d1e1adfbfca5def44552856e4f464';
const SIDE_CAR_SHA = 'sha256:abc';

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
    rustSidecarSourceCommit: SIDE_CAR_COMMIT,
    expectedRustSidecarSourceCommit: SIDE_CAR_COMMIT,
    rustSidecarBinarySha256: SIDE_CAR_SHA,
    expectedRustSidecarBinarySha256: SIDE_CAR_SHA,
    artifactGuard: { status: 'passed', passed: true },
    ...overrides,
  };
}

describe('Rust topology prefer decision', () => {
  it('uses Rust only for explicit prefer with eligible gate and passing artifact guard', () => {
    expect(evaluateRustTopologyPrefer(base())).toMatchObject({
      status: 'used-rust',
      usedRust: true,
      fallbackUsed: false,
      reason: 'gate-eligible-artifact-guard-passed',
      gateStatus: 'eligible',
      policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
      rustSidecarSourceCommit: SIDE_CAR_COMMIT,
      rustSidecarBinarySha256: SIDE_CAR_SHA,
    });
  });

  it('falls back visibly when prefer is requested with incremental cache coverage', () => {
    expect(evaluateRustTopologyPrefer(base({ isIncremental: true }))).toMatchObject({
      status: 'fallback-js',
      usedRust: false,
      fallbackUsed: true,
      reason: 'blocked-cache-mode',
    });
  });

  it('falls back visibly when current corpus is outside the fixed required set', () => {
    expect(evaluateRustTopologyPrefer(base({ currentCorpus: 'random-repo' }))).toMatchObject({
      status: 'fallback-js',
      reason: 'blocked-corpus-scope',
    });
  });

  it('falls back visibly when scanner comparison mismatches', () => {
    expect(evaluateRustTopologyPrefer(base({
      rustTopologyScanner: { ...matchedScanner, status: 'edge-mismatch', mismatches: 1 },
    }))).toMatchObject({
      status: 'fallback-js',
      reason: 'blocked-edge-mismatch',
    });
  });

  it('falls back visibly when source commit identity is not the quorum identity', () => {
    expect(evaluateRustTopologyPrefer(base({
      rustSidecarSourceCommit: 'different-commit',
    }))).toMatchObject({
      status: 'fallback-js',
      reason: 'blocked-sidecar-source-commit',
    });
  });

  it('falls back visibly when binary hash identity is not the approved identity', () => {
    expect(evaluateRustTopologyPrefer(base({
      rustSidecarBinarySha256: 'sha256:different',
    }))).toMatchObject({
      status: 'fallback-js',
      reason: 'blocked-sidecar-binary-sha256',
    });
  });

  it('falls back visibly when artifact guard fails', () => {
    expect(evaluateRustTopologyPrefer(base({
      artifactGuard: { status: 'failed', passed: false },
    }))).toMatchObject({
      status: 'fallback-js',
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
