import { renderTopologyMermaid } from '../_lib/topology-mermaid.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

{
  const md = renderTopologyMermaid({
    meta: { generated: '2026-05-01T00:00:00.000Z' },
    summary: { lens: 'runtime', sccCount: 1 },
    crossSubmoduleEdges: [
      { from: 'apps/web', to: 'packages/ui', count: 4 },
      { from: 'apps/web', to: 'packages/api', count: 2 },
    ],
    topFanIn: [
      { file: 'packages/ui/src/button.ts', count: 8 },
    ],
    topFanOut: [
      { file: 'apps/web/src/app.ts', count: 5 },
    ],
    sccs: [
      { size: 2, members: ['src/a.ts', 'src/b.ts'] },
    ],
    edges: [
      { from: 'src/a.ts', to: 'src/b.ts', typeOnly: false },
      { from: 'src/b.ts', to: 'src/a.ts', typeOnly: false },
    ],
  });

  assert('M1. renders a Markdown Mermaid artifact',
    md.startsWith('# Topology Mermaid') && md.includes('```mermaid'));
  assert('M2. renders the stable artifact contract sections',
    [
      '## How To Read This',
      '## Cross-Submodule Edges',
      '## Runtime Cycles',
      '## Hub Files',
      '## Omitted Detail / Limits',
      '## Citation Contract',
    ].every((section) => md.includes(section)),
    md);
  assert('M3. cross-submodule graph uses Mermaid flowchart syntax',
    md.includes('flowchart LR') &&
    md.includes('sub0["apps/web"]') &&
    md.includes('sub1["packages/ui"]') &&
    md.includes('sub0 -->|4| sub1'));
  assert('M4. cycle graph uses topology SCC members and internal edges',
    md.includes('scc0_0["src/a.ts"]') &&
    md.includes('scc0_1["src/b.ts"]') &&
    md.includes('scc0_0 --> scc0_1') &&
    md.includes('scc0_1 --> scc0_0'));
  assert('M5. hub files render from topFanIn and topFanOut as Markdown evidence',
    md.includes('packages/ui/src/button.ts') &&
    md.includes('8 inbound') &&
    md.includes('apps/web/src/app.ts') &&
    md.includes('5 outbound') &&
    md.includes('topology.json.topFanIn') &&
    md.includes('topology.json.topFanOut'),
    md);
  assert('M6. citation contract keeps topology.json authoritative',
    md.includes('visual companion') &&
    md.includes('not citation authority') &&
    md.includes('cite `topology.json`'),
    md);
}

{
  const md = renderTopologyMermaid({
    summary: { lens: 'runtime', sccCount: 0 },
    crossSubmoduleEdges: [],
    sccs: [],
    edges: [],
  });

  assert('M7. empty topology renders explicit no-edge, no-cycle, and no-hub notes',
    md.includes('No cross-submodule edges were observed') &&
    md.includes('No runtime cycles were observed') &&
    md.includes('No hub files were available'));
}

{
  const md = renderTopologyMermaid({
    summary: { lens: 'runtime', sccCount: 0 },
    crossSubmoduleEdges: [
      { from: 'a"b', to: 'x[y]', count: 1 },
    ],
    sccs: [],
    edges: [],
  });

  assert('M8. labels are escaped for Mermaid quoted labels',
    md.includes('sub0["a\\"b"]') && md.includes('sub1["x[y]"]'));
}

{
  const edges = Array.from({ length: 31 }, (_, i) => ({
    from: `pkg${i}`,
    to: 'core',
    count: i + 1,
  }));
  const md = renderTopologyMermaid({
    summary: { lens: 'runtime', sccCount: 0 },
    crossSubmoduleEdges: edges,
    sccs: [
      { size: 2, members: ['src/a.ts', 'src/b.ts'] },
      { size: 2, members: ['src/c.ts', 'src/d.ts'] },
    ],
    edges: [
      { from: 'src/a.ts', to: 'src/b.ts', typeOnly: false },
      { from: 'src/c.ts', to: 'src/d.ts', typeOnly: false },
    ],
  }, { edgeLimit: 3, cycleLimit: 1 });

  assert('M9. cross-edge cap reports shown and source counts',
    md.includes('Showing 3 of 31 cross-submodule edges (cap: 3).') &&
    md.includes('pkg30') &&
    md.includes('|31|') &&
    !md.includes('pkg0["pkg0"]'),
    md);
  assert('M10. cycle cap reports shown and source counts',
    md.includes('Showing 1 of 2 runtime cycles (cap: 1).') &&
    md.includes('SCC 1') &&
    !md.includes('SCC 2'),
    md);
}

{
  const md = renderTopologyMermaid({
    summary: { lens: 'runtime', sccCount: 1 },
    crossSubmoduleEdges: [],
    sccs: [
      { size: 2, members: ['src/a.ts', 'src/b.ts'] },
    ],
    edges: [
      { from: 'src/a.ts', to: 'src/missing.ts', typeOnly: false },
      { from: 'src/a.ts', to: 'src/b.ts', typeOnly: true },
    ],
  });

  assert('M11. cycle renderer does not emit dangling Mermaid node ids',
    !md.includes('undefined') && !md.includes('--> undefined'),
    md);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
