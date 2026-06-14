import assert from 'node:assert/strict';

const functionCloneArtifact = await import('../_lib/function-clone-artifact.mjs');

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

check('FCES1. function-clone artifact exposes builder, not version internals', () => {
  assert.equal(typeof functionCloneArtifact.buildFunctionCloneArtifact, 'function');

  for (const symbol of [
    'FUNCTION_CLONE_SCHEMA_VERSION',
    'FUNCTION_CLONE_NORMALIZED_VERSION',
  ]) {
    assert.equal(Object.hasOwn(functionCloneArtifact, symbol), false, symbol);
  }
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
