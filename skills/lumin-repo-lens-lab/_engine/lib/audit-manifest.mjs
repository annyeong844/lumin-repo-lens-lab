// _lib/audit-manifest.mjs
//
// Helpers for audit-repo.mjs manifest evidence and artifact enumeration.
// NO producer orchestration. Migrated manifest contracts call lumin-audit-core.

import { runAuditCoreJson, runAuditCoreJsonResultFile } from './audit-core.mjs';

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

export function buildManifestCloseoutUpdate({
  outDir,
  producerPerformancePath,
  rustAnalysis,
  topologyMermaidPath,
  auditSummaryPath,
  reviewPackPath,
}) {
  return runAuditCoreJson([
    'manifest-closeout-update',
    '--input', '-',
  ], 'buildManifestCloseoutUpdate', {
    input: JSON.stringify({
      output: outDir,
      producerPerformancePath,
      rustAnalysis,
      companion: {
        topologyMermaidPath,
        auditSummaryPath,
        reviewPackPath,
      },
    }),
  });
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

export function writeManifestFile(outDir, manifest) {
  return runAuditCoreJson([
    'manifest-write',
    '--output', outDir,
    '--input', '-',
  ], 'writeManifestFile', {
    input: JSON.stringify({ manifest: manifest ?? null }),
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
