// Regression guard for private-repo GitHub Actions minute usage.
//
// CI must remain available for large changes, but draft PRs opened by
// automation should not allocate runner minutes before the PR is ready.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const workflow = readFileSync(path.join(DIR, '.github', 'workflows', 'ci.yml'), 'utf8');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function sectionAfter(marker) {
  const idx = workflow.indexOf(marker);
  return idx === -1 ? '' : workflow.slice(idx);
}

const pullRequestSection = sectionAfter('pull_request:');
const testJobSection = sectionAfter('  test:');

assert('GHA1. CI can still be started manually',
  /^\s*workflow_dispatch:\s*$/m.test(workflow),
  workflow);

assert('GHA2. pull_request trigger includes ready_for_review',
  /pull_request:\s*\n\s+types:\s*\[[^\]]*ready_for_review[^\]]*\]/m.test(workflow),
  pullRequestSection.slice(0, 240));

assert('GHA3. pull_request trigger includes opened/synchronize/reopened',
  ['opened', 'synchronize', 'reopened'].every((event) =>
    new RegExp(`pull_request:\\s*\\n\\s+types:\\s*\\[[^\\]]*${event}[^\\]]*\\]`, 'm').test(workflow)),
  pullRequestSection.slice(0, 240));

assert('GHA4. test job skips draft pull requests before runner work',
  /if:\s*\$\{\{\s*github\.event_name\s*!=\s*'pull_request'\s*\|\|\s*github\.event\.pull_request\.draft\s*==\s*false\s*\}\}/m.test(workflow),
  testJobSection.slice(0, 320));

assert('GHA5. push to main/master still runs CI',
  /push:\s*\n\s+branches:\s*\[main,\s*master\]/m.test(workflow),
  workflow.slice(0, 160));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
