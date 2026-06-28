import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

{
  const fx = mkdtempSync(path.join(tmpdir(), 'fx-resolver-diagnostics-'));
  const out = path.join(fx, 'audit');
  try {
    mkdirSync(out, { recursive: true });
    writeFileSync(path.join(out, 'symbols.json'), JSON.stringify({
      uses: {
        resolvedInternal: 7,
        unresolvedInternal: 4,
        unresolvedInternalRatio: 0.3636,
        external: 2,
      },
      topUnresolvedSpecifiers: [
        { specifierPrefix: '@scope/orm', count: 2, example: '@scope/orm/client' },
      ],
      unresolvedInternalSpecifierRecords: [
        {
          specifier: '#app/config',
          consumerFile: 'packages/app/src/a.ts',
          kind: 'import',
          reason: 'hash-import-target-missing',
          resolverStage: 'hash-imports',
          matchedPattern: '#app/*',
          targetCandidates: ['packages/app/src/config'],
        },
        {
          specifier: '@scope/orm/client',
          consumerFile: 'apps/api/src/b.ts',
          kind: 'import',
          reason: 'workspace-generated-artifact-missing',
          resolverStage: 'workspace-package-subpath',
          hint: 'generated-artifact-missing',
          targetCandidates: ['packages/orm/client'],
          generatedArtifact: {
            policyVersion: 'generated-artifact-policy-v1',
            matchedPackage: '@scope/orm',
            targetSubpath: 'client',
            generatorFamily: 'prisma',
            confidence: 'strong',
            packageRoot: 'packages/orm',
          },
        },
        {
          specifier: 'app/routes/root',
          consumerFile: 'apps/web/src/c.ts',
          kind: 'import',
          reason: 'tsconfig-path-target-missing',
          resolverStage: 'tsconfig-paths',
          matchedPattern: 'app/*',
          targetCandidates: ['apps/web/app/routes/root'],
        },
      ],
      generatedConsumerBlindZones: [
        {
          reason: 'generated-consumer-blind-zone',
          sourceReason: 'workspace-generated-artifact-missing',
          specifier: '@scope/orm/client',
          consumerFile: 'apps/api/src/b.ts',
          matchedPackage: '@scope/orm',
          targetSubpath: 'client',
          generatorFamily: 'prisma',
          confidence: 'strong',
          candidatePath: 'packages/orm/client',
          status: 'missing',
          scopePackageRoot: 'packages/orm',
          mode: 'prepared',
          staleStatus: 'unknown',
          staleReason: 'generator-input-hash-not-recorded',
        },
      ],
    }, null, 2));

    execFileSync(process.execPath, [
      path.join(DIR, 'build-resolver-diagnostics.mjs'),
      '--root', fx,
      '--output', out,
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const capsPath = path.join(out, 'resolver-capabilities.json');
    const diagPath = path.join(out, 'resolver-diagnostics.json');
    const caps = JSON.parse(readFileSync(capsPath, 'utf8'));
    const diag = JSON.parse(readFileSync(diagPath, 'utf8'));

    const nodeImports = caps.families?.find((family) => family.family === 'node-imports');
    const tsconfig = caps.families?.find((family) => family.family === 'tsconfig-paths');

    assert('RD1. resolver-capabilities.json is written with deterministic schema',
      existsSync(capsPath) &&
        caps.schemaVersion === 'resolver-capabilities.v1' &&
        /^resolver-\d{4}-\d{2}-v\d+$/.test(caps.resolverVersion) &&
        Array.isArray(caps.conditionProfiles) &&
        caps.conditionProfiles[0]?.profileId === 'node-esm-default' &&
        nodeImports?.status === 'partial' &&
        nodeImports?.reasonCodes?.includes('hash-import-target-missing') &&
        tsconfig?.absenceClaimPolicy === 'fail-closed-when-relevant',
      JSON.stringify(caps, null, 2));

    assert('RD2. resolver-diagnostics.json preserves unresolved events and output levels',
      existsSync(diagPath) &&
        diag.schemaVersion === 'resolver-diagnostics.v1' &&
        diag.resolverVersion === caps.resolverVersion &&
        diag.capabilityArtifact === 'resolver-capabilities.json' &&
        diag.capabilityReference?.artifact === 'resolver-capabilities.json' &&
        diag.capabilityReference?.schemaVersion === caps.schemaVersion &&
        diag.capabilityReference?.resolverVersion === caps.resolverVersion &&
        diag.unresolvedImports?.some((item) =>
          item.specifier === '#app/config' &&
          item.family === 'node-imports' &&
          item.outputLevel === 'unresolved_with_reason' &&
          item.reason === 'hash-import-target-missing') &&
        diag.unresolvedImports?.some((item) =>
          item.specifier === '@scope/orm/client' &&
          item.family === 'generated-artifacts' &&
          item.generatedArtifact?.generatorFamily === 'prisma'),
      JSON.stringify(diag, null, 2));

    assert('RD3. resolver-diagnostics.json emits candidate targets and blind zones',
      diag.candidateTargets?.some((item) =>
        item.specifier === '#app/config' &&
        item.family === 'node-imports' &&
        item.outputLevel === 'candidate' &&
        item.proofUse === 'diagnostic-only' &&
        item.createsGraphEdge === false &&
        item.notResolvedBecause === 'hash-import-target-missing' &&
        item.candidatePaths?.includes('packages/app/src/config')) &&
        diag.blindZones?.some((zone) =>
          zone.reason === 'generated-consumer-blind-zone' &&
          zone.family === 'generated-artifacts' &&
          zone.affectedPackageScope === 'packages/orm' &&
          zone.blocksAbsenceClaims === true &&
          zone.staleStatus === 'unknown'),
      JSON.stringify(diag, null, 2));

    const hashImportZone = diag.blindZones?.find((zone) => zone.specifier === '#app/config');
    const generatedConsumerZone = diag.blindZones?.find((zone) =>
      zone.reason === 'generated-consumer-blind-zone');

    assert('RD3b. resolver blind zones declare candidate-relevant blocking policy',
      hashImportZone?.blockingScope === 'candidate-relevant' &&
        hashImportZone.relevancePolicy?.policyVersion === 'resolver-blind-zone-relevance.v1' &&
        hashImportZone.relevancePolicy?.mustNotBlockUnrelatedCandidates === true &&
        hashImportZone.relevancePolicy?.candidateRelevantWhen?.includes('target-candidate-file') &&
        hashImportZone.relevancePolicy?.candidateRelevantWhen?.includes('target-candidate-package-scope') &&
        hashImportZone.relevancePolicy?.candidateRelevantWhen?.includes('target-candidate-submodule'),
      JSON.stringify(hashImportZone, null, 2));

    assert('RD3c. generated consumer blind zones declare generated relevance policy',
      generatedConsumerZone?.blockingScope === 'candidate-relevant' &&
        generatedConsumerZone.relevancePolicy?.policyVersion === 'generated-blind-zone-relevance.v1' &&
        generatedConsumerZone.relevancePolicy?.mustNotBlockUnrelatedCandidates === true &&
        generatedConsumerZone.relevancePolicy?.candidateRelevantWhen?.includes('generated-consumer-scope') &&
        generatedConsumerZone.relevancePolicy?.candidateRelevantWhen?.includes('generated-consumer-target-submodule'),
      JSON.stringify(generatedConsumerZone, null, 2));

    assert('RD4. resolver-diagnostics exposes compact blocked candidate hints',
      diag.summary?.blockedCandidateHintCount === diag.blockedCandidateHints?.length &&
        diag.blockedCandidateHints?.some((hint) =>
          hint.family === 'node-imports' &&
          hint.reason === 'hash-import-target-missing' &&
          hint.specifier === '#app/config' &&
          hint.candidatePath === 'packages/app/src/config' &&
          hint.affectedPackageScope === 'packages/app' &&
          hint.blockingScope === 'candidate-relevant' &&
          hint.proofUse === 'blocks-absence-claim') &&
        diag.blockedCandidateHints?.some((hint) =>
          hint.family === 'generated-artifacts' &&
          hint.reason === 'generated-consumer-blind-zone' &&
          hint.specifier === '@scope/orm/client' &&
          hint.candidatePath === 'packages/orm/client' &&
          hint.affectedPackageScope === 'packages/orm' &&
          hint.relevance === 'generated-consumer-scope'),
      JSON.stringify(diag.blockedCandidateHints, null, 2));

    assert('RD5. resolver-diagnostics summary is sorted and machine-readable',
      diag.summary?.unresolvedInternal === 4 &&
        diag.summary?.blindZoneCount === diag.blindZones.length &&
        diag.summary?.blockedCandidateHintCount === diag.blockedCandidateHints.length &&
        diag.summary?.topFamilies?.[0]?.family === 'generated-artifacts' &&
        diag.summary?.topAffectedPackageScopes?.some((item) =>
          item.affectedPackageScope === 'packages/orm' &&
          item.count === 2) &&
        diag.summary?.topUnresolvedReasons?.some((item) =>
          item.reason === 'workspace-generated-artifact-missing' &&
          item.count === 1),
      JSON.stringify(diag.summary, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
