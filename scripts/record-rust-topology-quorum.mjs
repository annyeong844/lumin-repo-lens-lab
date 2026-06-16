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
import { hashFileSha256 } from '../_lib/rust-topology-prefer.mjs';
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
    '--rust-topology-timeout-ms',
    String(timeoutMs),
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
if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
  throw new Error('--timeout-ms must be a positive number');
}

const labSourceCommit = values['lab-source-commit'] ?? runGit(['rev-parse', 'HEAD']);
const rustSidecarSourceCommit = values['rust-sidecar-source-commit'];
const rustSidecarBinary = path.resolve(repoRoot, values['rust-topology-scanner-bin']);
const rustSidecarBinarySha256 = hashFileSha256(rustSidecarBinary);
const rustSidecarRoot = path.join(repoRoot, 'experiments', 'rust-sidecar', 'topology-scanner');
const labWorkingTreeClean = runGit(['status', '--porcelain']) === '';
const rustSidecarWorkingTreeClean = runGit(['status', '--porcelain', '--', '.'], rustSidecarRoot) === '';
const quorumPath = path.resolve(repoRoot, values.quorum);
const outputRoot = path.resolve(repoRoot, values['output-root']);
const sourceDiagnostics = {
  workingTreeClean: labWorkingTreeClean && rustSidecarWorkingTreeClean,
  sourceDirty: !(labWorkingTreeClean && rustSidecarWorkingTreeClean),
  labWorkingTreeClean,
  rustSidecarWorkingTreeClean,
  labSourceCommit,
  rustSidecarSourceCommit,
};

const result = await recordRustTopologyQuorumBatch({
  corpus: values.corpus,
  root: values.root,
  allRequired: values['all-required'],
  corpusRoots: values['corpus-root'],
  rootsJson: values['roots-json'],
  repeat,
  quorumPath,
  outputRoot,
  rustSidecarBinary,
  rustSidecarBinarySha256,
  rustSidecarSourceCommit,
  labSourceCommit,
  machineOs: `${platform()} ${release()}`,
  timeoutMs,
  runner: measureTopologyRunner,
  sourceState: () => sourceDiagnostics,
});

const gateCheckRoot = result?.rootMap?.[values['gate-check-corpus']];
if (!gateCheckRoot) {
  throw new Error(`M3 gate verification needs a root for ${values['gate-check-corpus']}`);
}
const gateCheck = runGateCheck({
  root: gateCheckRoot,
  outputRoot,
  rustSidecarBinary,
  quorumPath,
  corpus: values['gate-check-corpus'],
  timeoutMs,
});

const summary = renderQuorumSummary({
  evidence: result?.evidence,
  gateCheck,
  commands: result?.commands ?? [],
});
writeTextAtomic(path.join(repoRoot, 'baselines/m4-rust-topology-quorum-2026-06-15.md'), summary);
console.log(`[rust-topology-quorum] updated ${quorumPath}`);
