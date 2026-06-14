// Tests for `_lib/canon-draft.mjs` helper classifier pure functions — P3-2 Step 1.
//
// Covers canonical/classification-gates.md §10.1 (group) + §10.2 (single
// identity) exhaustively. This file is sibling to `test-canon-draft.mjs`
// (which covers type classification); helper coverage is separate so
// each P3 sub-phase has a clear test file.
//
// Pinning rules from docs/history/phases/p3/p3-2.md v2 §5.2:
//   - Group: Rule 0 (ANY_COLLISION_HELPER) / 1 (HELPER_DUPLICATE_STRONG) /
//     2 (HELPER_LOCAL_COMMON) / 3 (HELPER_DUPLICATE_REVIEW).
//   - Group Rule 0 scope: universal quantifier, not existential.
//   - Group precedence: DUPLICATE_STRONG wins over LOCAL_COMMON at fanIn ≥ 3
//     even for low-info helper names.
//   - Single: Rule 0 (severe) / 1 (low-signal) / 2 (central) / 3 (shared) /
//     4 (zero-internal-fan-in).
//   - Single precedence: low-signal fires ONLY when fanIn < 3; at threshold
//     central-helper wins.
//   - Contamination-unavailability: undefined contamination → Rule 0 unreachable.

import { LOW_INFO_HELPER_NAMES } from '../_lib/canon-draft-utils.mjs';
import {
  classifyHelperGroup,
  classifyHelperIdentity,
} from '../_lib/canon-draft-helpers.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ═══ GROUP CLASSIFIER — Rule 0 ANY_COLLISION_HELPER ═══

{
  const id1 = 'a.ts::foo', id2 = 'b.ts::foo';

  // G-R0-a. All severely-any-contaminated → ANY_COLLISION_HELPER
  const r1 = classifyHelperGroup({
    name: 'foo',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'severely-any-contaminated' },
      [id2]: { label: 'severely-any-contaminated' },
    },
  });
  assert('G-R0-a. all severely-contaminated → ANY_COLLISION_HELPER',
    r1.label === 'ANY_COLLISION_HELPER', `got=${r1.label}`);

  // G-R0-b. Mixed any-contaminated + severely-any-contaminated → ANY_COLLISION_HELPER
  const r2 = classifyHelperGroup({
    name: 'foo',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'any-contaminated' },
      [id2]: { label: 'severely-any-contaminated' },
    },
  });
  assert('G-R0-b. any-contaminated + severely-contaminated → ANY_COLLISION_HELPER',
    r2.label === 'ANY_COLLISION_HELPER', `got=${r2.label}`);

  // G-R0-marker. Marker is a warning glyph
  assert('G-R0-marker. ANY_COLLISION_HELPER marker contains ⚠',
    r1.marker.includes('⚠'), `got=${r1.marker}`);
}

// ═══ GROUP CLASSIFIER — Rule 0 scope (universal, NOT existential) ═══

{
  const id1 = 'a.ts::fetch', id2 = 'b.ts::fetch';

  // G-R0-SCOPE1. has-any only → NOT Rule 0
  const r1 = classifyHelperGroup({
    name: 'fetch',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'has-any' },
      [id2]: { label: 'has-any' },
    },
  });
  assert('G-R0-SCOPE1. has-any-only → NOT ANY_COLLISION_HELPER',
    r1.label !== 'ANY_COLLISION_HELPER', `got=${r1.label}`);

  // G-R0-SCOPE2. unknown-surface only → NOT Rule 0
  const r2 = classifyHelperGroup({
    name: 'fetch',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'unknown-surface' },
      [id2]: { label: 'unknown-surface' },
    },
  });
  assert('G-R0-SCOPE2. unknown-surface-only → NOT ANY_COLLISION_HELPER',
    r2.label !== 'ANY_COLLISION_HELPER', `got=${r2.label}`);

  // G-R0-SCOPE3. One severe + one clean → NOT Rule 0 (universal, not existential)
  const r3 = classifyHelperGroup({
    name: 'fetch',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {
      [id1]: { label: 'severely-any-contaminated' },
    },
  });
  assert('G-R0-SCOPE3. one severe + one clean → NOT ANY_COLLISION_HELPER',
    r3.label !== 'ANY_COLLISION_HELPER', `got=${r3.label}`);

  // G-R0-SCOPE4. Empty contaminationByIdentity → NOT Rule 0
  const r4 = classifyHelperGroup({
    name: 'fetch',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 5 },
    contaminationByIdentity: {},
  });
  assert('G-R0-SCOPE4. empty contamination map → NOT ANY_COLLISION_HELPER',
    r4.label !== 'ANY_COLLISION_HELPER', `got=${r4.label}`);
}

// ═══ GROUP CLASSIFIER — Rule 1 HELPER_DUPLICATE_STRONG ═══

{
  const id1 = 'a.ts::renderThing', id2 = 'b.ts::renderThing';

  // G-R1-a. maxFanIn=5, sumFanIn=5 (one helper heavily used)
  const r1 = classifyHelperGroup({
    name: 'renderThing',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 5, [id2]: 0 },
    contaminationByIdentity: {},
  });
  assert('G-R1-a. maxFanIn≥3, sumFanIn≥3 → HELPER_DUPLICATE_STRONG',
    r1.label === 'HELPER_DUPLICATE_STRONG', `got=${r1.label}`);

  // G-R1-b. maxFanIn=3, sumFanIn=5 (both shared)
  const r2 = classifyHelperGroup({
    name: 'renderThing',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 3, [id2]: 2 },
    contaminationByIdentity: {},
  });
  assert('G-R1-b. max=3, sum=5 → HELPER_DUPLICATE_STRONG',
    r2.label === 'HELPER_DUPLICATE_STRONG', `got=${r2.label}`);

  // G-R1-marker. Needs ❌
  assert('G-R1-marker. HELPER_DUPLICATE_STRONG marker is ❌',
    r1.marker === '❌', `got=${r1.marker}`);
}

// ═══ GROUP CLASSIFIER — Rule 1 wins over Rule 2 (low-info + high fanIn) ═══

{
  const id1 = 'a.ts::parse', id2 = 'b.ts::parse';

  // G-R1-vs-R2. `parse` ∈ LOW_INFO_HELPER_NAMES + fanIn ≥ 3
  // Rule 1 must fire BEFORE Rule 2; LOW_INFO label-set must not absorb heavily-used helpers.
  const r = classifyHelperGroup({
    name: 'parse',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 4, [id2]: 3 },
    contaminationByIdentity: {},
  });
  assert('G-R1-vs-R2. `parse` (low-info) + maxFanIn=4 → HELPER_DUPLICATE_STRONG (Rule 1 wins)',
    r.label === 'HELPER_DUPLICATE_STRONG', `got=${r.label}`);
}

// ═══ GROUP CLASSIFIER — Rule 2 HELPER_LOCAL_COMMON ═══

{
  const id1 = 'a.ts::get', id2 = 'b.ts::get';

  // G-R2-a. name ∈ LOW_INFO_HELPER_NAMES, low fanIn
  const r1 = classifyHelperGroup({
    name: 'get',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 1, [id2]: 2 },
    contaminationByIdentity: {},
  });
  assert('G-R2-a. `get` (low-info) + max fanIn=2 → HELPER_LOCAL_COMMON',
    r1.label === 'HELPER_LOCAL_COMMON', `got=${r1.label}`);

  // G-R2-b. `format` with zero fanIn on both
  const r2 = classifyHelperGroup({
    name: 'format',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 0, [id2]: 0 },
    contaminationByIdentity: {},
  });
  assert('G-R2-b. `format` (low-info) + fanIn 0 → HELPER_LOCAL_COMMON',
    r2.label === 'HELPER_LOCAL_COMMON', `got=${r2.label}`);
}

// ═══ GROUP CLASSIFIER — Rule 3 HELPER_DUPLICATE_REVIEW ═══

{
  const id1 = 'a.ts::unusualHelper', id2 = 'b.ts::unusualHelper';

  // G-R3-a. Non-low-info name + low fanIn
  const r1 = classifyHelperGroup({
    name: 'unusualHelper',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 1, [id2]: 2 },
    contaminationByIdentity: {},
  });
  assert('G-R3-a. non-low-info + low fanIn → HELPER_DUPLICATE_REVIEW',
    r1.label === 'HELPER_DUPLICATE_REVIEW', `got=${r1.label}`);

  // G-R3-b. Non-low-info name + maxFanIn=2, sumFanIn=4 — still Rule 3 because maxFanIn < 3
  const r2 = classifyHelperGroup({
    name: 'validateThing',
    identities: [id1, id2],
    fanInByIdentity: { [id1]: 2, [id2]: 2 },
    contaminationByIdentity: {},
  });
  assert('G-R3-b. max=2, sum=4 (non-low-info) → HELPER_DUPLICATE_REVIEW (Rule 1 needs maxFanIn ≥ 3)',
    r2.label === 'HELPER_DUPLICATE_REVIEW', `got=${r2.label}`);
}

// ═══ GROUP CLASSIFIER — Edge case: size 1 throws ═══

{
  let threw = false;
  try {
    classifyHelperGroup({
      name: 'foo',
      identities: ['a.ts::foo'],
      fanInByIdentity: { 'a.ts::foo': 5 },
      contaminationByIdentity: {},
    });
  } catch (_e) { threw = true; }
  assert('G-edge-size1. classifyHelperGroup with identities.length=1 throws',
    threw);
}

// ═══ SINGLE CLASSIFIER — Rule 0 severely-any-contaminated-helper ═══

{
  const id = 'a.ts::legacyHelper';

  // S-R0-a. Severe wins over high fanIn
  const r1 = classifyHelperIdentity({
    identity: id, fanIn: 10, contamination: { label: 'severely-any-contaminated' },
    exportedName: 'legacyHelper',
  });
  assert('S-R0-a. severe + fanIn=10 → severely-any-contaminated-helper',
    r1.label === 'severely-any-contaminated-helper', `got=${r1.label}`);

  // S-R0-b. Severe wins over low-info name
  const r2 = classifyHelperIdentity({
    identity: 'a.ts::get', fanIn: 1, contamination: { label: 'severely-any-contaminated' },
    exportedName: 'get',
  });
  assert('S-R0-b. severe + name=get → severely-any-contaminated-helper (Rule 0 before Rule 1)',
    r2.label === 'severely-any-contaminated-helper', `got=${r2.label}`);
}

// ═══ SINGLE CLASSIFIER — Rule 1 low-signal-helper-name ═══

{
  // S-R1-a. `get` + fanIn 0
  const r1 = classifyHelperIdentity({
    identity: 'a.ts::get', fanIn: 0, contamination: null, exportedName: 'get',
  });
  assert('S-R1-a. `get` + fanIn=0 → low-signal-helper-name',
    r1.label === 'low-signal-helper-name', `got=${r1.label}`);

  // S-R1-b. `parse` + fanIn 2
  const r2 = classifyHelperIdentity({
    identity: 'a.ts::parse', fanIn: 2, contamination: null, exportedName: 'parse',
  });
  assert('S-R1-b. `parse` + fanIn=2 → low-signal-helper-name',
    r2.label === 'low-signal-helper-name', `got=${r2.label}`);

  // S-R1-c. Non-low-info name + fanIn 2 → NOT low-signal
  const r3 = classifyHelperIdentity({
    identity: 'a.ts::doThing', fanIn: 2, contamination: null, exportedName: 'doThing',
  });
  assert('S-R1-c. non-low-info + fanIn=2 → NOT low-signal-helper-name',
    r3.label !== 'low-signal-helper-name', `got=${r3.label}`);
}

// ═══ SINGLE CLASSIFIER — Rule 2 central-helper ═══

{
  // S-R2-a. Non-low-info + fanIn=3
  const r1 = classifyHelperIdentity({
    identity: 'a.ts::renderThing', fanIn: 3, contamination: null,
    exportedName: 'renderThing',
  });
  assert('S-R2-a. fanIn=3 (non-low-info) → central-helper',
    r1.label === 'central-helper', `got=${r1.label}`);

  // S-R2-b. Low-info name + fanIn=3 → central-helper (Rule 2 wins over Rule 1)
  const r2 = classifyHelperIdentity({
    identity: 'a.ts::get', fanIn: 3, contamination: null, exportedName: 'get',
  });
  assert('S-R2-b. `get` + fanIn=3 → central-helper (Rule 2 at threshold beats Rule 1)',
    r2.label === 'central-helper', `got=${r2.label}`);

  // S-R2-c. High fanIn marker
  assert('S-R2-marker. central-helper marker is ✅',
    r1.marker === '✅', `got=${r1.marker}`);
}

// ═══ SINGLE CLASSIFIER — Rule 3 shared-helper ═══

{
  // S-R3-a. fanIn=1
  const r1 = classifyHelperIdentity({
    identity: 'a.ts::renderThing', fanIn: 1, contamination: null,
    exportedName: 'renderThing',
  });
  assert('S-R3-a. non-low-info + fanIn=1 → shared-helper',
    r1.label === 'shared-helper', `got=${r1.label}`);

  // S-R3-b. fanIn=2
  const r2 = classifyHelperIdentity({
    identity: 'a.ts::renderThing', fanIn: 2, contamination: null,
    exportedName: 'renderThing',
  });
  assert('S-R3-b. non-low-info + fanIn=2 → shared-helper',
    r2.label === 'shared-helper', `got=${r2.label}`);
}

// ═══ SINGLE CLASSIFIER — Rule 4 zero-internal-fan-in-helper ═══

{
  const r = classifyHelperIdentity({
    identity: 'a.ts::orphanHelper', fanIn: 0, contamination: null,
    exportedName: 'orphanHelper',
  });
  assert('S-R4. non-low-info + fanIn=0 → zero-internal-fan-in-helper',
    r.label === 'zero-internal-fan-in-helper', `got=${r.label}`);
}

// ═══ SINGLE CLASSIFIER — exportedName fallback from identity ═══

{
  // When exportedName is not supplied, derive from identity tail.
  // `a.ts::get` + fanIn 2 should still fire Rule 1.
  const r = classifyHelperIdentity({
    identity: 'src/util.ts::get',
    fanIn: 2,
    contamination: null,
    // exportedName omitted
  });
  assert('S-fallback. exportedName derived from identity tail → Rule 1 fires for `get`',
    r.label === 'low-signal-helper-name', `got=${r.label}`);
}

// ═══ LOW_INFO_HELPER_NAMES — sanity and coverage ═══

assert('CONST. LOW_INFO_HELPER_NAMES has 15 entries',
  LOW_INFO_HELPER_NAMES.length === 15, `got ${LOW_INFO_HELPER_NAMES.length}`);

assert('CONST. LOW_INFO_HELPER_NAMES is frozen',
  Object.isFrozen(LOW_INFO_HELPER_NAMES));

// Spot-check membership — `get`, `parse`, `format` IN; `render`, `compile` OUT
assert('CONST. `get` ∈ LOW_INFO_HELPER_NAMES',
  LOW_INFO_HELPER_NAMES.includes('get'));
assert('CONST. `render` ∉ LOW_INFO_HELPER_NAMES',
  !LOW_INFO_HELPER_NAMES.includes('render'));

// ═══ Contamination-unavailability sweep — broader than H17 ═══

{
  // Re-cover H17's sweep with a different shape — this is for visibility in
  // the helper-specific test file, not just test-classification-gates.
  let severeCount = 0, collisionCount = 0;
  for (const name of ['handler', 'foo', 'parse', 'x', 'makeLogger']) {
    for (const fi of [0, 1, 2, 3, 8]) {
      const r = classifyHelperIdentity({
        identity: `a.ts::${name}`, fanIn: fi, contamination: undefined,
        exportedName: name,
      });
      if (r.label === 'severely-any-contaminated-helper') severeCount++;
    }
    for (const [f1, f2] of [[5, 5], [1, 1], [3, 0]]) {
      const r = classifyHelperGroup({
        name,
        identities: [`a.ts::${name}`, `b.ts::${name}`],
        fanInByIdentity: { [`a.ts::${name}`]: f1, [`b.ts::${name}`]: f2 },
        contaminationByIdentity: {},
      });
      if (r.label === 'ANY_COLLISION_HELPER') collisionCount++;
    }
  }
  assert('C-SWEEP-1. fresh-AST mode: severely-any-contaminated-helper never fires (25 runs)',
    severeCount === 0, `saw ${severeCount}`);
  assert('C-SWEEP-2. fresh-AST mode: ANY_COLLISION_HELPER never fires (15 runs)',
    collisionCount === 0, `saw ${collisionCount}`);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
