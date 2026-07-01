// Regression guard for v1.9.9 "Product UX Pass":
//   - _lib/blind-zones.mjs detectBlindZones predicate
//   - audit-repo.mjs orchestrator (profile handling, manifest, skip
//     semantics)

import { execFileSync, execSync, spawnSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, rmSync, mkdtempSync, existsSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { detectBlindZones, formatBlindZonesSummary } from '../_lib/blind-zones.mjs';
import { detectMaintainerSelfAuditExcludes, mergeExcludes } from '../_lib/self-audit-excludes.mjs';
import { buildManifestEvidence } from '../_lib/audit-manifest.mjs';
import { renderAuditSummary } from '../_lib/audit-summary.mjs';
import { renderAuditReviewPack } from '../_lib/audit-review-pack.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const AUDIT_REPO = path.join(DIR, 'audit-repo.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ───────────────────────────────────────────────────────────
// A. _lib/blind-zones.mjs
// ───────────────────────────────────────────────────────────

// A0: audit-summary.latest.md is an artifact map, not a ranking engine.
{
  const typo = spawnSync(NODE, [AUDIT_REPO, '--profil', 'full'], { encoding: 'utf8' });
  assert('A0pre. audit-repo rejects unknown long options before falling back to defaults',
    typo.status === 2 &&
    /unknown option\(s\): --profil/.test(typo.stderr ?? ''),
    `status=${typo.status}, stderr=${typo.stderr}`);
  const badGeneratedMode = spawnSync(NODE, [
    AUDIT_REPO,
    '--root', DIR,
    '--generated-artifacts', 'run',
  ], { encoding: 'utf8' });
  assert('A0pre2. audit-repo rejects unsupported generated artifact modes',
    badGeneratedMode.status === 2 &&
      /unsupported --generated-artifacts mode: run/.test(badGeneratedMode.stderr ?? ''),
    `status=${badGeneratedMode.status}, stderr=${badGeneratedMode.stderr}`);

  const summary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-04-28T00:00:00.000Z' },
      profile: 'full',
      scanRange: { files: 10, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
      blindZones: [],
      livingAudit: {
        existingDocs: [{ path: 'docs/current/audit/lumin-structural-audit.md' }],
      },
    },
    checklistFacts: {
      E2_silent_catch: {
        gate: 'watch',
        count: 0,
        nonEmptyAnonymousCount: 1,
        unusedParamCount: 0,
        nonEmptyAnonymousSites: [{ file: 'src/errors.ts', line: 10 }],
      },
    },
    fixPlan: {
      summary: { REVIEW_FIX: 7 },
      reviewFixes: Array.from({ length: 7 }, (_, i) => ({
        finding: { file: `src/dead-${i}.ts`, line: i + 1, symbol: `Dead${i}` },
      })),
    },
    topology: {
      summary: { sccCount: 1 },
      sccs: [{ members: ['src/a.ts', 'src/b.ts', 'src/c.ts', 'src/d.ts'] }],
      largestFiles: [],
    },
    discipline: {
      totals: { ':any': 41, 'as any': 0, '@ts-ignore': 0, '@ts-expect-error': 108 },
      overallTopOffenders: [{ file: 'src/types.ts', total: 20, breakdown: { ':any': 20 } }],
    },
    callGraph: {
      summary: { semiDead: 2 },
      semiDeadList: [
        { file: 'src/test.ts', symbol: 'unusedImport', source: './helper' },
        { file: 'src/test2.ts', symbol: 'unusedImport2', source: './helper2' },
      ],
    },
    symbols: {
      meta: { supports: { anyContamination: true } },
      helperOwnersByIdentity: {
        'src/dirty.ts::dirtyHelper': {
          ownerFile: 'src/dirty.ts',
          exportedName: 'dirtyHelper',
          kind: 'FunctionDeclaration',
          line: 7,
          anyContamination: {
            label: 'severely-any-contaminated',
            labels: ['has-any', 'any-contaminated', 'severely-any-contaminated'],
            measurements: { explicitAnyCount: 3 },
          },
        },
      },
      typeOwnersByIdentity: {
        'src/dirty.ts::DirtyShape': {
          ownerFile: 'src/dirty.ts',
          exportedName: 'DirtyShape',
          kind: 'TSInterfaceDeclaration',
          line: 1,
          anyContamination: {
            label: 'severely-any-contaminated',
            labels: ['has-any', 'any-contaminated', 'severely-any-contaminated'],
            measurements: { explicitAnyCount: 3 },
          },
        },
      },
    },
  });

  assert('A0a. summary states that it is not a recommendation engine',
    summary.includes('# Audit Artifact Brief') &&
    summary.includes('not a recommendation engine') &&
    summary.includes('Do not paste it as the final user answer'),
    summary);
  assert('A0b. summary lists high-impact producer evidence as unranked measured cues',
    summary.includes('## Measured Cues (Unranked)') &&
    summary.includes('Runtime cycles: 1') &&
    summary.includes('Type-check escapes: 149') &&
    summary.includes('Call graph: semi-dead imports 2'),
    summary);
  assert('A0c. summary does not emit coding-agent prompts or numbered top recommendations',
    !summary.includes('Ask the coding agent:') &&
    !/^\d+\. /m.test(summary),
    summary);
  assert('A0d. artifact map names discipline and call-graph artifacts',
    summary.includes('`discipline.json`') &&
    summary.includes('`call-graph.json`') &&
    summary.includes('`symbols.json`'),
    summary);
  assert('A0e. summary exposes identity-level anyContamination as an unranked cue',
    summary.includes('Exported any-contamination: 1 severe type owner, 1 severe helper owner') &&
    summary.includes('symbols.json.typeOwnersByIdentity') &&
    summary.includes('src/dirty.ts::DirtyShape'),
    summary);
  assert('A0e2. full-profile summary reminds the model to offer optional expansion',
    summary.includes('## Expansion Hint') &&
    summary.includes('Full-profile evidence is available') &&
    summary.includes('full checklist로 펼쳐줘') &&
    summary.includes('formal report로 써줘') &&
    summary.includes('due-diligence handoff로 정리해줘'),
    summary);
  assert('A0e3. summary surfaces existing living audit docs as controller work',
    summary.includes('## Living Audit Tracking') &&
    summary.includes('docs/current/audit/lumin-structural-audit.md') &&
    summary.includes('Read and update the document before the final answer') &&
    summary.includes('NOT_RECHECKED') &&
    summary.includes('Do not ask a subagent to own this document'),
    summary);
  const caveatedPostWriteSummary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-04-28T00:00:00.000Z' },
      profile: 'quick',
      scanRange: { files: 1, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
      blindZones: [],
      postWrite: {
        requested: true,
        ran: true,
        silentNew: 0,
        baselineStatus: 'missing',
        scanRangeParity: 'baseline-missing',
        afterComplete: true,
      },
    },
  });
  assert('A0f. post-write summary caveats baseline-missing instead of saying clean zero',
    caveatedPostWriteSummary.includes('delta confidence is limited') &&
    caveatedPostWriteSummary.includes('baseline=missing') &&
    !caveatedPostWriteSummary.includes('## Expansion Hint') &&
    !caveatedPostWriteSummary.includes('found 0 new unplanned any-like escapes'),
    caveatedPostWriteSummary);
  const reviewPack = renderAuditReviewPack({
    manifest: { scanRange: { files: 10, languages: ['ts'], includeTests: true } },
    discipline: { totals: { ':any': 41 } },
    symbols: {
      meta: { supports: { anyContamination: true } },
      helperOwnersByIdentity: {
        'src/dirty.ts::dirtyHelper': {
          anyContamination: {
            label: 'severely-any-contaminated',
            labels: ['has-any', 'any-contaminated', 'severely-any-contaminated'],
            measurements: { explicitAnyCount: 3 },
          },
        },
      },
      typeOwnersByIdentity: {},
    },
  });
  assert('A0f. review pack Lane 2 reminds reviewer to inspect anyContamination owner maps',
    reviewPack.includes('Identity-level anyContamination: 0 severe type owners, 1 severe helper owner') &&
    reviewPack.includes('Inspect symbols.json owner maps') &&
    reviewPack.includes('Rust analyzer evidence is not available for this run; JS/TS clone and shape artifacts are not Rust evidence.') &&
    reviewPack.includes('Rust analyzer artifact not available in this run') &&
    !reviewPack.includes('Use rust-analyzer-health.latest.json, not JS/TS clone or shape artifacts, for Rust files.'),
    reviewPack);
  const reviewPackWithRustEvidence = renderAuditReviewPack({
    manifest: {
      scanRange: { files: 10, languages: ['ts', 'rs'], includeTests: true },
      rustAnalysis: {
        status: 'complete',
        available: true,
        files: 2,
        scanScope: { includeTests: false, exclude: ['generated'] },
      },
    },
    discipline: { totals: { ':any': 41 } },
    symbols: {
      meta: { supports: { anyContamination: true } },
      helperOwnersByIdentity: {},
      typeOwnersByIdentity: {},
    },
  });
  assert('A0f2. review pack only lists Rust analyzer artifact when manifest marks it available',
    reviewPackWithRustEvidence.includes('Use rust-analyzer-health.latest.json, not JS/TS clone or shape artifacts, for Rust files.') &&
    reviewPackWithRustEvidence.includes('Artifacts for the controller to inspect first: discipline.json, shape-index.json, function-clones.json, checklist-facts.json, symbols.json, rust-analyzer-health.latest.json') &&
    reviewPackWithRustEvidence.includes('Rust analyzer artifact available for 2 file(s) (production files only, 1 exclude pattern)'),
    reviewPackWithRustEvidence);
  const reviewPackWithResolverHints = renderAuditReviewPack({
    manifest: {
      scanRange: { files: 10, languages: ['ts'], includeTests: true },
      resolverDiagnostics: {
        blockedCandidateHintCount: 2,
        blockedCandidateHintSampleLimit: 10,
        blockedCandidateHintReasonCounts: [
          {
            reason: 'generated-consumer-blind-zone',
            count: 5,
            families: { 'generated-artifacts': 5 },
          },
          {
            reason: 'workspace-package-subpath-target-missing',
            count: 3,
            families: { 'workspace-packages': 3 },
          },
        ],
        blockedCandidateHintFamilyCounts: [
          {
            family: 'generated-artifacts',
            count: 5,
            reasons: { 'generated-consumer-blind-zone': 5 },
          },
          {
            family: 'workspace-packages',
            count: 3,
            reasons: { 'workspace-package-subpath-target-missing': 3 },
          },
        ],
        blockedCandidateHints: [
          {
            candidatePath: 'packages/core/src/dead.ts',
            specifier: '@repo/core/dead',
            reason: 'workspace-package-subpath-target-missing',
          },
          {
            candidatePath: 'packages/ui/src/Button.tsx',
            specifier: '@repo/ui/Button',
            reason: 'generated-consumer-blind-zone',
          },
        ],
      },
    },
    fixPlan: { summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 } },
    deadClassify: { summary: { excluded: {} } },
  });
  assert('A0f2. review pack Lane 3 warns about resolver blocked absence hints',
    reviewPackWithResolverHints.includes('Resolver blocked absence hints: 2') &&
      reviewPackWithResolverHints.includes('manifest sample limit 10') &&
      reviewPackWithResolverHints.includes('Resolver blocked absence distribution: reasons generated-consumer-blind-zone 5 (generated-artifacts 5), workspace-package-subpath-target-missing 3 (workspace-packages 3); families generated-artifacts 5 (generated-consumer-blind-zone 5), workspace-packages 3 (workspace-package-subpath-target-missing 3)') &&
      reviewPackWithResolverHints.includes('packages/core/src/dead.ts via @repo/core/dead (workspace-package-subpath-target-missing)') &&
      reviewPackWithResolverHints.includes('manifest.json.resolverDiagnostics.blockedCandidateHints') &&
      reviewPackWithResolverHints.includes('manifest.json.resolverDiagnostics.blockedCandidateHintReasonCounts') &&
      reviewPackWithResolverHints.includes('manifest.json.resolverDiagnostics.blockedCandidateHintFamilyCounts') &&
      reviewPackWithResolverHints.includes('resolver-diagnostics.json.blockedCandidateHints') &&
      reviewPackWithResolverHints.includes('before treating affected exports as absent'),
    reviewPackWithResolverHints);
  const manifestWithFrameworkResourceSurfaces = {
    meta: { generated: '2026-05-09T00:00:00.000Z' },
    profile: 'full',
    scanRange: { files: 12, languages: ['ts', 'js'], includeTests: true },
    confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
    blindZones: [],
    frameworkResourceSurfaces: {
      artifact: 'framework-resource-surfaces.json',
      totalFilesWithSurfaces: 4,
      byLane: {
        'framework-dispatch-entry': 2,
        'scaffold-template-resource': 1,
        'bundled-build-artifact': 1,
      },
      byConfidence: {
        grounded: 2,
        'resource-only': 1,
        'generated-output-review': 1,
      },
      topExamples: [
        {
          file: 'src/Button.stories.tsx',
          lanes: ['framework-dispatch-entry'],
          reasons: ['storybook-story-file'],
        },
      ],
    },
  };
  const summaryWithFrameworkResources = renderAuditSummary({
    manifest: manifestWithFrameworkResourceSurfaces,
  });
  assert('A0f3. summary surfaces framework/resource surface counts',
    summaryWithFrameworkResources.includes('Framework/resource surfaces: 4 files') &&
      summaryWithFrameworkResources.includes('framework-dispatch-entry 2') &&
      summaryWithFrameworkResources.includes('scaffold-template-resource 1') &&
      summaryWithFrameworkResources.includes('bundled-build-artifact 1') &&
      summaryWithFrameworkResources.includes('framework-resource-surfaces.json') &&
      summaryWithFrameworkResources.includes('before treating import absence as deadness'),
    summaryWithFrameworkResources);

  const reviewPackWithFrameworkResources = renderAuditReviewPack({
    manifest: manifestWithFrameworkResourceSurfaces,
    fixPlan: { summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 } },
    deadClassify: { summary: { excluded: {} } },
  });
  assert('A0f4. review pack Lane 3 mirrors framework/resource surface counts',
    reviewPackWithFrameworkResources.includes('Framework/resource surfaces: 4 files') &&
      reviewPackWithFrameworkResources.includes('framework-dispatch-entry 2') &&
      reviewPackWithFrameworkResources.includes('Read manifest.json.frameworkResourceSurfaces and framework-resource-surfaces.json'),
    reviewPackWithFrameworkResources);

  const manifestWithUnusedDependencies = {
    meta: { generated: '2026-05-24T00:00:00.000Z' },
    profile: 'full',
    scanRange: { files: 12, languages: ['ts', 'js'], includeTests: true },
    confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
    blindZones: [],
    artifactsProduced: ['unused-deps.json'],
    unusedDependencies: {
      artifact: 'unused-deps.json',
      schemaVersion: 'unused-deps.v1',
      policyVersion: 'unused-deps-review-policy-v1',
      status: 'complete',
      reviewUnusedCount: 2,
      mutedCount: 3,
      confidenceLimitedCount: 0,
      topReviewUnused: [
        { name: 'left-pad', packageRoot: '.', dependencyField: 'dependencies' },
      ],
    },
  };
  const summaryWithUnusedDependencies = renderAuditSummary({
    manifest: manifestWithUnusedDependencies,
  });
  const dependencySummaryLines = summaryWithUnusedDependencies
    .split('\n')
    .filter((line) => line.includes('Dependency hygiene'));
  assert('A0f5. summary surfaces unused-deps as review-only dependency hygiene counts',
    dependencySummaryLines.some((line) =>
      line.includes('2 review-only dependency declarations need inspection') &&
      line.includes('3 muted explanations') &&
      line.includes('manifest.json.unusedDependencies') &&
      line.includes('unused-deps.json') &&
      line.includes('before changing package manifests')) &&
      summaryWithUnusedDependencies.includes('`unused-deps.json`: review-only dependency declaration evidence') &&
      !summaryWithUnusedDependencies.includes('left-pad') &&
      !dependencySummaryLines.some((line) => /\b(safe|remove|delete|uninstall|drop|fix)\b/i.test(line)),
    summaryWithUnusedDependencies);

  const reviewPackWithUnusedDependencies = renderAuditReviewPack({
    manifest: manifestWithUnusedDependencies,
    fixPlan: { summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 } },
    deadClassify: { summary: { excluded: {} } },
  });
  const dependencyReviewLines = reviewPackWithUnusedDependencies
    .split('\n')
    .filter((line) => line.includes('Dependency hygiene'));
  assert('A0f6. review pack Lane 3 surfaces unused-deps without action wording',
    dependencyReviewLines.some((line) =>
      line.includes('Dependency hygiene review: inspect unused-deps.json before changing package manifests') &&
      line.includes('review-only=2; muted=3; confidence-limited=0')) &&
      reviewPackWithUnusedDependencies.includes('unused-deps.json') &&
      !reviewPackWithUnusedDependencies.includes('left-pad') &&
      !dependencyReviewLines.some((line) => /\b(safe|remove|delete|uninstall|drop|fix)\b/i.test(line)),
    reviewPackWithUnusedDependencies);

  const manifestWithUnavailableUnusedDependencies = {
    ...manifestWithUnusedDependencies,
    unusedDependencies: {
      artifact: 'unused-deps.json',
      schemaVersion: 'unused-deps.v1',
      status: 'unavailable',
      reason: 'input-artifact-missing',
      reviewUnusedCount: 0,
      mutedCount: 0,
      confidenceLimitedCount: 0,
    },
  };
  const unavailableSummary = renderAuditSummary({
    manifest: manifestWithUnavailableUnusedDependencies,
  });
  const unavailableReviewPack = renderAuditReviewPack({
    manifest: manifestWithUnavailableUnusedDependencies,
    fixPlan: { summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 } },
    deadClassify: { summary: { excluded: {} } },
  });
  assert('A0f7. dependency hygiene unavailable evidence renders incomplete instead of zero-clean',
    unavailableSummary.includes('Dependency hygiene: evidence incomplete; do not infer dependency declaration absence') &&
      unavailableReviewPack.includes('Dependency hygiene review: evidence incomplete; do not infer dependency declaration absence') &&
      unavailableSummary.includes('manifest.json.unusedDependencies') &&
      unavailableReviewPack.includes('unused-deps.json'),
    `${unavailableSummary}\n---\n${unavailableReviewPack}`);

  const manifestWithSfcEvidence = {
    meta: { generated: '2026-05-30T00:00:00.000Z' },
    profile: 'full',
    scanRange: { files: 12, languages: ['ts', 'vue', 'svelte', 'astro'], includeTests: true },
    confidence: { parseErrors: 0, unresolvedInternalRatio: 0 },
    blindZones: [
      { area: 'sfc-scan-gap', severity: 'scan-gap', details: { files: 3 } },
    ],
    sfcEvidence: {
      artifact: 'symbols.json',
      status: 'complete',
      scriptImportConsumerCount: 4,
      reachabilityOnlyCount: 2,
      reviewOnlyEvidenceCount: 13,
      totalEvidenceCount: 19,
      byLane: {
        scriptImportConsumers: 4,
        scriptSrcReachability: 2,
        styleAssetReferences: 3,
        templateComponentRefs: 5,
        globalComponentRegistrations: 2,
        generatedComponentManifests: 1,
        frameworkConventionComponents: 2,
      },
      scanGapStillApplies: true,
    },
  };
  const summaryWithSfcEvidence = renderAuditSummary({
    manifest: manifestWithSfcEvidence,
  });
  const sfcSummaryLines = summaryWithSfcEvidence
    .split('\n')
    .filter((line) => line.includes('SFC evidence'));
  assert('A0f8. summary surfaces SFC evidence as count-only review context',
    sfcSummaryLines.some((line) =>
      line.includes('19 records') &&
      line.includes('script imports 4') &&
      line.includes('template refs 5') &&
      line.includes('framework conventions 2') &&
      line.includes('manifest.json.sfcEvidence') &&
      line.includes('SFC arrays in `symbols.json`') &&
      line.includes('sfc-scan-gap still applies')) &&
      !sfcSummaryLines.some((line) => /\b(safe|remove|delete|uninstall|drop|fix)\b/i.test(line)),
    summaryWithSfcEvidence);

  const reviewPackWithSfcEvidence = renderAuditReviewPack({
    manifest: manifestWithSfcEvidence,
    fixPlan: { summary: { SAFE_FIX: 1, REVIEW_FIX: 0, DEGRADED: 0, MUTED: 0 } },
    deadClassify: { summary: { excluded: {} } },
  });
  const sfcReviewLines = reviewPackWithSfcEvidence
    .split('\n')
    .filter((line) => line.includes('SFC evidence review'));
  assert('A0f9. review pack Lane 3 surfaces SFC evidence without action wording',
    sfcReviewLines.some((line) =>
      line.includes('inspect manifest.json.sfcEvidence and SFC arrays in symbols.json') &&
      line.includes('template-refs=5') &&
      line.includes('review-only=13') &&
      line.includes('sfc-scan-gap still applies')) &&
      !sfcReviewLines.some((line) => /\b(safe|remove|delete|uninstall|drop|fix)\b/i.test(line)),
    reviewPackWithSfcEvidence);
}

// B1: Rust files present → scan-gap blind zone
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 100, rs: 42 } },
  });
  const rs = zones.find((z) => z.area === 'rs');
  assert('B1. Rust files trigger scan-gap (Do not make repo-wide absence claims)',
    rs && rs.severity === 'scan-gap' && rs.effect.includes('absence claims'),
    JSON.stringify(zones));
}

// B1b: SFC files are counted but not parsed yet → explicit scan-gap.
{
  const zones = detectBlindZones({
    triage: {
      shape: {
        totalFiles: 4,
        tsFiles: 1,
        jsFiles: 0,
        pyFiles: 0,
        goFiles: 0,
        sfcFiles: 3,
      },
      byLanguage: { ts: 1, vue: 1, svelte: 1, astro: 1 },
    },
  });
  const sfc = zones.find((z) => z.area === 'sfc-scan-gap');
  assert('B1b. SFC files trigger grouped scan-gap without per-extension noise',
    sfc &&
      sfc.severity === 'scan-gap' &&
      sfc.details.files === 3 &&
      sfc.details.languages.vue === 1 &&
      sfc.details.languages.svelte === 1 &&
      sfc.details.languages.astro === 1 &&
      !zones.some((z) => ['vue', 'svelte', 'astro'].includes(z.area)),
    JSON.stringify(zones));
}

// B2: Python files → precision-gap (method resolution)
{
  const zones = detectBlindZones({
    triage: { byLanguage: { py: 244 } },
  });
  const py = zones.find((z) => z.area === 'python-method-resolution');
  assert('B2. Python files trigger precision-gap (method-level claims degraded)',
    py && py.severity === 'precision-gap' &&
    py.effect.includes('Method-level'),
    JSON.stringify(zones));
}

// B2b: manifest confidence mirrors symbol-graph parse-error warnings.
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-manifest-parse-errors-'));
  try {
    writeFileSync(path.join(FX, 'triage.json'), JSON.stringify({
      shape: { totalFiles: 2, tsFiles: 1, jsFiles: 1 },
    }));
    writeFileSync(path.join(FX, 'symbols.json'), JSON.stringify({
      meta: {
        warnings: [
          { code: 'parse-errors', count: 2 },
        ],
      },
      uses: {
        unresolvedInternalRatio: 0,
        external: 0,
        resolvedInternal: 0,
        unresolvedInternal: 0,
      },
      filesWithParseErrors: ['src/a.js', 'src/b.js'],
    }));
    const evidence = buildManifestEvidence({
      root: FX,
      outDir: FX,
      includeTests: true,
      production: false,
    });
    assert('B2b. manifest confidence.parseErrors reads symbol warning code=parse-errors',
      evidence.confidence.parseErrors === 2,
      JSON.stringify(evidence.confidence));
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// B9: maintainer self-audit auto-excludes lab/corpus/generated mirrors
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-self-audit-excludes-'));
  try {
    mkdirSync(path.join(FX, '_lib'), { recursive: true });
    mkdirSync(path.join(FX, 'p6-corpus'), { recursive: true });
    mkdirSync(path.join(FX, 'output', 'corpus'), { recursive: true });
    mkdirSync(path.join(FX, 'skills', 'lumin-repo-lens-lab', '_engine'), { recursive: true });
    mkdirSync(path.join(FX, 'skills', 'lumin-repo-lens-lab', 'scripts'), { recursive: true });
    mkdirSync(path.join(FX, 'test-harness'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'lumin-repo-lens-lab-scripts', type: 'module' }));
    writeFileSync(path.join(FX, 'audit-repo.mjs'), 'export const root = 1;\n');

    const auto = detectMaintainerSelfAuditExcludes(FX);
    assert('B9a. maintainer checkout auto-excludes lab/corpus/generated mirrors',
      auto.includes('p6-corpus') &&
      auto.includes('output/corpus') &&
      auto.includes('skills/lumin-repo-lens-lab/_engine') &&
      auto.includes('skills/lumin-repo-lens-lab/scripts') &&
      auto.includes('test-harness'),
      JSON.stringify(auto));
    assert('B9b. user excludes are preserved and auto excludes are deduped',
      JSON.stringify(mergeExcludes(['output/corpus', 'custom'], auto).slice(0, 3)) ===
        JSON.stringify(['output/corpus', 'custom', 'p6-corpus']),
      JSON.stringify(mergeExcludes(['output/corpus', 'custom'], auto)));
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// B2b: Python files with unavailable extractor → scan-gap, not fake precision.
{
  const zones = detectBlindZones({
    triage: { shape: { totalFiles: 3, tsFiles: 2, jsFiles: 0, pyFiles: 1, goFiles: 0 } },
    symbols: {
      meta: {
        languageSupport: {
          python: { enabled: false, reason: 'python executable unavailable' },
        },
      },
    },
  });
  const py = zones.find((z) => z.area === 'python-scan-gap');
  assert('B2b. unavailable Python extractor triggers python-scan-gap',
    py && py.severity === 'scan-gap' && py.details.reason.includes('python'),
    JSON.stringify(zones));
}

// B2c: Go files with unavailable tree-sitter → scan-gap, not fake precision.
{
  const zones = detectBlindZones({
    triage: { shape: { totalFiles: 3, tsFiles: 2, jsFiles: 0, pyFiles: 0, goFiles: 1 } },
    symbols: {
      meta: {
        languageSupport: {
          go: { enabled: false, reason: 'tree-sitter unavailable' },
        },
      },
    },
  });
  const go = zones.find((z) => z.area === 'go-scan-gap');
  assert('B2c. unavailable Go extractor triggers go-scan-gap',
    go && go.severity === 'scan-gap' && go.details.reason.includes('tree-sitter'),
    JSON.stringify(zones));
}

// B3: High resolver blindness → confidence-gap
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 100 } },
    symbols: {
      uses: { unresolvedInternalRatio: 0.22, unresolvedInternal: 50 },
      topUnresolvedSpecifiers: [{ specifierPrefix: '@/' }],
    },
  });
  const r = zones.find((z) => z.area === 'resolver');
  assert('B3. unresolvedInternalRatio >= 15% triggers confidence-gap pointing at FP-36',
    r && r.severity === 'confidence-gap' && r.effect.includes('FP-36'),
    JSON.stringify(zones));
  assert('B3b. resolver blind-zone exposes threshold policy metadata',
    r?.details?.thresholdPolicy?.policyId === 'resolver-blind-zone-policy' &&
      r.details.thresholdPolicy.policyVersion === 'resolver-blind-zone-policy-v1' &&
      r.details.thresholdPolicy.thresholds?.unresolvedRatio === 0.15,
    JSON.stringify(r?.details, null, 2));
}

// B4: Low resolver blindness → no confidence-gap
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 100 } },
    symbols: {
      uses: { unresolvedInternalRatio: 0.02, unresolvedInternal: 3 },
    },
  });
  assert('B4. unresolvedInternalRatio < 15% does NOT trigger confidence-gap',
    !zones.find((z) => z.area === 'resolver'),
    JSON.stringify(zones));
}

// B4a: Low ratio but high absolute unresolved count still needs a resolver warning.
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 5000 } },
    symbols: {
      uses: { unresolvedInternalRatio: 0.07, unresolvedInternal: 1200 },
      topUnresolvedSpecifiers: [{ specifierPrefix: '@workspace/pkg', count: 120 }],
      unresolvedInternalSpecifierRecords: [
        { specifier: '@workspace/pkg/generated', reason: 'tsconfig-path-target-missing' },
        { specifier: '@workspace/pkg/generated2', reason: 'tsconfig-path-target-missing' },
        { specifier: '@workspace/pkg/subpath', reason: 'workspace-package-subpath-target-missing' },
      ],
    },
  });
  const r = zones.find((z) => z.area === 'resolver');
  assert('B4a. high absolute unresolved count triggers confidence-gap below ratio threshold',
      r && r.severity === 'confidence-gap' &&
      r.details.unresolvedInternal === 1200 &&
      r.details.trigger === 'absolute-count' &&
      r.details.topUnresolvedReasons?.[0]?.reason === 'tsconfig-path-target-missing' &&
      r.details.topUnresolvedReasons?.[0]?.count === 2,
    JSON.stringify(zones));
}

// B4a2: Resolver blind-zone details prefer the pre-grouped symbols summary when available.
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 5000 } },
    symbols: {
      uses: { unresolvedInternalRatio: 0.06, unresolvedInternal: 1300 },
      topUnresolvedSpecifiers: [{ specifierPrefix: '@workspace/', count: 800 }],
      unresolvedInternalSummaryByReason: {
        'workspace-package-subpath-target-missing': {
          count: 12,
          spaces: { type: 12, value: 0, unknown: 0 },
          resolverStages: { workspacePackageSubpath: 12 },
          examples: [{ specifier: '@workspace/types/foo', consumerFile: 'apps/web/src/a.ts' }],
        },
        'tsconfig-path-target-missing': {
          count: 4,
          spaces: { type: 1, value: 3, unknown: 0 },
          hints: { 'generated-artifact-missing': 4 },
          examples: [{ specifier: '@/generated/client', consumerFile: 'apps/web/src/b.ts' }],
        },
      },
      unresolvedInternalSpecifierRecords: [
        { specifier: '@/legacy', reason: 'legacy-record-only' },
      ],
    },
  });
  const r = zones.find((z) => z.area === 'resolver');
  assert('B4a2. resolver zone exposes grouped unresolved reasons from symbols summary',
    r?.details?.topUnresolvedReasons?.[0]?.reason === 'workspace-package-subpath-target-missing' &&
      r.details.topUnresolvedReasons[0].count === 12 &&
      r.details.topUnresolvedReasons[0].spaces?.type === 12 &&
      r.details.topUnresolvedReasons[0].spaces?.value === 0 &&
      r.details.topUnresolvedReasons[1].reason === 'tsconfig-path-target-missing' &&
      r.details.topUnresolvedReasons[1].spaces?.type === 1 &&
      r.details.topUnresolvedReasons[1].spaces?.value === 3 &&
      !r.details.topUnresolvedReasons.some((item) => item.reason === 'legacy-record-only'),
    JSON.stringify(zones));
}

// B4b: Low ratio but one concentrated unresolved prefix still needs a resolver warning.
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 5000 } },
    symbols: {
      uses: { unresolvedInternalRatio: 0.05, unresolvedInternal: 220 },
      topUnresolvedSpecifiers: [
        { specifierPrefix: '@workspace/', count: 190 },
        { specifierPrefix: '#/', count: 12 },
      ],
    },
  });
  const r = zones.find((z) => z.area === 'resolver');
  assert('B4b. concentrated unresolved prefix triggers confidence-gap below ratio threshold',
    r && r.severity === 'confidence-gap' &&
      r.details.trigger === 'prefix-concentration' &&
      r.details.topUnresolvedSpecifiers.includes('@workspace/'),
    JSON.stringify(zones));
}

// B5: Parse errors in symbols.meta.warnings → precision-gap
{
  const zones = detectBlindZones({
    symbols: {
      meta: { warnings: [{ kind: 'parse-errors', count: 3, message: 'x' }] },
    },
  });
  const p = zones.find((z) => z.area === 'parser');
  assert('B5. parse-errors in symbols.meta.warnings trigger precision-gap',
    p && p.severity === 'precision-gap',
    JSON.stringify(zones));
}

// B5b: Opaque CJS export forms are a precision gap; exact CJS exports alone are not.
{
  const zones = detectBlindZones({
    symbols: {
      cjsExportSurfaceByFile: {
        'src/exact.cjs': {
          exact: [{ name: 'foo', kind: 'exports-member', line: 1 }],
          opaque: [],
        },
        'src/opaque.cjs': {
          exact: [],
          opaque: [{ kind: 'module-exports-assignment', line: 3 }],
        },
      },
    },
  });
  const cjs = zones.find((z) => z.area === 'commonjs-export-surface');
  assert('B5b. opaque CJS export surface triggers precision-gap blind zone',
    cjs && cjs.severity === 'precision-gap' &&
      cjs.details.files === 1 &&
      cjs.details.opaqueForms[0].file === 'src/opaque.cjs',
    JSON.stringify(zones));
}

// B5c: Dynamic CommonJS require calls are a precision gap.
{
  const zones = detectBlindZones({
    symbols: {
      cjsRequireOpacity: [
        { consumerFile: 'src/consumer.js', line: 2, kind: 'dynamic-require' },
      ],
    },
  });
  const cjs = zones.find((z) => z.area === 'commonjs-dynamic-require');
  assert('B5c. dynamic CJS require triggers precision-gap blind zone',
    cjs && cjs.severity === 'precision-gap' &&
      cjs.details.files === 1 &&
      cjs.details.calls === 1 &&
      cjs.details.examples[0].consumerFile === 'src/consumer.js',
    JSON.stringify(zones));
}

// B6: Clean repo → no zones at all
{
  const zones = detectBlindZones({
    triage: { byLanguage: { ts: 100, tsx: 50 } },
    symbols: { uses: { unresolvedInternalRatio: 0.02 }, meta: { warnings: [] } },
  });
  assert('B6. clean repo (only supported langs, low blindness, no parse errors) → zero zones',
    zones.length === 0,
    JSON.stringify(zones));
}

// B7: formatBlindZonesSummary shape
{
  const zones = [
    { area: 'rs', severity: 'scan-gap', effect: 'x' },
    { area: 'py', severity: 'precision-gap', effect: 'x' },
    { area: 'resolver', severity: 'confidence-gap', effect: 'x' },
  ];
  const s = formatBlindZonesSummary(zones);
  assert('B7. formatBlindZonesSummary counts each severity',
    s.includes('1 scan-gap') && s.includes('1 precision-gap') && s.includes('1 confidence-gap'),
    s);
  assert('B8. formatBlindZonesSummary returns null when no zones',
    formatBlindZonesSummary([]) === null,
    `got: ${formatBlindZonesSummary([])}`);
}

// B9: resolver reason details are visible in console-oriented blind-zone summary.
{
  const zones = [
    {
      area: 'resolver',
      severity: 'confidence-gap',
      effect: 'x',
      details: {
        topUnresolvedReasons: [
          { reason: 'workspace-package-subpath-target-missing', count: 12 },
          { reason: 'tsconfig-path-target-missing', count: 4 },
        ],
      },
    },
  ];
  const s = formatBlindZonesSummary(zones);
  assert('B9. formatBlindZonesSummary includes top resolver unresolved reasons',
    s.includes('resolver reasons: workspace-package-subpath-target-missing 12, tsconfig-path-target-missing 4'),
    s);
}

// B10: audit-summary.latest.md points readers at unresolved reason summaries.
{
  const summary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-05-05T00:00:00.000Z' },
      profile: 'quick',
      scanRange: { files: 5000, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
      blindZones: [
        {
          area: 'resolver',
          severity: 'confidence-gap',
          effect: 'x',
          details: {
            topUnresolvedReasons: [
              { reason: 'workspace-package-subpath-target-missing', count: 12 },
              { reason: 'tsconfig-path-target-missing', count: 4 },
            ],
          },
        },
      ],
    },
  });
  assert('B10. audit summary surfaces resolver blind-zone reason counts',
    summary.includes('Resolver blind-zone reasons: workspace-package-subpath-target-missing 12, tsconfig-path-target-missing 4') &&
      summary.includes('symbols.json.unresolvedInternalSummaryByReason'),
    summary);
}

// B10b: audit-summary.latest.md surfaces top unresolved specifier roots when available.
{
  const summary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-05-05T00:00:00.000Z' },
      profile: 'quick',
      scanRange: { files: 5000, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
      resolverDiagnostics: {
        topSpecifierRoots: [
          {
            specifierRoot: '@scope/orm',
            count: 37,
            reasons: {
              'workspace-generated-artifact-missing': 29,
              'workspace-package-subpath-target-missing': 8,
            },
          },
          {
            specifierRoot: 'app',
            count: 11,
            reasons: {
              'tsconfig-path-target-missing': 11,
            },
          },
        ],
      },
      blindZones: [
        {
          area: 'resolver',
          severity: 'confidence-gap',
          effect: 'x',
          details: {
            topUnresolvedReasons: [
              { reason: 'workspace-generated-artifact-missing', count: 29 },
            ],
          },
        },
      ],
    },
  });
  assert('B10b. audit summary surfaces top unresolved specifier roots',
    summary.includes('Top unresolved roots: @scope/orm 37 (workspace-generated-artifact-missing 29, workspace-package-subpath-target-missing 8); app 11 (tsconfig-path-target-missing 11)') &&
      summary.includes('manifest.json.resolverDiagnostics.topSpecifierRoots'),
    summary);
}

// B10c: audit-summary.latest.md surfaces generated consumer blind-zone scopes.
{
  const summary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-05-05T00:00:00.000Z' },
      profile: 'quick',
      scanRange: { files: 5000, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0.02 },
      blindZones: [],
      generatedArtifacts: {
        generatedConsumerBlindZoneCount: 3,
        topGeneratedConsumerBlindZones: [
          {
            scopePackageRoot: 'packages/prisma',
            count: 2,
            statuses: {
              missing: 1,
              'present-but-out-of-scope': 1,
            },
            topSpecifiers: [
              { specifier: '@scope/prisma/enums', count: 2 },
            ],
          },
          {
            scopePackageRoot: 'packages/kysely',
            count: 1,
            statuses: {
              missing: 1,
            },
            topSpecifiers: [
              { specifier: '@scope/kysely/types', count: 1 },
            ],
          },
        ],
      },
    },
  });
  assert('B10c. audit summary surfaces generated consumer blind-zone scopes',
    summary.includes('Generated consumer blind zones: 3') &&
      summary.includes('packages/prisma 2 (missing 1, present-but-out-of-scope 1; @scope/prisma/enums 2)') &&
      summary.includes('manifest.json.generatedArtifacts.topGeneratedConsumerBlindZones') &&
      summary.includes('symbols.json.generatedConsumerBlindZones'),
    summary);
}

// B10d: audit-summary.latest.md surfaces candidate-relevant resolver package scopes.
{
  const summary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-05-05T00:00:00.000Z' },
      profile: 'quick',
      scanRange: { files: 5000, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
      resolverDiagnostics: {
        topAffectedPackageScopes: [
          { affectedPackageScope: 'packages/lib', count: 12 },
          { affectedPackageScope: 'apps/web', count: 4 },
        ],
      },
      blindZones: [
        {
          area: 'resolver',
          severity: 'confidence-gap',
          effect: 'x',
          details: {
            topUnresolvedReasons: [
              { reason: 'workspace-package-subpath-target-missing', count: 12 },
            ],
          },
        },
      ],
    },
  });
  assert('B10d. audit summary surfaces resolver affected package scopes',
    summary.includes('Resolver affected scopes: packages/lib 12; apps/web 4') &&
      summary.includes('manifest.json.resolverDiagnostics.topAffectedPackageScopes'),
    summary);
}

// B10e: audit-summary.latest.md surfaces candidate-level blocked absence hints.
{
  const summary = renderAuditSummary({
    manifest: {
      meta: { generated: '2026-05-05T00:00:00.000Z' },
      profile: 'quick',
      scanRange: { files: 5000, languages: ['ts'], includeTests: true },
      confidence: { parseErrors: 0, unresolvedInternalRatio: 0.06 },
      resolverDiagnostics: {
        blockedCandidateHintCount: 2,
        blockedCandidateHintSampleLimit: 10,
        blockedCandidateHintReasonCounts: [
          {
            reason: 'generated-consumer-blind-zone',
            count: 7,
            families: { 'generated-artifacts': 7 },
          },
          {
            reason: 'hash-import-target-missing',
            count: 2,
            families: { 'node-imports': 2 },
          },
        ],
        blockedCandidateHintFamilyCounts: [
          {
            family: 'generated-artifacts',
            count: 7,
            reasons: { 'generated-consumer-blind-zone': 7 },
          },
          {
            family: 'node-imports',
            count: 2,
            reasons: { 'hash-import-target-missing': 2 },
          },
        ],
        blockedCandidateHints: [
          {
            specifier: '#app/config',
            candidatePath: 'packages/app/src/config',
            affectedPackageScope: 'packages/app',
            reason: 'hash-import-target-missing',
          },
          {
            specifier: '@scope/orm/client',
            candidatePath: 'packages/orm/client',
            affectedPackageScope: 'packages/orm',
            reason: 'generated-consumer-blind-zone',
          },
        ],
      },
      blindZones: [
        {
          area: 'resolver',
          severity: 'confidence-gap',
          effect: 'x',
          details: {
            topUnresolvedReasons: [
              { reason: 'hash-import-target-missing', count: 1 },
              { reason: 'generated-consumer-blind-zone', count: 1 },
            ],
          },
        },
      ],
    },
  });
  assert('B10e. audit summary surfaces blocked candidate hints',
    summary.includes('Resolver blocked absence hints: 2') &&
      summary.includes('manifest sample limit 10') &&
      summary.includes('Resolver blocked absence distribution: reasons generated-consumer-blind-zone 7 (generated-artifacts 7), hash-import-target-missing 2 (node-imports 2); families generated-artifacts 7 (generated-consumer-blind-zone 7), node-imports 2 (hash-import-target-missing 2)') &&
      summary.includes('packages/app/src/config via #app/config (hash-import-target-missing)') &&
      summary.includes('packages/orm/client via @scope/orm/client (generated-consumer-blind-zone)') &&
      summary.includes('manifest.json.resolverDiagnostics.blockedCandidateHintReasonCounts') &&
      summary.includes('manifest.json.resolverDiagnostics.blockedCandidateHintFamilyCounts') &&
      summary.includes('manifest.json.resolverDiagnostics.blockedCandidateHints') &&
      summary.includes('resolver-diagnostics.json.blockedCandidateHints'),
    summary);
}

// ───────────────────────────────────────────────────────────
// C. audit-repo.mjs orchestrator
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-output-notes-'));
  const OUT = mkdtempSync(path.join(tmpdir(), 'fx-audit-output-out-'));
  try {
    mkdirSync(path.join(FX, 'src'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'fx-output-notes', type: 'module' }));
    writeFileSync(path.join(FX, 'src/a.ts'), 'export const a = 1;\n');

    const defaultOut = spawnSync(NODE, [
      AUDIT_REPO,
      '--root', FX,
      '--profile', 'quick',
      '--production',
    ], { cwd: DIR, encoding: 'utf8' });
    assert('O0a. default .audit output emits privacy note',
      defaultOut.status === 0 &&
      existsSync(path.join(FX, '.audit', 'manifest.json')) &&
      /privacy note: default artifacts are written/.test(defaultOut.stderr) &&
      defaultOut.stderr.includes('.audit/'),
      `${defaultOut.stdout}\n${defaultOut.stderr}`);

    const outsideOut = spawnSync(NODE, [
      AUDIT_REPO,
      '--root', FX,
      '--output', OUT,
      '--profile', 'quick',
      '--production',
    ], { cwd: DIR, encoding: 'utf8' });
    assert('O0b. explicit output outside root emits location note, not privacy warning',
      outsideOut.status === 0 &&
      /note: --output is outside --root/.test(outsideOut.stderr) &&
      !/privacy note: default artifacts/.test(outsideOut.stderr),
      `${outsideOut.stdout}\n${outsideOut.stderr}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
    rmSync(OUT, { recursive: true, force: true });
  }
}

{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-'));
  const OUT = path.join(FX, 'audit-out');

  try {
    // Minimal valid TS project
    mkdirSync(path.join(FX, 'src'), { recursive: true });
    mkdirSync(path.join(FX, 'scripts'), { recursive: true });
    mkdirSync(path.join(FX, 'docs/current/audit'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'fx', type: 'module' }));
    writeFileSync(path.join(FX, 'docs/current/audit/lumin-structural-audit.md'),
      '# Living Structural Audit\n\n## Tracked Items\n\n');
    const bigBody = Array.from({ length: 155 }, (_, i) => `  const x${i} = ${i};`).join('\n');
    writeFileSync(path.join(FX, 'src/a.ts'),
      "import { b } from './b';\n" +
      "export function a() { return b(); }\n" +
      "export function parseMaybe(raw: string) {\n" +
      "  try { return JSON.parse(raw); } catch { return null; }\n" +
      "}\n" +
      `export function hugeProd() {\n${bigBody}\n  return 0;\n` +
      "}\n");
    writeFileSync(path.join(FX, 'src/b.ts'),
      "export function b() { return 1; }\nexport function unused() { return 2; }\n");
    writeFileSync(path.join(FX, 'scripts/huge-smoke.mjs'),
      `export function hugeScript() {\n${bigBody}\n  return 0;\n}\n`);

    const cmd = `node audit-repo.mjs --root ${FX} --output ${OUT} --profile quick --production`;
    const out = execSync(cmd, { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8' });

    const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));
    const producerPerformance = JSON.parse(readFileSync(path.join(OUT, 'producer-performance.json'), 'utf8'));

    // O1: manifest has profile + commandsRun + scanRange + confidence +
    // resolverDiagnostics + blindZones + generatedArtifacts.
    assert('O1. manifest.json has the required top-level evidence sections',
      manifest.profile === 'quick' &&
      Array.isArray(manifest.commandsRun) &&
      manifest.scanRange !== undefined &&
      manifest.confidence !== undefined &&
      manifest.resolverDiagnostics !== undefined &&
      manifest.resolverDiagnostics?.resolverCapabilityArtifact === 'resolver-capabilities.json' &&
      manifest.resolverDiagnostics?.resolverDiagnosticsArtifact === 'resolver-diagnostics.json' &&
      Array.isArray(manifest.resolverDiagnostics?.topSpecifierRoots) &&
      Array.isArray(manifest.resolverDiagnostics?.topUnresolvedReasons) &&
      Array.isArray(manifest.blindZones) &&
      manifest.generatedArtifacts?.mode === 'default' &&
      manifest.generatedArtifacts?.generatedArtifactPolicyVersion === 'generated-artifact-policy-v1' &&
      manifest.generatedArtifacts?.executedGenerators === false &&
      Array.isArray(manifest.generatedArtifacts?.topGeneratedMisses),
      `keys: ${Object.keys(manifest).join(', ')}`);
    assert('O1b. manifest links producer-performance artifact and summary',
      manifest.performance?.artifact === 'producer-performance.json' &&
      typeof manifest.performance?.totalWallMs === 'number' &&
      manifest.performance?.producerCount === manifest.commandsRun.length &&
      manifest.artifactsProduced.includes('producer-performance.json'),
      JSON.stringify(manifest.performance));
    assert('O1c. producer-performance records comparable run metadata and producer timings',
      producerPerformance.schemaVersion === 'producer-performance.v1' &&
      producerPerformance.root === FX &&
      producerPerformance.output === OUT &&
      producerPerformance.profile === 'quick' &&
      producerPerformance.scanRange?.includeTests === false &&
      producerPerformance.scanRange?.production === true &&
      Array.isArray(producerPerformance.producers) &&
      producerPerformance.producers.length === manifest.commandsRun.length &&
      producerPerformance.producers.every((entry) =>
        typeof entry.name === 'string' &&
        typeof entry.wallMs === 'number' &&
        typeof entry.status === 'string') &&
      producerPerformance.summary?.producerCount === producerPerformance.producers.length,
      JSON.stringify(producerPerformance, null, 2));
    assert('O1d. producer-performance records artifact sizes and largest artifacts',
      typeof producerPerformance.artifacts?.totalBytes === 'number' &&
      producerPerformance.artifacts.totalBytes > 0 &&
      typeof producerPerformance.artifacts?.producedCount === 'number' &&
      Array.isArray(producerPerformance.artifacts?.largest) &&
      producerPerformance.artifacts.largest.length > 0 &&
      producerPerformance.artifacts.byName?.['symbols.json']?.bytes > 0 &&
      manifest.performance?.totalArtifactBytes === producerPerformance.artifacts.totalBytes &&
      Array.isArray(manifest.performance?.largestArtifacts) &&
      manifest.performance.largestArtifacts.length > 0,
      JSON.stringify({
        manifestPerformance: manifest.performance,
        artifacts: producerPerformance.artifacts,
      }, null, 2));
    assert('O1d2. producer-performance records orchestrator artifact read and parse counters',
      producerPerformance.artifactReads?.schemaVersion === 'artifact-read-metrics.v1' &&
      producerPerformance.artifactReads?.totalReadCount > 0 &&
      producerPerformance.artifactReads?.totalReadBytes > 0 &&
      typeof producerPerformance.artifactReads?.totalReadMs === 'number' &&
      typeof producerPerformance.artifactReads?.totalJsonParseMs === 'number' &&
      producerPerformance.artifactReads?.byName?.['symbols.json']?.readCount > 0 &&
      Array.isArray(producerPerformance.artifactReads?.largestReads) &&
      producerPerformance.artifactReads.largestReads.length > 0 &&
      manifest.performance?.artifactReadCount === producerPerformance.artifactReads.totalReadCount &&
      manifest.performance?.totalArtifactReadBytes === producerPerformance.artifactReads.totalReadBytes &&
      manifest.performance?.totalJsonParseMs === producerPerformance.artifactReads.totalJsonParseMs,
      JSON.stringify({
        manifestPerformance: manifest.performance,
        artifactReads: producerPerformance.artifactReads,
      }, null, 2));
    assert('O1e. producer-performance records honest orchestrator memory snapshots',
      producerPerformance.memory?.measurement === 'orchestrator-process-snapshots' &&
      producerPerformance.memory?.childPeakRssAvailable === false &&
      producerPerformance.producers.every((entry) =>
        typeof entry.memory?.before?.rssBytes === 'number' &&
        typeof entry.memory?.after?.rssBytes === 'number' &&
        typeof entry.memory?.delta?.rssBytes === 'number') &&
      typeof producerPerformance.summary?.maxObservedOrchestratorRssBytes === 'number',
      JSON.stringify(producerPerformance, null, 2));
    const symbolProducer = producerPerformance.producers.find((entry) =>
      entry.name === 'build-symbol-graph.mjs');
    const topologyProducer = producerPerformance.producers.find((entry) =>
      entry.name === 'measure-topology.mjs');
    assert('O1f. producer-performance records producer phase counters for heavy quick producers',
      producerPerformance.summary?.phaseSupportCount >= 2 &&
      Array.isArray(symbolProducer?.phases) &&
      symbolProducer.phases.some((phase) =>
        phase.name === 'snapshot' && typeof phase.wallMs === 'number') &&
      symbolProducer.phases.some((phase) =>
        phase.name === 'extract-changed-files' && typeof phase.wallMs === 'number') &&
      Array.isArray(topologyProducer?.phases) &&
      topologyProducer.phases.some((phase) =>
        phase.name === 'process-changed-files' && typeof phase.wallMs === 'number') &&
      topologyProducer.counters?.jsFilesProcessed > 0 &&
      topologyProducer.counters?.scannerFilesAttempted > 0 &&
      topologyProducer.counters?.scannerAcceptedFiles > 0 &&
      typeof topologyProducer.counters?.scannerFallbackFiles === 'number' &&
      typeof topologyProducer.counters?.oxcParseCalls === 'number' &&
      typeof topologyProducer.counters?.resolverMemoHits === 'number' &&
      typeof topologyProducer.counters?.resolverMemoMisses === 'number' &&
      manifest.performance?.phaseSupportCount === producerPerformance.summary.phaseSupportCount,
      JSON.stringify({
        manifestPerformance: manifest.performance,
        symbolProducer,
        topologyProducer,
      }, null, 2));
    const symbolCounters = symbolProducer?.counters ?? {};
    assert('O1f2. build-symbol-graph records extraction and graph counters',
      symbolCounters.snapshotFiles > 0 &&
      symbolCounters.changedFiles > 0 &&
      typeof symbolCounters.reusedFiles === 'number' &&
      symbolCounters.changedJsFiles > 0 &&
      symbolCounters.extractedFiles > 0 &&
      symbolCounters.fileDataFiles > 0 &&
      symbolCounters.definitionCount > 0 &&
      symbolCounters.useCount > 0 &&
      typeof symbolCounters.reExportCount === 'number' &&
      typeof symbolCounters.parseErrorCount === 'number' &&
      typeof symbolCounters.totalUses === 'number' &&
      typeof symbolCounters.resolvedInternalUses === 'number' &&
      typeof symbolCounters.unresolvedInternalUses === 'number' &&
      symbolCounters.symbolsJsonBytes > 0,
      JSON.stringify({ symbolProducer }, null, 2));
    const symbolPhaseNames = new Set((symbolProducer?.phases ?? []).map((phase) => phase.name));
    const expectedSymbolAssemblyPhases = [
      'assemble-file-data',
      'assemble-def-index',
      'assemble-namespace-reexports',
      'assemble-source-uses',
      'assemble-mdx-uses',
      'assemble-generated-blind-zones',
      'assemble-dead-candidates',
      'assemble-fan-in',
      'assemble-any-contamination',
    ];
    assert('O1f3. build-symbol-graph records assembly subphase timings',
      expectedSymbolAssemblyPhases.every((name) => symbolPhaseNames.has(name)),
      JSON.stringify({
        expectedSymbolAssemblyPhases,
        phases: symbolProducer?.phases,
      }, null, 2));
    const expectedSourceUsePhases = [
      'assemble-source-use-resolve',
      'assemble-source-use-external',
      'assemble-source-use-asset',
      'assemble-source-use-unresolved',
      'assemble-source-use-generated-virtual',
      'assemble-source-use-namespace-reexport',
      'assemble-source-use-resolved-internal',
    ];
    assert('O1f4. build-symbol-graph records source-use operation timings',
      expectedSourceUsePhases.every((name) => symbolPhaseNames.has(name)) &&
      typeof symbolCounters.sourceUseResolveMs === 'number' &&
      typeof symbolCounters.sourceUseResolverMemoHits === 'number' &&
      typeof symbolCounters.sourceUseResolverMemoMisses === 'number' &&
      typeof symbolCounters.sourceUseResolverStageRelativeAttempts === 'number' &&
      typeof symbolCounters.sourceUseResolverStageRelativeMs === 'number' &&
      typeof symbolCounters.sourceUseResolverStageRelativeCacheMisses === 'number' &&
      typeof symbolCounters.sourceUseResolverStageScopedTsconfigProbeHits === 'number' &&
      typeof symbolCounters.sourceUseResolverStageScopedTsconfigProbeMisses === 'number' &&
      typeof symbolCounters.sourceUseResolverStageExternalResults === 'number' &&
      typeof symbolCounters.sourceUseResolvedInternalBranchCount === 'number' &&
      typeof symbolCounters.sourceUseExternalBranchCount === 'number',
      JSON.stringify({
        expectedSourceUsePhases,
        phases: symbolProducer?.phases,
        counters: symbolCounters,
      }, null, 2));

    // O2: quick profile ran at least triage + symbols + classify + rank
    const steps = manifest.commandsRun.map((c) => c.step);
    assert('O2. quick profile ran triage + symbols + classify + rank',
      steps.includes('triage-repo.mjs') &&
      steps.includes('build-symbol-graph.mjs') &&
      steps.includes('build-resolver-diagnostics.mjs') &&
      steps.includes('classify-dead-exports.mjs') &&
      steps.includes('rank-fixes.mjs'),
      `ran: ${steps.join(', ')}`);

    // O3: quick profile did NOT run staleness or runtime
    assert('O3. quick profile did NOT run staleness or runtime-evidence',
      !steps.includes('measure-staleness.mjs') &&
      !steps.includes('merge-runtime-evidence.mjs') &&
      !steps.includes('build-call-graph.mjs') &&
      !steps.includes('check-barrel-discipline.mjs') &&
      !steps.includes('build-shape-index.mjs') &&
      !steps.includes('build-function-clone-index.mjs'),
      `ran: ${steps.join(', ')}`);

    // O4: artifactsProduced lists what was actually generated
    assert('O4. artifactsProduced enumerates files actually on disk',
      Array.isArray(manifest.artifactsProduced) &&
      manifest.artifactsProduced.includes('symbols.json') &&
      manifest.artifactsProduced.includes('resolver-capabilities.json') &&
      manifest.artifactsProduced.includes('resolver-diagnostics.json') &&
      manifest.artifactsProduced.includes('dead-classify.json') &&
      manifest.artifactsProduced.includes('checklist-facts.json') &&
      manifest.artifactsProduced.includes('audit-summary.latest.md'),
      JSON.stringify(manifest.artifactsProduced));
    assert('O4b. audit-summary.latest.md is written as an artifact map',
      typeof manifest.auditSummary?.path === 'string' &&
      manifest.auditSummary.path.endsWith('audit-summary.latest.md'),
      JSON.stringify(manifest.auditSummary));
    const summaryMd = readFileSync(path.join(OUT, 'audit-summary.latest.md'), 'utf8');
    assert('O4c. audit summary is an artifact brief, not a chat recommendation',
      summaryMd.includes('# Audit Artifact Brief') &&
      summaryMd.includes('not a recommendation engine') &&
      summaryMd.includes('## Read First') &&
      summaryMd.includes('## Measured Cues (Unranked)') &&
      summaryMd.includes('## Artifact Map') &&
      summaryMd.includes('## Guardrails') &&
      !summaryMd.includes('Ask the coding agent:'),
      summaryMd.slice(0, 500));
    assert('O4d. audit summary maps non-empty anonymous catch evidence without ranking it',
      summaryMd.includes('Catch handling:') &&
      summaryMd.includes('non-empty anonymous'),
      summaryMd);
    assert('O4e. audit summary tells the model to curate from raw artifacts',
      summaryMd.includes('Do not inherit ordering') &&
      summaryMd.includes('Re-rank by the user request'),
      summaryMd);
    assert('O4f. manifest records existing living audit docs for controller update',
      manifest.livingAudit?.existingDocs?.length === 1 &&
      manifest.livingAudit.existingDocs[0].path === 'docs/current/audit/lumin-structural-audit.md' &&
      manifest.livingAudit?.action === 'read-and-update-before-final-answer',
      JSON.stringify(manifest.livingAudit));
    assert('O4g. audit summary and console preview surface living audit tracking',
      summaryMd.includes('## Living Audit Tracking') &&
      summaryMd.includes('docs/current/audit/lumin-structural-audit.md') &&
      summaryMd.includes('Do not ask a subagent to own this document') &&
      out.includes('Living Audit Tracking'),
      `${summaryMd}\n--- stdout ---\n${out.slice(-1200)}`);
    const topologyMermaidMd = readFileSync(path.join(OUT, 'topology.mermaid.md'), 'utf8');
    assert('O4h. quick audit writes topology Mermaid as a human visual companion',
      manifest.artifactsProduced.includes('topology.mermaid.md') &&
      manifest.topologyMermaid?.path?.endsWith('topology.mermaid.md') &&
      summaryMd.includes('`topology.mermaid.md`') &&
      topologyMermaidMd.includes('# Topology Mermaid') &&
      topologyMermaidMd.includes('## How To Read This') &&
      topologyMermaidMd.includes('## Omitted Detail / Limits') &&
      topologyMermaidMd.includes('## Citation Contract') &&
      topologyMermaidMd.includes('not citation authority') &&
      (
        topologyMermaidMd.includes('```mermaid') ||
        (
          topologyMermaidMd.includes('No cross-submodule edges were observed') &&
          topologyMermaidMd.includes('No runtime cycles were observed')
        )
      ),
      JSON.stringify({
        topologyMermaid: manifest.topologyMermaid,
        artifacts: manifest.artifactsProduced,
        topologyMermaidMd: topologyMermaidMd.slice(0, 500),
      }));

    // O5: clean TS fixture produces no blind zones
    assert('O5. clean TS-only fixture produces zero blind zones',
      manifest.blindZones.length === 0,
      JSON.stringify(manifest.blindZones));

    // O6: confidence section has the FP-36 fields
    assert('O6. confidence.unresolvedInternalRatio and external fields populated',
      typeof manifest.confidence.unresolvedInternalRatio === 'number' &&
      typeof manifest.confidence.externalImports === 'number',
      JSON.stringify(manifest.confidence));
    const symbols = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    assert('O6b. symbols.json declares per-language extractor availability',
      symbols.meta?.languageSupport?.ts?.enabled === true &&
      symbols.meta?.languageSupport?.js?.enabled === true &&
      typeof symbols.meta?.languageSupport?.python?.enabled === 'boolean' &&
      typeof symbols.meta?.languageSupport?.go?.enabled === 'boolean',
      JSON.stringify(symbols.meta?.languageSupport));

    // O7: console output contains the "Next:" guidance line
    assert('O7. console output directs user to review blindZones before claims',
      out.includes('review manifest.blindZones'),
      `last 300 chars: ${out.slice(-300)}`);
    assert('O7b. console output reports produced artifacts without stale denominator',
      out.includes('artifacts: ') &&
      out.includes(' produced') &&
      !out.includes('/ 9') &&
      !out.includes('/ 12'),
      `last 300 chars: ${out.slice(-300)}`);
    assert('O7c. console output points to the human-readable summary',
      out.includes('audit-summary.latest.md'),
      `last 300 chars: ${out.slice(-300)}`);
    assert('O7d. console output includes an artifact-brief preview, not ranked recommendations',
      out.includes('[audit-repo] artifact brief preview:') &&
      out.includes('Read First') &&
      out.includes('Measured Cues') &&
      !out.includes('Worth Smoothing Next'),
      `last 600 chars: ${out.slice(-600)}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// D. orchestrator graceful skip with Python (triggers blind zone)
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-py-'));
  const OUT = path.join(FX, 'audit-out');
  try {
    mkdirSync(path.join(FX, 'src'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'fx', type: 'module' }));
    // A tiny TS consumer and a python file
    writeFileSync(path.join(FX, 'src/a.ts'),
      "export const x = 1;\n");
    writeFileSync(path.join(FX, 'src/helper.py'),
      "def hello():\n    return 'hi'\n");

    execSync(`node audit-repo.mjs --root ${FX} --output ${OUT} --profile quick --production`,
      { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });
    const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));

    const py = manifest.blindZones.find((z) =>
      z.area === 'python-method-resolution' || z.area === 'python-scan-gap');
    assert('O8. repo with Python files surfaces Python precision or scan gap',
      !!py && (py.severity === 'precision-gap' || py.severity === 'scan-gap'),
      `blindZones: ${JSON.stringify(manifest.blindZones)}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// E. orchestrator forwards --exclude to core producer steps
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-exclude-'));
  const OUT = path.join(FX, 'audit-out');
  try {
    mkdirSync(path.join(FX, 'src'), { recursive: true });
    mkdirSync(path.join(FX, 'apps/web/src'), { recursive: true });
    mkdirSync(path.join(FX, 'packages/prisma'), { recursive: true });
    mkdirSync(path.join(FX, 'output', 'corpus'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'fx', type: 'module', workspaces: ['apps/*', 'packages/*'] }));
    writeFileSync(path.join(FX, 'apps/web/package.json'),
      JSON.stringify({ name: 'web', type: 'module' }));
    writeFileSync(path.join(FX, 'packages/prisma/package.json'),
      JSON.stringify({
        name: '@scope/prisma',
        type: 'module',
        main: 'index.ts',
        bin: { 'prisma-enum-generator': './run-enum-generator.js' },
        scripts: { generate: 'prisma generate' },
        dependencies: { '@prisma/client': '1.0.0' },
      }));
    writeFileSync(path.join(FX, 'src/a.ts'),
      'export const live = 1;\n');
    writeFileSync(path.join(FX, 'packages/prisma/index.ts'),
      'export const prismaRoot = 1;\n');
    writeFileSync(path.join(FX, 'apps/web/src/consumer.ts'),
      "import { BookingStatus } from '@scope/prisma/enums';\n" +
      'export const status = BookingStatus.ACCEPTED;\n');
    writeFileSync(path.join(FX, 'output/corpus/leak.ts'),
      'export const leaked = 1;\n');

    execFileSync(process.execPath, [
      path.join(DIR, 'audit-repo.mjs'),
      '--root', FX,
      '--output', OUT,
      '--profile', 'quick',
      '--exclude', 'output',
      '--generated-artifacts', 'prepared',
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));
    const symbols = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const defFiles = Object.keys(symbols.defIndex ?? {});

    assert('O9a. audit-repo forwards --exclude to build-symbol-graph',
      !defFiles.some((f) => f.startsWith('output/')),
      `leaked defs: ${JSON.stringify(defFiles.filter((f) => f.startsWith('output/')))}`);
    assert('O9b. manifest.scanRange records excluded output range',
      manifest.scanRange?.excludes?.includes('output'),
      JSON.stringify(manifest.scanRange));
    assert('O9c. audit-repo forwards generated artifact mode into manifest evidence',
      manifest.generatedArtifacts?.mode === 'prepared' &&
        manifest.generatedArtifacts?.executedGenerators === false,
      JSON.stringify(manifest.generatedArtifacts));
    assert('O9d. audit-repo forwards generated artifact mode into build-symbol-graph',
      symbols.generatedConsumerBlindZones?.some((zone) =>
        zone.specifier === '@scope/prisma/enums' &&
        zone.mode === 'prepared' &&
        zone.staleStatus === 'unknown' &&
        zone.staleReason === 'generator-input-hash-not-recorded'),
      JSON.stringify(symbols.generatedConsumerBlindZones, null, 2));
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// F. maintainer self-audit auto-excludes lab/corpus/generated mirrors
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-self-scope-'));
  const OUT = path.join(FX, 'audit-out');
  try {
    mkdirSync(path.join(FX, '_lib'), { recursive: true });
    mkdirSync(path.join(FX, 'src'), { recursive: true });
    mkdirSync(path.join(FX, 'p6-corpus'), { recursive: true });
    mkdirSync(path.join(FX, 'output', 'corpus'), { recursive: true });
    mkdirSync(path.join(FX, 'skills', 'lumin-repo-lens-lab', '_engine'), { recursive: true });
    mkdirSync(path.join(FX, 'skills', 'lumin-repo-lens-lab', 'scripts'), { recursive: true });
    mkdirSync(path.join(FX, 'test-harness', 'lib'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'lumin-repo-lens-lab-scripts', type: 'module' }));
    writeFileSync(path.join(FX, 'audit-repo.mjs'), 'export const rootEntrypoint = 1;\n');
    writeFileSync(path.join(FX, 'src/live.ts'), 'export const live = 1;\n');
    writeFileSync(path.join(FX, 'p6-corpus/leak.ts'), 'export const p6Leak = 1;\n');
    writeFileSync(path.join(FX, 'output/corpus/leak.ts'), 'export const outputLeak = 1;\n');
    writeFileSync(path.join(FX, 'skills/lumin-repo-lens-lab/_engine/leak.mjs'), 'export const engineLeak = 1;\n');
    writeFileSync(path.join(FX, 'skills/lumin-repo-lens-lab/scripts/leak.mjs'), 'export const scriptLeak = 1;\n');
    writeFileSync(path.join(FX, 'test-harness/lib/leak.mjs'), 'export const harnessLeak = 1;\n');

    execFileSync(process.execPath, [
      path.join(DIR, 'audit-repo.mjs'),
      '--root', FX,
      '--output', OUT,
      '--profile', 'quick',
      '--production',
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));
    const symbols = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
    const defFiles = Object.keys(symbols.defIndex ?? {});

    assert('O13a. self-audit scanRange records automatic excludes',
      manifest.scanRange?.autoExcludes?.includes('p6-corpus') &&
      manifest.scanRange?.autoExcludes?.includes('output/corpus') &&
      manifest.scanRange?.autoExcludes?.includes('skills/lumin-repo-lens-lab/_engine') &&
      manifest.scanRange?.autoExcludes?.includes('skills/lumin-repo-lens-lab/scripts') &&
      manifest.scanRange?.autoExcludes?.includes('test-harness'),
      JSON.stringify(manifest.scanRange));
    assert('O13b. self-audit excludes lab/corpus/generated definitions',
      defFiles.includes('src/live.ts') &&
      !defFiles.some((f) =>
        f.startsWith('p6-corpus/') ||
        f.startsWith('output/corpus/') ||
        f.startsWith('skills/lumin-repo-lens-lab/_engine/') ||
        f.startsWith('skills/lumin-repo-lens-lab/scripts/') ||
        f.startsWith('test-harness/')),
      JSON.stringify(defFiles));
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// F2. scan-scope negation aliases all exclude tests through the orchestrator
// ───────────────────────────────────────────────────────────
{
  const FLAGS = [
    ['--production'],
    ['--no-tests'],
    ['--exclude-tests'],
    ['--include-tests=false'],
  ];

  for (const flagArgs of FLAGS) {
    const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-scope-'));
    const OUT = path.join(FX, 'audit-out');
    try {
      mkdirSync(path.join(FX, 'src'), { recursive: true });
      writeFileSync(path.join(FX, 'package.json'),
        JSON.stringify({ name: 'fx-scope', type: 'module' }));
      writeFileSync(path.join(FX, 'src/a.ts'),
        'export const prodOnly = 1;\n');
      writeFileSync(path.join(FX, 'src/a.test.ts'),
        'export const testOnly = 1;\n');

      execFileSync(process.execPath, [
        path.join(DIR, 'audit-repo.mjs'),
        '--root', FX,
        '--output', OUT,
        '--profile', 'quick',
        ...flagArgs,
      ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

      const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));
      const triage = JSON.parse(readFileSync(path.join(OUT, 'triage.json'), 'utf8'));
      const symbols = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
      const defFiles = Object.keys(symbols.defIndex ?? {});
      const label = flagArgs.join(' ');

      assert(`O11a. ${label} records production scanRange`,
        manifest.scanRange?.includeTests === false &&
        manifest.scanRange?.production === true &&
        manifest.scanRange?.languages?.includes('ts'),
        JSON.stringify(manifest.scanRange));
      assert(`O11b. ${label} triage excludes test files`,
        triage.shape?.testFiles === 0 &&
        triage.shape?.totalFiles === 1,
        JSON.stringify(triage.shape));
      assert(`O11c. ${label} symbol graph excludes test files`,
        !defFiles.some((f) => f.includes('.test.')),
        JSON.stringify(defFiles));
    } finally {
      rmSync(FX, { recursive: true, force: true });
    }
  }
}

// ───────────────────────────────────────────────────────────
// G. lifecycle artifacts are collected after opt-in modes run
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-lifecycle-artifacts-'));
  const OUT = path.join(FX, 'audit-out');
  const intent = path.join(FX, 'intent.json');
  try {
    mkdirSync(path.join(FX, 'src'), { recursive: true });
    writeFileSync(path.join(FX, 'package.json'),
      JSON.stringify({ name: 'fx-lifecycle', type: 'module' }));
    writeFileSync(path.join(FX, 'src/a.ts'),
      'export const live = 1;\n');
    writeFileSync(intent, JSON.stringify({
      names: ['newHelper'],
      shapes: [],
      files: ['src/new-helper.ts'],
      dependencies: [],
      plannedTypeEscapes: [],
    }));

    execFileSync(process.execPath, [
      path.join(DIR, 'audit-repo.mjs'),
      '--root', FX,
      '--output', OUT,
      '--profile', 'quick',
      '--pre-write',
      '--intent', intent,
      '--check-canon',
      '--sources', 'all',
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));
    const artifacts = manifest.artifactsProduced ?? [];

    assert('O12a. pre-write advisory is listed after lifecycle mode runs',
      artifacts.includes('pre-write-advisory.latest.json'),
      JSON.stringify(artifacts));
    assert('O12b. timestamped any-inventory pre snapshot is listed',
      artifacts.some((name) => /^any-inventory\.pre\..+\.json$/.test(name)),
      JSON.stringify(artifacts));
    assert('O12c. check-canon artifact is listed after lifecycle mode runs',
      artifacts.includes('canon-drift.json'),
      JSON.stringify(artifacts));
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// H. full profile staleness works when --root is a git subdirectory
// ───────────────────────────────────────────────────────────
{
  const FX = mkdtempSync(path.join(tmpdir(), 'fx-audit-repo-git-subdir-'));
  const ROOT = path.join(FX, 'workspace', 'tool');
  const OUT = path.join(FX, 'audit-out');
  try {
    mkdirSync(path.join(ROOT, 'src'), { recursive: true });
    writeFileSync(path.join(ROOT, 'package.json'),
      JSON.stringify({ name: 'fx-subdir', type: 'module' }));
    writeFileSync(path.join(ROOT, 'src/a.ts'),
      'export const live = 1;\nexport const unused = 2;\n' +
      'export function dirtyHelper(a: any, b: any, c: any) { return a; }\n' +
      'export interface DirtyShape { a: any; b: any; c: any }\n');
    writeFileSync(path.join(ROOT, 'src/web.ts'),
      "export interface WebState { id: string; status: 'idle' | 'running' }\n");
    writeFileSync(path.join(ROOT, 'src/daemon.ts'),
      "export type DaemonState = { status: 'idle' | 'running'; id: string };\n");

    execFileSync('git', ['init'], { cwd: FX, stdio: ['ignore', 'pipe', 'pipe'] });
    execFileSync('git', ['config', 'user.email', 'test@example.com'], { cwd: FX });
    execFileSync('git', ['config', 'user.name', 'Test User'], { cwd: FX });
    execFileSync('git', ['add', '.'], { cwd: FX });
    execFileSync('git', ['commit', '-m', 'fixture'], { cwd: FX, stdio: ['ignore', 'pipe', 'pipe'] });

    execFileSync(process.execPath, [
      path.join(DIR, 'audit-repo.mjs'),
      '--root', ROOT,
      '--output', OUT,
      '--profile', 'full',
      '--production',
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const manifest = JSON.parse(readFileSync(path.join(OUT, 'manifest.json'), 'utf8'));
    const checklist = JSON.parse(readFileSync(path.join(OUT, 'checklist-facts.json'), 'utf8'));
    const summaryMd = readFileSync(path.join(OUT, 'audit-summary.latest.md'), 'utf8');
    const reviewPackMd = readFileSync(path.join(OUT, 'audit-review-pack.latest.md'), 'utf8');
    const steps = manifest.commandsRun.map((c) => c.step);

    assert('O10a. full profile runs staleness when root is inside a larger git worktree',
      steps.includes('measure-staleness.mjs'),
      `ran: ${steps.join(', ')}; skipped: ${JSON.stringify(manifest.skipped)}`);
    assert('O10b. subdirectory-root staleness artifact is produced',
      manifest.artifactsProduced.includes('staleness.json'),
      JSON.stringify(manifest.artifactsProduced));
    assert('O10c. full profile emits optional checklist support artifacts',
      steps.includes('build-call-graph.mjs') &&
      steps.includes('check-barrel-discipline.mjs') &&
      steps.includes('build-shape-index.mjs') &&
      steps.includes('build-function-clone-index.mjs') &&
      manifest.artifactsProduced.includes('call-graph.json') &&
      manifest.artifactsProduced.includes('barrels.json') &&
      manifest.artifactsProduced.includes('shape-index.json') &&
      manifest.artifactsProduced.includes('function-clones.json'),
      `ran: ${steps.join(', ')}; artifacts: ${JSON.stringify(manifest.artifactsProduced)}`);
    assert('O10c2. full profile writes a reviewer-lane pack for Claude Code full review',
      manifest.artifactsProduced.includes('audit-review-pack.latest.md') &&
      manifest.reviewPack?.path?.endsWith('audit-review-pack.latest.md') &&
      reviewPackMd.includes('Audit Review Pack') &&
      reviewPackMd.includes('never calls external APIs') &&
      reviewPackMd.includes('Claude Code') &&
      reviewPackMd.includes('main-controller artifact brief') &&
      reviewPackMd.includes('codebase-reading assignment') &&
      reviewPackMd.includes('Subagent rule:') &&
      !reviewPackMd.includes('paste one whole lane into each chosen reviewer'),
      reviewPackMd);
    assert('O10d. full profile artifact brief maps exact shape-drift cue without ranking it',
      checklist.B1B2_shape_drift?.exactDuplicateGroups === 1 &&
      summaryMd.includes('Shape drift: exact groups 1') &&
      summaryMd.includes('shape-index.json'),
      summaryMd);
    assert('O10d2. full profile artifact brief maps function clone cues without ranking them',
      typeof checklist.B1_duplicate_implementation?.structureGroupCandidates === 'number' &&
      typeof checklist.B1_duplicate_implementation?.nearFunctionCandidates === 'number' &&
      summaryMd.includes('JS/TS function clone cues:') &&
      summaryMd.includes('near-function cues') &&
      summaryMd.includes('function-clones.json') &&
      reviewPackMd.includes('JS/TS function clone cues:') &&
      reviewPackMd.includes('near-function cues'),
      `${summaryMd}\n---\n${reviewPackMd}`);
    assert('O10e. full profile summary and review pack expose symbols anyContamination lane',
      summaryMd.includes('Exported any-contamination:') &&
      summaryMd.includes('symbols.json.typeOwnersByIdentity') &&
      reviewPackMd.includes('Identity-level anyContamination:') &&
      reviewPackMd.includes('Inspect symbols.json owner maps'),
      `${summaryMd}\n---\n${reviewPackMd}`);
  } finally {
    rmSync(FX, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
