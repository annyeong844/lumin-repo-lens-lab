import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

import { MODULE_EDGE_SCANNER_POLICY_VERSION } from './js-module-edge-scanner.mjs';
import { REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA } from './rust-topology-prefer-gate.mjs';

export const RUST_TOPOLOGY_PREFER_STATUSES = Object.freeze([
  'not-requested',
  'used-rust',
  'fallback-js',
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

function fallback({ reason, base }) {
  return {
    ...base,
    status: 'fallback-js',
    usedRust: false,
    fallbackUsed: true,
    reason,
  };
}

function sourceCommitMismatch({
  rustSidecarSourceCommit,
  expectedRustSidecarSourceCommit,
}) {
  if (!expectedRustSidecarSourceCommit) return false;
  if (!rustSidecarSourceCommit) return true;
  return rustSidecarSourceCommit !== expectedRustSidecarSourceCommit;
}

function binaryShaMismatch({
  rustSidecarBinarySha256,
  expectedRustSidecarBinarySha256,
}) {
  if (!expectedRustSidecarBinarySha256) return false;
  if (!rustSidecarBinarySha256) return true;
  return rustSidecarBinarySha256 !== expectedRustSidecarBinarySha256;
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
  rustSidecarSourceCommit,
  expectedRustSidecarSourceCommit,
  rustSidecarBinarySha256,
  expectedRustSidecarBinarySha256,
  rustSidecarBuildProfile = 'release',
  artifactGuard,
} = {}) {
  const base = {
    schemaVersion: 1,
    requested,
    mode,
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    rustSidecarBinary: rustSidecarBinary ?? null,
    rustSidecarSourceCommit: rustSidecarSourceCommit ?? null,
    expectedRustSidecarSourceCommit: expectedRustSidecarSourceCommit ?? null,
    rustSidecarBinarySha256: rustSidecarBinarySha256 ?? null,
    expectedRustSidecarBinarySha256: expectedRustSidecarBinarySha256 ?? null,
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
      fallbackUsed: false,
      reason: 'not-requested',
    };
  }
  if (isIncremental) return fallback({ reason: 'blocked-cache-mode', base });
  if (!REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.includes(currentCorpus)) {
    return fallback({ reason: 'blocked-corpus-scope', base });
  }
  if (!rustTopologyScanner) return fallback({ reason: 'blocked-unknown-sidecar-status', base });
  if (
    rustTopologyScanner.policyVersion &&
    rustTopologyScanner.policyVersion !== MODULE_EDGE_SCANNER_POLICY_VERSION
  ) {
    return fallback({ reason: 'blocked-policy-version', base });
  }
  if (rustTopologyScanner.reason === 'policy-version-mismatch') {
    return fallback({ reason: 'blocked-policy-version', base });
  }
  if (rustTopologyScanner.status !== 'matched') {
    return fallback({
      reason: SCANNER_TO_PREFER_REASON.get(rustTopologyScanner.status) ??
        'blocked-unknown-sidecar-status',
      base,
    });
  }
  if ((rustTopologyScanner.mismatches ?? 0) !== 0) {
    return fallback({ reason: 'blocked-unknown-sidecar-status', base });
  }
  if (!rustTopologyPreferGate) return fallback({ reason: 'blocked-gate-missing', base });
  if (rustTopologyPreferGate.status !== 'eligible') {
    const reason = rustTopologyPreferGate.reason === 'quorum-evidence-missing'
      ? 'blocked-quorum-missing'
      : 'blocked-gate-ineligible';
    return fallback({ reason, base });
  }
  if (sourceCommitMismatch({ rustSidecarSourceCommit, expectedRustSidecarSourceCommit })) {
    return fallback({ reason: 'blocked-sidecar-source-commit', base });
  }
  if (binaryShaMismatch({ rustSidecarBinarySha256, expectedRustSidecarBinarySha256 })) {
    return fallback({ reason: 'blocked-sidecar-binary-sha256', base });
  }
  if (artifactGuard?.status !== 'passed') {
    return fallback({ reason: 'blocked-artifact-contract', base });
  }
  return {
    ...base,
    status: 'used-rust',
    usedRust: true,
    fallbackUsed: false,
    reason: 'gate-eligible-artifact-guard-passed',
  };
}
