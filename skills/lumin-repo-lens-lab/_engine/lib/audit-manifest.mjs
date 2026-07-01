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

const LIVING_AUDIT_DOC_CANDIDATES = [
  'docs/current/audit/lumin-structural-audit.md',
  'LUMIN_REPO_LENS.md',
  'LUMIN_AUDIT.md',
  'TECH_DEBT_AUDIT.md',
];

function detectLivingAuditDocs(root) {
  const docs = [];
  for (const rel of LIVING_AUDIT_DOC_CANDIDATES) {
    const abs = path.join(root, rel);
    if (!existsSync(abs)) continue;
    docs.push({
      path: rel,
      absolutePath: abs,
    });
  }
  return {
    preferredPath: LIVING_AUDIT_DOC_CANDIDATES[0],
    existingDocs: docs,
    action: docs.length > 0
      ? 'read-and-update-before-final-answer'
      : 'create-only-on-explicit-tracking-request',
  };
}

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

function buildRustAnalysisSummaryFromFile(root, outDir) {
  const artifactPath = path.join(outDir, 'rust-analyzer-health.latest.json');
  if (!existsSync(artifactPath)) return null;
  return runAuditCoreJson([
    'rust-analysis-summary',
    '--root', root,
    '--artifact', artifactPath,
  ], 'buildRustAnalysisSummary');
}

function buildGeneratedArtifactsSummaryFromFile(root, outDir, symbols, {
  includeTests = true,
  excludes = [],
  generatedArtifactsMode = 'default',
} = {}) {
  const args = [
    'generated-artifacts-summary',
    '--root', root,
    '--generated-artifacts', generatedArtifactsMode,
    includeTests ? '--include-tests' : '--no-include-tests',
  ];
  if (symbols && typeof symbols === 'object') {
    args.push('--symbols', path.join(outDir, 'symbols.json'));
  }
  for (const exclude of excludes) {
    args.push('--exclude', exclude);
  }
  return runAuditCoreJson(args, 'buildGeneratedArtifactsSummary');
}

function buildArtifactSummaryFromFile(outDir, artifact, artifactName, artifactKind, label) {
  if (!artifact || typeof artifact !== 'object') return null;
  return runAuditCoreJson([
    'artifact-summary',
    '--artifact-kind', artifactKind,
    '--artifact', path.join(outDir, artifactName),
  ], label);
}

function pushArtifactPathArg(args, flag, artifact, outDir, artifactName) {
  if (!artifact || typeof artifact !== 'object') return;
  args.push(flag, path.join(outDir, artifactName));
}

function buildResolverDiagnosticsSummaryFromFile(outDir, {
  symbols = null,
  resolverCapabilities = null,
  resolverDiagnostics = null,
} = {}) {
  const args = ['resolver-diagnostics-summary'];
  pushArtifactPathArg(args, '--symbols', symbols, outDir, 'symbols.json');
  pushArtifactPathArg(
    args,
    '--resolver-capabilities',
    resolverCapabilities,
    outDir,
    'resolver-capabilities.json'
  );
  pushArtifactPathArg(
    args,
    '--resolver-diagnostics',
    resolverDiagnostics,
    outDir,
    'resolver-diagnostics.json'
  );
  return runAuditCoreJson(args, 'buildResolverDiagnosticsSummary');
}

function buildManifestCoreSummaryFromFile(root, outDir, {
  includeTests,
  production,
  excludes = [],
  autoExcludes = [],
  triage = null,
  symbols = null,
} = {}) {
  const args = [
    'manifest-core-summary',
    '--root', root,
    includeTests ? '--include-tests' : '--no-include-tests',
    production ? '--production' : '--no-production',
  ];
  pushArtifactPathArg(args, '--triage', triage, outDir, 'triage.json');
  pushArtifactPathArg(args, '--symbols', symbols, outDir, 'symbols.json');
  for (const exclude of excludes) {
    args.push('--exclude', exclude);
  }
  for (const autoExclude of autoExcludes) {
    args.push('--auto-exclude', autoExclude);
  }
  return runAuditCoreJson(args, 'buildManifestCoreSummary');
}

export function collectProducedArtifacts(outDir, options = {}) {
  const rustAnalysisUsable = options.rustAnalysisUsable ?? true;
  return runAuditCoreJson([
    'artifact-registry',
    '--output', outDir,
    ...(rustAnalysisUsable ? ['--rust-analysis-ran'] : []),
  ], 'collectProducedArtifacts');
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
  const resolverCapabilities = loadArtifact(outDir, 'resolver-capabilities.json', { onRead: onArtifactRead });
  const resolverDiagnostics = loadArtifact(outDir, 'resolver-diagnostics.json', { onRead: onArtifactRead });
  const frameworkResourceSurfaces = loadArtifact(outDir, 'framework-resource-surfaces.json', { onRead: onArtifactRead });
  const unusedDeps = loadArtifact(outDir, 'unused-deps.json', { onRead: onArtifactRead });
  const blockClones = loadArtifact(outDir, 'block-clones.json', { onRead: onArtifactRead });
  const entrySurface = loadArtifact(outDir, 'entry-surface.json', { onRead: onArtifactRead });
  const deadClassify = loadArtifact(outDir, 'dead-classify.json', { onRead: onArtifactRead });
  const rustAnalysis = buildRustAnalysisSummaryFromFile(root, outDir);
  const rustAnalysisForBlindZones = rustAnalysisRun?.ran === true ? rustAnalysis : null;

  const manifestCore = buildManifestCoreSummaryFromFile(root, outDir, {
    includeTests,
    production,
    excludes,
    autoExcludes,
    triage,
    symbols,
  });

  return {
    scanRange: manifestCore.scanRange,
    confidence: manifestCore.confidence,
    resolverDiagnostics: buildResolverDiagnosticsSummaryFromFile(outDir, {
      symbols,
      resolverCapabilities,
      resolverDiagnostics,
    }),
    blindZones: detectBlindZones({
      triage,
      symbols,
      deadClassify,
      entrySurface,
      resolverDiagnostics,
      rustAnalysis: rustAnalysisForBlindZones,
    }),
    rustAnalysis,
    generatedArtifacts: buildGeneratedArtifactsSummaryFromFile(root, outDir, symbols, {
      root,
      includeTests,
      excludes,
      generatedArtifactsMode,
    }),
    frameworkResourceSurfaces: buildArtifactSummaryFromFile(
      outDir,
      frameworkResourceSurfaces,
      'framework-resource-surfaces.json',
      'framework-resource-surfaces',
      'buildFrameworkResourceSurfacesSummary',
    ),
    unusedDependencies: buildArtifactSummaryFromFile(
      outDir,
      unusedDeps,
      'unused-deps.json',
      'unused-deps',
      'buildUnusedDependenciesSummary',
    ),
    blockClones: buildArtifactSummaryFromFile(
      outDir,
      blockClones,
      'block-clones.json',
      'block-clones',
      'buildBlockClonesSummary',
    ),
    sfcEvidence: manifestCore.sfcEvidence,
    livingAudit: detectLivingAuditDocs(root),
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
