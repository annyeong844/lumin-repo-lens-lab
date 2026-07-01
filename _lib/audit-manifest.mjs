// _lib/audit-manifest.mjs
//
// Helpers for audit-repo.mjs manifest evidence and artifact enumeration.
// NO producer orchestration. Migrated manifest contracts call lumin-audit-core.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { detectBlindZones } from './blind-zones.mjs';
import { loadIfExists as loadArtifact } from './artifacts.mjs';

function auditCoreBinary() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  const fallback = path.join(path.resolve(here, '..'), 'experiments', 'target', 'debug', exe);
  let cursor = here;
  for (;;) {
    const candidate = path.join(cursor, 'experiments', 'target', 'debug', exe);
    if (existsSync(candidate) || existsSync(path.join(cursor, 'experiments', 'Cargo.toml'))) {
      return candidate;
    }
    const parent = path.dirname(cursor);
    if (parent === cursor) return fallback;
    cursor = parent;
  }
}

function runAuditCoreJson(args, label, options = {}) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw new Error(`${label}: lumin-audit-core binary missing at ${command}; run cargo build --manifest-path experiments/Cargo.toml -p lumin-audit-core`);
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
    throw new Error(`${label}: lumin-audit-core binary missing at ${command}; run cargo build --manifest-path experiments/Cargo.toml -p lumin-audit-core`);
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

function buildManifestEvidenceSummaryFromFile(root, outDir, {
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  generatedArtifactsMode = 'default',
} = {}) {
  const args = [
    'manifest-evidence-summary',
    '--root', root,
    '--output', outDir,
    '--generated-artifacts', generatedArtifactsMode,
    includeTests ? '--include-tests' : '--no-include-tests',
    production ? '--production' : '--no-production',
  ];
  for (const exclude of excludes) {
    args.push('--exclude', exclude);
  }
  for (const autoExclude of autoExcludes) {
    args.push('--auto-exclude', autoExclude);
  }
  return runAuditCoreJson(args, 'buildManifestEvidenceSummary');
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
  } else if (options.rustAnalysisUsable ?? true) {
    args.push('--rust-analysis-ran');
  }
  return runAuditCoreJson(args, 'collectProducedArtifacts', runOptions);
}

export function buildArtifactSizeSummary(outDir, artifacts) {
  return runAuditCoreJson([
    'artifact-size-summary',
    '--output', outDir,
    '--input', '-',
  ], 'buildArtifactSizeSummary', {
    input: JSON.stringify(artifacts ?? []),
  });
}

export function buildArtifactReadMetricsSummary(input) {
  return runAuditCoreJson([
    'artifact-read-metrics-summary',
    '--input', '-',
  ], 'buildArtifactReadMetricsSummary', {
    input: JSON.stringify(input ?? {}),
  });
}

export const ARTIFACT_READ_EVENTS_SCHEMA_VERSION = 'lumin-audit-artifact-read-events.v1';

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

export function buildProducerPerformanceArtifactFromRuntime(input) {
  return runAuditCoreJson([
    'producer-performance-runtime-artifact',
    '--input', '-',
  ], 'buildProducerPerformanceArtifactFromRuntime', {
    input: JSON.stringify(input ?? {}),
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
  rustAnalysisUsable = true,
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
  } else if (rustAnalysisUsable) {
    args.push('--rust-analysis-ran');
  }
  return runAuditCoreJson(args, 'buildManifestFinalSummaryUpdate', options);
}

export function mergeRustAnalysisRun({ evidence = null, run }) {
  return runAuditCoreJson([
    'rust-analysis-run-merge',
    '--input', '-',
  ], 'mergeRustAnalysisRun', {
    input: JSON.stringify({ evidence, run }),
  });
}

export function buildLifecycleSummary(blocks) {
  return runAuditCoreJson([
    'lifecycle-summary',
    '--input', '-',
  ], 'buildLifecycleSummary', {
    input: JSON.stringify(blocks ?? {}),
  });
}

export function buildManifestMeta({
  generated,
  profile,
  root,
  outDir,
}) {
  return runAuditCoreJson([
    'manifest-meta',
    '--generated', generated,
    '--profile', profile,
    '--root', root,
    '--output', outDir,
  ], 'buildManifestMeta');
}

export function buildManifestRoot(input) {
  return runAuditCoreJson([
    'manifest-root',
    '--input', '-',
  ], 'buildManifestRoot', {
    input: JSON.stringify(input ?? {}),
  });
}

export function buildManifestEvidenceUpdate(evidence) {
  return runAuditCoreJson([
    'manifest-evidence-update',
    '--input', '-',
  ], 'buildManifestEvidenceUpdate', {
    input: JSON.stringify({ evidence }),
  });
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
  onArtifactRead,
}) {
  const triage = loadArtifact(outDir, 'triage.json', { onRead: onArtifactRead });
  const symbols = loadArtifact(outDir, 'symbols.json', { onRead: onArtifactRead });
  const resolverDiagnostics = loadArtifact(outDir, 'resolver-diagnostics.json', { onRead: onArtifactRead });
  loadArtifact(outDir, 'resolver-capabilities.json', { onRead: onArtifactRead });
  loadArtifact(outDir, 'framework-resource-surfaces.json', { onRead: onArtifactRead });
  loadArtifact(outDir, 'unused-deps.json', { onRead: onArtifactRead });
  loadArtifact(outDir, 'block-clones.json', { onRead: onArtifactRead });
  const entrySurface = loadArtifact(outDir, 'entry-surface.json', { onRead: onArtifactRead });
  const deadClassify = loadArtifact(outDir, 'dead-classify.json', { onRead: onArtifactRead });
  const manifestEvidence = buildManifestEvidenceSummaryFromFile(root, outDir, {
    includeTests,
    production,
    excludes,
    autoExcludes,
    generatedArtifactsMode,
  });
  const rustAnalysisForBlindZones = rustAnalysisRun?.ran === true ? manifestEvidence.rustAnalysis : null;

  return {
    scanRange: manifestEvidence.scanRange,
    confidence: manifestEvidence.confidence,
    resolverDiagnostics: manifestEvidence.resolverDiagnostics,
    blindZones: detectBlindZones({
      triage,
      symbols,
      deadClassify,
      entrySurface,
      resolverDiagnostics,
      rustAnalysis: rustAnalysisForBlindZones,
    }),
    rustAnalysis: manifestEvidence.rustAnalysis,
    generatedArtifacts: manifestEvidence.generatedArtifacts,
    frameworkResourceSurfaces: manifestEvidence.frameworkResourceSurfaces,
    unusedDependencies: manifestEvidence.unusedDependencies,
    blockClones: manifestEvidence.blockClones,
    sfcEvidence: manifestEvidence.sfcEvidence,
    livingAudit: manifestEvidence.livingAudit,
  };
}

export function refreshManifestEvidence(manifest, options) {
  const evidence = buildManifestEvidence(options);
  Object.assign(manifest, buildManifestEvidenceUpdate(evidence));
}
