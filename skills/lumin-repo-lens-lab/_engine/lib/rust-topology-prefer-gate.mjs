import { MODULE_EDGE_SCANNER_POLICY_VERSION } from './js-module-edge-scanner.mjs';
import { readJsonFile } from './artifacts.mjs';

export const RUST_TOPOLOGY_PREFER_QUORUM_PATH = 'baselines/rust-topology-prefer-quorum.json';
export const REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA = [
  'geulbat-phase1',
  'lab-self',
  'stable-source-clean',
  'nuxt-main',
];

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
  'wrapperElapsedMs',
  'sidecarElapsedMs',
  'sidecarStatus',
  'policyVersion',
  'machineOs',
];

const SIDECAR_FAILURE_STATUSES = new Set([
  'binary-not-found',
  'unsupported-platform',
  'timeout',
  'non-zero-exit',
  'invalid-json-output',
  'unsupported-file-type-or-syntax',
]);

function baseGate({
  mode,
  currentCorpus,
  rustTopologyScanner,
  quorumEvidence,
  policyVersion,
  status,
  reason,
  extra = {},
}) {
  const sidecarPolicyVersion = rustTopologyScanner?.rustPolicyVersion
    ?? rustTopologyScanner?.policyVersion
    ?? null;
  return {
    status,
    mode,
    scope: 'run',
    preferEnabled: false,
    jsRemainsOracle: true,
    reason,
    requiredCorpora: [...REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA],
    currentCorpus,
    currentCorpusSource: currentCorpus ? 'cli' : null,
    quorumEvidence: RUST_TOPOLOGY_PREFER_QUORUM_PATH,
    cacheMode: extra.cacheMode ?? null,
    mismatches: rustTopologyScanner?.mismatches ?? 0,
    filesCompared: rustTopologyScanner?.filesCompared ?? 0,
    sidecarStatus: rustTopologyScanner?.status ?? null,
    policyVersion,
    sidecarPolicyVersion,
    quorumEvidencePath: extra.quorumEvidencePath ?? RUST_TOPOLOGY_PREFER_QUORUM_PATH,
    ...extra,
  };
}

export function readRustTopologyPreferQuorum(filePath) {
  return readJsonFile(filePath, {
    tag: 'rust-topology-prefer-gate',
    strict: true,
  });
}

function hasRequiredRunFields(run) {
  return REQUIRED_QUORUM_RUN_FIELDS.every((field) => {
    const value = run?.[field];
    return value !== undefined && value !== null && value !== '';
  });
}

function cleanRunMatches(run, rustSidecarSourceCommit, policyVersion) {
  return (
    hasRequiredRunFields(run) &&
    run?.rustSidecarSourceCommit === rustSidecarSourceCommit &&
    run?.cacheMode === 'no-incremental' &&
    run?.mismatches === 0 &&
    run?.sidecarStatus === 'matched' &&
    run?.policyVersion === policyVersion
  );
}

function incompleteRequiredCorpora(quorumEvidence, policyVersion) {
  const sourceCommit = quorumEvidence?.rustSidecarSourceCommit;
  const runs = quorumEvidence?.runs ?? {};
  return REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.filter((corpus) => {
    const recentRuns = (Array.isArray(runs[corpus]) ? runs[corpus] : []).slice(-3);
    return recentRuns.length < 3 ||
      !recentRuns.every((run) => cleanRunMatches(run, sourceCommit, policyVersion));
  });
}

function cacheModeForCurrentCorpus(quorumEvidence, currentCorpus, policyVersion) {
  const sourceCommit = quorumEvidence?.rustSidecarSourceCommit;
  const runs = quorumEvidence?.runs?.[currentCorpus] ?? [];
  const cleanRun = runs.find((run) => cleanRunMatches(run, sourceCommit, policyVersion));
  return cleanRun?.cacheMode ?? null;
}

function missingConfiguredRequiredCorpora(quorumEvidence) {
  const declared = new Set(quorumEvidence?.requiredCorpora ?? []);
  return REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA
    .filter((corpus) => !declared.has(corpus));
}

export function evaluateRustTopologyPreferGate({
  mode = 'off',
  currentCorpus,
  rustTopologyScanner,
  quorumEvidence,
  quorumEvidencePath = RUST_TOPOLOGY_PREFER_QUORUM_PATH,
  policyVersion = MODULE_EDGE_SCANNER_POLICY_VERSION,
} = {}) {
  const gateExtra = { quorumEvidencePath };
  if (mode === 'off' || !rustTopologyScanner) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-mode-off',
      reason: 'rust-topology-scanner-off',
      extra: gateExtra,
    });
  }

  if (
    rustTopologyScanner.status === 'invalid-json-output' &&
    rustTopologyScanner.reason === 'policy-version-mismatch'
  ) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-policy-version',
      reason: 'policy-version-mismatch',
      extra: gateExtra,
    });
  }

  if (
    rustTopologyScanner.policyVersion &&
    rustTopologyScanner.policyVersion !== policyVersion
  ) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-policy-version',
      reason: 'policy-version-mismatch',
      extra: gateExtra,
    });
  }

  const mismatchStatusMap = {
    'count-mismatch': 'blocked-count-mismatch',
    'edge-mismatch': 'blocked-edge-mismatch',
    'risk-mismatch': 'blocked-risk-mismatch',
  };
  if (mismatchStatusMap[rustTopologyScanner.status]) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: mismatchStatusMap[rustTopologyScanner.status],
      reason: rustTopologyScanner.status,
      extra: gateExtra,
    });
  }

  if (SIDECAR_FAILURE_STATUSES.has(rustTopologyScanner.status)) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-sidecar-failure',
      reason: rustTopologyScanner.status,
      extra: gateExtra,
    });
  }

  if (rustTopologyScanner.status !== 'matched') {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-sidecar-failure',
      reason: 'unknown-sidecar-status',
      extra: gateExtra,
    });
  }

  if ((rustTopologyScanner.mismatches ?? 0) !== 0) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-sidecar-failure',
      reason: 'matched-status-with-nonzero-mismatches',
      extra: gateExtra,
    });
  }

  if (!currentCorpus) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-corpus-quorum',
      reason: 'current-corpus-not-declared',
      extra: gateExtra,
    });
  }

  if (!REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA.includes(currentCorpus)) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-corpus-quorum',
      reason: 'current-corpus-not-required',
      extra: gateExtra,
    });
  }

  if (
    !quorumEvidence ||
    !Array.isArray(quorumEvidence.requiredCorpora) ||
    quorumEvidence.requiredCorpora.length === 0
  ) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-corpus-quorum',
      reason: 'quorum-evidence-missing',
      extra: gateExtra,
    });
  }

  if (
    quorumEvidence.policyVersion &&
    quorumEvidence.policyVersion !== policyVersion
  ) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-policy-version',
      reason: 'quorum-policy-version-mismatch',
      extra: gateExtra,
    });
  }

  const missingRequiredCorpora = missingConfiguredRequiredCorpora(quorumEvidence);
  if (missingRequiredCorpora.length > 0) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-corpus-quorum',
      reason: 'required-corpora-not-declared',
      extra: { ...gateExtra, missingRequiredCorpora },
    });
  }

  const incompleteCorpora = incompleteRequiredCorpora(quorumEvidence, policyVersion);
  const cacheMode = cacheModeForCurrentCorpus(quorumEvidence, currentCorpus, policyVersion);
  if (incompleteCorpora.length > 0) {
    return baseGate({
      mode,
      currentCorpus,
      rustTopologyScanner,
      quorumEvidence,
      policyVersion,
      status: 'blocked-corpus-quorum',
      reason: 'required-corpus-history-incomplete',
      extra: { ...gateExtra, incompleteCorpora, cacheMode },
    });
  }

  return baseGate({
    mode,
    currentCorpus,
    rustTopologyScanner,
    quorumEvidence,
    policyVersion,
    status: 'eligible',
    reason: 'all-required-corpora-matched',
    extra: { ...gateExtra, cacheMode },
  });
}
