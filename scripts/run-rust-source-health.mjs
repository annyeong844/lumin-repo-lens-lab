#!/usr/bin/env node
import { parseArgs } from 'node:util';

import { runRustSourceHealth } from '../_lib/rust-source-health-runner.mjs';

const { values } = parseArgs({
  options: {
    root: { type: 'string' },
    output: { type: 'string' },
    'rust-source-health-bin': { type: 'string' },
    'sidecar-source-commit': { type: 'string' },
    'timeout-ms': { type: 'string', default: '60000' },
    threads: { type: 'string' },
    'worker-stack-bytes': { type: 'string', default: '16777216' },
  },
});

if (!values.root) throw new Error('--root is required');
if (!values.output) throw new Error('--output is required');
if (!values['rust-source-health-bin']) {
  throw new Error('--rust-source-health-bin is required');
}
if (!values['sidecar-source-commit']) {
  throw new Error('--sidecar-source-commit is required');
}

const timeoutMs = Number(values['timeout-ms']);
if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
  throw new Error('--timeout-ms must be a positive number');
}
const threadCount = values.threads === undefined ? undefined : Number(values.threads);
if (
  threadCount !== undefined &&
  (!Number.isInteger(threadCount) || threadCount <= 0)
) {
  throw new Error('--threads must be a positive integer');
}
const workerStackBytes = Number(values['worker-stack-bytes']);
if (!Number.isInteger(workerStackBytes) || workerStackBytes < 16777216) {
  throw new Error('--worker-stack-bytes must be an integer >= 16777216');
}

const result = await runRustSourceHealth({
  root: values.root,
  output: values.output,
  binary: values['rust-source-health-bin'],
  sidecarSourceCommit: values['sidecar-source-commit'],
  timeoutMs,
  threadCount,
  workerStackBytes,
});

console.log(`[rust-source-health] wrote ${result.output}`);
console.log(
  `[rust-source-health] files=${result.artifact.summary.files} skipped=${result.artifact.summary.skippedFiles} signals=${result.artifact.summary.signals}`,
);

