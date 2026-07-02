// _lib/audit-core.mjs
//
// Runtime bridge for migrated audit-core contracts.
// Owns locating, validating, building, and invoking the lumin-audit-core helper.

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

let auditCoreAutoBuildFailure = null;
let auditCoreBinaryCache = null;
const auditCoreContractCache = new Map();

const AUDIT_CORE_CONTRACT_PROBES = [
  [
    ['producer-performance-runtime-artifact'],
    'producer-performance-runtime-artifact: missing --input',
  ],
  [
    ['producer-performance-audit-run-artifact'],
    'producer-performance-audit-run-artifact: missing --input',
  ],
  [
    ['manifest-companion-update'],
    'manifest-companion-update: missing --input',
  ],
  [
    ['manifest-root-with-evidence'],
    'manifest-root-with-evidence: missing --input <path|->',
  ],
  [
    ['manifest-evidence-refresh-with-reads'],
    'manifest-evidence-refresh-with-reads: missing --root <repo>',
  ],
  [
    ['manifest-lifecycle-evidence-refresh'],
    'manifest-lifecycle-evidence-refresh: missing --input <path|->',
  ],
  [
    ['manifest-evidence-summary-with-reads'],
    'manifest-evidence-summary-with-reads: missing --root <repo>',
  ],
  [
    ['manifest-closeout-update'],
    'manifest-closeout-update: missing --input',
  ],
  [
    ['manifest-artifacts-produced-update'],
    'manifest-artifacts-produced-update: missing --output <dir>',
  ],
  [
    ['manifest-write'],
    'manifest-write: missing --output <dir>',
  ],
  [
    ['manifest-closeout-write'],
    'manifest-closeout-write: missing --input <path|->',
  ],
  [
    ['finalize-audit-run'],
    'finalize-audit-run: missing --input <path|->',
  ],
];

const RESULT_FILE_REQUIRED_SUBCOMMANDS = new Set([
  'manifest-root-with-evidence',
  'manifest-lifecycle-evidence-refresh',
  'manifest-evidence-summary-with-reads',
  'manifest-evidence-refresh-with-reads',
]);

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
  const cacheKey = JSON.stringify({
    here,
    platform: process.platform,
    arch: process.arch,
    platformOverride: process.env[platformEnv] ?? null,
    genericOverride: process.env.LUMIN_AUDIT_CORE_BIN ?? null,
    path: process.env.PATH ?? '',
    cargoTargetDir: process.env.CARGO_TARGET_DIR ?? null,
    noAutoBuild: process.env.LUMIN_AUDIT_CORE_NO_AUTO_BUILD ?? null,
  });
  const configuredOverrides = [process.env[platformEnv], process.env.LUMIN_AUDIT_CORE_BIN]
    .map((configured) => configured ? path.resolve(configured) : null)
    .filter(Boolean);
  const overrideSignatureKey = candidateSignatureKey(configuredOverrides);
  if (auditCoreBinaryCache?.key === cacheKey) {
    const signature = fileSignature(auditCoreBinaryCache.command);
    if (
      signature &&
      signature === auditCoreBinaryCache.signature &&
      overrideSignatureKey === auditCoreBinaryCache.overrideSignatureKey
    ) {
      return auditCoreBinaryCache.command;
    }
  }
  const remember = (command) => {
    auditCoreBinaryCache = {
      key: cacheKey,
      command,
      signature: fileSignature(command),
      overrideSignatureKey,
    };
    return command;
  };
  for (const resolved of configuredOverrides) {
    if (resolved && auditCoreCandidateSupportsCurrentContract(resolved)) return remember(resolved);
  }
  const packagedPlatform = path.resolve(here, '../bin', `${process.platform}-${process.arch}`, exe);
  if (auditCoreCandidateSupportsCurrentContract(packagedPlatform)) return remember(packagedPlatform);
  const packagedSourceManifest = path.resolve(here, '../rust', 'Cargo.toml');
  if (isLuminAuditCoreWorkspace(path.dirname(packagedSourceManifest))) {
    const built = auditCoreBinaryFromManifest(packagedSourceManifest, autoBuildCandidatePath(packagedSourceManifest, exe));
    if (built) return remember(built);
  }
  let cursor = here;
  for (;;) {
    const workspaceRoot = path.join(cursor, 'experiments');
    const manifest = path.join(workspaceRoot, 'Cargo.toml');
    if (isLuminAuditCoreWorkspace(workspaceRoot)) {
      const localCandidate = path.join(workspaceRoot, 'target', 'debug', exe);
      if (auditCoreCandidateSupportsCurrentContract(localCandidate)) return remember(localCandidate);
      const built = auditCoreBinaryFromManifest(manifest, autoBuildCandidatePath(manifest, exe));
      if (built) return remember(built);
    }
    const parent = path.dirname(cursor);
    if (parent === cursor) break;
    cursor = parent;
  }
  const pathBinary = executableOnPath(exe);
  if (pathBinary && auditCoreCandidateSupportsCurrentContract(pathBinary)) return remember(pathBinary);
  return remember(packagedPlatform);
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
  const signature = fileSignature(command);
  if (!signature) return false;
  const cached = auditCoreContractCache.get(command);
  if (cached?.signature === signature) return cached.supports;
  const supports = auditCoreBinarySupportsCurrentContract(command);
  auditCoreContractCache.set(command, { signature, supports });
  return supports;
}

function fileSignature(filePath) {
  try {
    const stat = statSync(filePath);
    if (!stat.isFile()) return null;
    return `${stat.size}:${stat.mtimeMs}:${stat.ctimeMs}`;
  } catch {
    return null;
  }
}

function candidateSignatureKey(commands) {
  return JSON.stringify(commands.map((command) => [command, fileSignature(command)]));
}

function auditCoreBinarySupportsCurrentContract(command) {
  for (const [args, expected] of AUDIT_CORE_CONTRACT_PROBES) {
    const result = spawnSync(command, args, {
      encoding: 'utf8',
    });
    if (result.error) return false;
    const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
    if (!output.includes(expected)) return false;
  }
  return auditCoreBinaryWritesResultFiles(command);
}

function auditCoreBinaryWritesResultFiles(command) {
  const tempDir = mkdtempSync(path.join(tmpdir(), 'lumin-audit-core-contract-'));
  const rootDir = path.join(tempDir, 'root');
  const outputDir = path.join(tempDir, 'out');
  const rootInputPath = path.join(tempDir, 'manifest-root-with-evidence.json');
  const lifecycleInputPath = path.join(tempDir, 'manifest-lifecycle-evidence-refresh.json');
  try {
    mkdirSync(rootDir, { recursive: true });
    mkdirSync(outputDir, { recursive: true });
    writeFileSync(path.join(outputDir, 'triage.json'), JSON.stringify({
      shape: { totalFiles: 1, tsFiles: 0, rsFiles: 1 },
      byLanguage: { rs: 1 },
    }));
    writeFileSync(path.join(outputDir, 'symbols.json'), JSON.stringify({
      uses: {
        external: 0,
        resolvedInternal: 0,
        unresolvedInternal: 0,
        unresolvedInternalRatio: 0,
      },
    }));
    writeFileSync(rootInputPath, JSON.stringify({
      generated: '2026-07-02T00:00:00.000Z',
      profile: 'quick',
      root: rootDir,
      output: outputDir,
      commandsRun: [],
      skipped: [],
      includeTests: true,
      production: false,
      generatedArtifactsMode: 'default',
    }));
    writeFileSync(lifecycleInputPath, JSON.stringify({
      manifest: {
        meta: { generated: '2026-07-02T00:00:00.000Z' },
        artifactsProduced: [],
      },
      lifecycle: {},
      evidence: {
        root: rootDir,
        output: outputDir,
        includeTests: true,
        production: false,
        generatedArtifactsMode: 'default',
      },
    }));

    const probes = [
      {
        subcommand: 'manifest-root-with-evidence',
        args: ['manifest-root-with-evidence', '--input', rootInputPath],
        requiredField: 'manifest',
      },
      {
        subcommand: 'manifest-lifecycle-evidence-refresh',
        args: ['manifest-lifecycle-evidence-refresh', '--input', lifecycleInputPath],
        requiredField: 'manifest',
      },
      {
        subcommand: 'manifest-evidence-summary-with-reads',
        args: [
          'manifest-evidence-summary-with-reads',
          '--root', rootDir,
          '--output', outputDir,
          '--include-tests',
          '--no-production',
        ],
        requiredField: 'evidence',
      },
      {
        subcommand: 'manifest-evidence-refresh-with-reads',
        args: [
          'manifest-evidence-refresh-with-reads',
          '--root', rootDir,
          '--output', outputDir,
          '--include-tests',
          '--no-production',
        ],
        requiredField: 'evidence',
      },
    ];

    for (const probe of probes) {
      const resultPath = path.join(tempDir, `${probe.subcommand}.json`);
      const result = spawnSync(command, [...probe.args, '--result-output', resultPath], {
        encoding: 'utf8',
      });
      if (result.error || result.status !== 0) return false;
      if ((result.stdout ?? '').trim().length > 0) return false;
      if (!existsSync(resultPath)) return false;
      const json = JSON.parse(readFileSync(resultPath, 'utf8'));
      if (!resultPayloadMatchesProbe(json, probe)) return false;
      if (!Array.isArray(json.artifactReads?.reads)) return false;
    }
    return true;
  } catch {
    return false;
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function resultPayloadMatchesProbe(json, probe) {
  const payload = json[probe.requiredField];
  return isObject(payload) &&
    isObject(payload.scanRange) &&
    typeof payload.scanRange.files === 'number';
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
  const subcommand = args?.[0];
  if (RESULT_FILE_REQUIRED_SUBCOMMANDS.has(subcommand)) {
    throw new Error(
      `${label}: ${subcommand} can emit repository-sized JSON and must use runAuditCoreJsonResultFile`
    );
  }
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
