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
  RUST_SOURCE_HEALTH_DEFAULT_EXCLUDE,
  RUST_SOURCE_HEALTH_DEFAULT_INCLUDE,
  RUST_SOURCE_HEALTH_PARSER,
  RUST_SOURCE_HEALTH_SCHEMA_VERSION,
  sortRustHealthArtifact,
  summarizeRustHealthArtifact,
  validateRustHealthFinalArtifact,
  validateRustHealthSidecarArtifact,
} from './rust-source-health-schema.mjs';

export class RustSourceHealthConfigError extends Error {}

const DEFAULT_INCLUDE = RUST_SOURCE_HEALTH_DEFAULT_INCLUDE;
const DEFAULT_EXCLUDE = RUST_SOURCE_HEALTH_DEFAULT_EXCLUDE;
const UTF8_DECODER = new TextDecoder('utf-8', {
  fatal: true,
  ignoreBOM: true,
});

function compareCodeUnit(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

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
    relativePath.startsWith('\\') ||
    relativePath.includes('\\') ||
    relativePath.includes(':') ||
    relativePath.split('/').some((segment) =>
      segment.length === 0 || segment === '.' || segment === '..'
    )
  ) {
    throw new RustSourceHealthConfigError(`unsafe rust source health path: ${relativePath}`);
  }
}

function sameStringList(left, right) {
  return Array.isArray(left) &&
    Array.isArray(right) &&
    left.length === right.length &&
    left.every((value, index) => value === right[index]);
}

function assertDefaultPathPolicy({ include, exclude }) {
  if (
    !sameStringList(include, DEFAULT_INCLUDE) ||
    !sameStringList(exclude, DEFAULT_EXCLUDE)
  ) {
    throw new RustSourceHealthConfigError(
      'custom rust source health path policy is not supported yet',
    );
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
    .sort((left, right) => compareCodeUnit(left.name, right.name));

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
  if (!root) throw new RustSourceHealthConfigError('root is required');
  assertDefaultPathPolicy({ include, exclude });
  if (
    !Number.isInteger(workerStackBytes) ||
    workerStackBytes < RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES
  ) {
    throw new RustSourceHealthConfigError(
      `workerStackBytes must be an integer >= ${RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES}`,
    );
  }
  if (
    threadCount !== undefined &&
    (!Number.isInteger(threadCount) || threadCount <= 0)
  ) {
    throw new RustSourceHealthConfigError(
      'threadCount must be a positive integer when provided',
    );
  }

  const rootAbs = path.resolve(root);
  if (!existsSync(rootAbs)) {
    throw new RustSourceHealthConfigError(
      `rust source health root not found: ${rootAbs}`,
    );
  }
  const rootStat = lstatSync(rootAbs);
  if (!rootStat.isDirectory()) {
    throw new RustSourceHealthConfigError(
      `rust source health root is not a directory: ${rootAbs}`,
    );
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
      compareCodeUnit(left.path, right.path),
    ),
    pathPolicy,
  };
}

export function runRustSourceHealthSidecar({ binary, input, timeoutMs = 60000 } = {}) {
  if (!binary) throw new RustSourceHealthConfigError('rust source health binary is required');
  if (!existsSync(binary)) {
    throw new RustSourceHealthConfigError(`rust source health binary not found: ${binary}`);
  }
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    throw new RustSourceHealthConfigError('timeoutMs must be a positive number');
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
  const coverageProblems = sidecarCoverageProblems({ artifact, input });
  if (coverageProblems.length > 0) {
    throw new Error(
      `invalid rust source health sidecar coverage: ${coverageProblems.join('; ')}`,
    );
  }
  return artifact;
}

function sidecarCoverageProblems({ artifact, input }) {
  const problems = [];
  const expected = new Map(
    (input?.files ?? []).map((file) => [file.path, file.sha256]),
  );
  const actual = artifact?.files ?? {};
  const expectedPaths = [...expected.keys()].sort();
  const actualPaths = Object.keys(actual).sort();

  if (JSON.stringify(actualPaths) !== JSON.stringify(expectedPaths)) {
    problems.push(
      `sidecar file coverage mismatch: expected ${expectedPaths.length} files but found ${actualPaths.length}`,
    );
  }
  const actualPathSet = new Set(actualPaths);
  const expectedPathSet = new Set(expectedPaths);
  const missingPaths = expectedPaths.filter((filePath) => !actualPathSet.has(filePath));
  const extraPaths = actualPaths.filter((filePath) => !expectedPathSet.has(filePath));
  if (missingPaths.length > 0) {
    problems.push(`sidecar missing files: ${missingPaths.join(', ')}`);
  }
  if (extraPaths.length > 0) {
    problems.push(`sidecar returned unexpected files: ${extraPaths.join(', ')}`);
  }

  for (const [filePath, expectedSha] of expected) {
    if (actual[filePath]?.sha256 !== expectedSha) {
      problems.push(`sidecar sha256 mismatch for ${filePath}`);
    }
  }

  if ((artifact?.skippedFiles ?? []).length !== 0) {
    problems.push('sidecar skippedFiles must be empty');
  }

  return problems;
}

export function buildFinalRustHealthArtifact({
  sidecarArtifact,
  skippedFiles = [],
  pathPolicy,
  sidecarSourceCommit,
  binarySha256,
} = {}) {
  if ((sidecarArtifact?.skippedFiles ?? []).length !== 0) {
    throw new Error('sidecar skippedFiles must be empty');
  }
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
  if (!binary) throw new RustSourceHealthConfigError('rust source health binary is required');
  const binaryPath = path.resolve(binary);
  if (!existsSync(binaryPath)) {
    throw new RustSourceHealthConfigError(
      `rust source health binary not found: ${binaryPath}`,
    );
  }
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
