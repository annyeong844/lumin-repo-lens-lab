import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, describe, expect, it } from "vitest";

import { detectTopologyDrift } from "../_lib/check-canon-topology.mjs";
import { TOPOLOGY_LABEL_SET } from "../_lib/check-canon-utils.mjs";

const workdir = mkdtempSync(path.join(tmpdir(), "vitest-topology-drift-"));

afterAll(() => {
  rmSync(workdir, { recursive: true, force: true });
});

function buildCanonTopologyMd({
  submodules,
  acyclic,
  cycles = [],
  crossEdges = [],
  oversize = [],
  workspaces = null,
}) {
  const lines = [];
  lines.push("# Topology canon (fixture)", "");
  lines.push("## 1. Submodule inventory", "");
  lines.push(
    "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |",
  );
  lines.push(
    "|-----------|------:|----:|---------:|----------:|-----|--------|------|",
  );
  for (const submodule of submodules) {
    lines.push(
      `| \`${submodule.name}\` | ${submodule.files} | ${submodule.loc} | ${submodule.inEdges} | ${submodule.outEdges} | ${submodule.sccMember ? "●" : "—"} | ${submodule.label} ✅ | |`,
    );
  }
  lines.push("", "## 2. Cross-submodule edges (top 30)", "");
  if (crossEdges.length === 0) {
    lines.push("_No cross-submodule edges observed._", "");
  } else {
    lines.push("| From | To | Count |", "|------|----|------:|");
    for (const edge of crossEdges) {
      lines.push(`| \`${edge.from}\` | \`${edge.to}\` | ${edge.count} |`);
    }
    lines.push("");
  }
  lines.push("## 3. Cycles (SCCs)", "");
  if (acyclic) {
    lines.push(
      "✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.",
      "",
    );
  } else {
    lines.push("❌ Cycles observed — canon invariant violation:", "");
    cycles.forEach((cycle, index) => {
      lines.push(
        `### Cycle ${index + 1} (size ${cycle.members.length}) — forbidden-cycle ❌`,
        "",
      );
      for (const member of cycle.members) lines.push(`- \`${member}\``);
      lines.push("");
    });
  }
  lines.push("## 4. Oversize files (≥ 400 LOC)", "");
  if (oversize.length === 0) {
    lines.push("_No oversize files observed._", "");
  } else {
    lines.push("| File | LOC | Status |", "|------|----:|--------|");
    for (const file of oversize) {
      lines.push(`| \`${file.file}\` | ${file.loc} | ${file.label} ⚠ |`);
    }
    lines.push("");
  }
  if (workspaces) {
    lines.push("## 5. Workspace boundaries", "");
    lines.push(
      "| Package | Path | Files | LOC |",
      "|---------|------|------:|----:|",
    );
    for (const workspace of workspaces) {
      lines.push(
        `| \`${workspace.name}\` | \`${workspace.path}\` | ${workspace.files} | ${workspace.loc} |`,
      );
    }
    lines.push("");
  }
  return lines.join("\n");
}

function writeCanon(name, spec) {
  const canonPath = path.join(workdir, name);
  writeFileSync(canonPath, buildCanonTopologyMd(spec), "utf8");
  return canonPath;
}

function topology({
  nodes = {},
  sccs = [],
  crossSubmoduleEdges,
  crossSubmoduleTop,
  largestFiles = [],
} = {}) {
  const result = {
    meta: { complete: true, generated: "2026-05-15T00:00:00Z" },
    summary: { lens: "runtime" },
    nodes,
    sccs,
    largestFiles,
  };
  if (crossSubmoduleEdges !== undefined) {
    result.crossSubmoduleEdges = crossSubmoduleEdges;
  }
  if (crossSubmoduleTop !== undefined) {
    result.crossSubmoduleTop = crossSubmoduleTop;
  }
  return result;
}

function detect(canonPath, observed, triage = null) {
  return detectTopologyDrift({
    canonPath,
    topology: observed,
    triage,
    canonLabelSet: TOPOLOGY_LABEL_SET,
  });
}

describe("topology canon drift detection", () => {
  it("Y-1/Y-2. reports missing canon and missing topology without false drift", () => {
    const missing = detect(path.join(workdir, "missing.md"), topology());
    expect(missing).toMatchObject({
      status: "skipped-missing-canon",
      drifts: [],
      reportMarkdown: null,
    });

    const canonPath = writeCanon("null-topology.md", {
      submodules: [],
      acyclic: true,
    });
    const absentTopology = detect(canonPath, null);
    expect(absentTopology.status).toBe("parse-error");
    expect(absentTopology.diagnostics.map((d) => d.reason)).toContain(
      "topology-input-missing",
    );
  });

  it("Y-3/Y-4/Y-5. detects submodule additions, removals, and SCC status changes with submodule identities", () => {
    const canonPath = writeCanon("submodule-drift.md", {
      submodules: [
        {
          name: "src",
          files: 1,
          loc: 10,
          inEdges: 1,
          outEdges: 1,
          sccMember: false,
          label: "shared-submodule",
        },
        {
          name: "gone",
          files: 1,
          loc: 10,
          inEdges: 0,
          outEdges: 0,
          sccMember: false,
          label: "isolated-submodule",
        },
      ],
      acyclic: true,
    });
    const result = detect(
      canonPath,
      topology({
        nodes: { "src/a.ts": { loc: 10 }, "lib/b.ts": { loc: 10 } },
        sccs: [{ members: ["src/a.ts", "lib/b.ts"] }],
        crossSubmoduleEdges: [
          { from: "src", to: "lib", count: 1 },
          { from: "lib", to: "src", count: 1 },
        ],
      }),
    );

    expect(result.status).toBe("drift");
    expect(result.drifts).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          category: "submodule-added",
          family: "added",
          identity: "lib",
        }),
        expect.objectContaining({
          category: "submodule-removed",
          family: "removed",
          identity: "gone",
        }),
        expect.objectContaining({
          category: "scc-status-changed",
          family: "structural-status-changed",
          identity: "src",
          canon: expect.objectContaining({ sccMember: false }),
          fresh: expect.objectContaining({ sccMember: true }),
        }),
      ]),
    );
    expect(
      result.drifts.every((drift) => drift.kind === "topology-drift"),
    ).toBe(true);
  });

  it("Y-6. rejects internally inconsistent canon SCC sections instead of emitting false SCC drift", () => {
    const canonPath = writeCanon("scc-disagreement.md", {
      submodules: [
        {
          name: "src",
          files: 1,
          loc: 10,
          inEdges: 1,
          outEdges: 1,
          sccMember: false,
          label: "shared-submodule",
        },
      ],
      acyclic: false,
      cycles: [{ members: ["src/a.ts"] }],
    });
    const result = detect(
      canonPath,
      topology({
        nodes: { "src/a.ts": { loc: 10 } },
        sccs: [{ members: ["src/a.ts"] }],
        crossSubmoduleEdges: [],
      }),
    );

    expect(result.status).toBe("parse-error");
    expect(result.drifts).toEqual([]);
    expect(JSON.stringify(result.diagnostics)).toMatch(/scc/i);
  });

  it("Y-7/Y-8/Y-9/Y-13. detects oversize and cross-edge drift with category-owned identity shapes", () => {
    const canonPath = writeCanon("edge-size-drift.md", {
      submodules: [
        {
          name: "src",
          files: 1,
          loc: 10,
          inEdges: 0,
          outEdges: 1,
          sccMember: false,
          label: "leaf-submodule",
        },
      ],
      acyclic: true,
      crossEdges: [{ from: "src", to: "old", count: 3 }],
      oversize: [{ file: "old-big.ts", loc: 500, label: "oversize" }],
    });
    const result = detect(
      canonPath,
      topology({
        nodes: { "src/a.ts": { loc: 10 } },
        crossSubmoduleEdges: [{ from: "src", to: "new", count: 7 }],
        largestFiles: [{ file: "new-huge.ts", loc: 1200 }],
      }),
    );

    expect(result.drifts).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          category: "oversize-changed",
          family: "content-shifted",
          identity: "old-big.ts",
        }),
        expect.objectContaining({
          category: "oversize-changed",
          family: "content-shifted",
          identity: "new-huge.ts",
        }),
        expect.objectContaining({
          category: "cross-edge-added",
          family: "added",
          identity: "src → new",
          fresh: expect.objectContaining({ count: 7 }),
        }),
        expect.objectContaining({
          category: "cross-edge-removed",
          family: "removed",
          identity: "src → old",
          canon: expect.objectContaining({ count: 3 }),
        }),
      ]),
    );
    expect(result.reportMarkdown).toContain("Display scope");
    expect(result.reportMarkdown).toContain("top-30");
  });

  it("Y-10/Y-11/Y-14. sorts fresh top-30 edges, prefers structured edges, and preserves clean/canonical label behavior", () => {
    const canonPath = writeCanon("clean.md", {
      submodules: [
        {
          name: "src",
          files: 1,
          loc: 10,
          inEdges: 0,
          outEdges: 0,
          sccMember: false,
          label: "isolated-submodule",
        },
      ],
      acyclic: true,
    });
    const clean = detect(
      canonPath,
      topology({
        nodes: { "src/a.ts": { loc: 10 } },
        crossSubmoduleEdges: [],
        crossSubmoduleTop: [{ edge: "stale → src", count: 99 }],
      }),
    );
    expect(clean.status).toBe("clean");
    expect(clean.drifts).toHaveLength(0);
    expect(clean.reportMarkdown).toContain("## 1. Summary");

    const crowdedCanon = writeCanon("crowded.md", {
      submodules: [
        {
          name: "hub",
          files: 1,
          loc: 1,
          inEdges: 0,
          outEdges: 0,
          sccMember: false,
          label: "isolated-submodule",
        },
      ],
      acyclic: true,
    });
    const edges = Array.from({ length: 31 }, (_, index) => ({
      from: `s${index}`,
      to: `s${index + 1}`,
      count: index === 30 ? 1 : 100,
    })).reverse();
    const crowded = detect(
      crowdedCanon,
      topology({
        nodes: { "hub/index.ts": { loc: 1 } },
        crossSubmoduleEdges: edges,
      }),
    );
    const addedEdges = crowded.drifts.filter(
      (drift) => drift.category === "cross-edge-added",
    );
    expect(addedEdges).toHaveLength(30);
    expect(addedEdges.some((drift) => drift.identity === "s30 → s31")).toBe(
      false,
    );
    expect(TOPOLOGY_LABEL_SET.size).toBe(8);
    expect(TOPOLOGY_LABEL_SET.has("forbidden-cycle")).toBe(true);
  });
});
