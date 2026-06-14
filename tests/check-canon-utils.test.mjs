import { describe, expect, it } from "vitest";

import {
  buildCanonDriftJsonObject,
  CATEGORY_TO_FAMILY,
  HELPER_LABEL_SET,
  makeDriftRecord,
  NAMING_LABEL_SET,
  parseHelperRegistryCanonText,
  parseNamingCanonText,
  parseTopologyCanonText,
  parseTypeOwnershipCanonText,
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

const TYPE_HEADER =
  "| Name | Identity | Owner | Fan-in | Status | Tags |\n" +
  "|------|----------|-------|-------:|--------|------|";

const HELPER_HEADER =
  "| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n" +
  "|------|----------|-------|-----------|-------:|--------|------|----------------------|";

function buildTopologyCanon({
  submodules = [],
  acyclic = true,
  cycles = [],
  crossEdges = [],
  oversize = [],
} = {}) {
  const lines = [
    "## 1. Submodule inventory",
    "",
    "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |",
    "|-----------|------:|----:|---------:|----------:|-----|--------|------|",
  ];
  for (const submodule of submodules) {
    lines.push(
      `| \`${submodule.name}\` | ${submodule.files} | ${submodule.loc} | ${submodule.inEdges} | ${submodule.outEdges} | ${
        submodule.sccMember ? "●" : "—"
      } | ${submodule.label} ✅ | |`,
    );
  }
  lines.push("", "## 2. Cross-submodule edges (top 30)", "");
  if (crossEdges.length > 0) {
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
  if (oversize.length > 0) {
    lines.push("| File | LOC | Status |", "|------|----:|--------|");
    for (const file of oversize) {
      lines.push(`| \`${file.file}\` | ${file.loc} | ${file.label} ⚠ |`);
    }
    lines.push("");
  }
  return lines.join("\n");
}

function buildNamingCanon({
  fileCohorts = [],
  symbolCohorts = [],
  outliers = null,
}) {
  const lines = [
    "## 1. File-naming cohorts",
    "",
    "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
    "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
  ];
  for (const cohort of fileCohorts) {
    lines.push(
      `| \`${cohort.cohort}\` | ${cohort.files} | \`${cohort.convention}\` | ${cohort.rate}% | ${
        cohort.outliers ?? 0
      } | ${cohort.label} ✅ |`,
    );
  }
  lines.push(
    "",
    "## 2. Symbol-naming cohorts",
    "",
    "| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |",
    "|--------------------------|------:|--------------------|----------------:|--------------:|--------|",
  );
  for (const cohort of symbolCohorts) {
    lines.push(
      `| \`${cohort.cohort}\` | ${cohort.items} | \`${cohort.convention}\` | ${cohort.rate}% | ${
        cohort.outliers ?? 0
      } | ${cohort.label} ✅ |`,
    );
  }
  lines.push("");
  if (outliers !== null) {
    lines.push(
      "## 3. Outliers",
      "",
      "| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |",
      "|----------|--------|------|--------------------|--------------------|--------|",
    );
    for (const outlier of outliers) {
      lines.push(
        `| \`${outlier.identity}\` | \`${outlier.cohort}\` | \`${outlier.name}\` | \`${outlier.observed}\` | \`${outlier.dominant}\` | ${outlier.label} ⚠ |`,
      );
    }
    lines.push("");
  }
  return lines.join("\n");
}

describe("check-canon utility parser contracts", () => {
  it("keeps type parser strictness tiers and drift JSON shape distinct", () => {
    const empty = parseTypeOwnershipCanonText({
      text: "",
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(empty.status).toBe("skipped-unrecognized-schema");
    expect(empty.records.size).toBe(0);

    const missingIdentity = parseTypeOwnershipCanonText({
      text:
        TYPE_HEADER.replace("| Identity ", "").replace("|----------", "") +
        "\n",
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(missingIdentity.status).toBe("parse-error");
    expect(missingIdentity.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          reason: "missing-required-column",
          column: "Identity",
        }),
      ]),
    );

    const unknownStatus = parseTypeOwnershipCanonText({
      text:
        TYPE_HEADER +
        "\n| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | bogus-label-xyz | |\n",
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(unknownStatus.status).toBe("parse-error");
    expect(
      unknownStatus.diagnostics.some((d) =>
        ["canon-parse-error", "unknown-status-label"].includes(d.reason),
      ),
    ).toBe(true);

    const cleanWithFanInSpace = parseTypeOwnershipCanonText({
      text:
        "| Name | Identity | Owner | Fan-in | Fan-in space | Status | Tags |\n" +
        "|------|----------|-------|-------:|--------------|--------|------|\n" +
        "| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | value 2, type 1, broad 0 | single-owner-strong ✅ | |\n",
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(cleanWithFanInSpace.status).toBe("clean");
    expect(cleanWithFanInSpace.records.get("src/foo.ts::Foo")).toMatchObject({
      exportedName: "Foo",
      ownerFile: "src/foo.ts",
      label: "single-owner-strong",
      fanIn: 3,
    });

    const prefixMemo = parseTypeOwnershipCanonText({
      text:
        "| Name | Note |\n|------|------|\n| stuff | info |\n\n" +
        TYPE_HEADER +
        "\n| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:12` | 3 | single-owner-strong ✅ | |\n",
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(prefixMemo.status).toBe("clean");
    expect(prefixMemo.records.has("src/foo.ts::Foo")).toBe(true);

    const memoOnly = parseTypeOwnershipCanonText({
      text: "| Name | Note |\n|------|------|\n| stuff | info |\n",
      canonLabelSet: TYPE_LABEL_SET,
    });
    expect(memoOnly.status).toBe("skipped-unrecognized-schema");

    const driftRecord = makeDriftRecord({
      kind: "type-drift",
      category: "owner-changed",
      identity: "src/foo.ts::Foo",
      canon: { label: "single-owner-strong" },
      fresh: { label: "single-owner-strong" },
      confidence: "high",
    });
    expect(driftRecord.family).toBe("structural-status-changed");
    expect(Object.keys(CATEGORY_TO_FAMILY)).toHaveLength(20);

    const driftObject = buildCanonDriftJsonObject({
      meta: {
        tool: "check-canon.mjs",
        generated: "2026-04-21T00:00:00Z",
        root: "/tmp/fake",
        canonDir: "/tmp/fake/canonical",
        scope: "fixture",
        strict: false,
      },
      perSource: {
        "type-ownership": {
          status: "clean",
          driftCount: 0,
          diagnostics: [],
        },
      },
      drifts: [],
    });
    expect(driftObject.summary).toMatchObject({
      sourcesRequested: 1,
      sourcesChecked: 1,
      sourcesSkipped: 0,
      driftCount: 0,
    });
  });

  it("keeps helper parser columns, status labels, unknown-signal, and signature pipes strict", () => {
    const renamedOwner = parseHelperRegistryCanonText({
      text: HELPER_HEADER.replace("| Owner ", "| OwnerFile ") + "\n",
      canonLabelSet: HELPER_LABEL_SET,
    });
    expect(renamedOwner.status).toBe("parse-error");

    const unknownStatus = parseHelperRegistryCanonText({
      text:
        HELPER_HEADER +
        "\n| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | bogus-helper-label | | |\n",
      canonLabelSet: HELPER_LABEL_SET,
    });
    expect(unknownStatus.status).toBe("parse-error");

    const clean = parseHelperRegistryCanonText({
      text:
        HELPER_HEADER +
        "\n| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | () => void | 3 | central-helper ✅ | | |\n" +
        "| `doBar` | `src/bar.ts::doBar` | `src/bar.ts:2` | () => T | 1 | shared-helper ⚠ | | any-contaminated |\n",
      canonLabelSet: HELPER_LABEL_SET,
    });
    expect(clean.status).toBe("clean");
    expect(clean.records.get("src/foo.ts::doFoo")).toMatchObject({
      exportedName: "doFoo",
      ownerFile: "src/foo.ts",
      label: "central-helper",
      fanIn: 3,
    });
    expect(clean.records.get("src/bar.ts::doBar")?.anyUnknownSignal).toBe(
      "any-contaminated",
    );

    const signaturePipes = parseHelperRegistryCanonText({
      text:
        HELPER_HEADER +
        "\n| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:5` | `(x: string | number) => void` | 3 | central-helper ✅ | | |\n" +
        "| `doBar` | `src/bar.ts::doBar` | `src/bar.ts:2` | (x: A \\| B) => void | 1 | shared-helper ⚠ | | |\n",
      canonLabelSet: HELPER_LABEL_SET,
    });
    expect(signaturePipes.status).toBe("clean");
    expect(
      signaturePipes.records.get("src/bar.ts::doBar")?.signature,
    ).toContain("A | B");
    expect(HELPER_LABEL_SET.size).toBe(9);
    expect(
      Object.keys(CATEGORY_TO_FAMILY).filter((k) =>
        k.startsWith("helper-drift::"),
      ),
    ).toHaveLength(5);
  });

  it("keeps topology multi-section parser failures visible", () => {
    const clean = parseTopologyCanonText({
      text: buildTopologyCanon({
        submodules: [
          {
            name: "src",
            files: 3,
            loc: 100,
            inEdges: 2,
            outEdges: 1,
            sccMember: false,
            label: "shared-submodule",
          },
        ],
        crossEdges: [{ from: "src", to: "lib", count: 5 }],
        oversize: [{ file: "src/giant.ts", loc: 500, label: "oversize" }],
      }),
      canonLabelSet: TOPOLOGY_LABEL_SET,
    });
    expect(clean.status).toBe("clean");
    expect(clean.inventory.get("src")).toMatchObject({
      label: "shared-submodule",
      inEdges: 2,
      sccMember: false,
    });
    expect(clean.crossEdges.has("src → lib")).toBe(true);
    expect(clean.oversize.has("src/giant.ts")).toBe(true);

    const disagreement = parseTopologyCanonText({
      text: [
        "## 1. Submodule inventory",
        "",
        "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |",
        "|-----------|------:|----:|---------:|----------:|-----|--------|------|",
        "| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |",
        "",
        "## 3. Cycles (SCCs)",
        "",
        "❌ Cycles observed — canon invariant violation:",
        "",
        "### Cycle 1 (size 2) — forbidden-cycle ❌",
        "",
        "- `src`",
        "- `lib`",
        "",
      ].join("\n"),
      canonLabelSet: TOPOLOGY_LABEL_SET,
    });
    expect(disagreement.status).toBe("parse-error");

    const malformedCrossEdge = parseTopologyCanonText({
      text: [
        "## 1. Submodule inventory",
        "",
        "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |",
        "|-----------|------:|----:|---------:|----------:|-----|--------|------|",
        "| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |",
        "",
        "## 2. Cross-submodule edges (top 30)",
        "",
        "| From | To |",
        "|------|----|",
        "| `src` | `lib` |",
        "",
        "## 3. Cycles (SCCs)",
        "",
        "✅ No submodule-level cycles observed.",
        "",
      ].join("\n"),
      canonLabelSet: TOPOLOGY_LABEL_SET,
    });
    expect(malformedCrossEdge.status).toBe("parse-error");
    expect(TOPOLOGY_LABEL_SET.size).toBe(8);
  });

  it("keeps naming required sections, placeholder normalization, and low-info filtering distinct", () => {
    const clean = parseNamingCanonText({
      text: buildNamingCanon({
        fileCohorts: [
          {
            cohort: "src",
            files: 3,
            convention: "kebab-case",
            rate: 100,
            label: "kebab-case-dominant",
          },
        ],
        symbolCohorts: [
          {
            cohort: "src::helper-export",
            items: 2,
            convention: "camelCase",
            rate: 100,
            label: "camelCase-dominant",
          },
        ],
      }),
      canonLabelSet: NAMING_LABEL_SET,
    });
    expect(clean.status).toBe("clean");
    expect(clean.fileCohorts.has("src")).toBe(true);
    expect(clean.symbolCohorts.has("src::helper-export")).toBe(true);
    expect(clean.outliers.size).toBe(0);

    const missingSymbolSection = parseNamingCanonText({
      text: [
        "## 1. File-naming cohorts",
        "",
        "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
        "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
        "| `src` | 3 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |",
        "",
      ].join("\n"),
      canonLabelSet: NAMING_LABEL_SET,
    });
    expect(missingSymbolSection.status).toBe("parse-error");
    expect(missingSymbolSection.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ reason: "missing-required-section" }),
      ]),
    );

    const emptySymbolSection = parseNamingCanonText({
      text: [
        "## 1. File-naming cohorts",
        "",
        "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
        "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
        "| `src` | 3 | `kebab-case` | 100% | 0 | kebab-case-dominant ✅ |",
        "",
        "## 2. Symbol-naming cohorts",
        "",
        "_No symbol-naming cohorts observed._",
        "",
      ].join("\n"),
      canonLabelSet: NAMING_LABEL_SET,
    });
    expect(emptySymbolSection.status).toBe("clean");
    expect(emptySymbolSection.symbolCohorts.size).toBe(0);

    const placeholders = parseNamingCanonText({
      text: [
        "## 1. File-naming cohorts",
        "",
        "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
        "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
        "| `src` | 5 | — | 40% | 0 | mixed-convention ⚠ |",
        "",
        "## 2. Symbol-naming cohorts",
        "",
        "| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |",
        "|--------------------------|------:|--------------------|----------------:|--------------:|--------|",
        "| `src::helper-export` | 2 | `—` | — | — | insufficient-evidence ℹ |",
        "",
        "## 3. Outliers",
        "",
        "| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |",
        "|----------|--------|------|--------------------|--------------------|--------|",
        "| `src/build.ts` | `src` | `build` | `camelCase` | `camelCase` | low-info-excluded ℹ |",
        "| `src/vite.config.ts` | `src` | `vite.config` | `mixed` | — | convention-outlier ⚠ |",
        "",
      ].join("\n"),
      canonLabelSet: NAMING_LABEL_SET,
    });
    expect(placeholders.status).toBe("clean");
    expect(placeholders.fileCohorts.get("src")?.dominantConvention).toBeNull();
    expect(
      placeholders.symbolCohorts.get("src::helper-export")?.dominantConvention,
    ).toBeNull();
    expect(placeholders.outliers.has("src/build.ts")).toBe(false);
    expect(
      placeholders.outliers.get("src/vite.config.ts")?.dominantConvention,
    ).toBeNull();

    expect(NAMING_LABEL_SET.size).toBe(10);
    expect(
      Object.keys(CATEGORY_TO_FAMILY).filter((k) =>
        k.startsWith("naming-drift::"),
      ),
    ).toHaveLength(5);
  });
});
