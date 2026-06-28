// Tests for `collectTopologyStructure` + `renderTopology` — P3-3 Step 2.
//
// DI-style: supply pre-canned `topology.json` + `triage.json` objects.
// End-to-end integration (real measure-topology.mjs + fixture repos) lives
// in `test-canon-draft-integration-topology.mjs`.
//
// Pinning rules from docs/history/phases/p3/p3-3.md v3 §5.3 + §5.3.1:
//   - PF-4: topology.json is primary input.
//   - PF-6: crossSubmoduleEdges full list = classification + §2 display source.
//           crossSubmoduleTop = degraded §2 display fallback only.
//   - §5.3.1 inventory source order:
//     1. triage.boundaries (monorepo)
//     2. triage.topDirs (single-package)
//     3. topology.nodes top-dir fallback (triage absent)
//     4. crossSubmoduleEdges AUGMENTS degrees only.
//   - Degraded mode when crossSubmoduleEdges absent: top-30 fallback,
//     classificationConfidence=medium, isolated-submodule suppressed.

import {
  collectTopologyStructure,
  renderTopology,
} from '../_lib/canon-draft-topology.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// Fixture builder
function mkTopology({
  nodes = {},
  crossSubmoduleEdges,
  crossSubmoduleTop,
  sccs = [],
  largestFiles = [],
  meta = {},
  summary = {},
} = {}) {
  const topo = {
    meta: { tool: 'm2s1-topology.mjs', generated: new Date().toISOString(), complete: true, ...meta },
    summary: { lens: 'runtime', ...summary },
    nodes,
    sccs,
    largestFiles,
  };
  if (crossSubmoduleEdges !== undefined) topo.crossSubmoduleEdges = crossSubmoduleEdges;
  if (crossSubmoduleTop !== undefined) topo.crossSubmoduleTop = crossSubmoduleTop;
  return topo;
}

// ═══ I1. Single-package mode, 3 submodules, full-list evidence ═══

{
  const topology = mkTopology({
    nodes: {
      '_lib/a.mjs': { loc: 100 },
      '_lib/b.mjs': { loc: 200 },
      'tests/t.mjs': { loc: 300 },
      'scripts/s.mjs': { loc: 50 },
    },
    crossSubmoduleEdges: [
      { from: 'tests', to: '_lib', count: 20 },
      { from: 'scripts', to: '_lib', count: 3 },
    ],
    crossSubmoduleTop: [
      { edge: 'tests → _lib', count: 20 },
      { edge: 'scripts → _lib', count: 3 },
    ],
  });
  const triage = {
    mode: 'single-package',
    topDirs: {
      '_lib': { files: 2, loc: 300 },
      'tests': { files: 1, loc: 300 },
      'scripts': { files: 1, loc: 50 },
    },
    boundaries: [],
  };
  const result = collectTopologyStructure({ topology, triage });
  assert('I1a. 3 submodule entries ({_lib, tests, scripts})',
    result.submodulesByPath.size === 3);
  const lib = result.submodulesByPath.get('_lib');
  assert('I1b. _lib has inDegree=23 (20+3), outDegree=0',
    lib?.inDegree === 23 && lib?.outDegree === 0);
  const tests = result.submodulesByPath.get('tests');
  assert('I1c. tests has inDegree=0, outDegree=20',
    tests?.inDegree === 0 && tests?.outDegree === 20);
  assert('I1d. meta.mode === "single-package"',
    result.meta.mode === 'single-package');
  assert('I1e. meta.crossEdgeSource === "full-list" (crossSubmoduleEdges present)',
    result.meta.crossEdgeSource === 'full-list');
  assert('I1f. meta.classificationConfidence === "high"',
    result.meta.classificationConfidence === 'high');
  assert('I1g. scripts (inDegree=0, outDegree=3) → leaf-submodule (outDegree > inDegree, inDegree < 5)',
    classifyCandidate(result, 'scripts') === 'leaf-submodule');
}

// Helper: classify a single submodule using the crossEdgeSource meta.
function classifyCandidate(result, name) {
  const entry = result.submodulesByPath.get(name);
  if (!entry) return null;
  // Inline classifier reproducing §11.1 — the aggregator itself doesn't
  // cache the label (renderer computes per-row); we recompute here for
  // test clarity.
  if (entry.sccMember) return 'cyclic-submodule';
  if (entry.inDegree === 0 && entry.outDegree === 0 && result.meta.crossEdgeSource === 'full-list') return 'isolated-submodule';
  if (entry.inDegree >= 5) return 'shared-submodule';
  if (entry.outDegree > entry.inDegree && entry.inDegree < 5) return 'leaf-submodule';
  return 'scoped-submodule';
}

// ═══ I2. §5.3.1 inventory source order — isolated submodule survives without cross-edges ═══

{
  const topology = mkTopology({
    nodes: {
      'a/foo.mjs': { loc: 100 },
      'b/bar.mjs': { loc: 200 },
      'c/baz.mjs': { loc: 50 },
    },
    crossSubmoduleEdges: [
      { from: 'a', to: 'b', count: 3 },
    ],
    crossSubmoduleTop: [{ edge: 'a → b', count: 3 }],
  });
  const triage = {
    mode: 'single-package',
    topDirs: {
      'a': { files: 1, loc: 100 },
      'b': { files: 1, loc: 200 },
      'c': { files: 1, loc: 50 },
    },
  };
  const result = collectTopologyStructure({ topology, triage });
  assert('I2a. inventory has 3 rows including `c` (no cross-edges)',
    result.submodulesByPath.size === 3 && result.submodulesByPath.has('c'));
  assert('I2b. `c` inDegree=0, outDegree=0',
    result.submodulesByPath.get('c').inDegree === 0 &&
    result.submodulesByPath.get('c').outDegree === 0);
  assert('I2c. `c` classifies to isolated-submodule in full-list mode',
    classifyCandidate(result, 'c') === 'isolated-submodule');
}

// ═══ I3. Degraded mode — crossSubmoduleEdges absent → top-30 fallback ═══

{
  const topology = mkTopology({
    nodes: {
      'a/x.mjs': { loc: 100 },
      'b/y.mjs': { loc: 100 },
    },
    // crossSubmoduleEdges omitted entirely (pre-P3-3-pre producer)
    crossSubmoduleTop: [{ edge: 'a → b', count: 5 }],
  });
  const triage = { mode: 'single-package', topDirs: { a: { files: 1, loc: 100 }, b: { files: 1, loc: 100 } } };
  const result = collectTopologyStructure({ topology, triage });
  assert('I3a. meta.crossEdgeSource === "top-30-only" (degraded)',
    result.meta.crossEdgeSource === 'top-30-only');
  assert('I3b. meta.classificationConfidence === "medium"',
    result.meta.classificationConfidence === 'medium');
  // Fallback: degrees computed from top-30
  const a = result.submodulesByPath.get('a');
  const b = result.submodulesByPath.get('b');
  assert('I3c. degrees still populated from top-30 fallback',
    a?.outDegree === 5 && b?.inDegree === 5);
}

// ═══ I4. SCC present → submodule flagged cyclic + sccs array populated ═══

{
  const topology = mkTopology({
    nodes: {
      'core/a.mjs': { loc: 100 },
      'core/b.mjs': { loc: 100 },
      'core/c.mjs': { loc: 100 },
    },
    crossSubmoduleEdges: [],
    crossSubmoduleTop: [],
    sccs: [{ size: 3, members: ['core/a.mjs', 'core/b.mjs', 'core/c.mjs'] }],
  });
  const triage = { mode: 'single-package', topDirs: { core: { files: 3, loc: 300 } } };
  const result = collectTopologyStructure({ topology, triage });
  const core = result.submodulesByPath.get('core');
  assert('I4a. core.sccMember === true',
    core?.sccMember === true);
  assert('I4b. result.sccs has 1 entry',
    result.sccs.length === 1);
  assert('I4c. classifier routes core to cyclic-submodule',
    classifyCandidate(result, 'core') === 'cyclic-submodule');
}

// ═══ I5. Oversize files filtered + classified ═══

{
  const topology = mkTopology({
    nodes: { 'big.ts': { loc: 500 }, 'huge.ts': { loc: 1200 } },
    crossSubmoduleEdges: [],
    largestFiles: [
      { file: 'huge.ts', loc: 1200 },
      { file: 'big.ts', loc: 500 },
      { file: 'small.ts', loc: 50 },  // should be filtered out
    ],
  });
  const result = collectTopologyStructure({ topology, triage: null });
  assert('I5a. oversizeFiles has 2 entries (< 400 LOC filtered)',
    result.oversizeFiles.length === 2);
  const huge = result.oversizeFiles.find((f) => f.file === 'huge.ts');
  const big = result.oversizeFiles.find((f) => f.file === 'big.ts');
  assert('I5b. huge.ts → extreme-oversize (loc=1200)',
    huge?.label === 'extreme-oversize');
  assert('I5c. big.ts → oversize (loc=500)',
    big?.label === 'oversize');
}

// ═══ I6. triage.json absent → topology.nodes top-dir fallback ═══

{
  const topology = mkTopology({
    nodes: {
      '_lib/a.mjs': { loc: 100 },
      '_lib/b.mjs': { loc: 100 },
      'tests/t.mjs': { loc: 100 },
    },
    crossSubmoduleEdges: [{ from: 'tests', to: '_lib', count: 2 }],
  });
  const result = collectTopologyStructure({ topology, triage: null });
  assert('I6a. inventory has 2 entries (_lib, tests) derived from top-dir',
    result.submodulesByPath.size === 2 &&
    result.submodulesByPath.has('_lib') &&
    result.submodulesByPath.has('tests'));
  assert('I6b. files/loc accumulated from nodes fallback',
    result.submodulesByPath.get('_lib').files === 2 &&
    result.submodulesByPath.get('_lib').loc === 200);
}

// ═══ I7. Monorepo mode — triage.boundaries populates workspaces ═══

{
  const topology = mkTopology({
    nodes: {
      'packages/a/src.ts': { loc: 100 },
      'packages/b/src.ts': { loc: 150 },
    },
    crossSubmoduleEdges: [{ from: 'packages/b', to: 'packages/a', count: 4 }],
  });
  const triage = {
    mode: 'monorepo-workspaces',
    boundaries: [
      { name: '@scope/a', path: 'packages/a', files: 1, loc: 100 },
      { name: '@scope/b', path: 'packages/b', files: 1, loc: 150 },
    ],
    topDirs: {},
  };
  const result = collectTopologyStructure({ topology, triage });
  assert('I7a. meta.mode === "monorepo-workspaces"',
    result.meta.mode === 'monorepo-workspaces');
  assert('I7b. workspaces populated with 2 entries',
    result.workspaces?.length === 2);
  assert('I7c. submodule keys match workspace paths',
    result.submodulesByPath.has('packages/a') && result.submodulesByPath.has('packages/b'));
}

// ═══ I8. topology.meta.complete=false → diagnostic + meta flag ═══

{
  const topology = mkTopology({
    nodes: {},
    crossSubmoduleEdges: [],
    meta: { tool: 'm2s1-topology.mjs', generated: new Date().toISOString(), complete: false },
  });
  const result = collectTopologyStructure({ topology, triage: null });
  assert('I8a. meta.topologyComplete === false',
    result.meta.topologyComplete === false);
  assert('I8b. diagnostic with reason topology-artifact-incomplete',
    result.diagnostics.some((d) => d.reason === 'topology-artifact-incomplete'));
}

// ═══ I9. Stale topology.json (> 24h) → stale diagnostic ═══

{
  const now = Date.parse('2026-04-21T12:00:00Z');
  const oldTs = now - 30 * 3600 * 1000; // 30h ago
  const topology = mkTopology({
    nodes: {},
    crossSubmoduleEdges: [],
    meta: { tool: 'm2s1-topology.mjs', generated: new Date(oldTs).toISOString(), complete: true },
  });
  const result = collectTopologyStructure({ topology, triage: null, nowMs: now });
  assert('I9a. meta.topologyStaleness === "stale"',
    result.meta.topologyStaleness === 'stale');
  assert('I9b. diagnostic with reason topology-artifact-stale',
    result.diagnostics.some((d) => d.reason === 'topology-artifact-stale'));
}

// ═══ I10. submodule-boundary-mismatch — edge endpoint not in inventory ═══

{
  const topology = mkTopology({
    nodes: { 'a/x.mjs': { loc: 100 } },
    crossSubmoduleEdges: [
      { from: 'a', to: 'ghostmod', count: 3 },  // ghostmod not in triage
    ],
  });
  const triage = {
    mode: 'single-package',
    topDirs: { 'a': { files: 1, loc: 100 } },
  };
  const result = collectTopologyStructure({ topology, triage });
  assert('I10. submodule-boundary-mismatch diagnostic when edge endpoint not in inventory',
    result.diagnostics.some((d) => d.reason === 'submodule-boundary-mismatch'));
}

// ═══ I11. Classifier NOT used on crossSubmoduleTop for degree math (PF-4 source-grep) ═══
//
// Asserted structurally in test-classification-gates.mjs TP15; here we
// verify runtime behavior — full-list and top-30 produce DIFFERENT degrees
// when the top-30 is truncated (i.e. full list has more entries than 30).
// We simulate this with a synthetic topology that has 35 cross edges in
// the full list but only 30 in top. Classification should favor full list.

{
  const edges = Array.from({ length: 35 }, (_, i) => ({ from: `mod_${i}`, to: 'hub', count: 1 }));
  const top30 = edges.slice(0, 30).map((e) => ({ edge: `${e.from} → ${e.to}`, count: e.count }));
  const topology = mkTopology({
    nodes: {
      'hub/x.mjs': { loc: 1 },
      ...Object.fromEntries(edges.map((e) => [`${e.from}/f.mjs`, { loc: 1 }])),
    },
    crossSubmoduleEdges: edges,
    crossSubmoduleTop: top30,
  });
  const triage = null;
  const result = collectTopologyStructure({ topology, triage });
  const hub = result.submodulesByPath.get('hub');
  assert('I11. hub inDegree=35 (from full crossSubmoduleEdges, NOT 30 from crossSubmoduleTop)',
    hub?.inDegree === 35, `got inDegree=${hub?.inDegree}`);
}

// ═══ I12. §2 display uses full-list top-30 sort, not legacy top order ═══

{
  const preferred = Array.from({ length: 30 }, (_, i) => ({
    from: `a${String(i).padStart(2, '0')}`,
    to: 'hub',
    count: 3,
  }));
  const lateTie = { from: 'z-last', to: 'hub', count: 3 };
  const topology = mkTopology({
    nodes: {
      'hub/x.mjs': { loc: 1 },
      ...Object.fromEntries([...preferred, lateTie].map((e) => [`${e.from}/x.mjs`, { loc: 1 }])),
    },
    crossSubmoduleEdges: [lateTie, ...preferred.slice().reverse()],
    crossSubmoduleTop: [{ edge: 'z-last → hub', count: 3 }],
  });
  const result = collectTopologyStructure({ topology, triage: null });
  const displayIds = new Set(result.crossEdgesForDisplay.map((e) => `${e.from} → ${e.to}`));
  assert('I12a. crossEdgesForDisplay has exactly 30 rows from full list',
    result.crossEdgesForDisplay.length === 30,
    `got ${result.crossEdgesForDisplay.length}`);
  assert('I12b. ASCII tie-break excludes z-last despite legacy top containing it',
    !displayIds.has('z-last → hub'),
    `display=${JSON.stringify(result.crossEdgesForDisplay)}`);
  assert('I12c. first row follows count desc + from/to ASCII tie-break',
    result.crossEdgesForDisplay[0]?.from === 'a00',
    `first=${JSON.stringify(result.crossEdgesForDisplay[0])}`);
}

// ═══ R1. Renderer — submodule inventory table ═══

{
  const topology = mkTopology({
    nodes: { 'a/x.mjs': { loc: 100 }, 'b/y.mjs': { loc: 50 } },
    crossSubmoduleEdges: [{ from: 'a', to: 'b', count: 2 }],
    crossSubmoduleTop: [{ edge: 'a → b', count: 2 }],
  });
  const triage = { mode: 'single-package', topDirs: { a: { files: 1, loc: 100 }, b: { files: 1, loc: 50 } } };
  const result = collectTopologyStructure({ topology, triage });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'TS/JS including tests' } });
  assert('R1a. # Topology draft header',
    md.includes('# Topology draft'));
  assert('R1b. §1 submodule inventory table with both submodules',
    md.includes('## 1. Submodule inventory') &&
    md.includes('`a`') && md.includes('`b`'));
  assert('R1c. FanInKind-analogue "CrossEdgeSource: full-list" line',
    md.includes('CrossEdgeSource: full-list'));
  assert('R1d. ClassificationConfidence: high',
    md.includes('ClassificationConfidence: high'));
}

// ═══ R2. Renderer — ✅ acyclic banner on SCC-free repo ═══

{
  const topology = mkTopology({ nodes: { 'a/x.mjs': { loc: 1 } }, crossSubmoduleEdges: [], sccs: [] });
  const result = collectTopologyStructure({ topology, triage: null });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'x' } });
  assert('R2. SCC-free repo → ✅ acyclic banner explicitly rendered (not silently empty)',
    md.includes('✅ No submodule-level cycles observed'));
}

// ═══ R3. Renderer — SCC member listing when present ═══

{
  const topology = mkTopology({
    nodes: { 'c/a.mjs': { loc: 1 }, 'c/b.mjs': { loc: 1 } },
    crossSubmoduleEdges: [],
    sccs: [{ size: 2, members: ['c/a.mjs', 'c/b.mjs'] }],
  });
  const result = collectTopologyStructure({ topology, triage: null });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'x' } });
  assert('R3a. SCC section shows the cycle with forbidden-cycle label',
    md.includes('forbidden-cycle') &&
    md.includes('Cycle 1') &&
    md.includes('`c/a.mjs`') &&
    md.includes('`c/b.mjs`'));
  assert('R3b. § header "❌ Cycles observed"',
    md.includes('❌ Cycles observed'));
}

// ═══ R4. Renderer — §5 workspace section OMITTED in single-package ═══

{
  const topology = mkTopology({ nodes: { 'a/x.mjs': { loc: 1 } }, crossSubmoduleEdges: [] });
  const triage = { mode: 'single-package', topDirs: { a: { files: 1, loc: 1 } } };
  const result = collectTopologyStructure({ topology, triage });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'x' } });
  assert('R4. single-package mode → §5 workspace section OMITTED',
    !md.includes('## 5. Workspace boundaries'));
}

// ═══ R5. Renderer — §5 workspace section RENDERED in monorepo ═══

{
  const topology = mkTopology({
    nodes: { 'packages/a/x.mjs': { loc: 1 } },
    crossSubmoduleEdges: [],
  });
  const triage = {
    mode: 'monorepo-workspaces',
    boundaries: [{ name: '@scope/a', path: 'packages/a', files: 1, loc: 1 }],
    topDirs: {},
  };
  const result = collectTopologyStructure({ topology, triage });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'x' } });
  assert('R5. monorepo mode → §5 rendered with workspace row',
    md.includes('## 5. Workspace boundaries') && md.includes('@scope/a'));
}

// ═══ R6. Renderer — degraded-mode warning in top-30-only mode ═══

{
  const topology = mkTopology({
    nodes: { 'a/x.mjs': { loc: 1 } },
    // crossSubmoduleEdges omitted → degraded
    crossSubmoduleTop: [{ edge: 'a → b', count: 1 }],
  });
  const result = collectTopologyStructure({ topology, triage: null });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'x' } });
  assert('R6a. degraded mode → header warning about top-30-only lens',
    md.includes('top-30 cross-edge lens'));
  assert('R6b. CrossEdgeSource: top-30-only in meta lines',
    md.includes('CrossEdgeSource: top-30-only'));
  assert('R6c. ClassificationConfidence: medium',
    md.includes('ClassificationConfidence: medium'));
}

// ═══ R7. Renderer — existing-canon observational header ═══

{
  const topology = mkTopology({ nodes: {}, crossSubmoduleEdges: [] });
  const result = collectTopologyStructure({ topology, triage: null });
  const md = renderTopology({
    ...result,
    meta: { ...result.meta, scope: 'x', existingCanon: true },
  });
  assert('R7. existingCanon=true → ⚠ Existing canon detected header for topology.md',
    md.includes('⚠ Existing canon detected') && md.includes('topology.md'));
}

// ═══ R8. Renderer — incomplete-topology warning ═══

{
  const topology = mkTopology({
    nodes: {},
    crossSubmoduleEdges: [],
    meta: { tool: 'm2s1-topology.mjs', generated: new Date().toISOString(), complete: false },
  });
  const result = collectTopologyStructure({ topology, triage: null });
  const md = renderTopology({ ...result, meta: { ...result.meta, scope: 'x' } });
  assert('R8. complete=false → header warning + TopologyComplete: false',
    md.includes('topology.json incomplete') && md.includes('TopologyComplete: false'));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
