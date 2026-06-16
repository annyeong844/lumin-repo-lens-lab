import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readdirSync,
  readFileSync,
} from 'node:fs';
import path from 'node:path';
import { TextDecoder } from 'node:util';

import { atomicWrite } from './atomic-write.mjs';
import {
  RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES,
  RUST_SOURCE_HEALTH_PARSER,
  RUST_SOURCE_HEALTH_SCHEMA_VERSION,
  sortRustHealthArtifact,
  summarizeRustHealthArtifact,
  validateRustHealthFinalArtifact,
  validateRustHealthSidecarArtifact,
} from './rust-source-health-schema.mjs';

const DEFAULT_INCLUDE = ['**/*.rs'];
const DEFAULT_EXCLUDE = ['target/**', 'vendor/**'];
const UTF8_DECODER = new TextDecoder('utf-8', { fatal: true });

function sha256Bytes(buffer) {
  return `sha256:${createHash('sha256').update(buffer).digest('hex')}`;
}

function hashFileSha256(filePath) {
  return sha256Bytes(readFileSync(filePath));
}

function toPosixRelative(relativePath) {
  return relativePath.split(path.sep).join('/');
}

function assertSafeRelativePath(relativePath) {
  if (
    relativePath.length === 0 ||
    path.isAbsolute(relativePath) ||
    relativePath.split('/').some((segment) => segment === '..')
  ) {
    throw new Error(`unsafe rust source health path: ${relativePath}`);
  }
}

export function hasPathSegment(relativePath, segment) {
  return (
    relativePath === segment ||
    relativePath.startsWith(`${segment}/`) ||
    relativePath.endsWith(`/${segment}`) ||
    relativePath.includes(`/${segment}/`)
  );
}

export function isExcludedByPathPolicy(relativePath) {
  return hasPathSegment(relativePath, 'target') ||
    hasPathSegment(relativePath, 'vendor');
}

function isRustSourcePath(relativePath) {
  return relativePath.endsWith('.rs');
}

function collectRustFiles({ rootAbs, dirAbs, files, skippedFiles }) {
  const entries = readdirSync(dirAbs, { withFileTypes: true })
    .sort((left, right) => left.name.localeCompare(right.name));

  for (const entry of entries) {
    const absolutePath = path.join(dirAbs, entry.name);
    const relativePath = toPosixRelative(path.relative(rootAbs, absolutePath));
    assertSafeRelativePath(relativePath);

    if (entry.isSymbolicLink()) continue;
    if (isExcludedByPathPolicy(relativePath)) continue;

    if (entry.isDirectory()) {
      collectRustFiles({ rootAbs, dirAbs: absolutePath, files, skippedFiles });
      continue;
    }

    if (!entry.isFile() || !isRustSourcePath(relativePath)) continue;

    const rawBytes = readFileSync(absolutePath);
    const sha256 = sha256Bytes(rawBytes);
    let text;
    try {
      text = UTF8_DECODER.decode(rawBytes);
    } catch {
      skippedFiles.push({ path: relativePath, reason: 'invalid-utf8' });
      continue;
    }

    files.push({ path: relativePath, sha256, text });
  }
}

export function collectRustSourceHealthInput({
  root,
  include = DEFAULT_INCLUDE,
  exclude = DEFAULT_EXCLUDE,
  threadCount,
  workerStackBytes = RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES,
} = {}) {
  if (!root) throw new Error('root is required');
  if (
    !Number.isInteger(workerStackBytes) ||
    workerStackBytes < RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES
  ) {
    throw new Error(
      `workerStackBytes must be an integer >= ${RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES}`,
    );
  }
  if (
    threadCount !== undefined &&
    (!Number.isInteger(threadCount) || threadCount <= 0)
  ) {
    throw new Error('threadCount must be a positive integer when provided');
  }

  const rootAbs = path.resolve(root);
  const rootStat = lstatSync(rootAbs);
  if (!rootStat.isDirectory()) {
    throw new Error(`rust source health root is not a directory: ${rootAbs}`);
  }

  const files = [];
  const skippedFiles = [];
  collectRustFiles({ rootAbs, dirAbs: rootAbs, files, skippedFiles });

  const pathPolicy = { include, exclude };
  const runtime = { workerStackBytes };
  if (threadCount !== undefined) runtime.threadCount = threadCount;

  return {
    input: {
      schemaVersion: RUST_SOURCE_HEALTH_SCHEMA_VERSION,
      root: rootAbs,
      files,
      pathPolicy,
      parser: {
        editionPolicy: RUST_SOURCE_HEALTH_PARSER.editionPolicy,
        edition: RUST_SOURCE_HEALTH_PARSER.edition,
        editionSource: RUST_SOURCE_HEALTH_PARSER.editionSource,
      },
      runtime,
    },
    skippedFiles: skippedFiles.sort((left, right) =>
      left.path.localeCompare(right.path),
    ),
    pathPolicy,
  };
}

export function runRustSourceHealthSidecar({ binary, input, timeoutMs = 60000 } = {}) {
  if (!binary) throw new Error('rust source health binary is required');
  if (!existsSync(binary)) {
    throw new Error(`rust source health binary not found: ${binary}`);
  }
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    throw new Error('timeoutMs must be a positive number');
  }

  const shouldUseShell =
    process.platform === 'win32' && /\.(cmd|bat)$/i.test(binary);
  const result = spawnSync(binary, [], {
    input: JSON.stringify(input),
    encoding: 'utf8',
    maxBuffer: 64 * 1024 * 1024,
    shell: shouldUseShell,
    timeout: timeoutMs,
    windowsHide: true,
  });

  if (result.error) {
    if (result.error.code === 'ETIMEDOUT') {
      throw new Error(`rust source health sidecar timed out after ${timeoutMs}ms`);
    }
    throw result.error;
  }
  if (result.status !== 0) {
    const stderr = String(result.stderr ?? '').trim();
    throw new Error(
      `rust source health sidecar exited ${result.status}${stderr ? `: ${stderr}` : ''}`,
    );
  }

  let artifact;
  try {
    artifact = JSON.parse(String(result.stdout ?? ''));
  } catch (error) {
    throw new Error(`invalid rust source health sidecar JSON: ${error.message}`);
  }

  const problems = validateRustHealthSidecarArtifact(artifact);
  if (problems.length > 0) {
    throw new Error(`invalid rust source health sidecar artifact: ${problems.join('; ')}`);
  }
  return artifact;
}

export function buildFinalRustHealthArtifact({
  sidecarArtifact,
  skippedFiles = [],
  pathPolicy,
  sidecarSourceCommit,
  binarySha256,
} = {}) {
  if (!sidecarSourceCommit) {
    throw new Error('sidecarSourceCommit is required');
  }
  const artifact = sortRustHealthArtifact({
    ...sidecarArtifact,
    skippedFiles: [...(sidecarArtifact?.skippedFiles ?? []), ...skippedFiles],
    meta: {
      ...(sidecarArtifact?.meta ?? {}),
      generated: new Date().toISOString(),
      sidecar: { sourceCommit: sidecarSourceCommit, binarySha256 },
      input: { pathPolicy },
    },
  });

  artifact.summary = summarizeRustHealthArtifact(artifact);

  const problems = validateRustHealthFinalArtifact(artifact);
  if (problems.length > 0) {
    throw new Error(`invalid final rust-health artifact: ${problems.join('; ')}`);
  }
  return artifact;
}

export function writeRustHealthArtifact({ output, artifact } = {}) {
  if (!output) throw new Error('output is required');
  const outputPath = path.resolve(output);
  mkdirSync(path.dirname(outputPath), { recursive: true });
  atomicWrite(outputPath, `${JSON.stringify(artifact, null, 2)}\n`);
  return outputPath;
}

export async function runRustSourceHealth({
  root,
  output,
  binary,
  sidecarSourceCommit,
  timeoutMs = 60000,
  threadCount,
  workerStackBytes = RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES,
} = {}) {
  const binaryPath = path.resolve(binary);
  const binarySha256 = hashFileSha256(binaryPath);
  const { input, skippedFiles, pathPolicy } = collectRustSourceHealthInput({
    root,
    threadCount,
    workerStackBytes,
  });
  const sidecarArtifact = runRustSourceHealthSidecar({
    binary: binaryPath,
    input,
    timeoutMs,
  });
  const artifact = buildFinalRustHealthArtifact({
    sidecarArtifact,
    skippedFiles,
    pathPolicy,
    sidecarSourceCommit,
    binarySha256,
  });
  const outputPath = writeRustHealthArtifact({ output, artifact });
  return { output: outputPath, artifact, input, skippedFiles };
}
