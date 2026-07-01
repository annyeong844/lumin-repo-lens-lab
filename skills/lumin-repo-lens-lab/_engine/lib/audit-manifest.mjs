// _lib/audit-manifest.mjs
//
// Helpers for audit-repo.mjs manifest evidence and artifact enumeration.
// NO producer orchestration. Migrated manifest contracts call lumin-audit-core.

import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
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

function runAuditCoreJson(args, label) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw new Error(`${label}: lumin-audit-core binary missing at ${command}; run cargo build --manifest-path experiments/Cargo.toml -p lumin-audit-core`);
  }
  const stdout = execFileSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  return JSON.parse(stdout);
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
  const rustAnalysisUsable = options.rustAnalysisUsable ?? true;
  return runAuditCoreJson([
    'artifact-registry',
    '--output', outDir,
    ...(rustAnalysisUsable ? ['--rust-analysis-ran'] : []),
  ], 'collectProducedArtifacts');
}

export function buildProducerPerformanceSummaryFromFile(artifactPath) {
  return runAuditCoreJson([
    'producer-performance-summary',
    '--artifact', artifactPath,
  ], 'buildProducerPerformanceSummary');
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
  manifest.scanRange = evidence.scanRange;
  manifest.confidence = evidence.confidence;
  manifest.resolverDiagnostics = evidence.resolverDiagnostics;
  manifest.blindZones = evidence.blindZones;
  manifest.rustAnalysis = evidence.rustAnalysis;
  manifest.generatedArtifacts = evidence.generatedArtifacts;
  manifest.frameworkResourceSurfaces = evidence.frameworkResourceSurfaces;
  manifest.unusedDependencies = evidence.unusedDependencies;
  manifest.blockClones = evidence.blockClones;
  manifest.sfcEvidence = evidence.sfcEvidence;
  manifest.livingAudit = evidence.livingAudit;
}
