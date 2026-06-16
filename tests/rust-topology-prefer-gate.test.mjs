import { describe, expect, it } from 'vitest';

import {
  evaluateRustTopologyPreferGate,
  REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA,
  RUST_TOPOLOGY_PREFER_QUORUM_PATH,
} from '../_lib/rust-topology-prefer-gate.mjs';
import { MODULE_EDGE_SCANNER_POLICY_VERSION } from '../_lib/js-module-edge-scanner.mjs';

const REQUIRED_CORPORA = REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA;

function cleanRun(corpus, index = 0) {
  return {
    labSourceCommit: `lab-${index}`,
    rustSidecarSourceCommit: '87116819c23d1e1adfbfca5def44552856e4f464',
    rustSidecarBinary: 'experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe',
    rustSidecarBinarySha256: 'sha256:abc',
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

function cleanQuorum() {
  return {
    schemaVersion: 1,
    requiredCorpora: REQUIRED_CORPORA,
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    rustSidecarSourceCommit: '87116819c23d1e1adfbfca5def44552856e4f464',
    rustSidecarBinarySha256: 'sha256:abc',
    runs: Object.fromEntries(
      REQUIRED_CORPORA.map((corpus) => [
        corpus,
        [cleanRun(corpus, 0), cleanRun(corpus, 1), cleanRun(corpus, 2)],
      ]),
    ),
  };
}

function matchedScanner(overrides = {}) {
  return {
    attempted: true,
    mode: 'compare',
    status: 'matched',
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    filesCompared: 701,
    mismatches: 0,
    mismatchSamples: [],
    sidecarTiming: { files: 701, elapsedMs: 559 },
    ...overrides,
  };
}

describe('Rust topology prefer gate', () => {
  it('marks a matched compare run eligible only when all required corpora have three clean no-incremental runs', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: cleanQuorum(),
    });

    expect(gate).toMatchObject({
      status: 'eligible',
      mode: 'compare',
      scope: 'run',
      preferEnabled: false,
      jsRemainsOracle: true,
      reason: 'all-required-corpora-matched',
      requiredCorpora: REQUIRED_CORPORA,
      currentCorpus: 'lab-self',
      currentCorpusSource: 'cli',
      quorumEvidence: RUST_TOPOLOGY_PREFER_QUORUM_PATH,
      cacheMode: 'no-incremental',
      mismatches: 0,
      filesCompared: 701,
      sidecarStatus: 'matched',
      policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
      sidecarPolicyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    });
  });

  it('blocks compare runs when the current corpus is not declared explicitly', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: cleanQuorum(),
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('current-corpus-not-declared');
  });

  it('blocks when the current corpus is not one of the fixed required corpora', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'random-repo',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: cleanQuorum(),
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('current-corpus-not-required');
  });

  it('maps bridge mismatch and failure states to exact gate statuses', () => {
    const cases = [
      ['count-mismatch', 'blocked-count-mismatch'],
      ['edge-mismatch', 'blocked-edge-mismatch'],
      ['risk-mismatch', 'blocked-risk-mismatch'],
      ['timeout', 'blocked-sidecar-failure'],
      ['non-zero-exit', 'blocked-sidecar-failure'],
      ['binary-not-found', 'blocked-sidecar-failure'],
      ['invalid-json-output', 'blocked-sidecar-failure'],
    ];

    for (const [scannerStatus, expectedGateStatus] of cases) {
      const gate = evaluateRustTopologyPreferGate({
        mode: 'compare',
        currentCorpus: 'lab-self',
        rustTopologyScanner: matchedScanner({
          status: scannerStatus,
          mismatches: scannerStatus.endsWith('mismatch') ? 1 : 0,
        }),
        quorumEvidence: cleanQuorum(),
      });

      expect(gate.status).toBe(expectedGateStatus);
    }
  });

  it('blocks unknown scanner statuses instead of treating quorum as enough', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner({ status: 'surprise-status' }),
      quorumEvidence: cleanQuorum(),
    });

    expect(gate.status).toBe('blocked-sidecar-failure');
    expect(gate.reason).toBe('unknown-sidecar-status');
  });

  it('blocks inconsistent matched scanner metadata with nonzero mismatches', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner({ mismatches: 1 }),
      quorumEvidence: cleanQuorum(),
    });

    expect(gate.status).toBe('blocked-sidecar-failure');
    expect(gate.reason).toBe('matched-status-with-nonzero-mismatches');
  });

  it('maps bridge policy mismatch reason to blocked-policy-version', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner({
        status: 'invalid-json-output',
        reason: 'policy-version-mismatch',
        rustPolicyVersion: 'wrong-policy',
      }),
      quorumEvidence: cleanQuorum(),
    });

    expect(gate.status).toBe('blocked-policy-version');
    expect(gate.reason).toBe('policy-version-mismatch');
    expect(gate.sidecarPolicyVersion).toBe('wrong-policy');
  });

  it('blocks when quorum evidence was recorded with a different scanner policy', () => {
    const quorum = cleanQuorum();
    quorum.policyVersion = 'old-policy';

    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: quorum,
    });

    expect(gate.status).toBe('blocked-policy-version');
    expect(gate.reason).toBe('quorum-policy-version-mismatch');
  });

  it('blocks when matched scanner metadata is missing policyVersion', () => {
    const scanner = matchedScanner();
    delete scanner.policyVersion;

    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: scanner,
      quorumEvidence: cleanQuorum(),
    });

    expect(gate.status).toBe('blocked-policy-version');
    expect(gate.reason).toBe('policy-version-mismatch');
  });

  it('blocks when quorum evidence is missing top-level policyVersion', () => {
    const quorum = cleanQuorum();
    delete quorum.policyVersion;

    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: quorum,
    });

    expect(gate.status).toBe('blocked-policy-version');
    expect(gate.reason).toBe('quorum-policy-version-mismatch');
  });

  it('blocks when quorum evidence has no approved binary sha256', () => {
    const quorum = cleanQuorum();
    delete quorum.rustSidecarBinarySha256;

    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: quorum,
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('quorum-binary-sha-missing');
  });

  it('blocks when quorum schemaVersion is missing or unsupported', () => {
    for (const schemaVersion of [undefined, 999]) {
      const quorum = cleanQuorum();
      if (schemaVersion === undefined) delete quorum.schemaVersion;
      else quorum.schemaVersion = schemaVersion;

      const gate = evaluateRustTopologyPreferGate({
        mode: 'prefer',
        currentCorpus: 'lab-self',
        rustTopologyScanner: matchedScanner(),
        quorumEvidence: quorum,
      });

      expect(gate.status).toBe('blocked-corpus-quorum');
      expect(gate.reason).toBe('quorum-schema-version-mismatch');
    }
  });

  it('blocks quorum runs that are matched but not full coverage', () => {
    const quorum = cleanQuorum();
    for (const runs of Object.values(quorum.runs)) {
      for (const run of runs) {
        run.fileCount = 1000;
        run.filesCompared = 1;
      }
    }

    const gate = evaluateRustTopologyPreferGate({
      mode: 'prefer',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: quorum,
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('required-corpus-history-incomplete');
    expect(gate.incompleteCorpora).toEqual(REQUIRED_CORPORA);
  });

  it('blocks quorum runs recorded with a different binary sha256', () => {
    const quorum = cleanQuorum();
    quorum.rustSidecarBinarySha256 = 'sha256:new-approved';
    for (const runs of Object.values(quorum.runs)) {
      for (const run of runs) run.rustSidecarBinarySha256 = 'sha256:old-recorded';
    }

    const gate = evaluateRustTopologyPreferGate({
      mode: 'prefer',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: quorum,
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('required-corpus-history-incomplete');
    expect(gate.incompleteCorpora).toEqual(REQUIRED_CORPORA);
  });

  it('blocks matched runs when quorum evidence is incomplete', () => {
    const quorum = cleanQuorum();
    quorum.runs['nuxt-main'] = [cleanRun('nuxt-main', 0), cleanRun('nuxt-main', 1)];

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

  it('does not let quorum evidence shrink the fixed required corpus set', () => {
    const quorum = cleanQuorum();
    quorum.requiredCorpora = ['lab-self'];
    quorum.runs = {
      'lab-self': [cleanRun('lab-self', 0), cleanRun('lab-self', 1), cleanRun('lab-self', 2)],
    };

    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
      quorumEvidence: quorum,
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('required-corpora-not-declared');
    expect(gate.missingRequiredCorpora).toEqual([
      'geulbat-phase1',
      'stable-source-clean',
      'nuxt-main',
    ]);
  });

  it('does not count quorum runs with missing audit evidence fields as clean', () => {
    const quorum = cleanQuorum();
    delete quorum.runs['nuxt-main'][2].machineOs;

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

  it('blocks when clean quorum runs are not consecutive in the latest history', () => {
    const quorum = cleanQuorum();
    quorum.runs['nuxt-main'] = [
      cleanRun('nuxt-main', 0),
      cleanRun('nuxt-main', 1),
      {
        ...cleanRun('nuxt-main', 2),
        mismatches: 1,
        sidecarStatus: 'risk-mismatch',
      },
      cleanRun('nuxt-main', 3),
    ];

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

  it('blocks matched runs when quorum evidence is missing', () => {
    const gate = evaluateRustTopologyPreferGate({
      mode: 'compare',
      currentCorpus: 'lab-self',
      rustTopologyScanner: matchedScanner(),
    });

    expect(gate.status).toBe('blocked-corpus-quorum');
    expect(gate.reason).toBe('quorum-evidence-missing');
  });
});
