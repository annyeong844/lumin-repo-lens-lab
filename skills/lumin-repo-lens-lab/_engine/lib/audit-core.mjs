// _lib/audit-core.mjs
//
// Runtime bridge for migrated audit-core contracts.
// Owns locating, validating, building, and invoking the lumin-audit-core helper.

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

let auditCoreAutoBuildFailure = null;

const AUDIT_CORE_CONTRACT_PROBES = [
  [
    'producer-performance-runtime-artifact',
    'producer-performance-runtime-artifact: missing --input',
  ],
  [
    'producer-performance-audit-run-artifact',
    'producer-performance-audit-run-artifact: missing --input',
  ],
  [
    'manifest-companion-update',
    'manifest-companion-update: missing --input',
  ],
  [
    'manifest-evidence-refresh-with-reads',
    'manifest-evidence-refresh-with-reads: missing --root <repo>',
  ],
  [
    'manifest-lifecycle-evidence-refresh',
    'manifest-lifecycle-evidence-refresh: missing --input <path|->',
  ],
  [
    'manifest-evidence-summary-with-reads',
    'manifest-evidence-summary-with-reads: missing --root <repo>',
  ],
  [
    'manifest-closeout-update',
    'manifest-closeout-update: missing --input',
  ],
  [
    'manifest-artifacts-produced-update',
    'manifest-artifacts-produced-update: missing --output <dir>',
  ],
  [
    'manifest-write',
    'manifest-write: missing --output <dir>',
  ],
  [
    'manifest-closeout-write',
    'manifest-closeout-write: missing --input <path|->',
  ],
];

function executableOnPath(exe) {
  for (const dir of (process.env.PATH ?? '').split(path.delimiter)) {
    if (!dir) continue;
    const candidate = path.join(dir, exe);
    if (existsSync(candidate)) return candidate;
  }
  return null;
}

function auditCoreBinary() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  const platformEnv = `LUMIN_AUDIT_CORE_BIN_${process.platform}_${process.arch}`
    .replace(/[^A-Z0-9_]/gi, '_')
    .toUpperCase();
  for (const configured of [process.env[platformEnv], process.env.LUMIN_AUDIT_CORE_BIN]) {
    const resolved = configured ? path.resolve(configured) : null;
    if (resolved && auditCoreCandidateSupportsCurrentContract(resolved)) return resolved;
  }
  const packagedPlatform = path.resolve(here, '../bin', `${process.platform}-${process.arch}`, exe);
  if (auditCoreCandidateSupportsCurrentContract(packagedPlatform)) return packagedPlatform;
  const packagedSourceManifest = path.resolve(here, '../rust', 'Cargo.toml');
  if (isLuminAuditCoreWorkspace(path.dirname(packagedSourceManifest))) {
    const built = auditCoreBinaryFromManifest(packagedSourceManifest, autoBuildCandidatePath(packagedSourceManifest, exe));
    if (built) return built;
  }
  let cursor = here;
  for (;;) {
    const workspaceRoot = path.join(cursor, 'experiments');
    const manifest = path.join(workspaceRoot, 'Cargo.toml');
    if (isLuminAuditCoreWorkspace(workspaceRoot)) {
      const localCandidate = path.join(workspaceRoot, 'target', 'debug', exe);
      if (auditCoreCandidateSupportsCurrentContract(localCandidate)) return localCandidate;
      const built = auditCoreBinaryFromManifest(manifest, autoBuildCandidatePath(manifest, exe));
      if (built) return built;
    }
    const parent = path.dirname(cursor);
    if (parent === cursor) break;
    cursor = parent;
  }
  const pathBinary = executableOnPath(exe);
  if (pathBinary && auditCoreCandidateSupportsCurrentContract(pathBinary)) return pathBinary;
  return packagedPlatform;
}

function isLuminAuditCoreWorkspace(workspaceRoot) {
  return existsSync(path.join(workspaceRoot, 'Cargo.toml')) &&
    existsSync(path.join(workspaceRoot, 'rust-common', 'Cargo.toml')) &&
    existsSync(path.join(workspaceRoot, 'rust-main', 'lumin-audit-core', 'Cargo.toml'));
}

function auditCoreBinaryFromManifest(manifestPath, candidate) {
  if (auditCoreCandidateSupportsCurrentContract(candidate)) {
    return candidate;
  }
  return ensureAuditCoreBuiltFromManifest(manifestPath, candidate)
    ? candidate
    : null;
}

function auditCoreCandidateSupportsCurrentContract(command) {
  return existsSync(command) && auditCoreBinarySupportsCurrentContract(command);
}

function auditCoreBinarySupportsCurrentContract(command) {
  for (const [subcommand, expected] of AUDIT_CORE_CONTRACT_PROBES) {
    const result = spawnSync(command, [subcommand], {
      encoding: 'utf8',
    });
    if (result.error) return false;
    const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
    if (!output.includes(expected)) return false;
  }
  return true;
}

function ensureAuditCoreBuiltFromManifest(manifestPath, candidate) {
  if (process.env.LUMIN_AUDIT_CORE_NO_AUTO_BUILD === '1') return false;
  try {
    execFileSync('cargo', [
      'build',
      '--manifest-path',
      manifestPath,
      '-p',
      'lumin-audit-core',
      '--locked',
      '--target-dir',
      path.dirname(path.dirname(candidate)),
    ], {
      cwd: path.dirname(manifestPath),
      stdio: 'inherit',
    });
  } catch (error) {
    auditCoreAutoBuildFailure = error?.message ?? String(error);
  }
  return auditCoreCandidateSupportsCurrentContract(candidate);
}

function autoBuildCandidatePath(manifestPath, exe) {
  const targetDir = process.env.CARGO_TARGET_DIR
    ? path.resolve(process.env.CARGO_TARGET_DIR)
    : path.join(
      tmpdir(),
      'lumin-audit-core-target',
      `${process.platform}-${process.arch}`,
      sourceKeyForPath(path.dirname(manifestPath))
    );
  return path.join(targetDir, 'debug', exe);
}

function sourceKeyForPath(sourcePath) {
  return path.resolve(sourcePath).replace(/[^A-Za-z0-9_.-]/g, '_').slice(-96);
}

function auditCorePlatformHint() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  const manifestPath = path.resolve(here, '../bin/audit-core-platforms.json');
  let supported = [];
  let packageScope = null;
  let sourceFallback = null;
  if (existsSync(manifestPath)) {
    try {
      const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
      packageScope = typeof manifest.packageScope === 'string' ? manifest.packageScope : null;
      sourceFallback = manifest.sourceFallback?.manifest ?? null;
      supported = (manifest.platforms ?? [])
        .map((platform) => platform.key)
        .filter((key) => typeof key === 'string' && key.length > 0)
        .sort();
    } catch {
      supported = [];
    }
  }
  const platformEnv = `LUMIN_AUDIT_CORE_BIN_${process.platform}_${process.arch}`
    .replace(/[^A-Z0-9_]/gi, '_')
    .toUpperCase();
  const supportedText = supported.length > 0
    ? ` packaged audit-core platforms: ${supported.join(', ')}.`
    : '';
  const sourceText = sourceFallback
    ? ` packaged source fallback: ${sourceFallback}.`
    : '';
  const scopeText = packageScope && !packageScope.startsWith('multi-platform')
    ? ` This skill package is scoped to ${packageScope}.`
    : '';
  const buildText = ' The wrapper can build a packaged or source-checkout lumin-audit-core helper for the current platform with cargo; set LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1 to disable that fallback.';
  const buildFailureText = auditCoreAutoBuildFailure
    ? ` Last auto-build failure: ${auditCoreAutoBuildFailure}.`
    : '';
  return `${supportedText}${sourceText}${scopeText} Provide ${platformEnv} or LUMIN_AUDIT_CORE_BIN for this platform, put ${exe} on PATH, or install a package built for ${process.platform}-${process.arch}.${buildText}${buildFailureText}`;
}

function missingAuditCoreBinaryError(label, command) {
  return new Error(`${label}: lumin-audit-core binary missing at ${command}.${auditCorePlatformHint()}`);
}

export function runAuditCoreJson(args, label, options = {}) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw missingAuditCoreBinaryError(label, command);
  }
  const childOptions = {
    encoding: 'utf8',
    stdio: [options.input === undefined ? 'ignore' : 'pipe', 'pipe', 'pipe'],
  };
  if (options.input !== undefined) childOptions.input = options.input;
  const stdout = execFileSync(command, args, childOptions);
  return JSON.parse(stdout);
}

export function runAuditCoreJsonResultFile(args, label, options = {}) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw missingAuditCoreBinaryError(label, command);
  }
  const tempDir = mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-'));
  const resultPath = path.join(tempDir, 'result.json');
  try {
    const childOptions = {
      encoding: 'utf8',
      stdio: [options.input === undefined ? 'ignore' : 'pipe', 'inherit', 'inherit'],
    };
    if (options.input !== undefined) childOptions.input = options.input;
    execFileSync(command, [...args, '--result-output', resultPath], childOptions);
    return JSON.parse(readFileSync(resultPath, 'utf8'));
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}
