#!/usr/bin/env node
import { parseArgs } from 'node:util';

import { runRustSourceHealth } from '../_lib/rust-source-health-runner.mjs';

class UsageError extends Error {}

function requireValue(value, message) {
  if (!value) throw new UsageError(message);
}

async function main() {
  let parsed;
  try {
    parsed = parseArgs({
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
  } catch (error) {
    throw new UsageError(error?.message ?? String(error));
  }

  const { values } = parsed;
  requireValue(values.root, '--root is required');
  requireValue(values.output, '--output is required');
  requireValue(values['rust-source-health-bin'], '--rust-source-health-bin is required');
  requireValue(values['sidecar-source-commit'], '--sidecar-source-commit is required');

  const timeoutMs = Number(values['timeout-ms']);
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    throw new UsageError('--timeout-ms must be a positive number');
  }
  const threadCount = values.threads === undefined ? undefined : Number(values.threads);
  if (
    threadCount !== undefined &&
    (!Number.isInteger(threadCount) || threadCount <= 0)
  ) {
    throw new UsageError('--threads must be a positive integer');
  }
  const workerStackBytes = Number(values['worker-stack-bytes']);
  if (!Number.isInteger(workerStackBytes) || workerStackBytes < 16777216) {
    throw new UsageError('--worker-stack-bytes must be an integer >= 16777216');
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
}

main().catch((error) => {
  console.error(error?.stack ?? error?.message ?? String(error));
  process.exitCode = error instanceof UsageError ? 2 : 1;
});
