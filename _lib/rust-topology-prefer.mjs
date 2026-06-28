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
  'blocked-quorum-invalid',
  'blocked-gate-missing',
  'blocked-gate-ineligible',
  'blocked-binary-not-found',
  'blocked-unsupported-platform',
  'blocked-timeout',
  'blocked-non-zero-exit',
  'blocked-invalid-json-output',
  'blocked-unsupported-file-type-or-syntax',
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
  ['unsupported-platform', 'blocked-unsupported-platform'],
  ['timeout', 'blocked-timeout'],
  ['non-zero-exit', 'blocked-non-zero-exit'],
  ['invalid-json-output', 'blocked-invalid-json-output'],
  ['unsupported-file-type-or-syntax', 'blocked-unsupported-file-type-or-syntax'],
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
  const normalized = globalThis.structuredClone(topology);
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

function sourceCommitMismatch({
  rustSidecarSourceCommit,
  expectedRustSidecarSourceCommit,
}) {
  if (!expectedRustSidecarSourceCommit) return true;
  if (!rustSidecarSourceCommit) return true;
  return rustSidecarSourceCommit !== expectedRustSidecarSourceCommit;
}

function binaryShaMismatch({
  rustSidecarBinarySha256,
  expectedRustSidecarBinarySha256,
}) {
  if (!expectedRustSidecarBinarySha256) return true;
  if (!rustSidecarBinarySha256) return true;
  return rustSidecarBinarySha256 !== expectedRustSidecarBinarySha256;
}

function hasMalformedMatchedScannerMetadata(rustTopologyScanner) {
  return rustTopologyScanner.mismatches !== 0 ||
    !Number.isFinite(rustTopologyScanner.filesCompared) ||
    rustTopologyScanner.filesCompared <= 0 ||
    rustTopologyScanner.policyVersion !== MODULE_EDGE_SCANNER_POLICY_VERSION;
}

function isPositiveInteger(value) {
  return Number.isInteger(value) && value > 0;
}

function hasFullCurrentCoverage({ currentFileCount, filesCompared }) {
  return isPositiveInteger(currentFileCount) &&
    isPositiveInteger(filesCompared) &&
    filesCompared === currentFileCount;
}

function hasValidPreferGateContract(rustTopologyPreferGate) {
  return rustTopologyPreferGate?.status === 'eligible' &&
    rustTopologyPreferGate.preferEnabled === false &&
    rustTopologyPreferGate.jsRemainsOracle === true;
}

export function evaluateRustTopologyPrefer({
  requested = false,
  mode = 'off',
  isIncremental = false,
  currentCorpus,
  rustTopologyScanner,
  rustTopologyPreferGate,
  currentFileCount,
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
    currentFileCount: currentFileCount ?? null,
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
  if (!rustTopologyScanner) return blocked({ reason: 'blocked-unknown-sidecar-status', base });
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
  if (rustTopologyScanner.policyVersion !== MODULE_EDGE_SCANNER_POLICY_VERSION) {
    return blocked({ reason: 'blocked-policy-version', base });
  }
  if (hasMalformedMatchedScannerMetadata(rustTopologyScanner)) {
    return blocked({ reason: 'blocked-unknown-sidecar-status', base });
  }
  if (!hasFullCurrentCoverage({
    currentFileCount,
    filesCompared: rustTopologyScanner.filesCompared,
  })) {
    return blocked({ reason: 'blocked-count-mismatch', base });
  }
  if (!rustTopologyPreferGate) return blocked({ reason: 'blocked-gate-missing', base });
  if (!hasValidPreferGateContract(rustTopologyPreferGate)) {
    const gateBlockReasonMap = new Map([
      ['quorum-evidence-missing', 'blocked-quorum-missing'],
      ['quorum-evidence-invalid', 'blocked-quorum-invalid'],
    ]);
    const reason = gateBlockReasonMap.get(rustTopologyPreferGate.reason) ??
      'blocked-gate-ineligible';
    return blocked({ reason, base });
  }
  if (sourceCommitMismatch({ rustSidecarSourceCommit, expectedRustSidecarSourceCommit })) {
    return blocked({ reason: 'blocked-sidecar-source-commit', base });
  }
  if (binaryShaMismatch({ rustSidecarBinarySha256, expectedRustSidecarBinarySha256 })) {
    return blocked({ reason: 'blocked-sidecar-binary-sha256', base });
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
