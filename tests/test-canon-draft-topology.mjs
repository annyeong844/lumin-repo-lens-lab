// Tests for `_lib/canon-draft.mjs` topology classifier pure functions — P3-3 Step 1.
//
// Covers canonical/classification-gates.md §11.1 (submodule), §11.2 (SCC),
// §11.3 (oversize file) exhaustively. Sibling to `test-canon-draft.mjs`
// (types) and `test-canon-draft-helpers.mjs` (helpers).
//
// Pinning rules from docs/history/phases/p3/p3-3.md v3 §5.2 + canonical §11:
//   - Rule 0 cyclic-submodule wins over high fan-in / centrality.
//   - Rule 1 isolated-submodule requires crossEdgeSource === 'full-list'
//     (PF-6 degraded-mode guard).
//   - Rule 2 shared-submodule at inDegree ≥ 5.
//   - Rule 3 leaf-submodule when outDegree > inDegree AND inDegree < 5.
//   - Rule 4 scoped-submodule fallback.
//   - SCC classifier v1 is constant `forbidden-cycle`.
//   - File classifier: < 400 → null; 400-999 → oversize; ≥ 1000 → extreme-oversize.

import {
  TOPOLOGY_LABELS,
  TOPOLOGY_UNCERTAIN_REASONS,
} from '../_lib/canon-draft-utils.mjs';
import {
  classifyTopologySubmodule,
  classifyTopologyScc,
  classifyTopologyFile,
} from '../_lib/canon-draft-topology.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ═══ SUBMODULE CLASSIFIER — Rule 0 cyclic-submodule ═══

{
  // S-R0-a. SCC + high in-degree → cyclic (Rule 0 wins over Rule 2 shared)
  const r1 = classifyTopologySubmodule({
    name: 'core', inDegree: 20, outDegree: 5, sccMember: true, crossEdgeSource: 'full-list',
  });
  assert('S-R0-a. SCC + high in-degree → cyclic-submodule (Rule 0 wins over shared)',
    r1.label === 'cyclic-submodule', `got=${r1.label}`);

  // S-R0-b. SCC + zero in/out → cyclic (Rule 0 wins over Rule 1 isolated)
  const r2 = classifyTopologySubmodule({
    name: 'orphan', inDegree: 0, outDegree: 0, sccMember: true, crossEdgeSource: 'full-list',
  });
  assert('S-R0-b. SCC + zero in/out → cyclic-submodule (Rule 0 wins over isolated)',
    r2.label === 'cyclic-submodule', `got=${r2.label}`);

  // S-R0-c. SCC + outDegree > inDegree → cyclic (Rule 0 wins over Rule 3 leaf)
  const r3 = classifyTopologySubmodule({
    name: 'leafy', inDegree: 1, outDegree: 10, sccMember: true, crossEdgeSource: 'full-list',
  });
  assert('S-R0-c. SCC + leaf pattern → cyclic-submodule (Rule 0 wins over leaf)',
    r3.label === 'cyclic-submodule', `got=${r3.label}`);

  assert('S-R0-marker. cyclic-submodule marker is ❌',
    r1.marker === '❌', `got=${r1.marker}`);
}

// ═══ SUBMODULE CLASSIFIER — Rule 1 isolated-submodule + degraded-mode guard ═══

{
  // S-R1-full. full-list mode + zero in/out → isolated
  const rFull = classifyTopologySubmodule({
    name: 'x', inDegree: 0, outDegree: 0, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R1-full. full-list + zero in/out → isolated-submodule',
    rFull.label === 'isolated-submodule', `got=${rFull.label}`);

  // S-R1-degraded. top-30-only mode + zero in/out → scoped (Rule 1 suppressed)
  const rDeg = classifyTopologySubmodule({
    name: 'x', inDegree: 0, outDegree: 0, sccMember: false, crossEdgeSource: 'top-30-only',
  });
  assert('S-R1-degraded. top-30-only + zero in/out → scoped-submodule (Rule 1 suppressed per §11.1)',
    rDeg.label === 'scoped-submodule', `got=${rDeg.label}`);

  // S-R1-partial. Any non-zero → Rule 1 doesn't fire even in full-list
  const rIn = classifyTopologySubmodule({
    name: 'x', inDegree: 1, outDegree: 0, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R1-partial-in. inDegree=1, outDegree=0 → NOT isolated (falls through)',
    rIn.label !== 'isolated-submodule');

  const rOut = classifyTopologySubmodule({
    name: 'x', inDegree: 0, outDegree: 1, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R1-partial-out. inDegree=0, outDegree=1 → NOT isolated (falls through)',
    rOut.label !== 'isolated-submodule');

  assert('S-R1-marker. isolated-submodule marker is ℹ',
    rFull.marker === 'ℹ', `got=${rFull.marker}`);
}

// ═══ SUBMODULE CLASSIFIER — Rule 2 shared-submodule ═══

{
  // S-R2-a. inDegree === 5 → shared (exact threshold)
  const r1 = classifyTopologySubmodule({
    name: '_lib', inDegree: 5, outDegree: 2, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R2-a. inDegree=5 (threshold) → shared-submodule',
    r1.label === 'shared-submodule', `got=${r1.label}`);

  // S-R2-b. inDegree === 20 → still shared
  const r2 = classifyTopologySubmodule({
    name: 'core', inDegree: 20, outDegree: 0, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R2-b. high inDegree (20) → shared-submodule',
    r2.label === 'shared-submodule');

  // S-R2-boundary. inDegree === 4 → NOT shared
  const r3 = classifyTopologySubmodule({
    name: 'x', inDegree: 4, outDegree: 1, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R2-boundary. inDegree=4 (below threshold) → NOT shared',
    r3.label !== 'shared-submodule');

  assert('S-R2-marker. shared-submodule marker is ✅',
    r1.marker === '✅');
}

// ═══ SUBMODULE CLASSIFIER — Rule 3 leaf-submodule ═══

{
  // S-R3-a. outDegree > inDegree AND inDegree < 5
  const r1 = classifyTopologySubmodule({
    name: 'app', inDegree: 1, outDegree: 8, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R3-a. outDegree=8 > inDegree=1, inDegree<5 → leaf-submodule',
    r1.label === 'leaf-submodule', `got=${r1.label}`);

  // S-R3-b. outDegree > inDegree but inDegree >= 5 → shared (Rule 2 wins)
  const r2 = classifyTopologySubmodule({
    name: 'busy', inDegree: 5, outDegree: 10, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R3-b. inDegree=5 + high outDegree → shared-submodule (Rule 2 before Rule 3)',
    r2.label === 'shared-submodule', `got=${r2.label}`);

  // S-R3-c. outDegree <= inDegree → NOT leaf
  const r3 = classifyTopologySubmodule({
    name: 'x', inDegree: 3, outDegree: 2, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R3-c. outDegree <= inDegree → NOT leaf-submodule',
    r3.label !== 'leaf-submodule');

  assert('S-R3-marker. leaf-submodule marker is ⚠',
    r1.marker === '⚠');
}

// ═══ SUBMODULE CLASSIFIER — Rule 4 scoped-submodule (fallback) ═══

{
  // S-R4-a. Balanced low degree
  const r1 = classifyTopologySubmodule({
    name: 'x', inDegree: 2, outDegree: 2, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R4-a. balanced low in/out → scoped-submodule',
    r1.label === 'scoped-submodule', `got=${r1.label}`);

  // S-R4-b. outDegree = inDegree = 4 (below shared threshold, not leaf-eligible)
  const r2 = classifyTopologySubmodule({
    name: 'x', inDegree: 4, outDegree: 4, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R4-b. in=out=4 → scoped-submodule',
    r2.label === 'scoped-submodule');

  // S-R4-c. in < out but in > 0 → scoped (not isolated)  — actually leaf
  // This hits Rule 3 (outDegree > inDegree AND inDegree < 5), so leaf.
  const r3 = classifyTopologySubmodule({
    name: 'x', inDegree: 2, outDegree: 3, sccMember: false, crossEdgeSource: 'full-list',
  });
  assert('S-R4-c. in=2, out=3 → leaf-submodule (Rule 3 fires)',
    r3.label === 'leaf-submodule');

  assert('S-R4-marker. scoped-submodule marker is ℹ',
    r1.marker === 'ℹ');
}

// ═══ SUBMODULE CLASSIFIER — crossEdgeSource parameter ═══

{
  // Parameter-form sanity: crossEdgeSource is a string literal enum.
  const r1 = classifyTopologySubmodule({
    name: 'x', inDegree: 0, outDegree: 0, sccMember: false, crossEdgeSource: 'full-list',
  });
  const r2 = classifyTopologySubmodule({
    name: 'x', inDegree: 0, outDegree: 0, sccMember: false, crossEdgeSource: 'top-30-only',
  });
  assert('S-SOURCE. isolated vs scoped branches on crossEdgeSource literal',
    r1.label === 'isolated-submodule' && r2.label === 'scoped-submodule');
}

// ═══ SCC CLASSIFIER — canonical §11.2 ═══

{
  const r1 = classifyTopologyScc({ sccIndex: 0, members: ['a.ts', 'b.ts'] });
  assert('SCC-1. size-2 SCC → forbidden-cycle',
    r1.label === 'forbidden-cycle', `got=${r1.label}`);
  assert('SCC-1-marker. marker is ❌',
    r1.marker === '❌');

  const r2 = classifyTopologyScc({ sccIndex: 5, members: Array.from({ length: 10 }, (_, i) => `f${i}.ts`) });
  assert('SCC-2. size-10 SCC → still forbidden-cycle (no sub-tiering in v1)',
    r2.label === 'forbidden-cycle');

  const r3 = classifyTopologyScc({ sccIndex: 99, members: [] });
  assert('SCC-3. empty members array → still returns forbidden-cycle (degenerate but consistent)',
    r3.label === 'forbidden-cycle');
}

// ═══ FILE CLASSIFIER — canonical §11.3 ═══

{
  // Below threshold → null
  assert('F-below-1. loc=0 → null',
    classifyTopologyFile({ file: 'empty.ts', loc: 0 }) === null);
  assert('F-below-2. loc=100 → null',
    classifyTopologyFile({ file: 'small.ts', loc: 100 }) === null);
  assert('F-below-3. loc=399 → null',
    classifyTopologyFile({ file: 'almost.ts', loc: 399 }) === null);

  // Oversize band: 400-999
  const r400 = classifyTopologyFile({ file: 'a.ts', loc: 400 });
  assert('F-oversize-low. loc=400 (threshold) → oversize',
    r400?.label === 'oversize');
  const r500 = classifyTopologyFile({ file: 'b.ts', loc: 500 });
  assert('F-oversize-mid. loc=500 → oversize',
    r500?.label === 'oversize');
  const r999 = classifyTopologyFile({ file: 'c.ts', loc: 999 });
  assert('F-oversize-high. loc=999 → oversize (just below extreme)',
    r999?.label === 'oversize');

  // Extreme-oversize: ≥ 1000
  const r1000 = classifyTopologyFile({ file: 'd.ts', loc: 1000 });
  assert('F-extreme-low. loc=1000 (threshold) → extreme-oversize',
    r1000?.label === 'extreme-oversize');
  const r5000 = classifyTopologyFile({ file: 'e.ts', loc: 5000 });
  assert('F-extreme-high. loc=5000 → extreme-oversize',
    r5000?.label === 'extreme-oversize');

  assert('F-oversize-marker. oversize marker is ⚠',
    r400?.marker === '⚠');
  assert('F-extreme-marker. extreme-oversize marker is ❌',
    r1000?.marker === '❌');

  // Non-numeric loc → null (defensive)
  assert('F-defensive-string. loc string → null',
    classifyTopologyFile({ file: 'bad.ts', loc: '500' }) === null);
  assert('F-defensive-undef. loc undefined → null',
    classifyTopologyFile({ file: 'bad.ts' }) === null);
}

// ═══ LABEL SET CONSTANTS ═══

assert('CONST. TOPOLOGY_LABELS has 8 entries',
  TOPOLOGY_LABELS.length === 8);
assert('CONST. TOPOLOGY_LABELS frozen',
  Object.isFrozen(TOPOLOGY_LABELS));
assert('CONST. cyclic-submodule IN label set',
  TOPOLOGY_LABELS.includes('cyclic-submodule'));
assert('CONST. shared-submodule IN label set',
  TOPOLOGY_LABELS.includes('shared-submodule'));
assert('CONST. forbidden-cycle IN label set',
  TOPOLOGY_LABELS.includes('forbidden-cycle'));
assert('CONST. extreme-oversize IN label set',
  TOPOLOGY_LABELS.includes('extreme-oversize'));

assert('CONST. TOPOLOGY_UNCERTAIN_REASONS has 3 entries',
  TOPOLOGY_UNCERTAIN_REASONS.length === 3);
assert('CONST. TOPOLOGY_UNCERTAIN_REASONS frozen',
  Object.isFrozen(TOPOLOGY_UNCERTAIN_REASONS));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
