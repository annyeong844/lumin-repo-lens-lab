// Tests for `_lib/canon-draft.mjs` naming classifier pure functions — P3-4 Step 1.
//
// Sibling to test-classification-gates.mjs TN1-TN15 (which cover the
// canonical mirror + cross-class interactions). This file drills into
// classifier + normalizer edge cases.

import {
  LOW_INFO_NAMES,
  LOW_INFO_HELPER_NAMES,
} from '../_lib/canon-draft-utils.mjs';
import {
  detectConvention,
  normalizeFileBasename,
  classifyNamingCohort,
  classifyNamingItem,
} from '../_lib/canon-draft-naming.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ═══ detectConvention — exhaustive ═══

{
  // Multi-segment patterns.
  assert('D1. `fooBar` → camelCase', detectConvention('fooBar') === 'camelCase');
  assert('D2. `fooBarBaz` → camelCase', detectConvention('fooBarBaz') === 'camelCase');
  assert('D3. `FooBar` → PascalCase', detectConvention('FooBar') === 'PascalCase');
  assert('D4. `foo-bar` → kebab-case', detectConvention('foo-bar') === 'kebab-case');
  assert('D5. `foo-bar-baz` → kebab-case', detectConvention('foo-bar-baz') === 'kebab-case');
  assert('D6. `foo_bar` → snake_case', detectConvention('foo_bar') === 'snake_case');
  assert('D7. `FOO_BAR` → UPPER_SNAKE', detectConvention('FOO_BAR') === 'UPPER_SNAKE');
  assert('D8. `MAX_RETRY_COUNT` → UPPER_SNAKE', detectConvention('MAX_RETRY_COUNT') === 'UPPER_SNAKE');

  // Single-segment defaults.
  assert('D9. `foo` → camelCase (lowercase default)', detectConvention('foo') === 'camelCase');
  assert('D10. `Foo` → PascalCase', detectConvention('Foo') === 'PascalCase');
  assert('D11. `FOO` → UPPER_SNAKE (bare uppercase)', detectConvention('FOO') === 'UPPER_SNAKE');

  // Mixed patterns.
  assert('D12. `Foo_bar` → mixed', detectConvention('Foo_bar') === 'mixed');
  assert('D13. `foo-Bar` → mixed', detectConvention('foo-Bar') === 'mixed');
  assert('D14. `foo_Bar_baz` → mixed', detectConvention('foo_Bar_baz') === 'mixed');

  // Edge cases.
  assert('D15. empty string → mixed', detectConvention('') === 'mixed');
  assert('D16. non-string → mixed', detectConvention(null) === 'mixed');
  assert('D17. `x` single-char lowercase → camelCase', detectConvention('x') === 'camelCase');
  assert('D18. `X` single-char uppercase → UPPER_SNAKE', detectConvention('X') === 'UPPER_SNAKE');
}

// ═══ normalizeFileBasename — all docs examples + edges ═══

{
  assert('N1. `_lib/canon-draft.mjs` → `canon-draft`',
    normalizeFileBasename('_lib/canon-draft.mjs') === 'canon-draft');
  assert('N2. `src/components/UserCard.tsx` → `UserCard`',
    normalizeFileBasename('src/components/UserCard.tsx') === 'UserCard');
  assert('N3. `tests/user-profile.test.tsx` → `user-profile`',
    normalizeFileBasename('tests/user-profile.test.tsx') === 'user-profile');
  assert('N4. `src/api.d.ts` → `api` (longest-extension first)',
    normalizeFileBasename('src/api.d.ts') === 'api');
  assert('N5. `src/legacy_module.js` → `legacy_module`',
    normalizeFileBasename('src/legacy_module.js') === 'legacy_module');
  assert('N6. `src/FOO.test.mjs` → `FOO`',
    normalizeFileBasename('src/FOO.test.mjs') === 'FOO');
  assert('N7. `src/Comp.stories.tsx` → `Comp`',
    normalizeFileBasename('src/Comp.stories.tsx') === 'Comp');
  assert('N8. `src/a.spec.ts` → `a`',
    normalizeFileBasename('src/a.spec.ts') === 'a');

  // Bare files (no dir).
  assert('N9. `plain.mjs` → `plain`',
    normalizeFileBasename('plain.mjs') === 'plain');
  // Mixed-slash path (Windows).
  assert('N10. `src\\win\\path.mjs` → `path`',
    normalizeFileBasename('src\\win\\path.mjs') === 'path');

  // Edge cases.
  assert('N11. empty string → empty',
    normalizeFileBasename('') === '');
  assert('N12. non-string → empty',
    normalizeFileBasename(null) === '');
  // No recognizable extension: returned as-is.
  assert('N13. `README` → `README`',
    normalizeFileBasename('README') === 'README');
}

// ═══ classifyNamingCohort — effective-size + dominance ═══

{
  // Insufficient-evidence at raw < 3.
  const r1 = classifyNamingCohort({
    cohortId: 'x',
    members: [{ name: 'fooBar' }, { name: 'bazQux' }],
    kind: 'symbol',
    lowInfoExclusions: new Set(),
  });
  assert('C1. raw 2 members → insufficient-evidence',
    r1.label === 'insufficient-evidence' && r1.dominantConvention === null);

  // Dominance ≥ 0.6 → *-dominant.
  const r2 = classifyNamingCohort({
    cohortId: 'x',
    members: [
      { name: 'fooBar' }, { name: 'bazQux' }, { name: 'zipZap' },
      { name: 'barFoo' }, { name: 'FooBar' },
    ],
    kind: 'symbol',
    lowInfoExclusions: new Set(),
  });
  assert('C2. 4 camelCase + 1 PascalCase (dominance 0.8) → camelCase-dominant',
    r2.label === 'camelCase-dominant' && r2.dominantConvention === 'camelCase');
  assert('C2b. consistencyRate === 0.8',
    Math.abs(r2.consistencyRate - 0.8) < 0.001);

  // Below threshold → mixed.
  const r3 = classifyNamingCohort({
    cohortId: 'x',
    members: [
      { name: 'fooBar' }, { name: 'bazQux' },
      { name: 'FooBar' }, { name: 'BazQux' },
      { name: 'foo-bar' },
    ],
    kind: 'symbol',
    lowInfoExclusions: new Set(),
  });
  assert('C3. 2 camelCase + 2 PascalCase + 1 kebab (all 0.4) → mixed-convention',
    r3.label === 'mixed-convention' && r3.dominantConvention === null);

  // File-kind cohort — normalization applied.
  const r4 = classifyNamingCohort({
    cohortId: '_lib',
    members: [
      { name: '_lib/canon-draft.mjs' },
      { name: '_lib/alias-map.mjs' },
      { name: '_lib/extract-ts.mjs' },
    ],
    kind: 'file',
    lowInfoExclusions: new Set(),
  });
  assert('C4. file cohort with kebab basenames → kebab-case-dominant (normalization applied)',
    r4.label === 'kebab-case-dominant' && r4.dominantConvention === 'kebab-case',
    `got=${JSON.stringify(r4)}`);

  // Effective-size: 10 raw, 8 low-info → effective 2 → insufficient.
  const lowInfo = new Set(['get', 'set', 'parse', 'format']);
  const r5 = classifyNamingCohort({
    cohortId: 'x',
    members: [
      { name: 'domainHelper' }, { name: 'otherHelper' },
      { name: 'get' }, { name: 'set' }, { name: 'parse' }, { name: 'format' },
      { name: 'get' }, { name: 'set' }, { name: 'parse' }, { name: 'format' },
    ],
    kind: 'symbol',
    lowInfoExclusions: lowInfo,
  });
  assert('C5. 10 raw with 8 low-info → effective 2 → insufficient-evidence',
    r5.label === 'insufficient-evidence' &&
    r5.effectiveMembers === 2 && r5.totalMembers === 10);

  // All low-info → insufficient.
  const r6 = classifyNamingCohort({
    cohortId: 'x',
    members: [{ name: 'get' }, { name: 'set' }, { name: 'parse' }, { name: 'format' }],
    kind: 'symbol',
    lowInfoExclusions: lowInfo,
  });
  assert('C6. all low-info → insufficient-evidence (effective 0)',
    r6.label === 'insufficient-evidence' && r6.effectiveMembers === 0);

  // Exactly 3 effective members at majority → dominant.
  const r7 = classifyNamingCohort({
    cohortId: 'x',
    members: [
      { name: 'fooBar' }, { name: 'bazQux' }, { name: 'zipZap' },
    ],
    kind: 'symbol',
    lowInfoExclusions: new Set(),
  });
  assert('C7. exactly 3 effective members, all camelCase → camelCase-dominant at 100%',
    r7.label === 'camelCase-dominant' && r7.consistencyRate === 1);

  // `mixed` is the fallback convention bucket, not a canonical dominant
  // convention label. Even at >= 0.6 dominance it must remain
  // mixed-convention so P3 drafts round-trip through the P5 parser.
  const r8 = classifyNamingCohort({
    cohortId: 'x',
    members: [
      { name: 'foo_Bar' }, { name: 'Foo-bar' }, { name: 'foo.Bar' },
      { name: 'bar:Baz' }, { name: 'alphaBeta' },
    ],
    kind: 'symbol',
    lowInfoExclusions: new Set(),
  });
  assert('C8. majority mixed fallback → mixed-convention, never mixed-dominant',
    r8.label === 'mixed-convention' && r8.dominantConvention === null,
    `got=${JSON.stringify(r8)}`);
}

// ═══ classifyNamingItem — Rule 0 low-info priority ═══

{
  // Rule 0 always wins.
  const r1 = classifyNamingItem({
    convention: 'camelCase', dominantConvention: 'camelCase', isLowInfo: true,
  });
  assert('I1. low-info + matches dominant → low-info-excluded (Rule 0 over Rule 2)',
    r1.label === 'low-info-excluded');

  const r2 = classifyNamingItem({
    convention: 'camelCase', dominantConvention: null, isLowInfo: true,
  });
  assert('I2. low-info + no dominant → low-info-excluded (Rule 0 over Rule 1)',
    r2.label === 'low-info-excluded');

  const r3 = classifyNamingItem({
    convention: 'PascalCase', dominantConvention: 'camelCase', isLowInfo: true,
  });
  assert('I3. low-info + differs from dominant → low-info-excluded (Rule 0 over Rule 3)',
    r3.label === 'low-info-excluded');

  // Rule 1 fires when no dominant.
  const r4 = classifyNamingItem({
    convention: 'camelCase', dominantConvention: null, isLowInfo: false,
  });
  assert('I4. not low-info + no dominant → convention-match (Rule 1)',
    r4.label === 'convention-match');

  // Rule 2: matches dominant.
  const r5 = classifyNamingItem({
    convention: 'camelCase', dominantConvention: 'camelCase', isLowInfo: false,
  });
  assert('I5. not low-info + matches dominant → convention-match (Rule 2)',
    r5.label === 'convention-match');

  // Rule 3: differs.
  const r6 = classifyNamingItem({
    convention: 'PascalCase', dominantConvention: 'camelCase', isLowInfo: false,
  });
  assert('I6. not low-info + differs from dominant → convention-outlier (Rule 3)',
    r6.label === 'convention-outlier');
}

// ═══ LOW_INFO sets integration ═══

{
  assert('L1. LOW_INFO_NAMES has Props',
    LOW_INFO_NAMES.includes('Props'));
  assert('L2. LOW_INFO_HELPER_NAMES has get',
    LOW_INFO_HELPER_NAMES.includes('get'));
  const combined = new Set([...LOW_INFO_NAMES, ...LOW_INFO_HELPER_NAMES]);
  assert('L3. combined low-info set is usable',
    combined.has('Props') && combined.has('get'));
}

// ═══ Integration: file cohort dominant extraction with normalization + low-info ═══

{
  // A cohort of _lib/*.mjs files. Multi-segment kebab + one snake_case outlier + one low-info file.
  const members = [
    { name: '_lib/canon-draft.mjs' },
    { name: '_lib/alias-map.mjs' },
    { name: '_lib/extract-ts.mjs' },
    { name: '_lib/resolver-core.mjs' },
    { name: '_lib/legacy_helper.mjs' },   // snake_case outlier
    { name: '_lib/get.mjs' },             // low-info (after normalization = `get`)
  ];
  const r = classifyNamingCohort({
    cohortId: '_lib',
    members,
    kind: 'file',
    lowInfoExclusions: new Set(['get']),
  });
  // Effective 5 (excluding `get`); 4 kebab + 1 snake → dominance 0.8 → kebab-case-dominant.
  assert('X1. realistic file cohort → kebab-case-dominant',
    r.label === 'kebab-case-dominant' && r.effectiveMembers === 5);
  assert('X2. consistencyRate 0.8 (4 of 5)',
    Math.abs(r.consistencyRate - 0.8) < 0.001);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
