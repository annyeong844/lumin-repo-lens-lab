// Release-blocking P1-3 integration test — docs/history/phases/p1/p1-3.md §5.6.
//
// One multi-source fixture exercises all four intent lookup paths (name,
// file, dependency, shape), plus canonical drift rendering, plus the
// advisory artifact round-trip.
//
// Thirteen assertions. FP-41 compound-component regression is NOT
// covered here — docs/history/phases/p1/p1-3.md §7 non-goals makes this explicit. FP-41
// pinning lives in tests/test-classify-facts-ast.mjs + test-corpus.mjs.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const PREWRITE = path.join(DIR, 'pre-write.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

// ── Fixture construction ─────────────────────────────────────
//
// - src/utils/date.ts::formatDate — exists, consumed by app.tsx.
// - src/types/gone.ts is DELIBERATELY ABSENT even though the canonical
//   file declares `GoneType` as owned by it. This forces one drift
//   entry (kind: 'ast-absent').

const fx = mkdtempSync(path.join(tmpdir(), 'pw-integration-'));
const out = mkdtempSync(path.join(tmpdir(), 'pw-integration-out-'));

try {
  write(fx, 'package.json', JSON.stringify({ name: 'pw-integration', type: 'module' }));
  write(fx, 'src/utils/date.ts', 'export const formatDate = (d) => d.toString();\n');
  write(fx, 'src/app.tsx',
    "import { formatDate } from './utils/date';\n" +
    "export const App = () => formatDate(new Date());\n");

  // Canonical file with the generated-canon header. Declares one owner
  // table with a single row for GoneType — whose declared ownerFile
  // does NOT exist in this fixture.
  write(fx, 'canonical/type-ownership.md',
    '# canonical/type-ownership.md — DRAFT\n' +
    '\n' +
    '> **Status:** draft, v1\n' +
    '> **Generated:** 2026-04-20T00:00:00Z\n' +
    '\n' +
    '### 2.1 Single owner (strong)\n' +
    '\n' +
    '| Type | Owner | Kind | Line | Fan-in | Status |\n' +
    '|---|---|---|---|---:|---|\n' +
    '| `GoneType` | `src/types/gone.ts` | TSTypeAliasDeclaration | 7 | 0 | ✅ |\n');

  const intent = {
    names: ['GoneType', 'formatDate', 'formatTimestamp'],
    shapes: [{ fields: ['year', 'month'] }],
    files: ['src/utils/new-helper.ts'],
    dependencies: ['dayjs'],
    plannedTypeEscapes: [{
      escapeKind: 'as-any',
      locationHint: 'src/x.ts::fn',
      reason: 'integration test',
    }],
  };
  const intentPath = path.join(out, 'intent.json');
  writeFileSync(intentPath, JSON.stringify(intent));

  // ── Run pre-write (default = cold-cache enabled) ───────────

  const stdout = execFileSync(NODE, [
    PREWRITE, '--root', fx, '--output', out, '--intent', intentPath,
  ], { stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8' });

  // ── 13 assertions per docs/history/phases/p1/p1-3.md §5.6 ─────────────────────

  // 1. Exit code 0 (execFileSync would have thrown on non-zero).
  assert('I1. exit code 0', stdout.length > 0);

  // 2. All sections present.
  const expectedSections = [
    '### Grounded facts',
    '### Agent review cues',
    '### Unavailable evidence',
    '### Already exists (reuse candidates)',
    '### New code candidates',
    '### Canonical drift',
    '### Planned type escapes (from Step 2 intent)',
  ];
  for (const section of expectedSections) {
    assert(`I2. section present — ${section.replace(/^### /, '')}`,
      stdout.includes(section),
      `stdout slice: ${stdout.slice(stdout.indexOf('### '), stdout.indexOf('### ') + 200)}`);
  }

  // 3. formatDate → EXISTS with grounded fan-in 1.
  assert('I3. formatDate renders as EXISTS with src/utils/date.ts',
    stdout.includes('EXISTS') && stdout.includes('src/utils/date.ts::formatDate'));

  // 4. formatTimestamp → near-name review cue pointing at formatDate.
  const reviewCueSection = stdout.slice(stdout.indexOf('### Agent review cues'));
  assert('I4. formatTimestamp produces an agent review cue for formatDate',
    reviewCueSection.includes('near exported name') && reviewCueSection.includes('formatDate'));

  // 5. GoneType → CANONICAL_EXISTS_AST_ABSENT under Already exists (no literal DRIFT).
  const alreadyExistsSection = stdout.slice(
    stdout.indexOf('### Already exists (reuse candidates)'),
    stdout.indexOf('###', stdout.indexOf('### Already exists (reuse candidates)') + 5),
  );
  assert('I5. GoneType appears under Already exists with CANONICAL_EXISTS_AST_ABSENT',
    alreadyExistsSection.includes('GoneType') &&
    alreadyExistsSection.includes('CANONICAL_EXISTS_AST_ABSENT'));
  assert('I5b. Already exists section does NOT contain the CANONICAL DRIFT: literal',
    !alreadyExistsSection.includes('CANONICAL DRIFT:'));

  // 6. Canonical drift section has exactly one CANONICAL DRIFT: for GoneType.
  const driftSection = stdout.slice(stdout.indexOf('### Canonical drift'));
  const driftMatches = driftSection.match(/CANONICAL DRIFT:/g) || [];
  assert('I6. Canonical drift section has one CANONICAL DRIFT: entry',
    driftMatches.length === 1);
  assert('I6b. drift entry references GoneType',
    driftSection.includes('GoneType'));

  // 7. Shape {year, month} → Unavailable evidence with shape-hash + P4.
  const unavailableSection = stdout.slice(stdout.indexOf('### Unavailable evidence'));
  assert('I7. Unavailable evidence section mentions shape-hash + P4',
    unavailableSection.includes('shape-hash') && unavailableSection.includes('P4'));

  // 8. src/utils/new-helper.ts → NEW_FILE (requires topology.meta.complete from cold-cache).
  const newCodeSection = stdout.slice(stdout.indexOf('### New code candidates'));
  assert('I8. New code candidates has NEW_FILE for src/utils/new-helper.ts',
    newCodeSection.includes('NEW_FILE') && newCodeSection.includes('src/utils/new-helper.ts'));

  // 9. dayjs → NEW_PACKAGE under New code candidates.
  assert('I9. dayjs renders as NEW_PACKAGE',
    newCodeSection.includes('NEW_PACKAGE') && newCodeSection.includes('dayjs'));

  // 10. Planned escape as-any row with reason.
  const plannedSection = stdout.slice(stdout.indexOf('### Planned type escapes'));
  assert('I10. planned escape as-any + reason renders',
    plannedSection.includes('as-any') && plannedSection.includes('integration test'));

  // 11. lookups[] name-first ordering in JSON artifact.
  const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
  const kinds = parsed.lookups.map((l) => l.kind);
  const nameIndex = kinds.indexOf('name');
  const fileIndex = kinds.indexOf('file');
  const depIndex = kinds.indexOf('dependency');
  const shapeIndex = kinds.indexOf('shape');
  assert('I11. lookups ordering: names before files, files before deps, deps before shapes',
    nameIndex < fileIndex && fileIndex < depIndex && depIndex < shapeIndex,
    `indices: name=${nameIndex}, file=${fileIndex}, dep=${depIndex}, shape=${shapeIndex}`);

  // 12. advisory.drift.length === 1 and references GoneType.
  assert('I12. advisory.drift has exactly 1 entry for GoneType',
    parsed.drift?.length === 1 && parsed.drift[0].intentName === 'GoneType');

  // 13. capabilities.identityFanIn === true.
  assert('I13. capabilities.identityFanIn === true (cold-cache produced symbols)',
    parsed.capabilities?.identityFanIn === true);

} finally {
  rmSync(fx, { recursive: true, force: true });
  rmSync(out, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
