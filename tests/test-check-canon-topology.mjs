// tests/test-check-canon-topology.mjs
//
// P5-3 Step 0 — RED test for `_lib/check-canon-topology.mjs`.
//
// Pins topology-drift 6-category enum, 3 sub-diffs (submodules / oversize /
// cross-edges), §1 inventory `SCC` column as authoritative (canon-drift.md
// v1.1 §5.c §1), §1/§3 disagreement → canon-parse-error, identity format
// per category (submodule path / `<from> → <to>` edge / ownerFile), and
// explicit top-30 count-desc sort before slice (p5-3.md §4.5.4).

import { writeFileSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';

import { detectTopologyDrift } from '../_lib/check-canon-topology.mjs';
import { TOPOLOGY_LABEL_SET } from '../_lib/check-canon-utils.mjs';

let passed = 0;
let failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed += 1; console.log(`  PASS  ${label}`); }
  else { failed += 1; console.log(`  FAIL  ${label}`); if (detail) console.log(`        ${detail}`); }
}

const workdir = mkdtempSync(path.join(tmpdir(), 'p5-3-engine-'));

// ── Canon MD builders ──────────────────────────────────────────

function buildCanonTopologyMd({ submodules, acyclic, cycles = [], crossEdges = [], oversize = [], workspaces = null }) {
  const lines = [];
  lines.push('# Topology canon (fixture)');
  lines.push('');
  lines.push('## 1. Submodule inventory');
  lines.push('');
  lines.push('| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |');
  lines.push('|-----------|------:|----:|---------:|----------:|-----|--------|------|');
  for (const s of submodules) {
    const sccMarker = s.sccMember ? '●' : '—';
    lines.push(`| \`${s.name}\` | ${s.files} | ${s.loc} | ${s.inEdges} | ${s.outEdges} | ${sccMarker} | ${s.label} ✅ | |`);
  }
  lines.push('');

  lines.push('## 2. Cross-submodule edges (top 30)');
  lines.push('');
  if (crossEdges.length === 0) {
    lines.push('_No cross-submodule edges observed._');
    lines.push('');
  } else {
    lines.push('| From | To | Count |');
    lines.push('|------|----|------:|');
    for (const e of crossEdges) {
      lines.push(`| \`${e.from}\` | \`${e.to}\` | ${e.count} |`);
    }
    lines.push('');
  }

  lines.push('## 3. Cycles (SCCs)');
  lines.push('');
  if (acyclic) {
    lines.push('✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.');
    lines.push('');
  } else {
    lines.push('❌ Cycles observed — canon invariant violation:');
    lines.push('');
    for (let i = 0; i < cycles.length; i += 1) {
      const c = cycles[i];
      lines.push(`### Cycle ${i + 1} (size ${c.members.length}) — forbidden-cycle ❌`);
      lines.push('');
      for (const m of c.members) lines.push(`- \`${m}\``);
      lines.push('');
    }
  }

  lines.push('## 4. Oversize files (≥ 400 LOC)');
  lines.push('');
  if (oversize.length === 0) {
    lines.push('_No oversize files observed._');
    lines.push('');
  } else {
    lines.push('| File | LOC | Status |');
    lines.push('|------|----:|--------|');
    for (const o of oversize) {
      lines.push(`| \`${o.file}\` | ${o.loc} | ${o.label} ⚠ |`);
    }
    lines.push('');
  }

  if (workspaces) {
    lines.push('## 5. Workspace boundaries');
    lines.push('');
    lines.push('| Package | Path | Files | LOC |');
    lines.push('|---------|------|------:|----:|');
    for (const w of workspaces) {
      lines.push(`| \`${w.name}\` | \`${w.path}\` | ${w.files} | ${w.loc} |`);
    }
    lines.push('');
  }

  return lines.join('\n');
}

function writeCanon(canonPath, spec) {
  writeFileSync(canonPath, buildCanonTopologyMd(spec), 'utf8');
}

// ── topology.json builder ──────────────────────────────────────

function buildTopology({ nodes = {}, sccs = [], crossSubmoduleEdges = null, crossSubmoduleTop = null, largestFiles = [] } = {}) {
  const t = {
    meta: { complete: true, generated: '2026-04-22T00:00:00Z' },
    nodes,
    sccs,
    largestFiles,
  };
  if (crossSubmoduleEdges !== null) t.crossSubmoduleEdges = crossSubmoduleEdges;
  if (crossSubmoduleTop !== null) t.crossSubmoduleTop = crossSubmoduleTop;
  return t;
}

// ── Y-1: missing canon → skipped-missing-canon ─────────────────

{
  const canonPath = path.join(workdir, 'y1-missing.md');
  const r = detectTopologyDrift({
    canonPath,
    topology: buildTopology({ nodes: {} }),
    triage: null,
    canonLabelSet: TOPOLOGY_LABEL_SET,
  });
  assert('Y-1a. missing canon → status=skipped-missing-canon',
    r.status === 'skipped-missing-canon', `status=${r.status}`);
  assert('Y-1b. missing canon → drifts empty + reportMarkdown null',
    r.drifts.length === 0 && r.reportMarkdown === null, '');
}

// ── Y-2: missing topology object → parse-error ─────────────────

{
  const canonPath = path.join(workdir, 'y2-null-topo.md');
  writeCanon(canonPath, { submodules: [], acyclic: true });
  const r = detectTopologyDrift({
    canonPath, topology: null, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET,
  });
  assert('Y-2. null topology → status=parse-error',
    r.status === 'parse-error', `status=${r.status}`);
}

// ── Y-3: submodule-added ───────────────────────────────────────

{
  const canonPath = path.join(workdir, 'y3-added.md');
  writeCanon(canonPath, {
    submodules: [{ name: 'src', files: 1, loc: 50, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' }],
    acyclic: true,
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 50 }, 'lib/b.ts': { loc: 20 } },
    crossSubmoduleEdges: [],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const added = r.drifts.filter((d) => d.category === 'submodule-added');
  assert('Y-3a. submodule-added = 1 when fresh gains a submodule',
    added.length === 1 && added[0].identity === 'lib',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('Y-3b. submodule-added family = added',
    added[0]?.family === 'added', `family=${added[0]?.family}`);
  assert('Y-3c. submodule-added identity has no :: and no →',
    !added[0]?.identity.includes('::') && !added[0]?.identity.includes('→'),
    `identity=${added[0]?.identity}`);
}

// ── Y-4: submodule-removed ─────────────────────────────────────

{
  const canonPath = path.join(workdir, 'y4-removed.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 50, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
      { name: 'gone', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'isolated-submodule' },
    ],
    acyclic: true,
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 50 } },
    crossSubmoduleEdges: [],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const removed = r.drifts.filter((d) => d.category === 'submodule-removed');
  assert('Y-4a. submodule-removed = 1 when canon has an extra submodule',
    removed.length === 1 && removed[0].identity === 'gone',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('Y-4b. submodule-removed identity has no :: and no →',
    !removed[0]?.identity.includes('::') && !removed[0]?.identity.includes('→'), '');
}

// ── Y-5: scc-status-changed ────────────────────────────────────

{
  const canonPath = path.join(workdir, 'y5-scc.md');
  // Canon says both submodules are acyclic (non-SCC members)
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 1, outEdges: 1, sccMember: false, label: 'shared-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 1, outEdges: 1, sccMember: false, label: 'shared-submodule' },
    ],
    acyclic: true,
  });
  // Fresh topology has them in an SCC → both sccMember flip to true
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    sccs: [{ members: ['src/a.ts', 'lib/b.ts'] }],
    crossSubmoduleEdges: [
      { from: 'src', to: 'lib', count: 1 },
      { from: 'lib', to: 'src', count: 1 },
    ],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const scc = r.drifts.filter((d) => d.category === 'scc-status-changed');
  assert('Y-5a. scc-status-changed = 2 when two submodules flip SCC',
    scc.length === 2,
    `drifts=${JSON.stringify(r.drifts.map((d) => d.category))}`);
  assert('Y-5b. scc-status-changed carries canon.sccMember=false, fresh.sccMember=true',
    scc.every((d) => d.canon.sccMember === false && d.fresh.sccMember === true),
    `records=${JSON.stringify(scc)}`);
  assert('Y-5c. scc-status-changed family = structural-status-changed',
    scc.every((d) => d.family === 'structural-status-changed'),
    `families=${scc.map((d) => d.family)}`);
}

// ── Y-6: §1 SCC / §3 disagreement → canon-parse-error ──────────

{
  const canonPath = path.join(workdir, 'y6-disagree.md');
  // §1 marks src as NON-SCC but §3 cycle member list INCLUDES src.
  // Hand-crafted MD rather than builder (builder enforces agreement).
  const md = [
    '## 1. Submodule inventory',
    '',
    '| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |',
    '|-----------|------:|----:|---------:|----------:|-----|--------|------|',
    '| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |',
    '',
    '## 3. Cycles (SCCs)',
    '',
    '❌ Cycles observed — canon invariant violation:',
    '',
    '### Cycle 1 (size 2) — forbidden-cycle ❌',
    '',
    '- `src`',
    '- `lib`',
    '',
  ].join('\n');
  writeFileSync(canonPath, md, 'utf8');
  const r = detectTopologyDrift({
    canonPath,
    topology: buildTopology({ nodes: { 'src/a.ts': { loc: 10 } }, sccs: [] }),
    triage: null,
    canonLabelSet: TOPOLOGY_LABEL_SET,
  });
  assert('Y-6a. §1/§3 disagreement → status=parse-error',
    r.status === 'parse-error', `status=${r.status}`);
  assert('Y-6b. parse-error diagnostic mentions scc agreement',
    r.diagnostics.some((d) => /scc/i.test(JSON.stringify(d))),
    `diagnostics=${JSON.stringify(r.diagnostics)}`);
  assert('Y-6c. no scc-status-changed emitted when canon is internally inconsistent',
    !r.drifts.some((d) => d.category === 'scc-status-changed'), '');
}

// ── Y-7: oversize-changed (added) ─────────────────────────────

{
  const canonPath = path.join(workdir, 'y7-oversize-added.md');
  writeCanon(canonPath, {
    submodules: [{ name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' }],
    acyclic: true,
  });
  const topology = buildTopology({
    nodes: { 'src/giant.ts': { loc: 500 } },
    largestFiles: [{ file: 'src/giant.ts', loc: 500 }],
    crossSubmoduleEdges: [],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const ov = r.drifts.filter((d) => d.category === 'oversize-changed');
  assert('Y-7a. oversize-changed = 1 when fresh gains an oversize file',
    ov.length === 1 && ov[0].identity === 'src/giant.ts',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('Y-7b. oversize-changed identity is file path (contains .ts)',
    /\.ts$/.test(ov[0]?.identity ?? ''), `identity=${ov[0]?.identity}`);
  assert('Y-7c. oversize-changed family = content-shifted',
    ov[0]?.family === 'content-shifted', `family=${ov[0]?.family}`);
}

// ── Y-8: cross-edge-added ──────────────────────────────────────

{
  const canonPath = path.join(workdir, 'y8-xedge-added.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
    ],
    acyclic: true,
    crossEdges: [],
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    crossSubmoduleEdges: [{ from: 'src', to: 'lib', count: 3 }],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const xa = r.drifts.filter((d) => d.category === 'cross-edge-added');
  assert('Y-8a. cross-edge-added = 1 when fresh has a new edge',
    xa.length === 1,
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('Y-8b. cross-edge-added identity is "<from> → <to>" literal with single spaces',
    xa[0]?.identity === 'src → lib',
    `identity=${xa[0]?.identity}`);
  assert('Y-8c. cross-edge-added carries fresh.count (no canon side)',
    typeof xa[0]?.fresh?.count === 'number' && xa[0].fresh.count === 3,
    `rec=${JSON.stringify(xa[0])}`);
  assert('Y-8d. cross-edge-added family = added',
    xa[0]?.family === 'added', `family=${xa[0]?.family}`);
}

// ── Y-9: cross-edge-removed ────────────────────────────────────

{
  const canonPath = path.join(workdir, 'y9-xedge-removed.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 1, sccMember: false, label: 'leaf-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 1, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
    ],
    acyclic: true,
    crossEdges: [{ from: 'src', to: 'lib', count: 5 }],
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    crossSubmoduleEdges: [],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const xr = r.drifts.filter((d) => d.category === 'cross-edge-removed');
  assert('Y-9a. cross-edge-removed = 1 when canon has an edge fresh dropped',
    xr.length === 1 && xr[0].identity === 'src → lib',
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('Y-9b. cross-edge-removed carries canon.count + canon.line (no fresh side)',
    typeof xr[0]?.canon?.count === 'number' && typeof xr[0]?.canon?.line === 'number',
    `rec=${JSON.stringify(xr[0])}`);
  assert('Y-9c. cross-edge-removed family = removed',
    xr[0]?.family === 'removed', `family=${xr[0]?.family}`);
}

// ── Y-10: top-30 sort-before-slice pin (Finding #4 / §4.5.4) ──

{
  // Canon has no cross-edges; fresh has 31 edges where the 31st-by-count
  // MUST fall outside the top-30 regardless of input ordering.
  const canonPath = path.join(workdir, 'y10-top30.md');
  // Build canon with many empty submodules to match fresh topology submodule set
  const sub = Array.from({ length: 32 }, (_, i) => ({
    name: `s${i}`, files: 1, loc: 1, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule',
  }));
  writeCanon(canonPath, { submodules: sub, acyclic: true, crossEdges: [] });

  // 31 edges: 30 with count=100, 1 with count=1. The count=1 edge is 31st.
  const edges = [];
  for (let i = 0; i < 30; i += 1) {
    edges.push({ from: `s${i}`, to: `s${(i + 1) % 32}`, count: 100 });
  }
  // The 31st edge — lowest count. If engine sorts count desc + slices 30, this is excluded.
  edges.push({ from: 's30', to: 's31', count: 1 });
  // Deliberately shuffle: put the low-count edge FIRST in the array.
  const shuffled = [edges[edges.length - 1], ...edges.slice(0, -1)];

  const nodes = {};
  for (const s of sub) nodes[`${s.name}/a.ts`] = { loc: 1 };

  const topology = buildTopology({
    nodes,
    crossSubmoduleEdges: shuffled,
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const xa = r.drifts.filter((d) => d.category === 'cross-edge-added');
  assert('Y-10a. top-30 slice honors count desc regardless of input ordering',
    xa.length === 30,
    `expected 30 edges in top-30; got ${xa.length}`);
  const identities = new Set(xa.map((d) => d.identity));
  assert('Y-10b. the count=1 edge (s30 → s31) is EXCLUDED from top-30',
    !identities.has('s30 → s31'),
    `s30 → s31 unexpectedly present`);
  assert('Y-10c. count=100 edges (e.g. s0 → s1) are INCLUDED',
    identities.has('s0 → s1'), 'top-30 missing expected high-count edge');
}

// ── Y-10d: legacy `crossSubmoduleTop` shape (`{edge, count}`) is supported ─
// Reviewer Finding #1 (2026-04-22): real `measure-topology.mjs` emits
// `crossSubmoduleTop` as `{edge: "<from> → <to>", count}` legacy strings.
// Engine must parse either shape, not access undefined `.from` / `.to`.

{
  const canonPath = path.join(workdir, 'y10d-legacy.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 1, sccMember: false, label: 'leaf-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 1, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
    ],
    acyclic: true,
    crossEdges: [],
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    // Legacy shape — what `measure-topology.mjs` actually produces:
    crossSubmoduleTop: [{ edge: 'src → lib', count: 5 }],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const xa = r.drifts.filter((d) => d.category === 'cross-edge-added');
  assert('Y-10d. legacy crossSubmoduleTop {edge, count} shape → identity parsed correctly',
    xa.length === 1 && xa[0].identity === 'src → lib' && !xa[0].identity.includes('undefined'),
    `drifts=${JSON.stringify(r.drifts)}`);
  assert('Y-10e. legacy shape: fresh.count preserved',
    xa[0]?.fresh?.count === 5, `count=${xa[0]?.fresh?.count}`);
}

// ── Y-10f: full structured crossSubmoduleEdges PREFERRED over legacy top ──

{
  const canonPath = path.join(workdir, 'y10f-pref.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 1, sccMember: false, label: 'leaf-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 1, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
    ],
    acyclic: true,
    crossEdges: [],
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    // Both present — structured edges is preferred; legacy top ignored.
    crossSubmoduleEdges: [{ from: 'src', to: 'lib', count: 10 }],
    crossSubmoduleTop: [{ edge: 'src → lib', count: 5 }],  // divergent count — structured wins
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const xa = r.drifts.filter((d) => d.category === 'cross-edge-added');
  assert('Y-10f. structured crossSubmoduleEdges PREFERRED when both are present',
    xa.length === 1 && xa[0].fresh.count === 10, `rec=${JSON.stringify(xa[0])}`);
}

// ── Y-10g: empty full list is authoritative over stale legacy top ──────────

{
  const canonPath = path.join(workdir, 'y10g-empty-full-list.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'isolated-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'isolated-submodule' },
    ],
    acyclic: true,
    crossEdges: [],
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    // Empty full-list means no cross-submodule edges; legacy top may be stale.
    crossSubmoduleEdges: [],
    crossSubmoduleTop: [{ edge: 'src → lib', count: 5 }],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  const xa = r.drifts.filter((d) => d.category === 'cross-edge-added');
  assert('Y-10g. empty structured crossSubmoduleEdges suppresses stale legacy top',
    xa.length === 0, `drifts=${JSON.stringify(r.drifts)}`);
}

// ── Y-11: clean (zero drift) ──────────────────────────────────

{
  const canonPath = path.join(workdir, 'y11-clean.md');
  writeCanon(canonPath, {
    submodules: [{ name: 'src', files: 1, loc: 50, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' }],
    acyclic: true,
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 50 } },
    crossSubmoduleEdges: [],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('Y-11a. clean run → status=clean, 0 drifts',
    r.status === 'clean' && r.drifts.length === 0,
    `status=${r.status}, drifts=${r.drifts.length}`);
  assert('Y-11b. clean run → reportMarkdown has §1 Summary',
    typeof r.reportMarkdown === 'string' && r.reportMarkdown.includes('## 1. Summary'),
    `md=${r.reportMarkdown?.slice(0, 200)}`);
  assert('Y-11c. clean run MD omits category sections',
    !r.reportMarkdown.includes('## 2. submodule-added') &&
    !r.reportMarkdown.includes('## 3. submodule-removed'), '');
}

// ── Y-12: kind=topology-drift invariant ───────────────────────

{
  const canonPath = path.join(workdir, 'y12-kind.md');
  writeCanon(canonPath, {
    submodules: [{ name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' }],
    acyclic: true,
  });
  const topology = buildTopology({ nodes: {}, crossSubmoduleEdges: [] });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('Y-12. every drift record has kind=topology-drift',
    r.drifts.length > 0 && r.drifts.every((d) => d.kind === 'topology-drift'),
    `kinds=${JSON.stringify(r.drifts.map((d) => d.kind))}`);
}

// ── Y-13: MD cross-edge row includes "Display scope: top-30" ──

{
  const canonPath = path.join(workdir, 'y13-display.md');
  writeCanon(canonPath, {
    submodules: [
      { name: 'src', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
      { name: 'lib', files: 1, loc: 10, inEdges: 0, outEdges: 0, sccMember: false, label: 'leaf-submodule' },
    ],
    acyclic: true,
    crossEdges: [],
  });
  const topology = buildTopology({
    nodes: { 'src/a.ts': { loc: 10 }, 'lib/b.ts': { loc: 10 } },
    crossSubmoduleEdges: [{ from: 'src', to: 'lib', count: 2 }],
  });
  const r = detectTopologyDrift({ canonPath, topology, triage: null, canonLabelSet: TOPOLOGY_LABEL_SET });
  assert('Y-13. cross-edge MD row includes "Display scope: top-30"',
    r.reportMarkdown.includes('Display scope') && r.reportMarkdown.includes('top-30'),
    `md=${r.reportMarkdown.slice(0, 600)}`);
}

// ── Y-14: TOPOLOGY_LABEL_SET mirrors §11.4 (8 entries) ─────────

{
  const expected = new Set([
    'cyclic-submodule', 'isolated-submodule', 'shared-submodule',
    'leaf-submodule', 'scoped-submodule', 'forbidden-cycle',
    'oversize', 'extreme-oversize',
  ]);
  assert('Y-14a. TOPOLOGY_LABEL_SET size = 8',
    TOPOLOGY_LABEL_SET.size === 8, `size=${TOPOLOGY_LABEL_SET.size}`);
  assert('Y-14b. TOPOLOGY_LABEL_SET equals §11.4 canonical set',
    [...expected].every((l) => TOPOLOGY_LABEL_SET.has(l)),
    `missing=${[...expected].filter((l) => !TOPOLOGY_LABEL_SET.has(l)).join(',')}`);
}

rmSync(workdir, { recursive: true, force: true });

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
