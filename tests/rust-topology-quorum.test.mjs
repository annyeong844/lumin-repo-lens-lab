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
    expect(() => validateRunRecord(missingCollector)).toThrow(/collector/);
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
