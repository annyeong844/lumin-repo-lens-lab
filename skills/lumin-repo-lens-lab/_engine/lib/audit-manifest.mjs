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
import {
  buildBlockedCandidateHintFamilyCounts,
  buildBlockedCandidateHintReasonCounts,
  sortCounterObject,
} from './resolver-blocked-hints.mjs';

const LIVING_AUDIT_DOC_CANDIDATES = [
  'docs/current/audit/lumin-structural-audit.md',
  'LUMIN_REPO_LENS.md',
  'LUMIN_AUDIT.md',
  'TECH_DEBT_AUDIT.md',
];

const RESOLVER_BLOCKED_CANDIDATE_HINT_SAMPLE_LIMIT = 10;

function languagesFromTriage(triage) {
  const byLanguage = triage?.byLanguage ?? triage?.languages ?? triage?.summary?.byLanguage;
  if (byLanguage && typeof byLanguage === 'object') return Object.keys(byLanguage);

  const shape = triage?.shape ?? {};
  const languages = [];
  if ((shape.tsFiles ?? 0) > 0) languages.push('ts');
  if ((shape.jsFiles ?? 0) > 0) languages.push('js');
  if ((shape.pyFiles ?? 0) > 0) languages.push('py');
  if ((shape.goFiles ?? 0) > 0) languages.push('go');
  if ((shape.rustFiles ?? shape.rsFiles ?? 0) > 0) languages.push('rs');
  return languages.length > 0 ? languages : null;
}

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

function countObjectFromSummary(summary) {
  if (!summary || typeof summary !== 'object') return null;
  const out = [];
  for (const [reason, value] of Object.entries(summary)) {
    const count = typeof value?.count === 'number'
      ? value.count
      : typeof value === 'number'
        ? value
        : null;
    if (count === null) continue;
    out.push({ reason, count });
  }
  return out.length ? out : null;
}

function unresolvedSpecifierRoot(specifier) {
  if (typeof specifier !== 'string' || specifier.length === 0) return null;
  if (/^[@~#]\//.test(specifier)) return specifier.slice(0, 2);
  if (specifier.startsWith('@')) {
    const parts = specifier.split('/');
    if (parts[0] && parts[1]) return `${parts[0]}/${parts[1]}`;
  }
  const first = specifier.split('/')[0];
  return first || null;
}

function topUnresolvedReasons(symbols) {
  const fromSummary = countObjectFromSummary(symbols?.unresolvedInternalSummaryByReason);
  const reasons = fromSummary ?? (() => {
    const counts = new Map();
    for (const record of symbols?.unresolvedInternalSpecifierRecords ?? []) {
      const reason = record?.reason ?? 'unknown-internal-resolution';
      counts.set(reason, (counts.get(reason) ?? 0) + 1);
    }
    return [...counts.entries()].map(([reason, count]) => ({ reason, count }));
  })();

  return reasons
    .sort((a, b) => b.count - a.count || a.reason.localeCompare(b.reason))
    .slice(0, 10);
}

function buildTopSpecifierRoots(symbols) {
  const groups = new Map();

  for (const record of symbols?.unresolvedInternalSpecifierRecords ?? []) {
    const specifierRoot = unresolvedSpecifierRoot(record?.specifier);
    if (!specifierRoot) continue;
    if (!groups.has(specifierRoot)) {
      groups.set(specifierRoot, {
        specifierRoot,
        count: 0,
        reasons: new Map(),
        examples: [],
      });
    }
    const group = groups.get(specifierRoot);
    const reason = record?.reason ?? 'unknown-internal-resolution';
    group.count++;
    group.reasons.set(reason, (group.reasons.get(reason) ?? 0) + 1);
    group.examples.push({
      specifier: record.specifier,
      consumerFile: record.consumerFile ?? null,
    });
  }

  return [...groups.values()]
    .map((group) => ({
      specifierRoot: group.specifierRoot,
      count: group.count,
      reasons: sortCounterObject(group.reasons),
      examples: group.examples
        .sort((a, b) =>
          `${a.consumerFile ?? ''}|${a.specifier ?? ''}`
            .localeCompare(`${b.consumerFile ?? ''}|${b.specifier ?? ''}`))
        .slice(0, 5),
    }))
    .sort((a, b) =>
      b.count - a.count ||
      a.specifierRoot.localeCompare(b.specifierRoot))
    .slice(0, 20);
}

function buildResolverDiagnosticsSummary(symbols, {
  resolverCapabilities = null,
  resolverDiagnostics = null,
} = {}) {
  const blockedCandidateHints = Array.isArray(resolverDiagnostics?.blockedCandidateHints)
    ? resolverDiagnostics.blockedCandidateHints.slice(0, RESOLVER_BLOCKED_CANDIDATE_HINT_SAMPLE_LIMIT)
    : [];
  return {
    resolverVersion:
      resolverDiagnostics?.resolverVersion ??
      resolverCapabilities?.resolverVersion ??
      null,
    resolverCapabilityArtifact: resolverCapabilities ? 'resolver-capabilities.json' : null,
    resolverDiagnosticsArtifact: resolverDiagnostics ? 'resolver-diagnostics.json' : null,
    unresolvedInternal: symbols?.uses?.unresolvedInternal ?? null,
    unresolvedInternalRatio: symbols?.uses?.unresolvedInternalRatio ?? null,
    blindZoneCount: resolverDiagnostics?.summary?.blindZoneCount ?? null,
    blockedCandidateHintCount: resolverDiagnostics?.summary?.blockedCandidateHintCount ?? null,
    blockedCandidateHintSampleLimit: resolverDiagnostics ? RESOLVER_BLOCKED_CANDIDATE_HINT_SAMPLE_LIMIT : null,
    blockedCandidateHints,
    blockedCandidateHintReasonCounts: buildBlockedCandidateHintReasonCounts(
      resolverDiagnostics?.blockedCandidateHints
    ),
    blockedCandidateHintFamilyCounts: buildBlockedCandidateHintFamilyCounts(
      resolverDiagnostics?.blockedCandidateHints
    ),
    candidateTargetCount: resolverDiagnostics?.summary?.candidateTargetCount ?? null,
    topFamilies: resolverDiagnostics?.summary?.topFamilies ?? [],
    topAffectedPackageScopes:
      resolverDiagnostics?.summary?.topAffectedPackageScopes ?? [],
    topUnresolvedReasons:
      resolverDiagnostics?.summary?.topUnresolvedReasons ?? topUnresolvedReasons(symbols),
    topSpecifierRoots:
      resolverDiagnostics?.summary?.topSpecifierRoots ?? buildTopSpecifierRoots(symbols),
    topUnresolvedSpecifiers: (symbols?.topUnresolvedSpecifiers ?? []).slice(0, 20),
  };
}

function numberOrZero(value) {
  return typeof value === 'number' && Number.isFinite(value) ? value : 0;
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

function buildSfcEvidenceSummary(symbols) {
  if (!symbols || typeof symbols !== 'object') return null;
  const uses = symbols.uses && typeof symbols.uses === 'object'
    ? symbols.uses
    : {};
  const byLane = {
    scriptImportConsumers: numberOrZero(uses.sfcScriptConsumers),
    scriptSrcReachability: numberOrZero(uses.sfcScriptSrcReachability),
    styleAssetReferences: numberOrZero(uses.sfcStyleAssetReferences),
    templateComponentRefs: numberOrZero(uses.sfcTemplateComponentRefs),
    globalComponentRegistrations: numberOrZero(uses.sfcGlobalComponentRegistrations),
    generatedComponentManifests: numberOrZero(uses.sfcGeneratedComponentManifests),
    frameworkConventionComponents: numberOrZero(uses.sfcFrameworkConventionComponents),
  };
  const totalEvidenceCount = Object.values(byLane)
    .reduce((sum, count) => sum + count, 0);
  if (totalEvidenceCount <= 0) return null;

  return {
    artifact: 'symbols.json',
    status: 'complete',
    scriptImportConsumerCount: byLane.scriptImportConsumers,
    reachabilityOnlyCount: byLane.scriptSrcReachability,
    reviewOnlyEvidenceCount:
      byLane.styleAssetReferences +
      byLane.templateComponentRefs +
      byLane.globalComponentRegistrations +
      byLane.generatedComponentManifests +
      byLane.frameworkConventionComponents,
    totalEvidenceCount,
    byLane,
    scanGapStillApplies: true,
  };
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

  const parseErrors = (() => {
    const w = (symbols?.meta?.warnings ?? []).find((x) =>
      x?.kind === 'parse-errors' || x?.type === 'parse-errors' || x?.code === 'parse-errors');
    return w?.count ?? symbols?.filesWithParseErrors?.length ?? 0;
  })();

  return {
    scanRange: {
      root,
      includeTests,
      production,
      excludes,
      autoExcludes,
      languages: languagesFromTriage(triage),
      files: triage?.summary?.files ?? triage?.files ?? triage?.shape?.totalFiles ?? null,
    },
    confidence: {
      parseErrors,
      unresolvedInternalRatio: symbols?.uses?.unresolvedInternalRatio ?? null,
      externalImports: symbols?.uses?.external ?? null,
      resolvedInternal: symbols?.uses?.resolvedInternal ?? null,
      unresolvedInternal: symbols?.uses?.unresolvedInternal ?? null,
    },
    resolverDiagnostics: buildResolverDiagnosticsSummary(symbols, {
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
    sfcEvidence: buildSfcEvidenceSummary(symbols),
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
