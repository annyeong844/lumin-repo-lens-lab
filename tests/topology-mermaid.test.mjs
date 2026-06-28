import { describe, expect, it } from "vitest";

import { renderTopologyMermaid } from "../_lib/topology-mermaid.mjs";

describe("topology Mermaid artifact rendering", () => {
  it("M1-M6. renders stable Markdown sections, Mermaid graphs, hub evidence, and citation contract", () => {
    const md = renderTopologyMermaid({
      meta: { generated: "2026-05-01T00:00:00.000Z" },
      summary: { lens: "runtime", sccCount: 1 },
      crossSubmoduleEdges: [
        { from: "apps/web", to: "packages/ui", count: 4 },
        { from: "apps/web", to: "packages/api", count: 2 },
      ],
      topFanIn: [{ file: "packages/ui/src/button.ts", count: 8 }],
      topFanOut: [{ file: "apps/web/src/app.ts", count: 5 }],
      sccs: [{ size: 2, members: ["src/a.ts", "src/b.ts"] }],
      edges: [
        { from: "src/a.ts", to: "src/b.ts", typeOnly: false },
        { from: "src/b.ts", to: "src/a.ts", typeOnly: false },
      ],
    });

    expect(md.startsWith("# Topology Mermaid")).toBe(true);
    expect(md).toContain("```mermaid");
    expect(
      [
        "## How To Read This",
        "## Cross-Submodule Edges",
        "## Runtime Cycles",
        "## Hub Files",
        "## Omitted Detail / Limits",
        "## Citation Contract",
      ].every((section) => md.includes(section)),
    ).toBe(true);
    expect(md).toContain("flowchart LR");
    expect(md).toContain('sub0["apps/web"]');
    expect(md).toContain('sub1["packages/ui"]');
    expect(md).toContain("sub0 -->|4| sub1");
    expect(md).toContain('scc0_0["src/a.ts"]');
    expect(md).toContain('scc0_1["src/b.ts"]');
    expect(md).toContain("scc0_0 --> scc0_1");
    expect(md).toContain("scc0_1 --> scc0_0");
    expect(md).toContain("packages/ui/src/button.ts");
    expect(md).toContain("8 inbound");
    expect(md).toContain("apps/web/src/app.ts");
    expect(md).toContain("5 outbound");
    expect(md).toContain("topology.json.topFanIn");
    expect(md).toContain("topology.json.topFanOut");
    expect(md).toContain("visual companion");
    expect(md).toContain("not citation authority");
    expect(md).toContain("cite `topology.json`");
  });

  it("M7. renders explicit empty-state notes", () => {
    const md = renderTopologyMermaid({
      summary: { lens: "runtime", sccCount: 0 },
      crossSubmoduleEdges: [],
      sccs: [],
      edges: [],
    });

    expect(md).toContain("No cross-submodule edges were observed");
    expect(md).toContain("No runtime cycles were observed");
    expect(md).toContain("No hub files were available");
  });

  it("M8. escapes quoted Mermaid labels", () => {
    const md = renderTopologyMermaid({
      summary: { lens: "runtime", sccCount: 0 },
      crossSubmoduleEdges: [{ from: 'a"b', to: "x[y]", count: 1 }],
      sccs: [],
      edges: [],
    });

    expect(md).toContain('sub0["a\\"b"]');
    expect(md).toContain('sub1["x[y]"]');
  });

  it("M9-M10. reports cross-edge and cycle caps with source counts", () => {
    const edges = Array.from({ length: 31 }, (_, i) => ({
      from: `pkg${i}`,
      to: "core",
      count: i + 1,
    }));
    const md = renderTopologyMermaid(
      {
        summary: { lens: "runtime", sccCount: 0 },
        crossSubmoduleEdges: edges,
        sccs: [
          { size: 2, members: ["src/a.ts", "src/b.ts"] },
          { size: 2, members: ["src/c.ts", "src/d.ts"] },
        ],
        edges: [
          { from: "src/a.ts", to: "src/b.ts", typeOnly: false },
          { from: "src/c.ts", to: "src/d.ts", typeOnly: false },
        ],
      },
      { edgeLimit: 3, cycleLimit: 1 },
    );

    expect(md).toContain("Showing 3 of 31 cross-submodule edges (cap: 3).");
    expect(md).toContain("pkg30");
    expect(md).toContain("|31|");
    expect(md).not.toContain('pkg0["pkg0"]');
    expect(md).toContain("Showing 1 of 2 runtime cycles (cap: 1).");
    expect(md).toContain("SCC 1");
    expect(md).not.toContain("SCC 2");
  });

  it("M11. does not emit dangling Mermaid node ids", () => {
    const md = renderTopologyMermaid({
      summary: { lens: "runtime", sccCount: 1 },
      crossSubmoduleEdges: [],
      sccs: [{ size: 2, members: ["src/a.ts", "src/b.ts"] }],
      edges: [
        { from: "src/a.ts", to: "src/missing.ts", typeOnly: false },
        { from: "src/a.ts", to: "src/b.ts", typeOnly: true },
      ],
    });

    expect(md).not.toContain("undefined");
    expect(md).not.toContain("--> undefined");
  });
});
