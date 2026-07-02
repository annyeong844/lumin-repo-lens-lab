import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

const auditManifest = await import('../_lib/audit-manifest.mjs');

let passed = 0;
let failed = 0;

function check(label, fn) {
  try {
    fn();
    passed++;
    console.log(`  PASS  ${label}`);
  } catch (error) {
    failed++;
    console.log(`  FAIL  ${label}\n        ${error?.message ?? error}`);
  }
}

check('AMES1. audit-manifest exposes manifest builders, not living-audit internals', () => {
  assert.equal(typeof auditManifest.buildManifestEvidence, 'function');
  assert.equal(typeof auditManifest.refreshManifestEvidence, 'function');
  assert.equal(typeof auditManifest.collectProducedArtifacts, 'function');
  assert.equal(typeof auditManifest.buildManifestArtifactsProducedUpdate, 'function');
  assert.equal(typeof auditManifest.buildManifestCloseoutUpdate, 'function');
  assert.equal(typeof auditManifest.buildManifestLifecycleUpdate, 'function');
  assert.equal(typeof auditManifest.executeBaseRuntime, 'function');
  assert.equal(typeof auditManifest.buildProducerPerformanceArtifactForAuditRun, 'function');

  for (const symbol of [
    'LIVING_AUDIT_DOC_CANDIDATES',
    'detectLivingAuditDocs',
    'mergeRustAnalysisRun',
    'buildArtifactSizeSummary',
    'buildArtifactReadMetricsSummary',
    'buildProducerPerformanceArtifactFromRuntime',
    'buildManifestMeta',
    'buildManifestEvidenceUpdate',
    'buildManifestFinalSummaryUpdate',
    'buildManifestCompanionUpdate',
    'ARTIFACT_READ_EVENTS_SCHEMA_VERSION',
    'buildLifecycleSummary',
  ]) {
    assert.equal(Object.hasOwn(auditManifest, symbol), false, symbol);
  }
});

check('AMES1i. audit-core wrapper ignores stale external binaries and falls back to current contract', () => {
  const previous = process.env.LUMIN_AUDIT_CORE_BIN;
  process.env.LUMIN_AUDIT_CORE_BIN = process.execPath;
  try {
    const update = auditManifest.buildManifestLifecycleUpdate({
      preWrite: {
        requested: true,
        ran: false,
        reason: 'stale-env-fallback-smoke',
      },
    });
    assert.equal(update.lifecycle.summaryOwner, 'lumin-audit-core');
    assert.equal(update.lifecycle.requestedCount, 1);
    assert.equal(update.lifecycle.notRunCount, 1);
  } finally {
    if (previous === undefined) delete process.env.LUMIN_AUDIT_CORE_BIN;
    else process.env.LUMIN_AUDIT_CORE_BIN = previous;
  }
});

check('AMES1g. lifecycle update wrapper leaves raw block placement and summary in audit-core', () => {
  const update = auditManifest.buildManifestLifecycleUpdate({
    preWrite: {
      requested: true,
      ran: true,
      engine: 'rust',
      language: 'rust',
      producer: 'lumin-rust-analyzer',
    },
    postWrite: null,
    canonDraft: {
      requested: true,
      ran: false,
      reason: 'unknown source',
    },
    checkCanon: null,
  });

  assert.equal(update.preWrite.requested, true);
  assert.equal(update.preWrite.ran, true);
  assert.equal(update.preWrite.engine, 'rust');
  assert.equal(Object.hasOwn(update, 'postWrite'), false);
  assert.equal(update.canonDraft.requested, true);
  assert.equal(update.canonDraft.ran, false);
  assert.equal(update.canonDraft.reason, 'unknown source');
  assert.equal(Object.hasOwn(update, 'checkCanon'), false);
  assert.equal(update.lifecycle.summaryOwner, 'lumin-audit-core');
  assert.equal(update.lifecycle.executionOwner, 'audit-repo.mjs');
  assert.equal(update.lifecycle.requestedCount, 2);
  assert.equal(update.lifecycle.ranCount, 1);
  assert.equal(update.lifecycle.notRunCount, 1);
  assert.equal(update.lifecycle.preWrite.status, 'complete');
  assert.equal(update.lifecycle.canonDraft.status, 'not-run');
});

check('AMES1h. artifacts-produced wrapper leaves manifest patch shape in audit-core', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-artifacts-produced-'));
  const out = path.join(fx, 'out');
  try {
    mkdirSync(out, { recursive: true });
    writeFileSync(path.join(out, 'triage.json'), '{}');
    writeFileSync(
      path.join(out, 'rust-analyzer-health.latest.json'),
      JSON.stringify({
        schemaVersion: 'lumin-rust-analyzer-health.v1',
      }),
    );

    assert.deepEqual(
      auditManifest.buildManifestArtifactsProducedUpdate(out, {
        rustAnalysis: {
          status: 'unavailable',
          available: false,
        },
      }),
      {
        artifactsProduced: ['triage.json'],
      },
    );
    assert.deepEqual(
      auditManifest.buildManifestArtifactsProducedUpdate(out, {
        rustAnalysis: {
          status: 'complete',
          available: true,
        },
      }).artifactsProduced,
      ['rust-analyzer-health.latest.json', 'triage.json'],
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES1j. closeout wrapper leaves final summary and companion patch in audit-core', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-closeout-'));
  const out = path.join(fx, 'out');
  try {
    mkdirSync(out, { recursive: true });
    const performancePath = path.join(out, 'producer-performance.json');
    writeFileSync(
      performancePath,
      JSON.stringify({
        schemaVersion: 'producer-performance.v1',
        summary: {
          producerCount: 1,
          okCount: 1,
          failedCount: 0,
          skippedCount: 0,
        },
        producers: [{ name: 'triage-repo.mjs', status: 'ok' }],
        skipped: [],
      }),
    );
    writeFileSync(path.join(out, 'audit-summary.latest.md'), '# Summary\n');
    writeFileSync(path.join(out, 'audit-review-pack.latest.md'), '# Review\n');

    const update = auditManifest.buildManifestCloseoutUpdate({
      outDir: out,
      producerPerformancePath: performancePath,
      rustAnalysis: {
        status: 'unavailable',
        available: false,
      },
      auditSummaryPath: 'C:/repo/.audit/audit-summary.latest.md',
      reviewPackPath: 'C:/repo/.audit/audit-review-pack.latest.md',
    });

    assert.equal(update.performance.producerCount, 1);
    assert.equal(update.orchestration.status, 'complete');
    assert.deepEqual(update.artifactsProduced, [
      'audit-review-pack.latest.md',
      'audit-summary.latest.md',
      'producer-performance.json',
    ]);
    assert.deepEqual(update.auditSummary, {
      path: 'C:/repo/.audit/audit-summary.latest.md',
      format: 'markdown',
    });
    assert.equal(update.reviewPack.format, 'markdown');
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES1f. produced artifact registry uses typed rustAnalysis block instead of JS usability fallback', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-produced-artifacts-'));
  const out = path.join(fx, 'out');
  try {
    mkdirSync(out, { recursive: true });
    writeFileSync(
      path.join(out, 'rust-analyzer-health.latest.json'),
      JSON.stringify({
        schemaVersion: 'lumin-rust-analyzer-health.v1',
        status: 'complete',
        available: true,
      }),
    );

    assert.equal(
      auditManifest.collectProducedArtifacts(out).includes('rust-analyzer-health.latest.json'),
      false,
    );
    assert.equal(
      auditManifest
        .collectProducedArtifacts(out, {
          rustAnalysis: {
            status: 'complete',
            available: true,
          },
        })
        .includes('rust-analyzer-health.latest.json'),
      true,
    );
    assert.equal(
      auditManifest
        .collectProducedArtifacts(out, {
          rustAnalysis: {
            status: 'unavailable',
            available: false,
          },
        })
        .includes('rust-analyzer-health.latest.json'),
      false,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES1e. refreshManifestEvidence applies the Rust-owned evidence patch', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-refresh-'));
  const out = path.join(fx, 'out');
  try {
    mkdirSync(out, { recursive: true });
    writeFileSync(
      path.join(out, 'triage.json'),
      JSON.stringify({
        shape: {
          totalFiles: 2,
          tsFiles: 1,
          rsFiles: 1,
        },
      }),
    );
    writeFileSync(
      path.join(out, 'symbols.json'),
      JSON.stringify({
        uses: {
          external: 0,
          resolvedInternal: 0,
          unresolvedInternal: 0,
          unresolvedInternalRatio: 0,
        },
      }),
    );
    writeFileSync(path.join(out, 'framework-resource-surfaces.json'), '{not-json');
    writeFileSync(
      path.join(out, 'rust-analyzer-health.latest.json'),
      JSON.stringify({
        schemaVersion: 'lumin-rust-analyzer-health.v1',
      }),
    );

    const reads = [];
    const manifest = {};
    auditManifest.refreshManifestEvidence(manifest, {
      root: fx,
      outDir: out,
      includeTests: false,
      production: true,
      onArtifactRead: (read) => reads.push(read),
    });

    assert.equal(manifest.scanRange.files, 2);
    assert.equal(manifest.scanRange.includeTests, false);
    assert.equal(manifest.scanRange.production, true);
    assert.ok(Array.isArray(manifest.blindZones));
    assert.equal(manifest.frameworkResourceSurfaces?.status, 'unavailable');
    assert.equal(manifest.frameworkResourceSurfaces?.reason?.kind, 'malformed-json');
    assert.equal(manifest.frameworkResourceSurfaces?.totalFilesWithSurfaces, null);
    assert.equal(manifest.frameworkResourceSurfaces?.totalSurfaceLanes, null);
    assert.ok(reads.some((read) => read.filePath.endsWith('triage.json')));
    assert.ok(reads.some((read) => read.filePath.endsWith('symbols.json')));
    assert.ok(reads.some((read) => read.filePath.endsWith('rust-analyzer-health.latest.json')));
    assert.ok(reads.some((read) =>
      read.filePath.endsWith('framework-resource-surfaces.json') &&
      read.ok === false &&
      read.bytes > 0
    ));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES1c. producer performance audit-run wrapper leaves audit context projection in audit-core', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-producer-performance-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'triage.json'), '{}');
    writeFileSync(path.join(out, 'rust-analyzer-health.latest.json'), '{}');
    const artifact = auditManifest.buildProducerPerformanceArtifactForAuditRun({
      generated: '2026-07-01T00:00:00.000Z',
      root: fx,
      outDir: out,
      profile: 'quick',
      includeTests: true,
      production: false,
      excludes: ['dist'],
      autoExcludes: ['.audit'],
      noIncremental: true,
      cacheRoot: path.join(out, '.cache'),
      clearIncrementalCache: true,
      generatedArtifactsMode: 'prepared',
      artifactReads: {
        schemaVersion: 'artifact-read-metrics.v1',
        measurement: 'audit-repo-orchestrator-json-reads',
        totalReadCount: 0,
        totalReadBytes: 0,
        totalReadMs: 0,
        totalJsonParseMs: 0,
        parseFailureCount: 0,
        byName: {},
      },
      rustAnalysis: {
        status: 'complete',
        available: true,
      },
      commandsRun: [{ step: 'triage-repo.mjs', status: 'ok', ms: 3 }],
      skipped: [{ step: 'emit-sarif.mjs', reason: 'not in --sarif mode' }],
    });

    assert.equal(artifact.schemaVersion, 'producer-performance.v1');
    assert.equal(artifact.profile, 'quick');
    assert.deepEqual(artifact.scanRange.excludes, ['dist']);
    assert.deepEqual(artifact.scanRange.autoExcludes, ['.audit']);
    assert.equal(artifact.cache.noIncremental, true);
    assert.equal(artifact.cache.clearIncrementalCache, true);
    assert.equal(artifact.generatedArtifacts.mode, 'prepared');
    assert.equal(artifact.summary.producerCount, 1);
    assert.equal(artifact.summary.okCount, 1);
    assert.equal(artifact.summary.skippedCount, 1);
    assert.equal(artifact.summary.artifactCount, 2);
    assert.ok(Object.hasOwn(artifact.artifacts.byName, 'rust-analyzer-health.latest.json'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES1b. buildManifestEvidence can merge rustAnalysis run state in audit-core', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-rust-run-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'rust-analyzer-health.latest.json'), JSON.stringify({
      schemaVersion: 'lumin-rust-analyzer.v1',
      policyVersion: 'lumin-rust-analyzer-policy.v1',
      meta: {
        producer: 'lumin-rust-analyzer',
        mode: 'rust-main',
        input: { root: fx },
      },
      summary: { files: 1, syntaxReviewSignals: 0 },
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
      rustAnalysisRun: {
        requested: true,
        ran: true,
        status: 'complete',
        rustFiles: 1,
        sourceCommit: 'abc123',
      },
      mergeRustAnalysisRun: true,
    });

    assert.equal(evidence.rustAnalysis?.requested, true);
    assert.equal(evidence.rustAnalysis?.ran, true);
    assert.equal(evidence.rustAnalysis?.status, 'complete');
    assert.equal(evidence.rustAnalysis?.available, true);
    assert.equal(evidence.rustAnalysis?.files, 1);
    assert.equal(evidence.rustAnalysis?.sourceCommit, 'abc123');
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2. buildManifestEvidence summarizes generated artifact misses', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-generated-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      uses: {
        unresolvedInternalRatio: 0.2,
        unresolvedInternal: 3,
      },
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '@scope/prisma/enums',
          consumerFile: 'apps/web/src/a.ts',
          reason: 'workspace-generated-artifact-missing',
          hint: 'generated-artifact-missing',
          generatedArtifact: {
            policyVersion: 'generated-artifact-policy-v1',
            generatorFamily: 'prisma',
            confidence: 'strong',
            matchedPackage: '@scope/prisma',
            targetSubpath: 'enums',
          },
        },
        {
          specifier: '@scope/prisma/enums',
          consumerFile: 'apps/web/src/b.ts',
          reason: 'workspace-generated-artifact-missing',
          hint: 'generated-artifact-missing',
          generatedArtifact: {
            policyVersion: 'generated-artifact-policy-v1',
            generatorFamily: 'prisma',
            confidence: 'strong',
            matchedPackage: '@scope/prisma',
            targetSubpath: 'enums',
          },
        },
        {
          specifier: '@scope/types/missing',
          consumerFile: 'apps/web/src/c.ts',
          reason: 'workspace-package-subpath-target-missing',
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.deepEqual(evidence.generatedArtifacts?.reasonSummary, {
      'workspace-generated-artifact-missing': 2,
    });
    assert.equal(evidence.generatedArtifacts?.mode, 'default');
    assert.equal(evidence.generatedArtifacts?.executedGenerators, false);
    assert.equal(evidence.generatedArtifacts?.generatedArtifactPolicyVersion, 'generated-artifact-policy-v1');
    assert.deepEqual(evidence.generatedArtifacts?.supportedGenerators, []);
    assert.deepEqual(evidence.generatedArtifacts?.topGeneratedMisses, [
      {
        specifier: '@scope/prisma/enums',
        matchedPackage: '@scope/prisma',
        targetSubpath: 'enums',
        count: 2,
        generatorFamily: 'prisma',
        confidence: 'strong',
      },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2c. buildManifestEvidence summarizes framework/resource surfaces', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-framework-resource-surfaces-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'framework-resource-surfaces.json'), JSON.stringify({
      schemaVersion: 'framework-resource-surfaces.v1',
      policyVersion: 'framework-resource-surface-policy-v1',
      files: [
        {
          file: 'src/Button.stories.tsx',
          surfaceLanes: [
            {
              lane: 'framework-dispatch-entry',
              capabilityPack: 'framework.storybook',
              confidence: 'grounded',
              framework: 'storybook',
              reason: 'storybook-story-file',
            },
          ],
        },
        {
          file: 'templates/controller.ts.hbs',
          surfaceLanes: [
            {
              lane: 'scaffold-template-resource',
              capabilityPack: 'surface.scaffold-template',
              confidence: 'resource-only',
              reason: 'handlebars-template-resource',
            },
          ],
        },
      ],
      summary: {
        totalFilesWithSurfaces: 2,
        totalSurfaceLanes: 2,
        byLane: {
          'framework-dispatch-entry': 1,
          'scaffold-template-resource': 1,
        },
        byCapabilityPack: {
          'framework.storybook': 1,
          'surface.scaffold-template': 1,
        },
        byConfidence: {
          grounded: 1,
          'resource-only': 1,
        },
      },
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.equal(evidence.frameworkResourceSurfaces?.artifact, 'framework-resource-surfaces.json');
    assert.equal(evidence.frameworkResourceSurfaces?.policyVersion, 'framework-resource-surface-policy-v1');
    assert.equal(evidence.frameworkResourceSurfaces?.totalFilesWithSurfaces, 2);
    assert.deepEqual(evidence.frameworkResourceSurfaces?.byLane, {
      'framework-dispatch-entry': 1,
      'scaffold-template-resource': 1,
    });
    assert.deepEqual(evidence.frameworkResourceSurfaces?.byCapabilityPack, {
      'framework.storybook': 1,
      'surface.scaffold-template': 1,
    });
    assert.deepEqual(evidence.frameworkResourceSurfaces?.topExamples, [
      {
        file: 'src/Button.stories.tsx',
        lanes: ['framework-dispatch-entry'],
        capabilityPacks: ['framework.storybook'],
        reasons: ['storybook-story-file'],
      },
      {
        file: 'templates/controller.ts.hbs',
        lanes: ['scaffold-template-resource'],
        capabilityPacks: ['surface.scaffold-template'],
        reasons: ['handlebars-template-resource'],
      },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2d. buildManifestEvidence summarizes unused dependency evidence', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-unused-deps-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'unused-deps.json'), JSON.stringify({
      schemaVersion: 'unused-deps.v1',
      policyVersion: 'unused-deps-review-policy-v1',
      status: 'complete',
      summary: {
        packageCount: 2,
        declaredDependencyCount: 5,
        usedCount: 1,
        reviewUnusedCount: 2,
        mutedCount: 2,
        confidenceLimitedCount: 0,
        unavailableCount: 0,
        byReason: {
          'external-import-consumer': 1,
          'no-observed-consumer': 2,
          'package-script-tool': 1,
          'ambient-types': 1,
        },
      },
      packages: [
        {
          packageDir: 'packages/app',
          manifestPath: 'packages/app/package.json',
          dependencies: [
            {
              name: 'left-pad',
              field: 'dependencies',
              status: 'review-unused',
              reason: 'no-observed-consumer',
              confidence: 'review',
            },
          ],
        },
        {
          packageDir: '.',
          manifestPath: 'package.json',
          dependencies: [
            {
              name: 'unused-lib',
              field: 'devDependencies',
              status: 'review-unused',
              reason: 'no-observed-consumer',
              confidence: 'review',
            },
            {
              name: 'tsx',
              field: 'devDependencies',
              status: 'muted',
              reason: 'package-script-tool',
              confidence: 'grounded',
            },
          ],
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.equal(evidence.unusedDependencies?.artifact, 'unused-deps.json');
    assert.equal(evidence.unusedDependencies?.schemaVersion, 'unused-deps.v1');
    assert.equal(evidence.unusedDependencies?.policyVersion, 'unused-deps-review-policy-v1');
    assert.equal(evidence.unusedDependencies?.status, 'complete');
    assert.equal(evidence.unusedDependencies?.declaredDependencyCount, 5);
    assert.equal(evidence.unusedDependencies?.reviewUnusedCount, 2);
    assert.equal(evidence.unusedDependencies?.mutedCount, 2);
    assert.deepEqual(evidence.unusedDependencies?.byReason, {
      'external-import-consumer': 1,
      'no-observed-consumer': 2,
      'package-script-tool': 1,
      'ambient-types': 1,
    });
    assert.deepEqual(evidence.unusedDependencies?.topReviewUnused, [
      {
        packageDir: '.',
        manifestPath: 'package.json',
        name: 'unused-lib',
        field: 'devDependencies',
        reason: 'no-observed-consumer',
        confidence: 'review',
      },
      {
        packageDir: 'packages/app',
        manifestPath: 'packages/app/package.json',
        name: 'left-pad',
        field: 'dependencies',
        reason: 'no-observed-consumer',
        confidence: 'review',
      },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2g. buildManifestEvidence mirrors block clone summary without source fragments', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-block-clones-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'block-clones.json'), JSON.stringify({
      schemaVersion: 'block-clones.v1',
      policyVersion: 'block-clone-review-policy-v1',
      status: 'complete',
      normalization: {
        policyId: 'block-clone-normalization-v1',
        mode: 'alpha-identifier',
      },
      thresholds: {
        policyId: 'block-clone-threshold-policy-v2',
        minTokens: 50,
        minLines: 5,
        minOccurrences: 2,
        maxInstancesPerGroup: 20,
        maxCandidateGroups: 1000,
        maxReviewGroups: 100,
        maxMutedGroups: 100,
        maxGroups: 40,
        maxTokensPerFile: 200000,
      },
      summary: {
        fileCount: 12,
        tokenCount: 3400,
        groupCount: 2,
        instanceCount: 5,
        reviewGroupCount: 1,
        mutedGroupCount: 1,
        skippedFileCount: 1,
        unavailableFileCount: 0,
      },
      noisePolicy: {
        policyId: 'block-clone-noise-policy-v1',
        reviewGroupCount: 1,
        mutedGroupCount: 1,
        mutedByReason: {
          'node-vitest-mirror-pair': 1,
        },
        candidateCapSaturated: false,
        reviewCapSaturated: false,
        mutedCapSaturated: false,
      },
      groups: [
        {
          id: 'block-clone:sha256:abc',
          claim: 'repeated normalized token region',
          instances: [
            { file: 'src/a.ts', startLine: 1, endLine: 8 },
            { file: 'src/b.ts', startLine: 2, endLine: 9 },
          ],
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.deepEqual(evidence.blockClones, {
      artifact: 'block-clones.json',
      schemaVersion: 'block-clones.v1',
      policyVersion: 'block-clone-review-policy-v1',
      status: 'complete',
      reviewOnly: true,
      normalizationPolicyId: 'block-clone-normalization-v1',
      normalizationMode: 'alpha-identifier',
      thresholdPolicyId: 'block-clone-threshold-policy-v2',
      noisePolicyId: 'block-clone-noise-policy-v1',
      thresholds: {
        minTokens: 50,
        minLines: 5,
        minOccurrences: 2,
        maxInstancesPerGroup: 20,
        maxTokensPerFile: 200000,
        maxCandidateGroups: 1000,
        maxReviewGroups: 100,
        maxMutedGroups: 100,
        maxGroups: 40,
      },
      fileCount: 12,
      tokenCount: 3400,
      groupCount: 2,
      instanceCount: 5,
      reviewGroupCount: 1,
      mutedGroupCount: 1,
      mutedByReason: {
        'node-vitest-mirror-pair': 1,
      },
      candidateCapSaturated: false,
      reviewCapSaturated: false,
      mutedCapSaturated: false,
      skippedFileCount: 1,
      unavailableFileCount: 0,
    });
    assert.equal(Object.hasOwn(evidence.blockClones, 'groups'), false);
    assert.equal(Object.hasOwn(evidence.blockClones, 'instances'), false);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2e. buildManifestEvidence preserves unavailable unused dependency status', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-unused-deps-unavailable-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'unused-deps.json'), JSON.stringify({
      schemaVersion: 'unused-deps.v1',
      policyVersion: 'unused-deps-review-policy-v1',
      status: 'unavailable',
      reason: 'input-artifact-missing',
      summary: {
        packageCount: 0,
        declaredDependencyCount: 0,
        usedCount: 0,
        reviewUnusedCount: 0,
        mutedCount: 0,
        confidenceLimitedCount: 0,
        unavailableCount: 0,
        byReason: {},
      },
      packages: [],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.equal(evidence.unusedDependencies?.artifact, 'unused-deps.json');
    assert.equal(evidence.unusedDependencies?.status, 'unavailable');
    assert.equal(evidence.unusedDependencies?.reason, 'input-artifact-missing');
    assert.equal(evidence.unusedDependencies?.reviewUnusedCount, 0);
    assert.deepEqual(evidence.unusedDependencies?.topReviewUnused, []);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2f. buildManifestEvidence tolerates malformed unused dependency package lists', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-unused-deps-malformed-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'unused-deps.json'), JSON.stringify({
      schemaVersion: 'unused-deps.v1',
      policyVersion: 'unused-deps-review-policy-v1',
      status: 'complete',
      summary: {
        packageCount: 1,
        declaredDependencyCount: 1,
        usedCount: 0,
        reviewUnusedCount: 1,
        mutedCount: 0,
        confidenceLimitedCount: 0,
        unavailableCount: 0,
        byReason: { 'no-observed-consumer': 1 },
      },
      packages: [
        {
          packageDir: '.',
          manifestPath: 'package.json',
          dependencies: {},
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.equal(evidence.unusedDependencies?.status, 'complete');
    assert.equal(evidence.unusedDependencies?.reviewUnusedCount, 1);
    assert.deepEqual(evidence.unusedDependencies?.topReviewUnused, []);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES2b. buildManifestEvidence summarizes generated consumer blind zones by scope', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-generated-consumer-zones-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      generatedConsumerBlindZones: [
        {
          reason: 'generated-consumer-blind-zone',
          sourceReason: 'workspace-generated-artifact-missing',
          specifier: '@scope/prisma/enums',
          consumerFile: 'apps/web/src/a.ts',
          matchedPackage: '@scope/prisma',
          targetSubpath: 'enums',
          candidatePath: 'packages/prisma/generated/enums.ts',
          status: 'missing',
          scopePackageRoot: 'packages/prisma',
          mode: 'default',
        },
        {
          reason: 'generated-consumer-blind-zone',
          sourceReason: 'workspace-generated-artifact-missing',
          specifier: '@scope/prisma/enums',
          consumerFile: 'apps/api/src/b.ts',
          matchedPackage: '@scope/prisma',
          targetSubpath: 'enums',
          candidatePath: 'packages/prisma/generated/enums.ts',
          status: 'present-but-out-of-scope',
          scanScopeReason: 'excluded',
          scopePackageRoot: 'packages/prisma',
          mode: 'prepared',
          staleStatus: 'unknown',
          staleReason: 'generator-input-hash-not-recorded',
        },
        {
          reason: 'generated-consumer-blind-zone',
          sourceReason: 'workspace-generated-artifact-missing',
          specifier: '@scope/kysely/types',
          consumerFile: 'apps/api/src/c.ts',
          matchedPackage: '@scope/kysely',
          targetSubpath: 'types',
          candidatePath: 'packages/kysely/generated/types.ts',
          status: 'missing',
          scopePackageRoot: 'packages/kysely',
          mode: 'default',
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.equal(evidence.generatedArtifacts?.generatedConsumerBlindZoneCount, 3);
    assert.deepEqual(evidence.generatedArtifacts?.topGeneratedConsumerBlindZones, [
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
        examples: [
          {
            specifier: '@scope/prisma/enums',
            consumerFile: 'apps/api/src/b.ts',
            candidatePath: 'packages/prisma/generated/enums.ts',
            status: 'present-but-out-of-scope',
            scanScopeReason: 'excluded',
            mode: 'prepared',
          },
          {
            specifier: '@scope/prisma/enums',
            consumerFile: 'apps/web/src/a.ts',
            candidatePath: 'packages/prisma/generated/enums.ts',
            status: 'missing',
            mode: 'default',
          },
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
        examples: [
          {
            specifier: '@scope/kysely/types',
            consumerFile: 'apps/api/src/c.ts',
            candidatePath: 'packages/kysely/generated/types.ts',
            status: 'missing',
            mode: 'default',
          },
        ],
      },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES3. generated present mode reports existing targets excluded by scan policy', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-generated-present-'));
  const out = path.join(fx, 'out');
  const generatedFile = path.join(fx, 'packages/prisma/generated/enums.ts');
  mkdirSync(path.dirname(generatedFile), { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(generatedFile, 'export enum Kind { A = "A" }\n');
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '@scope/prisma/generated/enums',
          consumerFile: 'apps/web/src/a.ts',
          reason: 'workspace-generated-artifact-missing',
          hint: 'generated-artifact-missing',
          targetCandidates: ['packages/prisma/generated/enums.ts'],
          generatedArtifact: {
            policyVersion: 'generated-artifact-policy-v1',
            generatorFamily: 'prisma',
            confidence: 'strong',
            matchedPackage: '@scope/prisma',
            targetSubpath: 'generated/enums',
          },
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
      excludes: ['packages/prisma/generated'],
      generatedArtifactsMode: 'present',
    });

    assert.equal(evidence.generatedArtifacts?.mode, 'present');
    assert.equal(evidence.generatedArtifacts?.presentButOutOfScopeCount, 1);
    assert.deepEqual(evidence.generatedArtifacts?.presentButOutOfScope, [
      {
        specifier: '@scope/prisma/generated/enums',
        consumerFile: 'apps/web/src/a.ts',
        matchedPackage: '@scope/prisma',
        targetSubpath: 'generated/enums',
        candidatePath: 'packages/prisma/generated/enums.ts',
        reason: 'present-but-out-of-scope',
        mode: 'present',
      },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES4. generated prepared mode marks existing excluded targets as stale-unknown', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-generated-prepared-'));
  const out = path.join(fx, 'out');
  const generatedFile = path.join(fx, 'packages/prisma/generated/enums.ts');
  mkdirSync(path.dirname(generatedFile), { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(generatedFile, 'export enum Kind { A = "A" }\n');
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '@scope/prisma/generated/enums',
          consumerFile: 'apps/web/src/a.ts',
          reason: 'workspace-generated-artifact-missing',
          hint: 'generated-artifact-missing',
          targetCandidates: [generatedFile],
          generatedArtifact: {
            policyVersion: 'generated-artifact-policy-v1',
            generatorFamily: 'prisma',
            confidence: 'strong',
            matchedPackage: '@scope/prisma',
            targetSubpath: 'generated/enums',
          },
        },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
      excludes: ['packages/prisma/generated'],
      generatedArtifactsMode: 'prepared',
    });

    assert.equal(evidence.generatedArtifacts?.mode, 'prepared');
    assert.equal(evidence.generatedArtifacts?.presentButOutOfScope?.[0]?.staleStatus, 'unknown');
    assert.equal(
      evidence.generatedArtifacts?.presentButOutOfScope?.[0]?.staleReason,
      'generator-input-hash-not-recorded'
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES5. buildManifestEvidence summarizes resolver unresolved roots and reasons', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-resolver-diagnostics-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      uses: {
        resolvedInternal: 7,
        unresolvedInternalRatio: 0.31,
        unresolvedInternal: 4,
        external: 2,
      },
      topUnresolvedSpecifiers: [
        { specifierPrefix: '@scope/orm', count: 3, example: '@scope/orm/client' },
      ],
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '@scope/orm/client',
          consumerFile: 'apps/api/src/a.ts',
          reason: 'workspace-generated-artifact-missing',
          resolverStage: 'workspace-package-subpath',
          hint: 'generated-artifact-missing',
          typeOnly: false,
        },
        {
          specifier: '@scope/orm/client',
          consumerFile: 'apps/web/src/b.ts',
          reason: 'workspace-generated-artifact-missing',
          resolverStage: 'workspace-package-subpath',
          hint: 'generated-artifact-missing',
          typeOnly: true,
        },
        {
          specifier: '@scope/orm/helpers',
          consumerFile: 'apps/web/src/c.ts',
          reason: 'workspace-package-subpath-target-missing',
          resolverStage: 'workspace-package-subpath',
        },
        {
          specifier: 'app/routes/root',
          consumerFile: 'apps/web/src/d.ts',
          reason: 'tsconfig-path-target-missing',
          resolverStage: 'tsconfig-paths',
        },
      ],
      unresolvedInternalSummaryByReason: {
        'workspace-generated-artifact-missing': { count: 2 },
        'workspace-package-subpath-target-missing': { count: 1 },
        'tsconfig-path-target-missing': { count: 1 },
      },
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.equal(evidence.resolverDiagnostics?.unresolvedInternal, 4);
    assert.equal(evidence.resolverDiagnostics?.unresolvedInternalRatio, 0.31);
    assert.deepEqual(evidence.resolverDiagnostics?.topUnresolvedReasons, [
      { reason: 'workspace-generated-artifact-missing', count: 2 },
      { reason: 'tsconfig-path-target-missing', count: 1 },
      { reason: 'workspace-package-subpath-target-missing', count: 1 },
    ]);
    assert.deepEqual(evidence.resolverDiagnostics?.topSpecifierRoots, [
      {
        specifierRoot: '@scope/orm',
        count: 3,
        reasons: {
          'workspace-generated-artifact-missing': 2,
          'workspace-package-subpath-target-missing': 1,
        },
        examples: [
          { specifier: '@scope/orm/client', consumerFile: 'apps/api/src/a.ts' },
          { specifier: '@scope/orm/client', consumerFile: 'apps/web/src/b.ts' },
          { specifier: '@scope/orm/helpers', consumerFile: 'apps/web/src/c.ts' },
        ],
      },
      {
        specifierRoot: 'app',
        count: 1,
        reasons: {
          'tsconfig-path-target-missing': 1,
        },
        examples: [
          { specifier: 'app/routes/root', consumerFile: 'apps/web/src/d.ts' },
        ],
      },
    ]);
    assert.deepEqual(evidence.resolverDiagnostics?.topUnresolvedSpecifiers, [
      { specifierPrefix: '@scope/orm', count: 3, example: '@scope/orm/client' },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES5s. buildManifestEvidence summarizes SFC evidence counts without raw records', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-sfc-evidence-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      uses: {
        sfcScriptConsumers: 4,
        sfcScriptSrcReachability: 2,
        sfcStyleAssetReferences: 3,
        sfcTemplateComponentRefs: 5,
        sfcGlobalComponentRegistrations: 2,
        sfcGeneratedComponentManifests: 1,
        sfcFrameworkConventionComponents: 2,
      },
      sfcTemplateComponentRefs: [
        { tagName: 'SecretCard', consumerFile: 'src/App.vue' },
      ],
      sfcGlobalComponentRegistrations: [
        { componentName: 'GlobalSecret', consumerFile: 'src/main.ts' },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    assert.deepEqual(evidence.sfcEvidence, {
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
    });
    assert(
      !JSON.stringify(evidence.sfcEvidence).includes('SecretCard') &&
        !JSON.stringify(evidence.sfcEvidence).includes('GlobalSecret'),
      JSON.stringify(evidence.sfcEvidence)
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('AMES5b. manifest resolver blind zone uses resolver-diagnostics summary when present', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'audit-manifest-resolver-blind-zone-'));
  const out = path.join(fx, 'out');
  mkdirSync(out, { recursive: true });
  try {
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      uses: {
        resolvedInternal: 7,
        unresolvedInternalRatio: 0.31,
        unresolvedInternal: 4,
        external: 2,
      },
      topUnresolvedSpecifiers: [
        { specifierPrefix: '@legacy/fallback', count: 4, example: '@legacy/fallback/a' },
      ],
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '@legacy/fallback/a',
          consumerFile: 'apps/api/src/legacy.ts',
          reason: 'legacy-symbols-fallback',
        },
      ],
      unresolvedInternalSummaryByReason: {
        'legacy-symbols-fallback': { count: 4 },
      },
    }));
    writeFileSync(path.join(out, 'resolver-diagnostics.json'), JSON.stringify({
      schemaVersion: 'resolver-diagnostics.v1',
      resolverVersion: 'resolver-2026-05-v1',
      summary: {
        unresolvedInternal: 4,
        unresolvedInternalRatio: 0.31,
        blindZoneCount: 3,
        candidateTargetCount: 2,
        unresolvedImportCount: 4,
        blockedCandidateHintCount: 3,
        reasonCounts: {
          'workspace-package-subpath-target-missing': 3,
          'hash-import-target-missing': 1,
        },
        topFamilies: [
          { family: 'workspace-packages', count: 3 },
          { family: 'node-imports', count: 1 },
        ],
        topAffectedPackageScopes: [
          { affectedPackageScope: 'packages/lib', count: 2 },
          { affectedPackageScope: 'packages/app', count: 1 },
        ],
        topUnresolvedReasons: [
          { reason: 'workspace-package-subpath-target-missing', count: 3 },
          { reason: 'hash-import-target-missing', count: 1 },
        ],
        topSpecifierRoots: [
          {
            specifierRoot: '@scope/lib',
            count: 3,
            reasons: {
              'workspace-package-subpath-target-missing': 3,
            },
            examples: [
              { specifier: '@scope/lib/missing', consumerFile: 'apps/web/src/a.ts' },
            ],
          },
        ],
      },
      blindZones: [
        {
          family: 'workspace-packages',
          reason: 'workspace-package-subpath-target-missing',
          specifier: '@scope/lib/missing',
          importer: 'apps/web/src/a.ts',
          affectedPackageScope: 'packages/lib',
          blocksAbsenceClaims: true,
          relevance: 'target-candidate-package-scope',
        },
        {
          family: 'node-imports',
          reason: 'hash-import-target-missing',
          specifier: '#config',
          importer: 'packages/app/src/a.ts',
          affectedPackageScope: 'packages/app',
          blocksAbsenceClaims: true,
          relevance: 'affected-package-scope',
        },
      ],
      blockedCandidateHints: [
        {
          family: 'workspace-packages',
          reason: 'workspace-package-subpath-target-missing',
          specifier: '@scope/lib/missing',
          importer: 'apps/web/src/a.ts',
          affectedPackageScope: 'packages/lib',
          blockingScope: 'candidate-relevant',
          relevance: 'target-candidate-package-scope',
          proofUse: 'blocks-absence-claim',
          candidatePath: 'packages/lib/missing.ts',
        },
        {
          family: 'node-imports',
          reason: 'hash-import-target-missing',
          specifier: '#config',
          importer: 'packages/app/src/a.ts',
          affectedPackageScope: 'packages/app',
          blockingScope: 'candidate-relevant',
          relevance: 'affected-package-scope',
          proofUse: 'blocks-absence-claim',
          candidatePath: 'packages/app/src/config.ts',
        },
        {
          family: 'generated-artifacts',
          reason: 'generated-consumer-blind-zone',
          specifier: '@scope/generated/client',
          importer: 'packages/app/src/use-client.ts',
          affectedPackageScope: 'packages/generated',
          blockingScope: 'candidate-relevant',
          relevance: 'generated-consumer-scope',
          proofUse: 'blocks-absence-claim',
          candidatePath: 'packages/generated/client.ts',
        },
      ],
      candidateTargets: [
        { specifier: '@scope/lib/missing', candidates: ['packages/lib/missing.ts'] },
        { specifier: '#config', candidates: ['packages/app/src/config.ts'] },
      ],
      unresolvedImports: [
        { specifier: '@scope/lib/missing', consumerFile: 'apps/web/src/a.ts' },
      ],
    }));

    const evidence = auditManifest.buildManifestEvidence({
      root: fx,
      outDir: out,
      includeTests: true,
      production: false,
    });

    const resolverZone = evidence.blindZones.find((zone) => zone?.area === 'resolver');
    assert.ok(resolverZone, JSON.stringify(evidence.blindZones, null, 2));
    assert.equal(resolverZone.details?.sourceArtifact, 'resolver-diagnostics.json');
    assert.equal(resolverZone.details?.resolverVersion, 'resolver-2026-05-v1');
    assert.equal(resolverZone.details?.blindZoneCount, 3);
    assert.equal(resolverZone.details?.candidateTargetCount, 2);
    assert.equal(resolverZone.details?.unresolvedImportCount, 4);
    assert.equal(evidence.resolverDiagnostics?.blockedCandidateHintCount, 3);
    assert.equal(evidence.resolverDiagnostics?.blockedCandidateHintSampleLimit, 10);
    assert.deepEqual(evidence.resolverDiagnostics?.blockedCandidateHints, [
      {
        family: 'workspace-packages',
        reason: 'workspace-package-subpath-target-missing',
        specifier: '@scope/lib/missing',
        importer: 'apps/web/src/a.ts',
        affectedPackageScope: 'packages/lib',
        blockingScope: 'candidate-relevant',
        relevance: 'target-candidate-package-scope',
        proofUse: 'blocks-absence-claim',
        candidatePath: 'packages/lib/missing.ts',
      },
      {
        family: 'node-imports',
        reason: 'hash-import-target-missing',
        specifier: '#config',
        importer: 'packages/app/src/a.ts',
        affectedPackageScope: 'packages/app',
        blockingScope: 'candidate-relevant',
        relevance: 'affected-package-scope',
        proofUse: 'blocks-absence-claim',
        candidatePath: 'packages/app/src/config.ts',
      },
      {
        family: 'generated-artifacts',
        reason: 'generated-consumer-blind-zone',
        specifier: '@scope/generated/client',
        importer: 'packages/app/src/use-client.ts',
        affectedPackageScope: 'packages/generated',
        blockingScope: 'candidate-relevant',
        relevance: 'generated-consumer-scope',
        proofUse: 'blocks-absence-claim',
        candidatePath: 'packages/generated/client.ts',
      },
    ]);
    assert.deepEqual(evidence.resolverDiagnostics?.blockedCandidateHintReasonCounts, [
      {
        reason: 'generated-consumer-blind-zone',
        count: 1,
        families: { 'generated-artifacts': 1 },
      },
      {
        reason: 'hash-import-target-missing',
        count: 1,
        families: { 'node-imports': 1 },
      },
      {
        reason: 'workspace-package-subpath-target-missing',
        count: 1,
        families: { 'workspace-packages': 1 },
      },
    ]);
    assert.deepEqual(evidence.resolverDiagnostics?.blockedCandidateHintFamilyCounts, [
      {
        family: 'generated-artifacts',
        count: 1,
        reasons: { 'generated-consumer-blind-zone': 1 },
      },
      {
        family: 'node-imports',
        count: 1,
        reasons: { 'hash-import-target-missing': 1 },
      },
      {
        family: 'workspace-packages',
        count: 1,
        reasons: { 'workspace-package-subpath-target-missing': 1 },
      },
    ]);
    assert.deepEqual(resolverZone.details?.reasonCounts, {
      'workspace-package-subpath-target-missing': 3,
      'hash-import-target-missing': 1,
    });
    assert.deepEqual(resolverZone.details?.topFamilies, [
      { family: 'workspace-packages', count: 3 },
      { family: 'node-imports', count: 1 },
    ]);
    assert.deepEqual(evidence.resolverDiagnostics?.topAffectedPackageScopes, [
      { affectedPackageScope: 'packages/lib', count: 2 },
      { affectedPackageScope: 'packages/app', count: 1 },
    ]);
    assert.deepEqual(resolverZone.details?.topAffectedPackageScopes, [
      { affectedPackageScope: 'packages/lib', count: 2 },
      { affectedPackageScope: 'packages/app', count: 1 },
    ]);
    assert.deepEqual(resolverZone.details?.topUnresolvedReasons, [
      { reason: 'workspace-package-subpath-target-missing', count: 3 },
      { reason: 'hash-import-target-missing', count: 1 },
    ]);
    assert.deepEqual(resolverZone.details?.topSpecifierRoots, [
      {
        specifierRoot: '@scope/lib',
        count: 3,
        reasons: {
          'workspace-package-subpath-target-missing': 3,
        },
        examples: [
          { specifier: '@scope/lib/missing', consumerFile: 'apps/web/src/a.ts' },
        ],
      },
    ]);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
