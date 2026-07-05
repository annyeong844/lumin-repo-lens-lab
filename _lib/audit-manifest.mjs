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
  resultFile = false,
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
  const runner = resultFile ? runAuditCoreJsonResultFile : runAuditCoreJson;
  return runner(args, label, runOptions);
}

function buildManifestEvidenceSummaryWithReadsFromFile(root, outDir, options = {}) {
  return runManifestEvidenceCommand(
    'manifest-evidence-summary-with-reads',
    'buildManifestEvidenceSummary',
    root,
    outDir,
    { ...options, resultFile: true },
  );
}

function buildManifestEvidenceRefreshWithReadsFromFile(root, outDir, options = {}) {
  return runManifestEvidenceCommand(
    'manifest-evidence-refresh-with-reads',
    'refreshManifestEvidence',
    root,
    outDir,
    { ...options, resultFile: true },
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
  return runJsonInputCommand(
    'artifact-read-metrics-summary',
    'buildArtifactReadMetricsSummary',
    input ?? {},
  );
}

const ARTIFACT_READ_EVENTS_SCHEMA_VERSION = 'lumin-audit-artifact-read-events.v1';

function runJsonInputCommand(command, label, input, { args = [] } = {}) {
  return runAuditCoreJson([command, '--input', '-', ...args], label, {
    input: JSON.stringify(input ?? {}),
  });
}

function runJsonInputResultFileCommand(command, label, input) {
  return runAuditCoreJsonResultFile([command, '--input', '-'], label, {
    input: JSON.stringify(input ?? {}),
  });
}

export function writeTopologyMermaidWithAuditCore({ topology, outputPath, options = {} }) {
  return runJsonInputResultFileCommand(
    'topology-mermaid-render',
    'writeTopologyMermaid',
    {
      schemaVersion: 'lumin-topology-mermaid-render-request.v1',
      topology,
      outputPath,
      options,
    },
  );
}

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

export function finalizeAuditRun({
  manifest,
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
  topologyMermaidPath = null,
  auditSummaryPath = null,
  reviewPackPath = null,
}) {
  return runJsonInputCommand(
    'finalize-audit-run',
    'finalizeAuditRun',
    {
      manifest: manifest ?? null,
      context: {
        generated,
        root,
        output: outDir,
        profile,
        includeTests,
        production,
        excludes,
        autoExcludes,
        noIncremental,
        cacheRoot,
        clearIncrementalCache,
        generatedArtifactsMode,
      },
      observations: {
        artifactReads,
        rustAnalysis,
        commandsRun,
        skipped,
      },
      rustAnalysis,
      companion: {
        topologyMermaidPath,
        auditSummaryPath,
        reviewPackPath,
      },
    },
  );
}

export function executeBaseRuntime(request) {
  return runJsonInputCommand('execute-base-runtime', 'executeBaseRuntime', request);
}

export function executeCanonDraftLifecycle(request) {
  return runJsonInputCommand('execute-canon-draft', 'executeCanonDraftLifecycle', request);
}

export function executeCheckCanonLifecycle(request) {
  return runJsonInputCommand('execute-check-canon', 'executeCheckCanonLifecycle', request);
}

export function resolvePreWriteRoute(request) {
  return runJsonInputCommand('pre-write-route', 'resolvePreWriteRoute', request);
}

export function executeRustPreWriteLifecycle(request) {
  return runJsonInputResultFileCommand(
    'execute-rust-pre-write',
    'executeRustPreWriteLifecycle',
    request,
  );
}

export function executeJsPreWriteLifecycle(request) {
  return runJsonInputResultFileCommand(
    'execute-js-pre-write',
    'executeJsPreWriteLifecycle',
    request,
  );
}

export function executePostWriteLifecycle(request) {
  return runJsonInputResultFileCommand(
    'execute-post-write',
    'executePostWriteLifecycle',
    request,
  );
}

export function applyLifecycleExitPolicy(request) {
  return runJsonInputCommand('lifecycle-exit-policy', 'applyLifecycleExitPolicy', request);
}

export function evaluateLifecycleRequestGuard(request) {
  return runJsonInputCommand(
    'lifecycle-request-guard',
    'evaluateLifecycleRequestGuard',
    request,
  );
}

export function buildManifestCloseoutUpdate({
  outDir,
  producerPerformancePath,
  rustAnalysis,
  topologyMermaidPath,
  auditSummaryPath,
  reviewPackPath,
}) {
  return runJsonInputCommand(
    'manifest-closeout-update',
    'buildManifestCloseoutUpdate',
    {
      output: outDir,
      producerPerformancePath,
      rustAnalysis,
      companion: {
        topologyMermaidPath,
        auditSummaryPath,
        reviewPackPath,
      },
    },
  );
}

export function buildManifestLifecycleUpdate(blocks) {
  return runJsonInputCommand(
    'manifest-lifecycle-update',
    'buildManifestLifecycleUpdate',
    blocks,
  );
}

export function buildManifestRootWithEvidence({
  generated,
  profile,
  root,
  outDir,
  commandsRun = [],
  skipped = [],
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  generatedArtifactsMode = 'default',
  rustAnalysisRun = null,
  rustAnalysisRan = false,
  onArtifactRead,
}) {
  const result = runJsonInputResultFileCommand(
    'manifest-root-with-evidence',
    'buildManifestRootWithEvidence',
    {
      generated,
      profile,
      root,
      output: outDir,
      commandsRun,
      skipped,
      includeTests,
      production,
      excludes,
      autoExcludes,
      generatedArtifactsMode,
      rustAnalysisRun,
      rustAnalysisRan: rustAnalysisRun ? false : rustAnalysisRan,
    },
  );
  observeRustArtifactReads(result.artifactReads, onArtifactRead);
  return result.manifest ?? {};
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

export function applyLifecycleAndRefreshManifestEvidence({
  manifest,
  lifecycle,
  root,
  outDir,
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  generatedArtifactsMode = 'default',
  rustAnalysisRun = null,
  rustAnalysisRan = false,
  onArtifactRead,
}) {
  const result = runJsonInputResultFileCommand(
    'manifest-lifecycle-evidence-refresh',
    'applyLifecycleAndRefreshManifestEvidence',
    {
      manifest: manifest ?? null,
      lifecycle: lifecycle ?? {},
      evidence: {
        root,
        output: outDir,
        includeTests,
        production,
        excludes,
        autoExcludes,
        generatedArtifactsMode,
        rustAnalysisRun,
        rustAnalysisRan: rustAnalysisRun ? false : rustAnalysisRan,
      },
    },
  );
  observeRustArtifactReads(result.artifactReads, onArtifactRead);
  return result.manifest ?? manifest;
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
