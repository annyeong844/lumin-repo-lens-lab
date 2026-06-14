import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';

import {
  buildGeneratedConsumerBlindZones,
  generatedConsumerBlindZoneRelevance,
  generatedArtifactRelevance,
  generatedArtifactRelevantTaint,
} from '../_lib/generated-blind-zone-relevance.mjs';

function submoduleOf(file) {
  const parts = String(file ?? '').replace(/\\/g, '/').split('/');
  if ((parts[0] === 'apps' || parts[0] === 'packages') && parts.length >= 2) {
    return `${parts[0]}/${parts[1]}`;
  }
  return parts[0] || 'root';
}

function generatedRecord(overrides = {}) {
  return {
    specifier: '@scope/prisma/enums',
    consumerFile: 'apps/web/page.ts',
    reason: 'workspace-generated-artifact-missing',
    hint: 'generated-artifact-missing',
    targetCandidates: ['packages/prisma/generated/enums.ts'],
    generatedArtifact: {
      policyVersion: 'generated-artifact-policy-v1',
      matchedPackage: '@scope/prisma',
      packageRoot: 'packages/prisma',
      targetSubpath: 'generated/enums',
      generatorFamily: 'prisma',
      confidence: 'strong',
    },
    ...overrides,
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

check('GBZR1. candidate inside generated package root is relevant provider surface', () => {
  const relevance = generatedArtifactRelevance(
    { file: 'packages/prisma/index.ts', symbol: 'PrismaEnums' },
    generatedRecord(),
    { submoduleOf },
  );

  assert.deepEqual(relevance, {
    impact: 'provider-surface-unresolved',
    relevance: 'matched-package-root',
  });
});

check('GBZR2. target candidate submodule is relevant when package root is absent', () => {
  const relevance = generatedArtifactRelevance(
    { file: 'packages/prisma/client.ts', symbol: 'PrismaClient' },
    generatedRecord({
      generatedArtifact: {
        policyVersion: 'generated-artifact-policy-v1',
        matchedPackage: '@scope/prisma',
        targetSubpath: 'generated/enums',
        generatorFamily: 'prisma',
        confidence: 'strong',
      },
    }),
    { submoduleOf },
  );

  assert.deepEqual(relevance, {
    impact: 'provider-surface-unresolved',
    relevance: 'target-candidate-submodule',
  });
});

check('GBZR3. consumer submodule alone is not relevant provider-surface proof', () => {
  const relevance = generatedArtifactRelevance(
    { file: 'apps/web/components/Button.tsx', symbol: 'Button' },
    generatedRecord(),
    { submoduleOf },
  );

  assert.equal(relevance, null);
});

check('GBZR4. consumer-only generated miss does not create finding taint', () => {
  const taint = generatedArtifactRelevantTaint(
    { file: 'apps/web/components/Button.tsx', symbol: 'Button' },
    [generatedRecord()],
    { submoduleOf },
  );

  assert.equal(taint, null);
});

check('GBZR5. generated consumer blind-zone inventory records missing generated target scope', () => {
  const zones = buildGeneratedConsumerBlindZones({
    unresolvedInternalSpecifierRecords: [generatedRecord()],
  }, {
    root: 'C:/repo',
    includeTests: true,
    exclude: [],
    mode: 'default',
  });

  assert.deepEqual(zones, [{
    reason: 'generated-consumer-blind-zone',
    sourceReason: 'workspace-generated-artifact-missing',
    specifier: '@scope/prisma/enums',
    consumerFile: 'apps/web/page.ts',
    matchedPackage: '@scope/prisma',
    targetSubpath: 'generated/enums',
    generatorFamily: 'prisma',
    confidence: 'strong',
    candidatePath: 'packages/prisma/generated/enums.ts',
    status: 'missing',
    scopePackageRoot: 'packages/prisma',
    mode: 'default',
  }]);
});

check('GBZR6. generated consumer blind-zone marks present generated files excluded by scan policy', () => {
  const fx = mkdtempSync(path.join(tmpdir(), 'generated-consumer-zone-'));
  try {
    const generatedFile = path.join(fx, 'packages/prisma/generated/enums.ts');
    mkdirSync(path.dirname(generatedFile), { recursive: true });
    writeFileSync(generatedFile, 'export const GeneratedEnum = 1;\n');

    const zones = buildGeneratedConsumerBlindZones({
      unresolvedInternalSpecifierRecords: [generatedRecord({
        targetCandidates: [generatedFile],
      })],
    }, {
      root: fx,
      includeTests: true,
      exclude: ['packages/prisma/generated'],
      mode: 'prepared',
    });

    assert.equal(zones.length, 1);
    assert.equal(zones[0]?.status, 'present-but-out-of-scope');
    assert.equal(zones[0]?.scanScopeReason, 'excluded');
    assert.equal(zones[0]?.staleStatus, 'unknown');
    assert.equal(zones[0]?.staleReason, 'generator-input-hash-not-recorded');
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

check('GBZR7. generated consumer blind-zone relevance is scoped to generated package surface', () => {
  const zone = buildGeneratedConsumerBlindZones({
    unresolvedInternalSpecifierRecords: [generatedRecord()],
  }, {
    root: 'C:/repo',
  })[0];

  assert.deepEqual(
    generatedConsumerBlindZoneRelevance(
      { file: 'packages/prisma/model.ts', symbol: 'ModelName' },
      zone,
      { submoduleOf },
    ),
    {
      impact: 'consumer-surface-unresolved',
      relevance: 'generated-consumer-scope',
    },
  );
  assert.equal(
    generatedConsumerBlindZoneRelevance(
      { file: 'apps/web/components/Button.tsx', symbol: 'Button' },
      zone,
      { submoduleOf },
    ),
    null,
  );
});

check('GBZR8. generated consumer blind-zone can create structured soft taint', () => {
  const zone = buildGeneratedConsumerBlindZones({
    unresolvedInternalSpecifierRecords: [generatedRecord()],
  }, {
    root: 'C:/repo',
  })[0];
  const taint = generatedArtifactRelevantTaint(
    { file: 'packages/prisma/model.ts', symbol: 'ModelName' },
    [],
    { submoduleOf, generatedConsumerBlindZones: [zone] },
  );

  assert.equal(taint?.kind, 'generated-artifact-missing-relevant');
  assert.equal(taint?.reason, 'generated-consumer-blind-zone');
  assert.equal(taint?.impact, 'consumer-surface-unresolved');
  assert.equal(taint?.relevance, 'generated-consumer-scope');
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
