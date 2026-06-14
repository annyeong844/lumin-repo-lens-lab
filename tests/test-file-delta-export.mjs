import assert from 'node:assert/strict';

const fileDelta = await import('../_lib/post-write-file-delta.mjs');

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

check('FDES1. file-delta exposes delta APIs, not path normalizer internals', () => {
  assert.equal(typeof fileDelta.computeFileDelta, 'function');
  assert.equal(typeof fileDelta.repoRelativeFileList, 'function');
  assert.equal(Object.hasOwn(fileDelta, 'normalizeRepoRelativePath'), false);
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
