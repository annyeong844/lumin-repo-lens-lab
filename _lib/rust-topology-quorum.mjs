import { existsSync, mkdirSync } from 'node:fs';
import path from 'node:path';

import { atomicWrite } from './atomic-write.mjs';
import { readJsonFile } from './artifacts.mjs';
import { MODULE_EDGE_SCANNER_POLICY_VERSION } from './js-module-edge-scanner.mjs';
import {
  REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA,
  RUST_TOPOLOGY_PREFER_QUORUM_SCHEMA_VERSION,
  RUST_TOPOLOGY_PREFER_QUORUM_PATH,
} from './rust-topology-prefer-gate.mjs';

export const QUORUM_SCHEMA_VERSION = RUST_TOPOLOGY_PREFER_QUORUM_SCHEMA_VERSION;
export const DEFAULT_M4_QUORUM_OUTPUT_ROOT = 'baselines/m4-rust-topology-quorum';

const REQUIRED_RUN_FIELDS = [
  'labSourceCommit',
  'rustSidecarSourceCommit',
  'rustSidecarBinary',
  'rustSidecarBinarySha256',
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

function isPositiveInteger(value) {
  return Number.isInteger(value) && value > 0;
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
  if (!isPositiveInteger(record.fileCount)) {
    throw new Error(`run record fileCount must be a positive integer: ${record.fileCount}`);
  }
  if (!isPositiveInteger(record.filesCompared)) {
    throw new Error(`run record filesCompared must be a positive integer: ${record.filesCompared}`);
  }
  if (record.filesCompared !== record.fileCount) {
    throw new Error('run record filesCompared must equal fileCount for M5 full-coverage quorum');
  }
  if (!record.collector || typeof record.collector !== 'object') {
    throw new Error('run record missing collector source diagnostics');
  }
  for (const field of [
    'workingTreeClean',
    'sourceDirty',
    'labWorkingTreeClean',
    'rustSidecarWorkingTreeClean',
  ]) {
    if (typeof record.collector[field] !== 'boolean') {
      throw new Error(`collector source diagnostic must be boolean: ${field}`);
    }
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
  if (evidence.policyVersion !== MODULE_EDGE_SCANNER_POLICY_VERSION) {
    throw new Error(`quorum evidence policyVersion mismatch: ${evidence.policyVersion}`);
  }
  if (!evidence.rustSidecarBinarySha256) {
    throw new Error('quorum evidence missing rustSidecarBinarySha256');
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
  if (checked.rustSidecarBinarySha256 !== base.rustSidecarBinarySha256) {
    throw new Error('run record rustSidecarBinarySha256 differs from quorum evidence');
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
  rustSidecarBinarySha256,
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
    rustSidecarBinarySha256,
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

export function readOrCreateQuorumEvidence(filePath, rustSidecarSourceCommit, rustSidecarBinarySha256) {
  return readQuorumEvidence(filePath) ?? defaultEvidence(rustSidecarSourceCommit, rustSidecarBinarySha256);
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

export function defaultEvidence(rustSidecarSourceCommit, rustSidecarBinarySha256) {
  return {
    schemaVersion: QUORUM_SCHEMA_VERSION,
    requiredCorpora: [...REQUIRED_RUST_TOPOLOGY_PREFER_CORPORA],
    policyVersion: MODULE_EDGE_SCANNER_POLICY_VERSION,
    rustSidecarSourceCommit,
    rustSidecarBinarySha256,
    runs: {},
  };
}

function nextRunOutputDir(outputRoot, corpus, runIndex) {
  return path.join(outputRoot, corpus, `run-${String(runIndex).padStart(3, '0')}`);
}

function isSummaryCleanRun(run, evidence) {
  const collector = run?.collector;
  return (
    run?.sidecarStatus === 'matched' &&
    run?.mismatches === 0 &&
    run?.cacheMode === 'no-incremental' &&
    run?.rustSidecarSourceCommit === evidence?.rustSidecarSourceCommit &&
    run?.rustSidecarBinarySha256 === evidence?.rustSidecarBinarySha256 &&
    run?.policyVersion === evidence?.policyVersion &&
    isPositiveInteger(run?.fileCount) &&
    isPositiveInteger(run?.filesCompared) &&
    run.filesCompared === run.fileCount &&
    collector?.sourceDirty === false &&
    collector?.workingTreeClean === true &&
    collector?.labWorkingTreeClean === true &&
    collector?.rustSidecarWorkingTreeClean === true
  );
}

function latestThreeStatus(runs = [], evidence) {
  const recent = runs.slice(-3);
  const clean = recent.length === 3 && recent.every((run) => isSummaryCleanRun(run, evidence));
  return clean ? 'clean' : 'incomplete';
}

export async function recordRustTopologyQuorum({
  corpus,
  root,
  quorumPath = RUST_TOPOLOGY_PREFER_QUORUM_PATH,
  outputRoot = DEFAULT_M4_QUORUM_OUTPUT_ROOT,
  rustSidecarBinary,
  rustSidecarBinarySha256,
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
  const evidence = readOrCreateQuorumEvidence(
    quorumPath,
    rustSidecarSourceCommit,
    rustSidecarBinarySha256,
  );
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
    '--rust-topology-timeout-ms',
    String(timeoutMs),
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
    rustSidecarBinarySha256,
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
    lines.push(`| \`${corpus}\` | ${runs.length} | ${latestThreeStatus(runs, evidence)} | ${last.filesCompared ?? 0} | ${last.mismatches ?? 0} | ${last.commandWallElapsedMs ?? 0} | ${last.scannerBridgeElapsedMs ?? 0} | ${last.sidecarElapsedMs ?? 0} |`);
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
