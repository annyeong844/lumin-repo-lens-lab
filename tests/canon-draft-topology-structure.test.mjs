import { describe, expect, it } from "vitest";

import {
  classifyTopologySubmodule,
  collectTopologyStructure,
  renderTopology,
} from "../_lib/canon-draft-topology.mjs";

const NOW = Date.parse("2026-05-15T00:00:00Z");

function makeTopology({
  nodes = {},
  crossSubmoduleEdges,
  crossSubmoduleTop,
  sccs = [],
  largestFiles = [],
  meta = {},
  summary = {},
} = {}) {
  const topology = {
    meta: {
      tool: "m2s1-topology.mjs",
      generated: new Date(NOW).toISOString(),
      complete: true,
      ...meta,
    },
    summary: { lens: "runtime", ...summary },
    nodes,
    sccs,
    largestFiles,
  };
  if (crossSubmoduleEdges !== undefined) {
    topology.crossSubmoduleEdges = crossSubmoduleEdges;
  }
  if (crossSubmoduleTop !== undefined) {
    topology.crossSubmoduleTop = crossSubmoduleTop;
  }
  return topology;
}

function collect(topology, triage = null) {
  return collectTopologyStructure({ topology, triage, nowMs: NOW });
}

function classify(result, name) {
  const entry = result.submodulesByPath.get(name);
  if (!entry) return null;
  return classifyTopologySubmodule({
    name,
    inDegree: entry.inDegree,
    outDegree: entry.outDegree,
    sccMember: entry.sccMember,
    crossEdgeSource: result.meta.crossEdgeSource,
  }).label;
}

function render(result, meta = {}) {
  return renderTopology({
    ...result,
    meta: {
      ...result.meta,
      source: "topology.json",
      scope: "TS/JS including tests",
      generatedAt: "2026-05-15T00:00:00.000Z",
      ...meta,
    },
  });
}

describe("topology canon aggregation and rendering", () => {
  it("I1/I2. builds full-list submodule inventory without dropping isolated rows", () => {
    const topology = makeTopology({
      nodes: {
        "_lib/a.mjs": { loc: 100 },
        "_lib/b.mjs": { loc: 200 },
        "tests/t.mjs": { loc: 300 },
        "scripts/s.mjs": { loc: 50 },
        "c/baz.mjs": { loc: 10 },
      },
      crossSubmoduleEdges: [
        { from: "tests", to: "_lib", count: 20 },
        { from: "scripts", to: "_lib", count: 3 },
      ],
      crossSubmoduleTop: [
        { edge: "tests → _lib", count: 20 },
        { edge: "scripts → _lib", count: 3 },
      ],
    });
    const triage = {
      mode: "single-package",
      topDirs: {
        _lib: { files: 2, loc: 300 },
        tests: { files: 1, loc: 300 },
        scripts: { files: 1, loc: 50 },
        c: { files: 1, loc: 10 },
      },
    };
    const result = collect(topology, triage);

    expect(result.submodulesByPath.size).toBe(4);
    expect(result.submodulesByPath.get("_lib")).toMatchObject({
      files: 2,
      loc: 300,
      inDegree: 23,
      outDegree: 0,
    });
    expect(result.submodulesByPath.get("tests")).toMatchObject({
      inDegree: 0,
      outDegree: 20,
    });
    expect(result.meta).toMatchObject({
      mode: "single-package",
      crossEdgeSource: "full-list",
      classificationConfidence: "high",
    });
    expect(classify(result, "scripts")).toBe("leaf-submodule");
    expect(classify(result, "c")).toBe("isolated-submodule");
  });

  it("I3/R6. degrades to top-30 lens with medium confidence and visible warnings", () => {
    const result = collect(
      makeTopology({
        nodes: { "a/x.mjs": { loc: 100 }, "b/y.mjs": { loc: 100 } },
        crossSubmoduleTop: [{ edge: "a → b", count: 5 }],
      }),
      {
        mode: "single-package",
        topDirs: { a: { files: 1, loc: 100 }, b: { files: 1, loc: 100 } },
      },
    );
    const md = render(result);

    expect(result.meta.crossEdgeSource).toBe("top-30-only");
    expect(result.meta.classificationConfidence).toBe("medium");
    expect(result.submodulesByPath.get("a").outDegree).toBe(5);
    expect(result.submodulesByPath.get("b").inDegree).toBe(5);
    expect(classify(result, "a")).toBe("leaf-submodule");
    expect(md).toContain("Submodule classification derived from top-30");
    expect(md).toContain("CrossEdgeSource: top-30-only");
    expect(md).toContain("ClassificationConfidence: medium");
  });

  it("I4/I5/I7/I8/I9/R3/R5/R7/R8. surfaces SCC, oversize, workspace, stale, and incomplete evidence", () => {
    const result = collect(
      makeTopology({
        meta: {
          complete: false,
          generated: "2026-05-13T00:00:00.000Z",
        },
        nodes: {
          "packages/core/a.mjs": { loc: 100 },
          "packages/core/b.mjs": { loc: 100 },
          "packages/app/main.mjs": { loc: 50 },
        },
        crossSubmoduleEdges: [],
        sccs: [
          {
            size: 2,
            members: ["packages/core/a.mjs", "packages/core/b.mjs"],
          },
        ],
        largestFiles: [
          { file: "huge.ts", loc: 1200 },
          { file: "big.ts", loc: 500 },
          { file: "small.ts", loc: 50 },
        ],
      }),
      {
        mode: "monorepo-workspaces",
        boundaries: [
          { name: "core", path: "packages/core", files: 2, loc: 200 },
          { name: "app", path: "packages/app", files: 1, loc: 50 },
        ],
      },
    );
    const md = render(result, { existingCanon: true });

    expect(result.workspaces).toHaveLength(2);
    expect(result.submodulesByPath.get("packages/core").sccMember).toBe(true);
    expect(result.sccs).toHaveLength(1);
    expect(result.oversizeFiles.map((file) => file.label)).toEqual([
      "extreme-oversize",
      "oversize",
    ]);
    expect(result.diagnostics.map((diag) => diag.reason)).toEqual(
      expect.arrayContaining([
        "topology-artifact-incomplete",
        "topology-artifact-stale",
      ]),
    );
    expect(md).toContain("Existing canon detected");
    expect(md).toContain("TopologyComplete: false");
    expect(md).toContain("topology.json is stale");
    expect(md).toContain("Cycles observed");
    expect(md).toContain("forbidden-cycle");
    expect(md).toContain("Workspace boundaries");
    expect(md).toContain("extreme-oversize");
  });

  it("I10/I11/I12/R2. prefers full cross-edge lists, reports boundary mismatches, and sorts display top 30", () => {
    const edges = Array.from({ length: 31 }, (_, i) => ({
      from: `s${String(i).padStart(2, "0")}`,
      to: "hub",
      count: i === 30 ? 1 : 100,
    }));
    const topology = makeTopology({
      nodes: { "hub/index.mjs": { loc: 1 }, "known/a.mjs": { loc: 1 } },
      crossSubmoduleEdges: [
        { from: "known", to: "missing", count: 2 },
        ...edges,
      ],
      crossSubmoduleTop: [{ edge: "stale → hub", count: 999 }],
    });
    const result = collect(topology, {
      mode: "single-package",
      topDirs: { hub: { files: 1, loc: 1 }, known: { files: 1, loc: 1 } },
    });
    const md = render(result);

    expect(
      result.diagnostics.some(
        (d) => d.reason === "submodule-boundary-mismatch",
      ),
    ).toBe(true);
    expect(result.crossEdgesForDisplay).toHaveLength(30);
    expect(result.crossEdgesForDisplay[0]).toMatchObject({
      from: "s00",
      to: "hub",
      count: 100,
    });
    expect(result.crossEdgesForDisplay.some((e) => e.from === "s30")).toBe(
      false,
    );
    expect(result.crossEdgesForDisplay.some((e) => e.from === "stale")).toBe(
      false,
    );
    expect(md).toContain("No submodule-level cycles observed");
    expect(md).toContain("submodule-boundary-mismatch");
  });
});
