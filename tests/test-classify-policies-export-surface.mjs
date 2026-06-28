import assert from 'node:assert/strict';

const classifyPolicies = await import('../_lib/classify-policies.mjs');

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

check('CPES1. classify-policies does not expose legacy framework sentinel helpers', () => {
  for (const symbol of [
    'isCoreSentinel',
    'detectNuxtNitro',
    'isNuxtNitroSentinel',
  ]) {
    assert.equal(Object.hasOwn(classifyPolicies, symbol), false, symbol);
  }
});

check('CPES2. classify-policies does not re-export non-public policy actions', () => {
  for (const symbol of [
    'ACTION_NONE',
    'ACTION_REVIEW_HINT',
  ]) {
    assert.equal(Object.hasOwn(classifyPolicies, symbol), false, symbol);
  }
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
