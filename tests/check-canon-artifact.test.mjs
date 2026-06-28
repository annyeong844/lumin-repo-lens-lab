import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, describe, expect, it } from "vitest";

import {
  loadHelperRegistryCanon,
  loadNamingCanon,
  loadTopologyCanon,
  loadTypeOwnershipCanon,
  writeCanonDriftArtifacts,
} from "../_lib/check-canon-artifact.mjs";
import {
  HELPER_LABEL_SET,
  NAMING_LABEL_SET,
  TOPOLOGY_LABEL_SET,
} from "../_lib/check-canon-utils.mjs";

const TYPE_LABEL_SET = new Set([
  "zero-internal-fan-in",
  "low-signal-type-name",
  "DUPLICATE_STRONG",
  "DUPLICATE_REVIEW",
  "LOCAL_COMMON_NAME",
  "single-owner-strong",
  "single-owner-weak",
  "severely-any-contaminated",
  "ANY_COLLISION",
]);

const workdir = mkdtempSync(
  path.join(tmpdir(), "vitest-check-canon-artifact-"),
);

afterAll(() => {
  rmSync(workdir, { recursive: true, force: true });
});

describe("check-canon artifact loader and writer contracts", () => {
  it("loaders distinguish missing canon files from empty drift", () => {
    const type = loadTypeOwnershipCanon({
      canonPath: path.join(workdir, "missing-type.md"),
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(type.status).toBe("skipped-missing-canon");
    expect(type.records.size).toBe(0);
    expect(type.diagnostics.some((d) => /absent|missing/i.test(d.reason))).toBe(
      true,
    );

    const helper = loadHelperRegistryCanon({
      canonPath: path.join(workdir, "missing-helper.md"),
      canonLabelSet: HELPER_LABEL_SET,
    });
    expect(helper.status).toBe("skipped-missing-canon");

    const topology = loadTopologyCanon({
      canonPath: path.join(workdir, "missing-topology.md"),
      canonLabelSet: TOPOLOGY_LABEL_SET,
    });
    expect(topology.status).toBe("skipped-missing-canon");

    const naming = loadNamingCanon({
      canonPath: path.join(workdir, "missing-naming.md"),
      canonLabelSet: NAMING_LABEL_SET,
    });
    expect(naming.status).toBe("skipped-missing-canon");
  });

  it("loaders delegate real files and preserve lineCount", () => {
    const typePath = path.join(workdir, "type-ownership.md");
    writeFileSync(
      typePath,
      "| Name | Identity | Owner | Fan-in | Status | Tags |\n" +
        "|------|----------|-------|-------:|--------|------|\n" +
        "| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | single-owner-strong ✅ | |\n",
      "utf8",
    );
    const type = loadTypeOwnershipCanon({
      canonPath: typePath,
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(type.status).toBe("clean");
    expect(type.records.size).toBe(1);
    expect(type.lineCount).toBeGreaterThan(0);

    const helperPath = path.join(workdir, "helper-registry.md");
    writeFileSync(
      helperPath,
      "| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n" +
        "|------|----------|-------|-----------|-------:|--------|------|----------------------|\n" +
        "| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | central-helper ✅ | | |\n",
      "utf8",
    );
    expect(
      loadHelperRegistryCanon({
        canonPath: helperPath,
        canonLabelSet: HELPER_LABEL_SET,
      }),
    ).toMatchObject({ status: "clean", lineCount: expect.any(Number) });

    const topologyPath = path.join(workdir, "topology.md");
    writeFileSync(
      topologyPath,
      [
        "## 1. Submodule inventory",
        "",
        "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |",
        "|-----------|------:|----:|---------:|----------:|-----|--------|------|",
        "| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |",
        "",
        "## 3. Cycles (SCCs)",
        "",
        "✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.",
        "",
      ].join("\n"),
      "utf8",
    );
    const topology = loadTopologyCanon({
      canonPath: topologyPath,
      canonLabelSet: TOPOLOGY_LABEL_SET,
    });
    expect(topology.status).toBe("clean");
    expect(topology.inventory.size).toBe(1);
    expect(topology.lineCount).toBeGreaterThan(0);

    const namingPath = path.join(workdir, "naming.md");
    writeFileSync(
      namingPath,
      [
        "## 1. File-naming cohorts",
        "",
        "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
        "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
        "| `src` | 3 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |",
        "",
        "## 2. Symbol-naming cohorts",
        "",
        "| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |",
        "|--------------------------|------:|--------------------|----------------:|--------------:|--------|",
        "",
      ].join("\n"),
      "utf8",
    );
    const naming = loadNamingCanon({
      canonPath: namingPath,
      canonLabelSet: NAMING_LABEL_SET,
    });
    expect(naming.status).toBe("clean");
    expect(naming.fileCohorts.size).toBe(1);
    expect(naming.lineCount).toBeGreaterThan(0);
  });

  it("writer emits JSON, conditionally emits Markdown, and overwrites stale foreign sources", () => {
    const outA = path.join(workdir, "out-a");
    const driftObject = {
      meta: {
        tool: "check-canon.mjs",
        generated: "2026-04-21T00:00:00Z",
        root: "/x",
        canonDir: "/x/canonical",
        scope: "fixture",
        strict: false,
      },
      summary: {
        sourcesRequested: 1,
        sourcesChecked: 1,
        sourcesSkipped: 0,
        driftCount: 0,
      },
      perSource: {
        "type-ownership": {
          status: "clean",
          driftCount: 0,
          reportPath: path.join(outA, "canon-drift.type-ownership.md"),
          diagnostics: [],
        },
      },
      drifts: [],
    };

    const written = writeCanonDriftArtifacts({
      output: outA,
      driftObject,
      reportMarkdown: "# Type-ownership canon drift\n\nclean\n",
      source: "type-ownership",
    });
    expect(existsSync(written.jsonPath)).toBe(true);
    expect(written.reportPath).toMatch(/canon-drift\.type-ownership\.md$/);
    expect(existsSync(written.reportPath)).toBe(true);

    const outB = path.join(workdir, "out-b");
    const withoutMd = writeCanonDriftArtifacts({
      output: outB,
      driftObject: {
        meta: {},
        summary: {
          sourcesRequested: 1,
          sourcesChecked: 0,
          sourcesSkipped: 1,
          driftCount: 0,
        },
        perSource: {
          "type-ownership": {
            status: "skipped-missing-canon",
            driftCount: 0,
            diagnostics: [],
          },
        },
        drifts: [],
      },
      reportMarkdown: null,
      source: "type-ownership",
    });
    expect(existsSync(withoutMd.jsonPath)).toBe(true);
    expect(withoutMd.reportPath).toBeNull();

    const outC = path.join(workdir, "out-c");
    writeCanonDriftArtifacts({
      output: outC,
      driftObject: {
        meta: {},
        summary: {
          sourcesRequested: 1,
          sourcesChecked: 1,
          sourcesSkipped: 0,
          driftCount: 5,
        },
        perSource: {
          "helper-registry": {
            status: "drift",
            driftCount: 5,
            diagnostics: [],
          },
        },
        drifts: [{ kind: "helper-drift", identity: "old" }],
      },
      reportMarkdown: null,
      source: "helper-registry",
    });
    writeCanonDriftArtifacts({
      output: outC,
      driftObject: {
        meta: {},
        summary: {
          sourcesRequested: 1,
          sourcesChecked: 1,
          sourcesSkipped: 0,
          driftCount: 0,
        },
        perSource: {
          "type-ownership": {
            status: "clean",
            driftCount: 0,
            diagnostics: [],
          },
        },
        drifts: [],
      },
      reportMarkdown: null,
      source: "type-ownership",
    });
    const after = JSON.parse(
      readFileSync(path.join(outC, "canon-drift.json"), "utf8"),
    );
    expect(after.perSource["helper-registry"]).toBeUndefined();
    expect(after.perSource["type-ownership"]).toBeTruthy();
    expect(after.drifts).toEqual([]);
  });
});
