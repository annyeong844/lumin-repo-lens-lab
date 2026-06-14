import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

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

check('HD1. hooks manifest declares Phase 1 runner events', () => {
  const manifest = JSON.parse(readFileSync(path.join(ROOT, 'hooks', 'hooks.json'), 'utf8'));
  assert.equal(typeof manifest.hooks, 'object');
  assert.deepEqual(Object.keys(manifest.hooks).sort(), [
    'PostToolBatch',
    'PreToolUse',
    'Stop',
    'UserPromptSubmit',
  ]);
});

check('HD2. hook doctor exits 0 and reports root plus manifest status', () => {
  const out = execFileSync('node', ['scripts/hook-doctor.mjs'], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.match(out, /hook doctor/i);
  assert.match(out, /workspaceRoot:/);
  assert.match(out, /auditRoot:/);
  assert.match(out, /hooks[\\/]hooks\.json/);
  assert.match(out, /activeHookEvents: 4/);
  assert.match(out, /events: PostToolBatch, PreToolUse, Stop, UserPromptSubmit/);
  assert.match(out, /preimageStore:/);
  assert.match(out, /eventStore:/);
});

if (failed) {
  console.error(`\n${passed} passed, ${failed} failed`);
  process.exit(1);
}

console.log(`\n${passed} passed, ${failed} failed`);
