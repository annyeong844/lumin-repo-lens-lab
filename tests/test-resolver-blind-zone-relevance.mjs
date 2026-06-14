import assert from 'node:assert/strict';

import {
  RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION,
  resolverBlindZoneRelevance,
  resolverBlindZoneRelevantTaint,
} from '../_lib/resolver-blind-zone-relevance.mjs';
import { computeFindingProvenance } from '../_lib/finding-provenance.mjs';
import { tierForFinding } from '../_lib/ranking.mjs';

function submoduleOf(file) {
  const parts = String(file ?? '').replace(/\\/g, '/').replace(/^\.\//, '').split('/');
  if ((parts[0] === 'apps' || parts[0] === 'packages') && parts.length >= 2) {
    return `${parts[0]}/${parts[1]}`;
  }
  return parts[0] || 'root';
}

function workspaceMiss(overrides = {}) {
  return {
    specifier: '@scope/lib/missing',
    consumerFile: 'apps/web/src/page.ts',
    reason: 'workspace-package-subpath-target-missing',
    resolverStage: 'workspace-package-subpath',
    outputLevel: 'unresolved_with_reason',
    targetCandidates: ['packages/lib/src/missing.ts'],
    family: 'workspace-packages',
    ...overrides,
  };
}

function safeAction() {
  return {
    kind: 'demote_export_declaration',
    proofComplete: true,
    actionBlockers: [],
    strongerActionBlockers: [],
  };
}

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

check('RBZR0. resolver relevance policy version is exported for diagnostics', () => {
  assert.equal(RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION, 'resolver-blind-zone-relevance.v1');
});

check('RBZR1. target candidate package scope is relevant only to same package findings', () => {
  assert.deepEqual(
    resolverBlindZoneRelevance(
      { file: 'packages/lib/src/foo.ts', symbol: 'foo' },
      workspaceMiss(),
      { submoduleOf },
    ),
    {
      impact: 'resolver-surface-unresolved',
      relevance: 'target-candidate-package-scope',
      severity: 'soft',
    },
  );

  assert.equal(
    resolverBlindZoneRelevance(
      { file: 'packages/ui/src/Button.tsx', symbol: 'Button' },
      workspaceMiss(),
      { submoduleOf },
    ),
    null,
  );
});

check('RBZR2. explicit affectedPackageScope scopes relevance without repo-global blocking', () => {
  assert.deepEqual(
    resolverBlindZoneRelevance(
      { file: 'packages/api/src/router.ts', symbol: 'router' },
      workspaceMiss({
        targetCandidates: [],
        affectedPackageScope: 'packages/api',
        family: 'conditional-exports',
        reason: 'condition-profile-ambiguous',
      }),
      { submoduleOf },
    ),
    {
      impact: 'resolver-surface-unresolved',
      relevance: 'affected-package-scope',
      severity: 'soft',
    },
  );

  assert.equal(
    resolverBlindZoneRelevance(
      { file: 'packages/web/src/router.ts', symbol: 'router' },
      workspaceMiss({
        targetCandidates: [],
        affectedPackageScope: 'packages/api',
        family: 'conditional-exports',
        reason: 'condition-profile-ambiguous',
      }),
      { submoduleOf },
    ),
    null,
  );
});

check('RBZR3. exact target candidate file remains a blocking unresolved match', () => {
  assert.deepEqual(
    resolverBlindZoneRelevance(
      { file: 'packages/lib/src/missing.ts', symbol: 'missing' },
      workspaceMiss(),
      { submoduleOf },
    ),
    {
      impact: 'resolver-surface-unresolved',
      relevance: 'target-candidate-file',
      severity: 'blocking',
    },
  );
});

check('RBZR4. generated artifact records stay owned by generated relevance helpers', () => {
  assert.equal(
    resolverBlindZoneRelevance(
      { file: 'packages/prisma/index.ts', symbol: 'PrismaEnums' },
      workspaceMiss({
        reason: 'workspace-generated-artifact-missing',
        hint: 'generated-artifact-missing',
        generatedArtifact: {
          policyVersion: 'generated-artifact-policy-v1',
          packageRoot: 'packages/prisma',
        },
        targetCandidates: ['packages/prisma/generated/enums.ts'],
      }),
      { submoduleOf },
    ),
    null,
  );

  assert.equal(
    resolverBlindZoneRelevance(
      { file: 'packages/prisma/model.ts', symbol: 'ModelName' },
      {
        family: 'generated-artifacts',
        reason: 'generated-consumer-blind-zone',
        affectedPackageScope: 'packages/prisma',
        candidatePath: 'packages/prisma/generated/enums.ts',
      },
      { submoduleOf },
    ),
    null,
  );
});

check('RBZR5. generic resolver relevance creates structured soft taint', () => {
  const taint = resolverBlindZoneRelevantTaint(
    { file: 'packages/lib/src/foo.ts', symbol: 'foo' },
    [workspaceMiss()],
    { submoduleOf },
  );

  assert.equal(taint?.kind, 'resolver-blind-zone-relevant');
  assert.equal(taint?.reason, 'workspace-package-subpath-target-missing');
  assert.equal(taint?.family, 'workspace-packages');
  assert.equal(taint?.impact, 'resolver-surface-unresolved');
  assert.equal(taint?.relevance, 'target-candidate-package-scope');
  assert.equal(taint?.total, 1);
});

check('RBZR6. computeFindingProvenance uses generic resolver relevance without repo-wide taint', () => {
  const relevant = computeFindingProvenance(
    { file: 'packages/lib/src/foo.ts', symbol: 'foo' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [workspaceMiss()],
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    },
  );
  const unrelated = computeFindingProvenance(
    { file: 'packages/ui/src/Button.tsx', symbol: 'Button' },
    {
      filesWithParseErrors: [],
      unresolvedInternalSpecifiers: [workspaceMiss()],
      submoduleOf,
      astEvidence: 'ast-ident-ref-count',
      astCount: 0,
    },
  );

  assert.equal(
    relevant.taintedBy.some((t) => t.kind === 'resolver-blind-zone-relevant'),
    true,
    JSON.stringify(relevant.taintedBy),
  );
  assert.equal(relevant.resolverConfidence, 'medium');
  assert.deepEqual(unrelated.taintedBy, []);
  assert.equal(unrelated.resolverConfidence, 'high');
});

check('RBZR7. generic resolver soft taint demotes SAFE_FIX to REVIEW_FIX with blocker detail', () => {
  const result = tierForFinding({
    file: 'packages/lib/src/foo.ts',
    line: 1,
    symbol: 'foo',
    bucket: 'C',
    safeAction: safeAction(),
    taintedBy: [{
      kind: 'resolver-blind-zone-relevant',
      reason: 'workspace-package-subpath-target-missing',
      family: 'workspace-packages',
      specifier: '@scope/lib/missing',
      impact: 'resolver-surface-unresolved',
      relevance: 'target-candidate-package-scope',
      effect: '...',
    }],
  }, {
    resolver: { unresolvedRatio: 0.01 },
  });

  assert.equal(result.tier, 'REVIEW_FIX');
  assert.match(result.reason, /resolver-blind-zone/);
  assert.equal(result.blockedPromotion, true);
  assert.equal(result.blockedBy?.[0]?.family, 'workspace-packages');
  assert.equal(result.blockedBy?.[0]?.relevance, 'target-candidate-package-scope');
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
