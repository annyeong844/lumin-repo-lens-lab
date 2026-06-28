import assert from 'node:assert/strict';

const definitionId = await import('../_lib/definition-id.mjs');

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

check('DIES1. definition-id exposes OXC helper, not raw id builder', () => {
  assert.equal(typeof definitionId.definitionIdFromOxcNode, 'function');
  assert.equal(Object.hasOwn(definitionId, 'makeDefinitionId'), false);
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
