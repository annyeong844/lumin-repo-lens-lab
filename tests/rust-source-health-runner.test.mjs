import { createHash } from 'node:crypto';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  buildFinalRustHealthArtifact,
  collectRustSourceHealthInput,
  runRustSourceHealth,
} from '../_lib/rust-source-health-runner.mjs';
import {
  RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES,
  RUST_SOURCE_HEALTH_PARSER,
  RUST_SOURCE_HEALTH_POLICY_VERSION,
  RUST_SOURCE_HEALTH_SCHEMA_VERSION,
  summarizeRustHealthArtifact,
  validateRustHealthFinalArtifact,
} from '../_lib/rust-source-health-schema.mjs';

function tempDir(name) {
  const dir = path.join(
    os.tmpdir(),
    `${name}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
  );
  mkdirSync(dir, { recursive: true });
  return dir;
}

function writeText(file, text) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, text, 'utf8');
}

function writeBytes(file, bytes) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, Buffer.from(bytes));
}

function hashFile(file) {
  return `sha256:${createHash('sha256').update(readFileSync(file)).digest('hex')}`;
}

function writeFakeSidecar(dir, body) {
  mkdirSync(dir, { recursive: true });
  const script = path.join(dir, 'fake-rust-source-health.mjs');
  const binary = path.join(
    dir,
    process.platform === 'win32'
      ? 'fake-rust-source-health.cmd'
      : 'fake-rust-source-health.sh',
  );
  writeFileSync(script, body, 'utf8');
  if (process.platform === 'win32') {
    writeFileSync(
      binary,
      `@echo off\r\n"${process.execPath}" "%~dp0\\fake-rust-source-health.mjs"\r\n`,
      'utf8',
    );
  } else {
    writeFileSync(
      binary,
      `#!/usr/bin/env sh\n"${process.execPath}" "$(dirname "$0")/fake-rust-source-health.mjs"\n`,
      'utf8',
    );
    chmodSync(binary, 0o755);
  }
  return binary;
}

function fakeSidecarBody({ capturePath, malformed = false, hang = false } = {}) {
  if (hang) return 'setTimeout(() => {}, 10000);\n';
  if (malformed) return "process.stdout.write('{bad json');\n";
  return `
import { writeFileSync } from 'node:fs';

let stdin = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', (chunk) => { stdin += chunk; });
process.stdin.on('end', () => {
  const request = JSON.parse(stdin);
  if (${JSON.stringify(capturePath)}) {
    writeFileSync(${JSON.stringify(capturePath)}, stdin, 'utf8');
  }
  const files = {};
  for (const file of request.files) {
    files[file.path] = {
      sha256: file.sha256,
      facts: {
        items: 1,
        functions: 1,
        maxFunctionLines: 1,
        unsafeBlocks: 0,
        unsafeFunctions: 0
      },
      signals: [],
      parse: { ok: true, errors: [] },
      path: {
        classifications: file.path.includes('/tests/') ? ['test'] : ['source'],
        suppressed: false
      }
    };
  }
  const artifact = {
    schemaVersion: ${RUST_SOURCE_HEALTH_SCHEMA_VERSION},
    meta: {
      producer: 'rust-source-health',
      mode: 'syntax-only',
      parser: {
        kind: ${JSON.stringify(RUST_SOURCE_HEALTH_PARSER.kind)},
        version: ${JSON.stringify(RUST_SOURCE_HEALTH_PARSER.version)},
        editionPolicy: ${JSON.stringify(RUST_SOURCE_HEALTH_PARSER.editionPolicy)},
        edition: ${JSON.stringify(RUST_SOURCE_HEALTH_PARSER.edition)},
        editionSource: ${JSON.stringify(RUST_SOURCE_HEALTH_PARSER.editionSource)}
      },
      policy: {
        version: ${JSON.stringify(RUST_SOURCE_HEALTH_POLICY_VERSION)},
        thresholds: { maxFunctionLines: 80, maxImplLines: 200 }
      },
      runtime: {
        threadCount: request.runtime.threadCount ?? 1,
        workerStackBytes: request.runtime.workerStackBytes
      },
      limits: ['syntax-only', 'no-type-info', 'no-trait-solving', 'no-borrow-check']
    },
    summary: {
      files: Object.keys(files).length,
      skippedFiles: 0,
      parseErrorFiles: 0,
      parseErrors: 0,
      functions: Object.keys(files).length,
      unsafeBlocks: 0,
      unsafeFunctions: 0,
      signals: 0,
      signalsByKind: {}
    },
    skippedFiles: [],
    files
  };
  process.stdout.write(JSON.stringify(artifact));
});
`;
}

describe('Rust source health runner', () => {
  it('writes a final artifact with wrapper provenance and runtime input', async () => {
    const dir = tempDir('lumin-rust-health-runner-happy');
    const root = path.join(dir, 'repo');
    const output = path.join(dir, 'out', 'rust-health.json');
    const capturePath = path.join(dir, 'request.json');
    writeText(path.join(root, 'src', 'lib.rs'), 'pub fn main() {}\n');
    const binary = writeFakeSidecar(dir, fakeSidecarBody({ capturePath }));

    const result = await runRustSourceHealth({
      root,
      output,
      binary,
      sidecarSourceCommit: 'abc123',
      timeoutMs: 5000,
    });

    const request = JSON.parse(readFileSync(capturePath, 'utf8'));
    expect(request.runtime.workerStackBytes).toBe(
      RUST_SOURCE_HEALTH_DEFAULT_WORKER_STACK_BYTES,
    );
    expect(request.runtime.threadCount).toBeUndefined();
    expect(request.files.map((file) => file.path)).toEqual(['src/lib.rs']);
    expect(result.output).toBe(path.resolve(output));
    expect(existsSync(output)).toBe(true);
    expect(result.artifact.meta.sidecar).toEqual({
      sourceCommit: 'abc123',
      binarySha256: hashFile(binary),
    });
    expect(result.artifact.meta.input.pathPolicy).toEqual({
      include: ['**/*.rs'],
      exclude: ['target/**', 'vendor/**'],
    });
    expect(result.artifact.summary).toEqual(
      summarizeRustHealthArtifact(result.artifact),
    );
    expect(result.artifact.meta.rustTopologyPrefer).toBeUndefined();
    expect(result.artifact.meta.rustTopologyScanner).toBeUndefined();
  });

  it('records invalid UTF-8 as skipped-file evidence and omits it from sidecar input', async () => {
    const dir = tempDir('lumin-rust-health-runner-invalid-utf8');
    const root = path.join(dir, 'repo');
    const output = path.join(dir, 'rust-health.json');
    const capturePath = path.join(dir, 'request.json');
    writeText(path.join(root, 'src', 'lib.rs'), 'pub fn ok() {}\n');
    writeBytes(path.join(root, 'src', 'bad.rs'), [0xff, 0xfe, 0xfd]);
    const binary = writeFakeSidecar(dir, fakeSidecarBody({ capturePath }));

    const result = await runRustSourceHealth({
      root,
      output,
      binary,
      sidecarSourceCommit: 'abc123',
      timeoutMs: 5000,
    });

    const request = JSON.parse(readFileSync(capturePath, 'utf8'));
    expect(request.files.map((file) => file.path)).toEqual(['src/lib.rs']);
    expect(result.artifact.skippedFiles).toEqual([
      { path: 'src/bad.rs', reason: 'invalid-utf8' },
    ]);
    expect(result.artifact.summary.skippedFiles).toBe(1);
  });

  it('applies path policy with root-relative slash paths and does not follow symlinks', () => {
    const dir = tempDir('lumin-rust-health-runner-path-policy');
    const root = path.join(dir, 'repo');
    writeText(path.join(root, 'target', 'generated.rs'), 'fn generated() {}\n');
    writeText(path.join(root, 'vendor', 'lib.rs'), 'fn vendored() {}\n');
    writeText(path.join(root, 'src', 'targeted.rs'), 'fn targeted() {}\n');
    writeText(path.join(root, 'src', 'vendor_name.rs'), 'fn vendor_name() {}\n');
    try {
      symlinkSync(
        path.join(root, 'src', 'targeted.rs'),
        path.join(root, 'src', 'linked.rs'),
        'file',
      );
    } catch {
      // Some Windows configurations disallow symlink creation. The runner still
      // covers the symlink branch on systems that permit the fixture.
    }

    const { input, skippedFiles } = collectRustSourceHealthInput({ root });

    expect(input.files.map((file) => file.path)).toEqual([
      'src/targeted.rs',
      'src/vendor_name.rs',
    ]);
    expect(skippedFiles).toEqual([]);
    for (const file of input.files) {
      expect(path.isAbsolute(file.path)).toBe(false);
      expect(file.path.includes('..')).toBe(false);
      expect(file.path.includes('\\')).toBe(false);
    }
  });

  it('appends wrapper-owned skipped files before final summary validation', () => {
    const sidecarArtifact = {
      schemaVersion: RUST_SOURCE_HEALTH_SCHEMA_VERSION,
      meta: {
        producer: 'rust-source-health',
        mode: 'syntax-only',
        parser: { ...RUST_SOURCE_HEALTH_PARSER },
        policy: {
          version: RUST_SOURCE_HEALTH_POLICY_VERSION,
          thresholds: { maxFunctionLines: 80, maxImplLines: 200 },
        },
        runtime: { threadCount: 1, workerStackBytes: 16777216 },
        limits: ['syntax-only', 'no-type-info', 'no-trait-solving', 'no-borrow-check'],
      },
      summary: {
        files: 0,
        skippedFiles: 0,
        parseErrorFiles: 0,
        parseErrors: 0,
        functions: 0,
        unsafeBlocks: 0,
        unsafeFunctions: 0,
        signals: 0,
        signalsByKind: {},
      },
      skippedFiles: [],
      files: {},
    };

    const artifact = buildFinalRustHealthArtifact({
      sidecarArtifact,
      skippedFiles: [{ path: 'src/bad.rs', reason: 'invalid-utf8' }],
      pathPolicy: { include: ['**/*.rs'], exclude: ['target/**', 'vendor/**'] },
      sidecarSourceCommit: 'abc123',
      binarySha256: `sha256:${'a'.repeat(64)}`,
    });

    expect(artifact.summary.skippedFiles).toBe(1);
    expect(validateRustHealthFinalArtifact(artifact)).toEqual([]);
  });

  it('passes runtime thread count and rejects too-small worker stack before sidecar execution', async () => {
    const dir = tempDir('lumin-rust-health-runner-runtime');
    const root = path.join(dir, 'repo');
    const output = path.join(dir, 'rust-health.json');
    const capturePath = path.join(dir, 'request.json');
    writeText(path.join(root, 'src', 'lib.rs'), 'pub fn main() {}\n');
    const binary = writeFakeSidecar(dir, fakeSidecarBody({ capturePath }));

    await runRustSourceHealth({
      root,
      output,
      binary,
      sidecarSourceCommit: 'abc123',
      timeoutMs: 5000,
      threadCount: 2,
    });
    const request = JSON.parse(readFileSync(capturePath, 'utf8'));
    expect(request.runtime.threadCount).toBe(2);

    const rejectedCapturePath = path.join(dir, 'rejected-request.json');
    const rejectedBinary = writeFakeSidecar(
      path.join(dir, 'rejected-bin'),
      fakeSidecarBody({ capturePath: rejectedCapturePath }),
    );
    await expect(
      runRustSourceHealth({
        root,
        output: path.join(dir, 'rejected.json'),
        binary: rejectedBinary,
        sidecarSourceCommit: 'abc123',
        timeoutMs: 5000,
        workerStackBytes: 8388608,
      }),
    ).rejects.toThrow(/workerStackBytes/);
    expect(existsSync(rejectedCapturePath)).toBe(false);
  });

  it('treats malformed sidecar JSON as a hard failure without partial output', async () => {
    const dir = tempDir('lumin-rust-health-runner-invalid-json');
    const root = path.join(dir, 'repo');
    const output = path.join(dir, 'rust-health.json');
    writeText(path.join(root, 'src', 'lib.rs'), 'pub fn main() {}\n');
    const binary = writeFakeSidecar(dir, fakeSidecarBody({ malformed: true }));

    await expect(
      runRustSourceHealth({
        root,
        output,
        binary,
        sidecarSourceCommit: 'abc123',
        timeoutMs: 5000,
      }),
    ).rejects.toThrow(/invalid rust source health sidecar JSON/);
    expect(existsSync(output)).toBe(false);
  });

  it('treats sidecar timeout as a hard failure without partial output', async () => {
    const dir = tempDir('lumin-rust-health-runner-timeout');
    const root = path.join(dir, 'repo');
    const output = path.join(dir, 'rust-health.json');
    writeText(path.join(root, 'src', 'lib.rs'), 'pub fn main() {}\n');
    const binary = writeFakeSidecar(dir, fakeSidecarBody({ hang: true }));

    await expect(
      runRustSourceHealth({
        root,
        output,
        binary,
        sidecarSourceCommit: 'abc123',
        timeoutMs: 50,
      }),
    ).rejects.toThrow(/timed out/);
    expect(existsSync(output)).toBe(false);
  });

  it('writes only the requested rust-health artifact boundary', async () => {
    const dir = tempDir('lumin-rust-health-runner-boundary');
    const root = path.join(dir, 'repo');
    const output = path.join(root, 'baselines', 'rust-health.json');
    writeText(path.join(root, 'src', 'lib.rs'), 'pub fn main() {}\n');
    const binary = writeFakeSidecar(dir, fakeSidecarBody());

    await runRustSourceHealth({
      root,
      output,
      binary,
      sidecarSourceCommit: 'abc123',
      timeoutMs: 5000,
    });

    expect(existsSync(output)).toBe(true);
    expect(existsSync(path.join(root, 'topology.json'))).toBe(false);
    expect(existsSync(path.join(root, 'topology.sarif'))).toBe(false);
    expect(existsSync(path.join(root, 'baselines', 'rust-topology-prefer-quorum.json'))).toBe(false);
  });
});
