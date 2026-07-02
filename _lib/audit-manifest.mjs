// _lib/audit-manifest.mjs
//
// Helpers for audit-repo.mjs manifest evidence and artifact enumeration.
// NO producer orchestration. Migrated manifest contracts call lumin-audit-core.

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

let auditCoreAutoBuildFailure = null;

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
  const pathBinary = executableOnPath(exe);
  if (pathBinary && auditCoreCandidateSupportsCurrentContract(pathBinary)) return pathBinary;
  const packagedSourceManifest = path.resolve(here, '../rust', 'Cargo.toml');
  if (existsSync(packagedSourceManifest)) {
    const candidate = path.resolve(here, '../rust', 'target', 'debug', exe);
    return auditCoreBinaryFromManifest(packagedSourceManifest, candidate);
  }
  const packagedManifest = path.resolve(here, '../bin/audit-core-platforms.json');
  const hasPackagedManifest = existsSync(packagedManifest);
  const fallback = path.join(path.resolve(here, '..'), 'experiments', 'target', 'debug', exe);
  let cursor = here;
  for (;;) {
    const candidate = path.join(cursor, 'experiments', 'target', 'debug', exe);
    if (existsSync(candidate)) {
      const manifest = path.join(cursor, 'experiments', 'Cargo.toml');
      return existsSync(manifest) ? auditCoreBinaryFromManifest(manifest, candidate) : candidate;
    }
    if (existsSync(path.join(cursor, 'experiments', 'Cargo.toml'))) {
      return ensureAuditCoreBuiltFromManifest(
        path.join(cursor, 'experiments', 'Cargo.toml'),
        candidate
      );
    }
    const parent = path.dirname(cursor);
    if (parent === cursor) return hasPackagedManifest ? packagedPlatform : fallback;
    cursor = parent;
  }
}

function auditCoreBinaryFromManifest(manifestPath, candidate) {
  if (auditCoreCandidateSupportsCurrentContract(candidate)) {
    return candidate;
  }
  return ensureAuditCoreBuiltFromManifest(manifestPath, candidate);
}

function auditCoreCandidateSupportsCurrentContract(command) {
  return existsSync(command) && auditCoreBinarySupportsCurrentContract(command);
}

function auditCoreBinarySupportsCurrentContract(command) {
  const result = spawnSync(command, ['manifest-evidence-summary-with-reads'], {
    encoding: 'utf8',
  });
  if (result.error) return false;
  const output = `${result.stdout ?? ''}\n${result.stderr ?? ''}`;
  return output.includes('manifest-evidence-summary-with-reads: missing --root <repo>');
}

function ensureAuditCoreBuiltFromManifest(manifestPath, candidate) {
  if (process.env.LUMIN_AUDIT_CORE_NO_AUTO_BUILD === '1') return candidate;
  try {
    execFileSync('cargo', [
      'build',
      '--manifest-path',
      manifestPath,
      '-p',
      'lumin-audit-core',
    ], {
      cwd: path.dirname(manifestPath),
      stdio: 'inherit',
    });
  } catch (error) {
    auditCoreAutoBuildFailure = error?.message ?? String(error);
  }
  return candidate;
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

function runAuditCoreJson(args, label, options = {}) {
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

function runAuditCoreJsonResultFile(args, label, options = {}) {
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

function runManifestEvidenceCommand(command, label, root, outDir, {
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  generatedArtifactsMode = 'default',
  rustAnalysisRun = null,
  mergeRustAnalysisRun = false,
} = {}) {
  const args = [
    command,
    '--root', root,
    '--output', outDir,
    '--generated-artifacts', generatedArtifactsMode,
    includeTests ? '--include-tests' : '--no-include-tests',
    production ? '--production' : '--no-production',
  ];
  const runOptions = {};
  if (mergeRustAnalysisRun && rustAnalysisRun) {
    args.push('--rust-analysis-run-block', '-');
    runOptions.input = JSON.stringify(rustAnalysisRun);
  } else if (rustAnalysisRun?.ran === true) {
    args.push('--rust-analysis-ran');
  }
  for (const exclude of excludes) {
    args.push('--exclude', exclude);
  }
  for (const autoExclude of autoExcludes) {
    args.push('--auto-exclude', autoExclude);
  }
  return runAuditCoreJson(args, label, runOptions);
}

function buildManifestEvidenceSummaryWithReadsFromFile(root, outDir, options = {}) {
  return runManifestEvidenceCommand(
    'manifest-evidence-summary-with-reads',
    'buildManifestEvidenceSummary',
    root,
    outDir,
    options,
  );
}

function buildManifestEvidenceRefreshWithReadsFromFile(root, outDir, options = {}) {
  return runManifestEvidenceCommand(
    'manifest-evidence-refresh-with-reads',
    'refreshManifestEvidence',
    root,
    outDir,
    options,
  );
}

export function collectProducedArtifacts(outDir, options = {}) {
  const args = [
    'artifact-registry',
    '--output', outDir,
  ];
  const runOptions = {};
  if (Object.hasOwn(options, 'rustAnalysis')) {
    args.push('--rust-analysis-block', '-');
    runOptions.input = JSON.stringify(options.rustAnalysis ?? null);
  }
  return runAuditCoreJson(args, 'collectProducedArtifacts', runOptions);
}

export function buildManifestArtifactsProducedUpdate(outDir, options = {}) {
  const args = [
    'manifest-artifacts-produced-update',
    '--output', outDir,
  ];
  const runOptions = {};
  if (Object.hasOwn(options, 'rustAnalysis')) {
    args.push('--rust-analysis-block', '-');
    runOptions.input = JSON.stringify(options.rustAnalysis ?? null);
  }
  return runAuditCoreJson(args, 'buildManifestArtifactsProducedUpdate', runOptions);
}

function buildArtifactReadMetricsSummary(input) {
  return runAuditCoreJson([
    'artifact-read-metrics-summary',
    '--input', '-',
  ], 'buildArtifactReadMetricsSummary', {
    input: JSON.stringify(input ?? {}),
  });
}

const ARTIFACT_READ_EVENTS_SCHEMA_VERSION = 'lumin-audit-artifact-read-events.v1';

export function createArtifactReadMetrics({ rootDir, largestLimit = 10 } = {}) {
  const reads = [];

  function observeRead(record) {
    reads.push({
      filePath: record?.filePath ?? 'unknown',
      bytes: record?.bytes ?? 0,
      readMs: record?.readMs ?? 0,
      jsonParseMs: record?.jsonParseMs ?? 0,
      ok: record?.ok !== false,
    });
  }

  function summary() {
    return buildArtifactReadMetricsSummary({
      schemaVersion: ARTIFACT_READ_EVENTS_SCHEMA_VERSION,
      rootDir,
      largestLimit,
      reads,
    });
  }

  return { observeRead, summary };
}

export function buildProducerPerformanceArtifactForAuditRun({
  generated,
  root,
  outDir,
  profile,
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  noIncremental = false,
  cacheRoot,
  clearIncrementalCache = false,
  generatedArtifactsMode = 'default',
  artifactReads,
  rustAnalysis = null,
  commandsRun = [],
  skipped = [],
}) {
  const args = [
    'producer-performance-audit-run-artifact',
    '--input', '-',
    '--generated', generated,
    '--root', root,
    '--output', outDir,
    '--profile', profile,
    includeTests ? '--include-tests' : '--no-include-tests',
    production ? '--production' : '--no-production',
    '--cache-root', cacheRoot,
    ...(noIncremental ? ['--no-incremental'] : []),
    ...(clearIncrementalCache ? ['--clear-incremental-cache'] : []),
    '--generated-artifacts', generatedArtifactsMode,
  ];
  for (const exclude of excludes) args.push('--exclude', exclude);
  for (const autoExclude of autoExcludes) args.push('--auto-exclude', autoExclude);
  return runAuditCoreJson(args, 'buildProducerPerformanceArtifactForAuditRun', {
    input: JSON.stringify({
      artifactReads,
      rustAnalysis,
      commandsRun,
      skipped,
    }),
  });
}

export function executeBasePlan(request) {
  return runAuditCoreJson([
    'execute-base-plan',
    '--input', '-',
  ], 'executeBasePlan', {
    input: JSON.stringify(request ?? {}),
  });
}

export function executeBaseRuntime(request) {
  return runAuditCoreJson([
    'execute-base-runtime',
    '--input', '-',
  ], 'executeBaseRuntime', {
    input: JSON.stringify(request ?? {}),
  });
}

export function executeCanonDraftLifecycle(request) {
  return runAuditCoreJson([
    'execute-canon-draft',
    '--input', '-',
  ], 'executeCanonDraftLifecycle', {
    input: JSON.stringify(request ?? {}),
  });
}

export function executeCheckCanonLifecycle(request) {
  return runAuditCoreJson([
    'execute-check-canon',
    '--input', '-',
  ], 'executeCheckCanonLifecycle', {
    input: JSON.stringify(request ?? {}),
  });
}

export function resolvePreWriteRoute(request) {
  return runAuditCoreJson([
    'pre-write-route',
    '--input', '-',
  ], 'resolvePreWriteRoute', {
    input: JSON.stringify(request ?? {}),
  });
}

export function executeRustPreWriteLifecycle(request) {
  return runAuditCoreJsonResultFile([
    'execute-rust-pre-write',
    '--input', '-',
  ], 'executeRustPreWriteLifecycle', {
    input: JSON.stringify(request ?? {}),
  });
}

export function executePostWriteLifecycle(request) {
  return runAuditCoreJsonResultFile([
    'execute-post-write',
    '--input', '-',
  ], 'executePostWriteLifecycle', {
    input: JSON.stringify(request ?? {}),
  });
}

export function applyLifecycleExitPolicy(request) {
  return runAuditCoreJson([
    'lifecycle-exit-policy',
    '--input', '-',
  ], 'applyLifecycleExitPolicy', {
    input: JSON.stringify(request ?? {}),
  });
}

export function evaluateLifecycleRequestGuard(request) {
  return runAuditCoreJson([
    'lifecycle-request-guard',
    '--input', '-',
  ], 'evaluateLifecycleRequestGuard', {
    input: JSON.stringify(request ?? {}),
  });
}

export function buildOrchestrationPlan({
  profile = 'quick',
  sarif = false,
  preWrite = false,
  postWrite = false,
  canonDraft = false,
  checkCanon = false,
  rustAnalyzer = false,
} = {}) {
  return runAuditCoreJson([
    'orchestration-plan',
    '--profile', profile,
    ...(sarif ? ['--sarif'] : []),
    ...(preWrite ? ['--pre-write'] : []),
    ...(postWrite ? ['--post-write'] : []),
    ...(canonDraft ? ['--canon-draft'] : []),
    ...(checkCanon ? ['--check-canon'] : []),
    ...(rustAnalyzer ? ['--rust-analyzer'] : []),
  ], 'buildOrchestrationPlan');
}

export function buildManifestFinalSummaryUpdate({
  outDir,
  producerPerformancePath,
  rustAnalysis,
}) {
  const args = [
    'manifest-final-summary-update',
    '--output', outDir,
    '--producer-performance', producerPerformancePath,
  ];
  const options = {};
  if (rustAnalysis !== undefined) {
    args.push('--rust-analysis-block', '-');
    options.input = JSON.stringify(rustAnalysis ?? null);
  }
  return runAuditCoreJson(args, 'buildManifestFinalSummaryUpdate', options);
}

export function buildManifestLifecycleUpdate(blocks) {
  return runAuditCoreJson([
    'manifest-lifecycle-update',
    '--input', '-',
  ], 'buildManifestLifecycleUpdate', {
    input: JSON.stringify(blocks ?? {}),
  });
}

export function buildManifestRoot(input) {
  return runAuditCoreJson([
    'manifest-root',
    '--input', '-',
  ], 'buildManifestRoot', {
    input: JSON.stringify(input ?? {}),
  });
}

export function buildManifestCompanionUpdate(input) {
  return runAuditCoreJson([
    'manifest-companion-update',
    '--input', '-',
  ], 'buildManifestCompanionUpdate', {
    input: JSON.stringify(input ?? {}),
  });
}

function observeRustArtifactReads(artifactReads, onArtifactRead) {
  if (!onArtifactRead) return;
  for (const read of artifactReads?.reads ?? []) {
    onArtifactRead(read);
  }
}

export function buildManifestEvidence({
  root,
  outDir,
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  generatedArtifactsMode = 'default',
  rustAnalysisRun = null,
  mergeRustAnalysisRun = false,
  onArtifactRead,
}) {
  const manifestEvidence = buildManifestEvidenceSummaryWithReadsFromFile(root, outDir, {
    includeTests,
    production,
    excludes,
    autoExcludes,
    generatedArtifactsMode,
    rustAnalysisRun,
    mergeRustAnalysisRun,
  });
  observeRustArtifactReads(manifestEvidence.artifactReads, onArtifactRead);
  return manifestEvidence.evidence ?? {};
}

export function refreshManifestEvidence(manifest, options) {
  const result = buildManifestEvidenceRefreshWithReadsFromFile(options.root, options.outDir, options);
  observeRustArtifactReads(result.artifactReads, options.onArtifactRead);
  Object.assign(
    manifest,
    result.evidence ?? {},
  );
}
