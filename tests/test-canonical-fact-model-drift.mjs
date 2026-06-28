// Tests for canonical/fact-model.md §3.9 drift — P2-0 step 1.
//
// This test is the forcing function that keeps the `any-inventory.mjs`
// producer and `_lib/extract-ts-escapes.mjs` in lockstep with the
// canonical schema. Drift in either direction fails.
//
// Canonical anchors:
//   - canonical/fact-model.md §3.9 — type-escape shape + escapeKind enum
//   - docs/history/phases/p2/session.md §4.1 — producer emission contract
//   - docs/history/phases/p2/p2-0.md §5.2 — this test's role as Step 1 forcing function

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Canonical authoritative enum ─────────────────────────────
//
// Hardcoded here as the P2-0 pin. The test extracts the list from the
// canonical markdown and asserts equality AND order.

const EXPECTED_ESCAPE_KINDS = [
  'explicit-any',
  'as-any',
  'angle-any',
  'as-unknown-as-T',
  'rest-any-args',
  'index-sig-any',
  'generic-default-any',
  'ts-ignore',
  'ts-expect-error',
  'no-explicit-any-disable',
  'jsdoc-any',
];

const factModelPath = path.join(DIR, 'canonical', 'fact-model.md');
const factModelText = readFileSync(factModelPath, 'utf8');

// ── T1-T3. Extract the escapeKind bullet list from §3.9 ─────

// §3.9 has a canonical "escapeKind is one of:" block with bullets like:
//   - `explicit-any` — description
// Parse those bullets, preserving order.
function extractEscapeKindsFromMarkdown(text) {
  const marker = '`escapeKind` is one of:';
  const idx = text.indexOf(marker);
  if (idx < 0) return null;
  const rest = text.slice(idx);
  // Stop at the next blank line followed by a non-bullet paragraph.
  const lines = rest.split('\n');
  const kinds = [];
  for (let i = 1; i < lines.length; i++) {
    const line = lines[i];
    const m = line.match(/^-\s+`([^`]+)`\s*—/);
    if (m) {
      kinds.push(m[1]);
    } else if (kinds.length > 0 && line.trim().length === 0) {
      // Bullet block ended.
      break;
    }
  }
  return kinds;
}

const parsed = extractEscapeKindsFromMarkdown(factModelText);

assert('T1. canonical §3.9 "escapeKind is one of:" block is present',
  parsed !== null && parsed.length > 0);

assert('T2. canonical §3.9 enumerates exactly 11 escapeKinds',
  parsed && parsed.length === 11,
  `got ${parsed?.length}: ${JSON.stringify(parsed)}`);

assert('T3. canonical §3.9 escapeKinds match expected list AND order',
  parsed && JSON.stringify(parsed) === JSON.stringify(EXPECTED_ESCAPE_KINDS),
  `canonical=${JSON.stringify(parsed)}\n        expected=${JSON.stringify(EXPECTED_ESCAPE_KINDS)}`);

// ── T4-T5. P2-0 amendment fields present ────────────────────
//
// The JSON example in §3.9 must carry occurrenceKey + normalizedCodeShape
// (added in P2-0 amendment 2026-04-20).

assert('T4. canonical §3.9 example carries `occurrenceKey` field',
  /"occurrenceKey"\s*:/.test(factModelText),
  'occurrenceKey missing from the §3.9 type-escape JSON example');

assert('T5. canonical §3.9 example carries `normalizedCodeShape` field',
  /"normalizedCodeShape"\s*:/.test(factModelText),
  'normalizedCodeShape missing from the §3.9 type-escape JSON example');

// ── T6-T7. Documentation for the new fields present ─────────

assert('T6. canonical §3.9 explains `normalizedCodeShape` (token-aware / string-literal preserved)',
  /normalizedCodeShape.*token-aware|token-aware.*normalizedCodeShape/is.test(factModelText) ||
  /normalizedCodeShape[\s\S]{0,400}string.*literal/i.test(factModelText),
  'no explanation of normalizedCodeShape normalization rule');

assert('T7. canonical §3.9 explains `occurrenceKey` hash composition',
  /occurrenceKey[\s\S]{0,400}sha256/i.test(factModelText) &&
  /occurrenceKey[\s\S]{0,600}file.*escapeKind.*normalizedCodeShape.*insideExportedIdentity/is.test(factModelText),
  'occurrenceKey composition (sha256 of file|escapeKind|normalizedCodeShape|insideExportedIdentity) not documented');

// ── T8-T12. Precedence rules present (P2-0 amendment) ───────

const precedencePairs = [
  ['rest-any-args', 'explicit-any'],
  ['index-sig-any', 'explicit-any'],
  ['generic-default-any', 'explicit-any'],
  ['angle-any', 'explicit-any'],
  ['as-unknown-as-T', 'as-any'],
];
for (const [winner, over] of precedencePairs) {
  const re = new RegExp(`\`${winner}\`\\s+wins over\\s+\`${over}\``, 'i');
  assert(`T8-12. canonical §3.9 documents precedence: ${winner} > ${over}`,
    re.test(factModelText),
    `precedence rule "${winner} wins over ${over}" not found`);
}

// ── T13. P2-0 amendment date noted ──────────────────────────

assert('T13. canonical §3.9 marks the P2-0 amendment (2026-04-20)',
  /P2-0 amendment.*2026-04-20/i.test(factModelText),
  'P2-0 amendment date not documented — silent schema changes forbidden');

// ── T14. PLANNED_ESCAPE_KEYS drift pin (D-3 fix, 2026-04-21) ────
//
// The intent validator's normalized plannedTypeEscapes shape must match
// what post-write-delta reads. A new field added to canonical §3.9 but
// not to PLANNED_ESCAPE_KEYS would silently drop at validator
// normalization, leaving post-write-delta with `plannedEntry.newField
// === undefined`. This test locks the expected key set.

{
  const { PLANNED_ESCAPE_KEYS, PLANNED_ESCAPE_ALL_KEYS } =
    await import('../_lib/pre-write-intent.mjs');

  assert('T14a. PLANNED_ESCAPE_KEYS.required = escapeKind, locationHint, reason',
    JSON.stringify([...PLANNED_ESCAPE_KEYS.required]) ===
    JSON.stringify(['escapeKind', 'locationHint', 'reason']));

  assert('T14b. PLANNED_ESCAPE_KEYS.optional = codeShape, alternativeConsidered',
    JSON.stringify([...PLANNED_ESCAPE_KEYS.optional]) ===
    JSON.stringify(['codeShape', 'alternativeConsidered']));

  assert('T14c. PLANNED_ESCAPE_ALL_KEYS union has exactly 5 entries',
    PLANNED_ESCAPE_ALL_KEYS.length === 5);

  // Cross-module drift pin: validator normalizes to PLANNED_ESCAPE_KEYS,
  // post-write-delta reads named fields directly. Assert the validator's
  // normalized output shape matches the constant exactly — a key added
  // to the constant but missed in the normalization loop surfaces here
  // (validator would leave it out → delta sees undefined).
  const { validateIntent } = await import('../_lib/pre-write-intent.mjs');
  const sample = {
    names: [], shapes: [], files: [], dependencies: [],
    plannedTypeEscapes: [{
      escapeKind: 'as-any',
      locationHint: 'src/x.ts::foo',
      reason: 'test',
      codeShape: 'x as any',
      alternativeConsidered: 'narrow',
    }],
  };
  const validated = validateIntent(sample);
  assert('T14d. validator normalization surfaces every PLANNED_ESCAPE_KEYS key on the full-fixture entry',
    validated.ok === true &&
    PLANNED_ESCAPE_ALL_KEYS.every((k) => k in validated.intent.plannedTypeEscapes[0]));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
