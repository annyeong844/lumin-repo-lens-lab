import { describe, expect, it } from "vitest";

import {
  LOW_INFO_HELPER_NAMES,
  LOW_INFO_NAMES,
} from "../_lib/canon-draft-utils.mjs";
import {
  collectNamingCohorts,
  renderNaming,
} from "../_lib/canon-draft-naming.mjs";

const ROOT = "/fx";
const LOW_INFO_ALL = new Set(LOW_INFO_NAMES);
const LOW_INFO_HELP = new Set(LOW_INFO_HELPER_NAMES);

function makeExtractFn(perFile) {
  return (absFile) =>
    perFile.get(absFile) ?? { defs: [], uses: [], reExports: [] };
}

function makeSubmoduleOf(resolves) {
  return (absFile) => resolves.get(absFile) ?? "root";
}

function collect({ files, perFile = new Map(), resolves, extractFn }) {
  return collectNamingCohorts({
    files,
    root: ROOT,
    extractFn: extractFn ?? makeExtractFn(perFile),
    submoduleOf: makeSubmoduleOf(
      resolves ?? new Map(files.map((file) => [file, "_lib"])),
    ),
    lowInfoNames: LOW_INFO_ALL,
    lowInfoHelperNames: LOW_INFO_HELP,
  });
}

function render(result, meta = {}) {
  return renderNaming({
    ...result,
    meta: { ...result.meta, scope: "TS/JS including tests", ...meta },
  });
}

describe("naming cohort aggregation", () => {
  it("I1. records a file cohort and surfaces a snake_case file outlier", () => {
    const files = [
      "/fx/_lib/canon-draft.mjs",
      "/fx/_lib/alias-map.mjs",
      "/fx/_lib/extract-ts.mjs",
      "/fx/_lib/resolver-core.mjs",
      "/fx/_lib/legacy_helper.mjs",
    ];
    const result = collect({ files });

    expect([...result.fileCohorts.keys()]).toEqual(["_lib"]);
    expect(result.fileCohorts.get("_lib").members).toHaveLength(5);
    expect(result.fileCohorts.get("_lib").classification).toMatchObject({
      label: "kebab-case-dominant",
      consistencyRate: expect.closeTo(0.8),
    });
    expect(result.perItemRows).toEqual([
      expect.objectContaining({
        identity: "_lib/legacy_helper.mjs",
        itemLabel: "convention-outlier",
      }),
    ]);
  });

  it("I2/I3. splits symbol cohorts by type, helper, and constant ownership", () => {
    const file = "/fx/_lib/mixed.mjs";
    const result = collect({
      files: [file],
      perFile: new Map([
        [
          file,
          {
            defs: [
              { name: "UserType", kind: "TSInterfaceDeclaration", line: 1 },
              { name: "FooAlias", kind: "TSTypeAliasDeclaration", line: 2 },
              { name: "MAX_RETRY", kind: "const-var", line: 3 },
              { name: "DEFAULT", kind: "const-var", line: 4 },
              {
                name: "parseIt",
                kind: "const-var",
                line: 5,
                initType: "ArrowFunctionExpression",
              },
              { name: "helperFn", kind: "FunctionDeclaration", line: 6 },
            ],
            uses: [],
            reExports: [],
          },
        ],
      ]),
    });

    expect(result.symbolCohorts.get("_lib::type-export").members).toHaveLength(
      2,
    );
    expect(
      result.symbolCohorts.get("_lib::helper-export").members,
    ).toHaveLength(2);
    expect(
      result.symbolCohorts.get("_lib::constant-export").members,
    ).toHaveLength(2);
  });

  it("I2. classifies symbol cohort dominance and symbol outlier identity", () => {
    const file = "/fx/_lib/u.mjs";
    const result = collect({
      files: [file],
      perFile: new Map([
        [
          file,
          {
            defs: [
              "parseJson",
              "stringifyJson",
              "fetchData",
              "renderThing",
              "computeThing",
              "doTheThing",
              "mkLogger",
              "validateInput",
              "normalizePath",
              "MyBadFunc",
            ].map((name, index) => ({
              name,
              kind: "FunctionDeclaration",
              line: index + 1,
            })),
            uses: [],
            reExports: [],
          },
        ],
      ]),
    });

    expect(result.symbolCohorts.get("_lib::helper-export")).toMatchObject({
      members: expect.arrayContaining([
        expect.objectContaining({ name: "MyBadFunc" }),
      ]),
      classification: {
        label: "camelCase-dominant",
        dominantConvention: "camelCase",
        consistencyRate: expect.closeTo(0.9),
      },
    });
    expect(result.perItemRows.find((row) => row.name === "MyBadFunc")).toEqual(
      expect.objectContaining({
        identity: "_lib/u.mjs::MyBadFunc",
        itemLabel: "convention-outlier",
      }),
    );
  });

  it("I4/I5. file cohorts include no-export and parse-error files while diagnostics stay visible", () => {
    const files = ["/fx/_lib/ok.mjs", "/fx/_lib/broken.mjs"];
    const result = collect({
      files,
      extractFn: (absFile) => {
        if (absFile === "/fx/_lib/broken.mjs") {
          throw new Error("simulated parse error");
        }
        return {
          defs: [{ name: "okFn", kind: "FunctionDeclaration", line: 1 }],
          uses: [],
          reExports: [],
        };
      },
    });

    expect(result.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          reason: "parse-error",
          target: "_lib/broken.mjs",
        }),
      ]),
    );
    expect(
      result.symbolCohorts.get("_lib::helper-export").members,
    ).toHaveLength(1);
    expect(result.fileCohorts.get("_lib").members).toHaveLength(2);
  });

  it("I6/I7. low-info item rows and metadata counts stay explicit", () => {
    const file = "/fx/_lib/u.mjs";
    const result = collect({
      files: [file],
      perFile: new Map([
        [
          file,
          {
            defs: [
              "parseJson",
              "stringifyJson",
              "fetchData",
              "renderThing",
              "get",
            ].map((name, index) => ({
              name,
              kind: "FunctionDeclaration",
              line: index + 1,
            })),
            uses: [],
            reExports: [],
          },
        ],
      ]),
    });

    expect(result.perItemRows.find((row) => row.name === "get")).toMatchObject({
      itemLabel: "low-info-excluded",
    });
    expect(
      result.symbolCohorts.get("_lib::helper-export").classification,
    ).toMatchObject({ label: "camelCase-dominant" });
    expect(result.meta).toMatchObject({
      filesScanned: 1,
      fileCohortCount: 1,
      symbolCohortCount: 1,
    });
  });
});

describe("naming renderer", () => {
  it("R1. renders file cohorts, symbol cohorts, outliers, and identity shape metadata", () => {
    const file = "/fx/_lib/canon-draft.mjs";
    const result = collect({
      files: [
        "/fx/_lib/canon-draft.mjs",
        "/fx/_lib/alias-map.mjs",
        "/fx/_lib/extract-ts.mjs",
      ],
      perFile: new Map([
        [
          file,
          {
            defs: [
              { name: "foo", kind: "FunctionDeclaration", line: 1 },
              { name: "bar", kind: "FunctionDeclaration", line: 2 },
              { name: "baz", kind: "FunctionDeclaration", line: 3 },
              { name: "BAD_NAME", kind: "FunctionDeclaration", line: 4 },
            ],
            uses: [],
            reExports: [],
          },
        ],
      ]),
    });
    const md = render(result);

    expect(md).toContain("## 1. File-naming cohorts");
    expect(md).toContain("`_lib`");
    expect(md).toContain("## 2. Symbol-naming cohorts");
    expect(md).toContain("`_lib::helper-export`");
    expect(md).toContain("## 3. Outliers");
    expect(md).toContain("BAD_NAME");
    expect(md).toContain("CohortIdentityShape: submodule | submodule::kind");
  });

  it("R2/R4. omits the outlier section for clean empty or zero-outlier drafts", () => {
    const clean = render(
      collect({
        files: ["/fx/_lib/a.mjs", "/fx/_lib/b.mjs", "/fx/_lib/c.mjs"],
        perFile: new Map(
          ["/fx/_lib/a.mjs", "/fx/_lib/b.mjs", "/fx/_lib/c.mjs"].map((file) => [
            file,
            { defs: [], uses: [], reExports: [] },
          ]),
        ),
      }),
    );
    const empty = render(
      collect({
        files: [],
        resolves: new Map(),
      }),
      { scope: "x" },
    );

    expect(clean).not.toContain("## 3. Outliers");
    expect(empty).toContain("# Naming conventions draft");
    expect(empty).toContain("_No file-naming cohorts observed._");
    expect(empty).not.toContain("## 3. Outliers");
  });

  it("R3. existingCanon renders an observational naming header", () => {
    const md = render(collect({ files: [], resolves: new Map() }), {
      existingCanon: true,
      scope: "x",
    });

    expect(md).toContain("⚠ Existing canon detected");
    expect(md).toContain("naming.md");
  });
});
