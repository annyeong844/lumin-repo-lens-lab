import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { renderHelperRegistry } from "../_lib/canon-draft-helpers.mjs";
import { renderNaming } from "../_lib/canon-draft-naming.mjs";
import { renderTopology } from "../_lib/canon-draft-topology.mjs";
import { renderTypeOwnership } from "../_lib/canon-draft-types.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const driftText = readFileSync(
  path.join(ROOT, "canonical", "canon-drift.md"),
  "utf8",
);

function extractHeaderColumns(markdown, startAfter = 0) {
  const lines = markdown.split(/\r?\n/);
  for (let i = startAfter; i < lines.length; i += 1) {
    const line = lines[i];
    if (!line.trimStart().startsWith("|")) continue;
    if (/^\s*\|[\s:|-]+\|\s*$/.test(line)) continue;
    const cells = line
      .split("|")
      .map((cell) => cell.trim())
      .filter(Boolean);
    if (cells.length >= 2 && cells.every((cell) => /^[A-Z]/.test(cell))) {
      return { cells, lineIndex: i };
    }
  }
  return null;
}

function extractHeaderUnder(markdown, headingText) {
  const idx = markdown.indexOf(headingText);
  if (idx < 0) return null;
  return extractHeaderColumns(
    markdown,
    markdown.slice(0, idx).split(/\r?\n/).length,
  );
}

function expectCanonDriftColumns(columns, section) {
  for (const column of columns) {
    expect(driftText, `${section} lists column ${column}`).toContain(
      `\`${column}\``,
    );
  }
}

describe("canon drift renderer table contract", () => {
  it("RT-T. type ownership renderer header matches canon-drift section 5.a", () => {
    const ownerFile = "src/foo.ts";
    const exportedName = "Foo";
    const identity = `${ownerFile}::${exportedName}`;
    const typeDefsByIdentity = new Map([
      [
        identity,
        {
          name: exportedName,
          ownerFile,
          line: 10,
          fanIn: 1,
          kind: "alias",
          anyContamination: undefined,
        },
      ],
    ]);
    const identitiesByName = new Map([[exportedName, [identity]]]);
    const typeUsesByIdentity = new Map([[identity, { fanIn: 1 }]]);

    const md = renderTypeOwnership({
      typeDefsByIdentity,
      identitiesByName,
      typeUsesByIdentity,
      diagnostics: [],
      meta: { scope: "test-scope", source: "fixture" },
    });

    const header = extractHeaderColumns(md);
    const expected = [
      "Name",
      "Identity",
      "Owner",
      "Fan-in",
      "Fan-in space",
      "Status",
      "Tags",
    ];

    expect(header).not.toBeNull();
    expect(header.cells).toEqual(expected);
    expectCanonDriftColumns(expected, "section 5.a");
  });

  it("RT-H. helper registry renderer header matches canon-drift section 5.b", () => {
    const ownerFile = "src/foo.ts";
    const exportedName = "doFoo";
    const identity = `${ownerFile}::${exportedName}`;
    const helperDefsByIdentity = new Map([
      [
        identity,
        {
          name: exportedName,
          ownerFile,
          line: 20,
          fanIn: 1,
          kind: "function",
          signature: "(x: number) => number",
          paramCount: 1,
          returnKind: "primitive",
          anyContamination: undefined,
        },
      ],
    ]);
    const helpersByName = new Map([[exportedName, [identity]]]);

    const md = renderHelperRegistry({
      helperDefsByIdentity,
      helpersByName,
      distinctConsumerFiles: new Map(),
      diagnostics: [],
      meta: { scope: "test-scope", source: "fixture" },
    });

    const header = extractHeaderColumns(md);
    const expected = [
      "Name",
      "Identity",
      "Owner",
      "Signature",
      "Fan-in",
      "Status",
      "Tags",
      "Any / unknown signal",
    ];

    expect(header).not.toBeNull();
    expect(header.cells).toEqual(expected);
    expectCanonDriftColumns(expected, "section 5.b");
  });

  it("RT-Y. topology renderer headers match canon-drift section 5.c", () => {
    const submodulesByPath = new Map([
      [
        "src/lib",
        {
          name: "src/lib",
          files: 1,
          loc: 50,
          inDegree: 0,
          outDegree: 0,
          sccMember: false,
        },
      ],
    ]);

    const md = renderTopology({
      submodulesByPath,
      crossEdgesForDisplay: [{ from: "src/a", to: "src/b", count: 2 }],
      sccs: [],
      oversizeFiles: [
        { file: "src/giant.ts", loc: 450, label: "oversize", marker: "!" },
      ],
      workspaces: null,
      diagnostics: [],
      meta: {
        scope: "test-scope",
        source: "fixture",
        crossEdgeSource: "full-list",
      },
    });

    const inventory = extractHeaderUnder(md, "## 1. Submodule inventory");
    const crossEdges = extractHeaderUnder(md, "## 2. Cross-submodule edges");
    const oversize = extractHeaderUnder(md, "## 4. Oversize files");
    const expectedInventory = [
      "Submodule",
      "Files",
      "LOC",
      "In-edges",
      "Out-edges",
      "SCC",
      "Status",
      "Tags",
    ];
    const expectedCrossEdges = ["From", "To", "Count"];
    const expectedOversize = ["File", "LOC", "Status"];

    expect(inventory).not.toBeNull();
    expect(inventory.cells).toEqual(expectedInventory);
    expect(crossEdges).not.toBeNull();
    expect(crossEdges.cells).toEqual(expectedCrossEdges);
    expect(oversize).not.toBeNull();
    expect(oversize.cells).toEqual(expectedOversize);
    expectCanonDriftColumns(
      [...expectedInventory, ...expectedCrossEdges, ...expectedOversize],
      "section 5.c",
    );
  });

  it("RT-N. naming renderer headers match canon-drift section 5.d", () => {
    const fileCohort = {
      cohortId: "src/lib",
      members: [{ file: "src/lib/a.ts" }, { file: "src/lib/b.ts" }],
      classification: {
        label: "consistent-kebab-case",
        marker: "ok",
        consistencyRate: 1.0,
        dominantConvention: "kebab-case",
      },
    };
    const symbolCohort = {
      cohortId: "src/lib::helper-export",
      members: [{ name: "doA" }, { name: "doB" }],
      classification: {
        label: "consistent-camelCase",
        marker: "ok",
        consistencyRate: 1.0,
        dominantConvention: "camelCase",
      },
    };

    const md = renderNaming({
      fileCohorts: new Map([[fileCohort.cohortId, fileCohort]]),
      symbolCohorts: new Map([[symbolCohort.cohortId, symbolCohort]]),
      perItemRows: [
        {
          cohortId: "src/lib",
          itemLabel: "convention-outlier",
          identity: "src/lib/WEIRD.ts",
          cohort: "src/lib",
          name: "WEIRD.ts",
          observedConvention: "UPPERCASE",
          dominantConvention: "kebab-case",
          status: "outlier",
        },
      ],
      diagnostics: [],
      meta: { scope: "test-scope", source: "fixture" },
    });

    const fileHeader = extractHeaderUnder(md, "## 1. File-naming cohorts");
    const symbolHeader = extractHeaderUnder(md, "## 2. Symbol-naming cohorts");
    const expectedFile = [
      "Cohort (submodule)",
      "Files",
      "DominantConvention",
      "ConsistencyRate",
      "OutliersCount",
      "Status",
    ];
    const expectedSymbol = [
      "Cohort (submodule::kind)",
      "Items",
      "DominantConvention",
      "ConsistencyRate",
      "OutliersCount",
      "Status",
    ];

    expect(fileHeader).not.toBeNull();
    expect(fileHeader.cells).toEqual(expectedFile);
    expect(symbolHeader).not.toBeNull();
    expect(symbolHeader.cells).toEqual(expectedSymbol);
    expectCanonDriftColumns(
      [
        "Cohort (submodule)",
        "Files",
        "DominantConvention",
        "ConsistencyRate",
        "OutliersCount",
        "Status",
        "Cohort (submodule::kind)",
        "Items",
      ],
      "section 5.d",
    );
  });
});
