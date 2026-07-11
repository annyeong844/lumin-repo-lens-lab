import { describe, expect, it } from "vitest";

import { lookupFile } from "../_lib/pre-write-lookup-file.mjs";

function buildTopology({ nodes = {}, edges = [], complete = false } = {}) {
  return {
    meta: {
      tool: "measure-topology.mjs",
      ...(complete ? { complete: true } : {}),
    },
    nodes,
    edges,
  };
}

function buildSymbols({ defIndex = {}, filesWithParseErrors = [] } = {}) {
  return {
    meta: {
      schemaVersion: 3,
      supports: {
        anyContamination: false,
        identityFanIn: true,
        reExportRecords: "file-level",
      },
    },
    defIndex,
    filesWithParseErrors,
  };
}

describe("pre-write file lookup status evidence", () => {
  it("establishes FILE_EXISTS only from positive topology or defIndex evidence", () => {
    const topologyResult = lookupFile("src/utils/date.ts", {
      topology: buildTopology({
        nodes: { "src/utils/date.ts": { loc: 42 } },
        complete: true,
      }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(topologyResult).toMatchObject({
      kind: "file",
      intentFile: "src/utils/date.ts",
      result: "FILE_EXISTS",
      loc: 42,
    });
    expect(topologyResult.citations.join(" ")).toContain("grounded");

    const defIndexResult = lookupFile("src/legacy/old.ts", {
      topology: null,
      symbols: buildSymbols({
        defIndex: {
          "src/legacy/old.ts": { someExport: { kind: "const", line: 1 } },
        },
      }),
      root: "/root",
    });
    expect(defIndexResult.result).toBe("FILE_EXISTS");
    expect(defIndexResult.loc).toBeNull();
    expect(defIndexResult.inboundFanIn).toBeNull();
    expect(defIndexResult.inboundFanInConfidence).toBe("unavailable");
  });

  it("requires complete topology and no parse error before returning NEW_FILE", () => {
    const newFile = lookupFile("src/utils/time.ts", {
      topology: buildTopology({
        nodes: { "src/existing.ts": { loc: 10 } },
        complete: true,
      }),
      symbols: buildSymbols({ filesWithParseErrors: [] }),
      root: "/root",
    });
    expect(newFile.result).toBe("NEW_FILE");

    const topologyAbsent = lookupFile("src/utils/time.ts", {
      topology: null,
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(topologyAbsent.result).toBe("FILE_STATUS_UNKNOWN");
    expect(topologyAbsent.citations.join(" ")).toContain("확인 불가");
    expect(topologyAbsent.citations.join(" ")).toContain("topology");

    const incompleteTopology = lookupFile("src/utils/time.ts", {
      topology: buildTopology({
        nodes: { "src/existing.ts": { loc: 10 } },
        complete: false,
      }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(incompleteTopology.result).toBe("FILE_STATUS_UNKNOWN");

    const defIndexAbsentOnly = lookupFile("src/any.ts", {
      topology: null,
      symbols: buildSymbols({ defIndex: {} }),
      root: "/root",
    });
    expect(defIndexAbsentOnly.result).toBe("FILE_STATUS_UNKNOWN");

    const parseError = lookupFile("src/broken.ts", {
      topology: buildTopology({
        nodes: { "src/clean.ts": { loc: 20 } },
        complete: true,
      }),
      symbols: buildSymbols({ filesWithParseErrors: ["src/broken.ts"] }),
      root: "/root",
    });
    expect(parseError.result).toBe("FILE_STATUS_UNKNOWN");
    expect(parseError.citations.join(" ")).toMatch(/parse/i);
  });

  it("detects domain clusters as watch cues without claiming semantic reuse", () => {
    const prefixCluster = lookupFile("lib/cardNewsService.js", {
      topology: buildTopology({
        nodes: {
          "lib/cardNewsGenerator.js": { loc: 120 },
          "lib/cardNewsPlanner.js": { loc: 80 },
          "lib/cardNewsJobStore.js": { loc: 40 },
          "lib/other.js": { loc: 10 },
        },
        complete: true,
      }),
      symbols: buildSymbols({ filesWithParseErrors: [] }),
      root: "/root",
    });
    expect(prefixCluster.result).toBe("NEW_FILE");
    expect(prefixCluster.domainCluster).toMatchObject({
      kind: "DOMAIN_CLUSTER_DETECTED",
      matchCount: 3,
      prefixPath: "lib/cardNews",
      totalLoc: 240,
    });

    const suffixCluster = lookupFile("_lib/artifact-loader.mjs", {
      topology: buildTopology({
        nodes: {
          "_lib/artifacts.mjs": { loc: 80 },
          "_lib/check-canon-artifact.mjs": { loc: 40 },
          "_lib/post-write-artifact.mjs": { loc: 30 },
          "_lib/pre-write-artifact.mjs": { loc: 30 },
          "_lib/shape-index-artifact.mjs": { loc: 120 },
          "_lib/symbol-graph-artifact.mjs": { loc: 100 },
          "_lib/other.mjs": { loc: 10 },
        },
        complete: true,
      }),
      symbols: buildSymbols({ filesWithParseErrors: [] }),
      root: "/root",
    });
    expect(suffixCluster.domainCluster).toMatchObject({
      kind: "DOMAIN_CLUSTER_DETECTED",
      matchKind: "domain-token",
      matchCount: 6,
    });
    expect(suffixCluster.domainCluster.examples.map((e) => e.file)).toEqual(
      expect.arrayContaining([
        "_lib/post-write-artifact.mjs",
        "_lib/artifacts.mjs",
      ]),
    );

    const strongPrefix = lookupFile("src/utils/merge-with-defaults.util.ts", {
      topology: buildTopology({
        nodes: {
          "src/utils/merge-with-values.util.ts": { loc: 44 },
          "src/utils/deep-merge.util.ts": { loc: 25 },
        },
        complete: true,
      }),
      symbols: buildSymbols({ filesWithParseErrors: [] }),
      root: "/root",
    });
    expect(strongPrefix.domainCluster).toMatchObject({
      kind: "DOMAIN_CLUSTER_DETECTED",
      matchCount: 1,
      prefixPath: "src/utils/mergeWith",
    });
    expect(strongPrefix.domainCluster.examples).toHaveLength(1);
    expect(strongPrefix.domainCluster.examples[0].file).toBe(
      "src/utils/merge-with-values.util.ts",
    );
  });

  it("keeps boundary status not evaluated without planned edge endpoints", () => {
    const withEdges = lookupFile("src/a.ts", {
      topology: buildTopology({
        nodes: { "src/a.ts": { loc: 1 }, "src/b.ts": { loc: 1 } },
        edges: [
          { from: "src/b.ts", to: "src/a.ts" },
          { from: "src/c.ts", to: "src/a.ts" },
        ],
        complete: true,
      }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(withEdges.inboundFanIn).toBe(2);
    expect(withEdges.inboundFanInConfidence).toBe("grounded");
    expect(withEdges.boundary.status).toBe("NOT_EVALUATED");
    expect(withEdges.boundary.rule).toBeNull();

    const blanketAllow = lookupFile("src/new.ts", {
      topology: buildTopology({ nodes: {}, complete: true }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(blanketAllow.boundary.status).toBe("NOT_EVALUATED");

    const endpointsAbsent = lookupFile("src/new.ts", {
      topology: buildTopology({ nodes: {}, complete: true }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(endpointsAbsent.boundary.status).toBe("NOT_EVALUATED");
    expect(endpointsAbsent.citations.join(" ")).toMatch(/planned-edge/i);
  });

  it("tags tests, normalizes backslashes, and exposes stable discriminator fields", () => {
    const testPath = lookupFile("tests/foo.test.ts", {
      topology: buildTopology({ nodes: {}, complete: true }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(testPath.tags).toContain("test-only");

    const backslash = lookupFile("src\\utils\\date.ts", {
      topology: buildTopology({
        nodes: { "src/utils/date.ts": { loc: 10 } },
        complete: true,
      }),
      symbols: buildSymbols(),
      root: "/root",
    });
    expect(backslash.result).toBe("FILE_EXISTS");
    expect(
      typeof backslash.submodule === "string" || backslash.submodule === null,
    ).toBe(true);
    expect(backslash.kind).toBe("file");
    expect(backslash.intentFile).toBe("src/utils/date.ts");
  });
});
