import assert from 'node:assert/strict';
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

const evidence = await import('../_lib/generated-artifact-evidence.mjs');

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

check('GAE1. build output requires files coverage plus build-like script evidence', () => {
  const packet = evidence.generatedOutputArtifactEvidence({
    name: '@scope/bundle',
    files: ['dist'],
    scripts: { build: 'vite build' },
  }, './dist/bundle.js', 'exports["."]');

  assert.equal(packet?.policyVersion, 'generated-artifact-policy-v1');
  assert.equal(packet?.generatorFamily, 'build-output');
  assert.equal(packet?.confidence, 'strong');
  assert.equal(packet?.matchedPackage, '@scope/bundle');
  assert.equal(packet?.targetSubpath, 'dist/bundle.js');
  assert(packet?.evidence?.some((item) =>
    item.kind === 'package-files' &&
    item.field === 'files' &&
    item.matched === 'dist'));
  assert(packet?.evidence?.some((item) =>
    item.kind === 'package-script' &&
    item.field === 'scripts.build' &&
    item.matched === 'vite build'));
});

check('GAE1b. generated artifact identity constants are exported from the policy module', () => {
  assert.equal(evidence.GENERATED_ARTIFACT_POLICY_VERSION, 'generated-artifact-policy-v1');
  assert.equal(evidence.GENERATED_ARTIFACT_MISSING_HINT, 'generated-artifact-missing');
  assert.equal(evidence.GENERATED_ARTIFACT_MISSING_REASON, 'workspace-generated-artifact-missing');
});

check('GAE2. files-only build output remains weak and does not produce strong generated evidence', () => {
  const packet = evidence.generatedOutputArtifactEvidence({
    name: '@scope/bundle',
    files: ['dist'],
  }, './dist/bundle.js', 'exports["."]');

  assert.equal(packet, null);
});

check('GAE3. static output requires explicit script output path evidence', () => {
  const packet = evidence.generatedOutputArtifactEvidence({
    name: '@scope/css-output',
    files: ['style.min.css'],
    scripts: { build: 'postcss ./style.css -o ./style.min.css' },
  }, './style.min.css', 'exports["./style.min.css"]');

  assert.equal(packet?.generatorFamily, 'static-artifact');
  assert.equal(packet?.confidence, 'strong');
  assert.equal(packet?.targetSubpath, 'style.min.css');
  assert(packet?.evidence?.some((item) =>
    item.kind === 'script-output-path' &&
    item.field === 'scripts.build' &&
    item.matched === 'style.min.css'));
});

check('GAE3b. relative generated artifact evidence requires exact package script output path', () => {
  const root = path.join(tmpdir(), 'gae-relative-generated');
  rmSync(root, { recursive: true, force: true });
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({
    name: 'relative-generated-fixture',
    scripts: {
      tailwind: 'tailwindcss --input ./src/styles.css --output ./src/tailwind.generated.css',
    },
  }));
  const fromFile = path.join(root, 'src', 'consumer.ts');
  const target = path.join(root, 'src', 'tailwind.generated.css');

  const packet = evidence.generatedRelativeArtifactEvidence(root, fromFile, target);

  assert.equal(packet?.policyVersion, 'generated-artifact-policy-v1');
  assert.equal(packet?.generatorFamily, 'local-generated-asset');
  assert.equal(packet?.confidence, 'strong');
  assert.equal(packet?.matchedPackage, 'relative-generated-fixture');
  assert.equal(packet?.packageRoot, '.');
  assert.equal(packet?.targetSubpath, 'src/tailwind.generated.css');
  assert(packet?.evidence?.some((item) =>
    item.kind === 'script-output-path' &&
    item.field === 'scripts.tailwind' &&
    item.matched === 'src/tailwind.generated.css'));
  assert.equal(evidence.generatedRelativeArtifactEvidence(
    root,
    fromFile,
    path.join(root, 'src', 'ordinary.css'),
  ), null);
});

check('GAE4. path-segment evidence is supporting only and exposes the generated hint', () => {
  const root = path.resolve('repo');
  const candidate = path.join(root, 'packages/generated/generated/client');
  const packet = evidence.generatedArtifactForTargetCandidates(root, [candidate]);

  assert.equal(evidence.GENERATED_ARTIFACT_MISSING_HINT, 'generated-artifact-missing');
  assert.equal(packet?.policyVersion, 'generated-artifact-policy-v1');
  assert.equal(packet?.generatorFamily, 'path-segment');
  assert.equal(packet?.confidence, 'supporting');
  assert.equal(packet?.targetSubpath, 'packages/generated/generated/client');
  assert.equal(evidence.isStrongGeneratedArtifact(packet), false);
  assert.equal(evidence.unresolvedGeneratedArtifactHintForCandidates([candidate]), 'generated-artifact-missing');
});

check('GAE5. workspace subpath evidence matches normalized target subpaths', () => {
  const packet = {
    policyVersion: 'generated-artifact-policy-v1',
    generatorFamily: 'prisma',
    confidence: 'strong',
    targetSubpath: 'enums',
    evidence: [],
  };
  const entry = {
    legacySubpath: true,
    generatedSubpathEvidence: [packet],
  };

  assert.equal(evidence.generatedWorkspaceSubpathEvidence(entry, 'enums.ts'), packet);
  assert.equal(evidence.generatedWorkspaceSubpathEvidence(entry, 'client'), null);
});

check('GAE6. generated artifact identity strings are not hardcoded outside the policy module', () => {
  const sourceFiles = [
    '_lib/audit-manifest.mjs',
    '_lib/resolver-core.mjs',
    '_lib/finding-provenance.mjs',
    '_lib/generated-blind-zone-relevance.mjs',
    '_lib/ranking.mjs',
  ];
  for (const file of sourceFiles) {
    const src = readFileSync(file, 'utf8');
    assert.equal(src.includes("'workspace-generated-artifact-missing'"), false, file);
    assert.equal(src.includes('"workspace-generated-artifact-missing"'), false, file);
    assert.equal(src.includes("'generated-artifact-missing'"), false, file);
    assert.equal(src.includes('"generated-artifact-missing"'), false, file);
    assert.equal(src.includes("'generated-artifact-policy-v1'"), false, file);
    assert.equal(src.includes('"generated-artifact-policy-v1"'), false, file);
  }
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
