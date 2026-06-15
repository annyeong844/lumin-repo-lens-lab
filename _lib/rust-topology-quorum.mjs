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
    ...(evidence ?? {}),
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
