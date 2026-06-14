// Regression guard for the generated tests/README.md.
//
// Before 1.9.0, tests/README.md was hand-edited. Four consecutive
// releases drifted some part of it. v1.9.0 made the file a generated
// artifact; v1.9.1 closed the CHANGELOG-count leak; v1.9.2 made the
// test itself fully hermetic — it no longer mutates the real repo's
// README or creates temp files under the real tests/ directory.
// Everything happens in a helper-managed temp repo.
//
// This matters because the previous version would leave the working
// tree dirty if the process was killed mid-test (SIGKILL, parallel
// runner, Ctrl-C). Now the worst that happens is a leftover tmpdir
// under /tmp which is cheap to ignore.
//
// Assertions (T1-T8) verify:
//   T1  --check passes when README is in sync
//   T2  --check exits non-zero when README has drifted
//   T3  drift report points the reader at the fix command
//   T4  generator writes README without error
//   T5  --check passes after regeneration
//   T6  generated README carries the do-not-edit banner
//   T7  generated README does NOT present an authoritative count —
//       guards against drift-prone INJECTED forms only (historical
//       prose from CHANGELOG subjects is intentionally untouched;
//       see CHANGELOG v1.9.2 for the claim-scoping rationale)
//   T8  adding a suite without a description surfaces a Maintainer note
//   T9  pre-write suites have generated README descriptions
//   T10 current suite inventory has generated README descriptions

import { execSync } from 'node:child_process';
import { readFileSync, cpSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createTempRepoFixture } from './_helpers/temp-repo-fixture.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// v1.9.2: full hermeticity. Create a temp copy of the files the
// generator reads and writes. The real repo is never touched. If the
// process dies mid-test, only a /tmp dir leaks — zero impact on the
// working tree.
const FX = createTempRepoFixture({
  prefix: 'fx-test-doc-',
  packageJson: { type: 'module' },
});
const FIXTURE = FX.root;
cpSync(path.join(DIR, 'CHANGELOG.md'), FX.path('CHANGELOG.md'));
cpSync(path.join(DIR, 'scripts'), FX.path('scripts'), { recursive: true });
cpSync(path.join(DIR, 'tests'), FX.path('tests'), { recursive: true });
// package.json is supplied by the fixture helper so the copy looks like a
// regular repo root if anything else is added later.

const FIXTURE_README = path.join(FIXTURE, 'tests/README.md');
const FIXTURE_GEN    = path.join(FIXTURE, 'scripts/update-test-doc.mjs');

function run(cmd) {
  try {
    return { ok: true, out: execSync(cmd, { cwd: FIXTURE, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }) };
  } catch (e) {
    return { ok: false, out: (e.stdout || '') + (e.stderr || '') };
  }
}

try {
  // T1. --check on the (generated) README should pass. The real repo
  // already has an up-to-date README, so the fixture copy does too.
  const r1 = run(`node ${FIXTURE_GEN} --check`);
  assert('T1. --check passes when README is in sync with CHANGELOG+suites',
    r1.ok && r1.out.includes('up to date'),
    r1.out);

  // T2. introduce drift: append garbage to README, --check should fail
  FX.write('tests/README.md', FX.read('tests/README.md') + '\n<!-- injected drift -->\n');
  const r2 = run(`node ${FIXTURE_GEN} --check`);
  assert('T2. --check exits non-zero when README has drifted',
    !r2.ok && r2.out.includes('DRIFT'),
    r2.out);

  // T3. the drift report must indicate how to fix it
  assert('T3. drift report suggests running update-test-doc',
    r2.out.includes('update-test-doc'),
    `report: ${r2.out.slice(0, 300)}`);

  // T4. regenerating writes the correct content back
  const r3 = run(`node ${FIXTURE_GEN}`);
  assert('T4. generator writes README without error',
    r3.ok && r3.out.includes('wrote'),
    r3.out);

  // T5. after regen, --check passes again
  const r4 = run(`node ${FIXTURE_GEN} --check`);
  assert('T5. --check passes after regeneration',
    r4.ok,
    r4.out);

  // T6. generated README carries the DO-NOT-EDIT banner
  const genContent = readFileSync(FIXTURE_README, 'utf8');
  assert('T6. generated README carries explicit do-not-edit marker',
    genContent.includes('GENERATED FILE') && genContent.includes('do not edit'),
    `first 200 chars: ${genContent.slice(0, 200)}`);

  // T7. generated README does NOT carry a hardcoded CURRENT total.
  // Historical prose like "104 test assertions pass" inside a release
  // subject extracted from CHANGELOG is factual record, not drift.
  // The drift-prone forms are: per-release bullet counts we inject
  // ourselves, and totals presented as the authoritative number.
  const countPatterns = [
    /\*\*v\d+\.\d+\.\d+\*\*\s*\(\d+\)/,           // bullet count suffix
    /\*\*\d+\s+assertions\*\*/i,                  // bold assertion count
    /\*\*total:?\s+\d+\*\*/i,                     // bold total
    /\d+\s+assertions\s+across\s+\d+\s+suites/i,  // grand-total prose
  ];
  const leakingPattern = countPatterns.find((p) => p.test(genContent));
  assert('T7. generated README does NOT present an authoritative assertion count',
    leakingPattern === undefined,
    `pattern ${leakingPattern} matched. Snippet: ` +
    (leakingPattern ? JSON.stringify(genContent.match(leakingPattern)?.[0]) : ''));

  // T8. maintainer-note surfacing. Adding a suite without registering
  // its description should show up as a "Maintainer note" section in
  // the generated README. Create the new suite IN THE FIXTURE, not the
  // real repo.
  FX.write('tests/test-z-zz-temp.mjs', 'console.log("dummy — for test-update-test-doc only");\n');
  run(`node ${FIXTURE_GEN}`);
  const afterAdd = readFileSync(FIXTURE_README, 'utf8');
  assert('T8. adding a suite without a description surfaces a Maintainer note',
    afterAdd.includes('## Maintainer note') && afterAdd.includes('test-z-zz-temp.mjs'),
    `README tail after adding suite: ${afterAdd.slice(-500)}`);

  // T9. pre-write suites are the current test-reform focus. They should not
  // remain in the generated README as anonymous entries after the wiki
  // inventory has named their protected invariants.
  const preWriteLines = genContent
    .split('\n')
    .filter((line) => line.includes('tests/test-pre-write-'));
  const missingPreWriteDescriptions = preWriteLines.filter((line) =>
    line.includes('(no description'));
  assert('T9. generated README gives pre-write suites explicit descriptions',
    missingPreWriteDescriptions.length === 0,
    `missing descriptions:\n${missingPreWriteDescriptions.join('\n')}`);

  // T10. The current suite inventory should not have anonymous README rows.
  // T8 still proves the generator surfaces new omissions; this guard keeps the
  // checked-in suite set from accepting leftover maintainer-note entries.
  const missingDescriptionLines = genContent
    .split('\n')
    .filter((line) => line.includes('(no description'));
  assert('T10. generated README gives every current suite an explicit description',
    missingDescriptionLines.length === 0,
    `missing descriptions:\n${missingDescriptionLines.join('\n')}`);
} finally {
  FX.cleanup();
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
