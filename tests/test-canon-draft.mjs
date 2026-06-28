// Tests for `_lib/canon-draft.mjs` pure helpers — P3-1 Step 1.
//
// Coverage per docs/history/phases/p3/p3-1.md v2 §5.2:
//   - classifyTypeNameGroup: every Rule 0–3 branch + precedence pins
//   - ANY_COLLISION scope pins (P1-2 reviewer fix):
//     has-any only / unknown-surface only / mixed → NOT ANY_COLLISION
//   - classifySingleIdentity: every Rule 0–4 branch
//   - escapeMdCell + codeCell markdown hygiene
//
// Step 0 drift-lock lives in `tests/test-classification-gates.mjs`;
// this file tests the pure-function behavior.

import {
  LOW_INFO_NAMES,
  escapeMdCell,
  codeCell,
} from '../_lib/canon-draft-utils.mjs';
import {
  classifyTypeNameGroup,
  classifySingleIdentity,
} from '../_lib/canon-draft-types.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── LOW_INFO_NAMES contract ─────────────────────────────

assert('LI1. LOW_INFO_NAMES is an array',
  Array.isArray(LOW_INFO_NAMES));
assert('LI2. LOW_INFO_NAMES is frozen (attempt to push throws in strict mode)',
  Object.isFrozen(LOW_INFO_NAMES));
assert('LI3. "Props" in LOW_INFO_NAMES',
  LOW_INFO_NAMES.includes('Props'));
assert('LI4. LOW_INFO_NAMES.length === 16',
  LOW_INFO_NAMES.length === 16);

// ── classifyTypeNameGroup Rule 0 — ANY_COLLISION ───────

{
  const id1 = 'a.ts::Foo', id2 = 'b.ts::Foo';
  const r = classifyTypeNameGroup({
    name: 'Foo',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 20, [id2]: 20 },  // high fan-in should NOT override Rule 0
    contaminationByIdentity: {
      [id1]: { label: 'any-contaminated' },
      [id2]: { label: 'severely-any-contaminated' },
    },
  });
  assert('G-R0a. all-contaminated 2 identities → ANY_COLLISION',
    r.label === 'ANY_COLLISION');
  assert('G-R0b. ANY_COLLISION marker is ⚠',
    r.marker === '⚠');
  assert('G-R0c. high fan-in does NOT override Rule 0',
    r.label === 'ANY_COLLISION');  // deliberate re-pin
}

// ── Rule 0 SCOPE pins (reviewer P1-2) — non-contaminated labels do NOT trigger

{
  // has-any only — mild; NOT Rule 0.
  const id1 = 'a.ts::X', id2 = 'b.ts::X';
  const r = classifyTypeNameGroup({
    name: 'X',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'has-any' },
      [id2]: { label: 'has-any' },
    },
  });
  assert('G-R0-SCOPE1. has-any only (both) → NOT ANY_COLLISION (falls through to Rule 1)',
    r.label === 'DUPLICATE_STRONG');
}

{
  // unknown-surface only — safe-boundary, not contamination. NOT Rule 0.
  const id1 = 'a.ts::Y', id2 = 'b.ts::Y';
  const r = classifyTypeNameGroup({
    name: 'Y',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'unknown-surface' },
      [id2]: { label: 'unknown-surface' },
    },
  });
  assert('G-R0-SCOPE2. unknown-surface only (both) → NOT ANY_COLLISION',
    r.label === 'DUPLICATE_STRONG');
}

{
  // Mixed: one contaminated + one has-any → Rule 0 is universal, not
  // existential. Single non-contaminated member breaks Rule 0.
  const id1 = 'a.ts::Z', id2 = 'b.ts::Z';
  const r = classifyTypeNameGroup({
    name: 'Z',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'any-contaminated' },
      [id2]: { label: 'has-any' },
    },
  });
  assert('G-R0-SCOPE3. mixed (any-contaminated + has-any) → NOT ANY_COLLISION',
    r.label === 'DUPLICATE_STRONG');
}

// ── classifyTypeNameGroup Rule 1 — DUPLICATE_STRONG ───

{
  const id1 = 'a.ts::Bar', id2 = 'b.ts::Bar';
  const r = classifyTypeNameGroup({
    name: 'Bar',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 0 },   // max=5, sum=5 → Rule 1
    contaminationByIdentity: {},
  });
  assert('G-R1a. max≥3 AND sum≥3 → DUPLICATE_STRONG',
    r.label === 'DUPLICATE_STRONG');
  assert('G-R1b. marker is ❌',
    r.marker === '❌');
}

{
  // Precedence pin: `Result` is in LOW_INFO_NAMES but high fan-in →
  // Rule 1 wins over Rule 2. Canonical §3 rationale.
  const id1 = 'a.ts::Result', id2 = 'b.ts::Result';
  const r = classifyTypeNameGroup({
    name: 'Result',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 18, [id2]: 3 },
    contaminationByIdentity: {},
  });
  assert('G-R1c. Rule 1 fires BEFORE Rule 2 — Result+high-fanIn → DUPLICATE_STRONG (not LOCAL_COMMON_NAME)',
    r.label === 'DUPLICATE_STRONG');
}

// ── classifyTypeNameGroup Rule 2 — LOCAL_COMMON_NAME ───

{
  // LOW_INFO_NAMES + max fanIn < 3 → LOCAL_COMMON_NAME.
  const id1 = 'a.ts::Props', id2 = 'b.ts::Props';
  const r = classifyTypeNameGroup({
    name: 'Props',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 1, [id2]: 2 },  // max=2, sum=3 but max<3 → NOT Rule 1
    contaminationByIdentity: {},
  });
  assert('G-R2a. low-info name + max-fanIn<3 → LOCAL_COMMON_NAME',
    r.label === 'LOCAL_COMMON_NAME');
  assert('G-R2b. marker is ⚠',
    r.marker === '⚠');
}

{
  // Every LOW_INFO_NAMES entry is actually recognized.
  const id1 = 'a.ts::Options', id2 = 'b.ts::Options';
  const r = classifyTypeNameGroup({
    name: 'Options',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 0, [id2]: 0 },
    contaminationByIdentity: {},
  });
  assert('G-R2c. "Options" (LOW_INFO_NAMES entry) → LOCAL_COMMON_NAME',
    r.label === 'LOCAL_COMMON_NAME');
}

// ── classifyTypeNameGroup Rule 3 — DUPLICATE_REVIEW ────

{
  // 2 identities, non-low-info, low fan-in → fallback.
  const id1 = 'a.ts::Xyz', id2 = 'b.ts::Xyz';
  const r = classifyTypeNameGroup({
    name: 'Xyz',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 1, [id2]: 1 },
    contaminationByIdentity: {},
  });
  assert('G-R3a. non-low-info name + low fan-in → DUPLICATE_REVIEW',
    r.label === 'DUPLICATE_REVIEW');
  assert('G-R3b. marker is ⚠',
    r.marker === '⚠');
}

// ── classifyTypeNameGroup input validation ──────

{
  let threw = false;
  try { classifyTypeNameGroup({ name: 'Foo', identities: ['a.ts::Foo'], fanInByIdentity: {}, contaminationByIdentity: {} }); }
  catch { threw = true; }
  assert('G-VALID. single-identity input → throws (use classifySingleIdentity)',
    threw);
}

// ── classifySingleIdentity Rule 0 — severely-any-contaminated ────

{
  // Severely-contaminated — wins over any fan-in.
  const r = classifySingleIdentity({
    identity: 'a.ts::Big',
    fanIn: 100,
    kind: 'TSInterfaceDeclaration',
    contamination: { label: 'severely-any-contaminated' },
  });
  assert('S-R0a. severely-any-contaminated → severely-any-contaminated label',
    r.label === 'severely-any-contaminated');
  assert('S-R0b. high fanIn does NOT promote severely-contaminated to single-owner-strong',
    r.label !== 'single-owner-strong');
}

{
  // `any-contaminated` (non-severe) does NOT trigger single-identity Rule 0.
  const r = classifySingleIdentity({
    identity: 'a.ts::Moderate',
    fanIn: 5,
    kind: 'TSInterfaceDeclaration',
    contamination: { label: 'any-contaminated' },
  });
  assert('S-R0c. any-contaminated (non-severe) falls through to fan-in rules',
    r.label === 'single-owner-strong');
}

// ── classifySingleIdentity Rule 1 — low-signal-type-name ──

{
  const r = classifySingleIdentity({
    identity: 'a.ts::T',
    fanIn: 2,
    kind: 'TSTypeAliasDeclaration',
    contamination: null,
  });
  assert('S-R1a. TSTypeAliasDeclaration + 1-char name + fanIn<3 → low-signal-type-name',
    r.label === 'low-signal-type-name');
}

{
  // 1-char alias with fanIn ≥ 3 → Rule 2 (promoted, not low-signal).
  const r = classifySingleIdentity({
    identity: 'a.ts::T',
    fanIn: 5,
    kind: 'TSTypeAliasDeclaration',
    contamination: null,
  });
  assert('S-R1b. TSTypeAliasDeclaration + 1-char name + fanIn≥3 → single-owner-strong (NOT low-signal)',
    r.label === 'single-owner-strong');
}

{
  // 1-char but not TSTypeAliasDeclaration → Rule 1 doesn't fire.
  const r = classifySingleIdentity({
    identity: 'a.ts::X',
    fanIn: 2,
    kind: 'TSInterfaceDeclaration',
    contamination: null,
  });
  assert('S-R1c. 1-char name on interface → NOT low-signal (kind gate)',
    r.label === 'single-owner-weak');
}

// ── classifySingleIdentity Rule 2 / 3 / 4 — fan-in tiers ───

{
  const r = classifySingleIdentity({
    identity: 'a.ts::Long',
    fanIn: 5,
    kind: 'TSInterfaceDeclaration',
    contamination: null,
  });
  assert('S-R2. fanIn≥3 → single-owner-strong',
    r.label === 'single-owner-strong');
  assert('S-R2b. marker is ✅',
    r.marker === '✅');
}

{
  const r = classifySingleIdentity({
    identity: 'a.ts::Weak',
    fanIn: 2,
    kind: 'TSInterfaceDeclaration',
    contamination: null,
  });
  assert('S-R3a. fanIn=2 → single-owner-weak',
    r.label === 'single-owner-weak');
}

{
  const r = classifySingleIdentity({
    identity: 'a.ts::Weak1',
    fanIn: 1,
    kind: 'TSInterfaceDeclaration',
    contamination: null,
  });
  assert('S-R3b. fanIn=1 → single-owner-weak',
    r.label === 'single-owner-weak');
}

{
  const r = classifySingleIdentity({
    identity: 'a.ts::Zero',
    fanIn: 0,
    kind: 'TSInterfaceDeclaration',
    contamination: null,
  });
  assert('S-R4. fanIn=0 → zero-internal-fan-in',
    r.label === 'zero-internal-fan-in');
}

// ── escapeMdCell ───────────────────────────────

assert('E1. escapeMdCell("clean") is idempotent',
  escapeMdCell('clean') === 'clean');
assert('E2. escapeMdCell(null) is empty string',
  escapeMdCell(null) === '');
assert('E3. escapeMdCell("a|b") escapes pipe',
  escapeMdCell('a|b') === 'a\\|b');
assert('E4. escapeMdCell with backslash escapes it first',
  escapeMdCell('a\\b') === 'a\\\\b');
assert('E5. escapeMdCell collapses newline to space',
  escapeMdCell('a\nb') === 'a b');

// ── codeCell ─────────────────────────────────

assert('C1. codeCell("") returns empty string (no `` rendering)',
  codeCell('') === '');
assert('C2. codeCell(null) returns empty string',
  codeCell(null) === '');
assert('C3. codeCell("x") wraps in single backticks',
  codeCell('x') === '`x`');
assert('C4. codeCell with single backtick → double-backtick wrap (CommonMark)',
  codeCell('a`b') === '`` a`b ``');

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
