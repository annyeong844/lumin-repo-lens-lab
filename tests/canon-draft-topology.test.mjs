import { describe, expect, it } from "vitest";

import {
  TOPOLOGY_LABELS,
  TOPOLOGY_UNCERTAIN_REASONS,
} from "../_lib/canon-draft-utils.mjs";
import {
  classifyTopologyFile,
  classifyTopologyScc,
  classifyTopologySubmodule,
} from "../_lib/canon-draft-topology.mjs";

function classifySubmodule(input) {
  return classifyTopologySubmodule({
    name: "x",
    inDegree: 0,
    outDegree: 0,
    sccMember: false,
    crossEdgeSource: "full-list",
    ...input,
  });
}

describe("topology canon classifiers", () => {
  it("S-R0. cyclic-submodule wins over shared, isolated, and leaf patterns", () => {
    expect(
      classifySubmodule({
        name: "core",
        inDegree: 20,
        outDegree: 5,
        sccMember: true,
      }),
    ).toEqual({ label: "cyclic-submodule", marker: "❌" });
    expect(
      classifySubmodule({
        name: "orphan",
        inDegree: 0,
        outDegree: 0,
        sccMember: true,
      }).label,
    ).toBe("cyclic-submodule");
    expect(
      classifySubmodule({
        name: "leafy",
        inDegree: 1,
        outDegree: 10,
        sccMember: true,
      }).label,
    ).toBe("cyclic-submodule");
  });

  it("S-R1/S-R2/S-R3/S-R4. applies isolated, shared, leaf, and scoped rules in order", () => {
    expect(
      classifySubmodule({
        inDegree: 0,
        outDegree: 0,
        crossEdgeSource: "full-list",
      }),
    ).toEqual({ label: "isolated-submodule", marker: "ℹ" });
    expect(
      classifySubmodule({
        inDegree: 0,
        outDegree: 0,
        crossEdgeSource: "top-30-only",
      }).label,
    ).toBe("scoped-submodule");
    expect(classifySubmodule({ inDegree: 5, outDegree: 10 }).label).toBe(
      "shared-submodule",
    );
    expect(classifySubmodule({ inDegree: 4, outDegree: 1 }).label).not.toBe(
      "shared-submodule",
    );
    expect(classifySubmodule({ inDegree: 1, outDegree: 8 })).toEqual({
      label: "leaf-submodule",
      marker: "⚠",
    });
    expect(classifySubmodule({ inDegree: 2, outDegree: 2 })).toEqual({
      label: "scoped-submodule",
      marker: "ℹ",
    });
  });

  it("SCC. keeps the v1 SCC classifier as forbidden-cycle", () => {
    expect(
      classifyTopologyScc({ sccIndex: 0, members: ["a.ts", "b.ts"] }),
    ).toEqual({
      label: "forbidden-cycle",
      marker: "❌",
    });
    expect(
      classifyTopologyScc({
        sccIndex: 5,
        members: Array.from({ length: 10 }, (_, i) => `f${i}.ts`),
      }).label,
    ).toBe("forbidden-cycle");
    expect(classifyTopologyScc({ sccIndex: 99, members: [] }).label).toBe(
      "forbidden-cycle",
    );
  });

  it("F. preserves file-size thresholds and defensive non-number handling", () => {
    expect(classifyTopologyFile({ file: "empty.ts", loc: 0 })).toBeNull();
    expect(classifyTopologyFile({ file: "small.ts", loc: 399 })).toBeNull();
    expect(classifyTopologyFile({ file: "big.ts", loc: 400 })).toEqual({
      label: "oversize",
      marker: "⚠",
    });
    expect(classifyTopologyFile({ file: "big.ts", loc: 999 }).label).toBe(
      "oversize",
    );
    expect(classifyTopologyFile({ file: "huge.ts", loc: 1000 })).toEqual({
      label: "extreme-oversize",
      marker: "❌",
    });
    expect(classifyTopologyFile({ file: "weird.ts", loc: "400" })).toBeNull();
    expect(classifyTopologyFile({ file: "unknown.ts" })).toBeNull();
  });

  it("CONST. exposes frozen canonical topology label and uncertainty sets", () => {
    expect(TOPOLOGY_LABELS).toHaveLength(8);
    expect(Object.isFrozen(TOPOLOGY_LABELS)).toBe(true);
    expect(TOPOLOGY_LABELS).toEqual(
      expect.arrayContaining([
        "cyclic-submodule",
        "shared-submodule",
        "forbidden-cycle",
        "extreme-oversize",
      ]),
    );
    expect(TOPOLOGY_UNCERTAIN_REASONS).toHaveLength(3);
    expect(Object.isFrozen(TOPOLOGY_UNCERTAIN_REASONS)).toBe(true);
  });
});
