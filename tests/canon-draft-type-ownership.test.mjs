import { describe, expect, it } from "vitest";

import {
  collectTypeIdentities,
  renderTypeOwnership,
} from "../_lib/canon-draft-types.mjs";

function makeSymbols({
  defIndex = {},
  fanInByIdentity = {},
  fanInByIdentitySpace = {},
  reExportsByFile = {},
} = {}) {
  return {
    meta: {
      tool: "build-symbol-graph.mjs",
      generated: "2026-04-21T00:00:00Z",
      root: "/fake",
      supports: {
        identityFanIn: true,
        identityFanInSpace: true,
        reExportRecords: "file-level",
      },
    },
    defIndex,
    fanInByIdentity,
    fanInByIdentitySpace,
    reExportsByFile,
  };
}

function typeDef(name, kind, line) {
  return { [name]: { name, kind, line } };
}

function makeHash(ch) {
  return `sha256:${ch.repeat(64)}`;
}

function makeShapeIndex(facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    groupsByHash[fact.hash] ??= [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  return {
    schemaVersion: "shape-index.v1",
    meta: { complete },
    facts,
    groupsByHash,
    diagnostics: [],
  };
}

function collect(symbols) {
  return collectTypeIdentities({ symbols, root: "/fake" });
}

function render(result, options = {}) {
  return renderTypeOwnership({
    typeDefsByIdentity: result.typeDefsByIdentity,
    identitiesByName: result.identitiesByName,
    typeUsesByIdentity: result.typeUsesByIdentity,
    diagnostics: result.diagnostics,
    meta: { scope: "TS/JS including tests", source: "symbols.json" },
    ...options,
  });
}

describe("type ownership aggregation", () => {
  it("I1/I1f. aggregates a single type owner and renders fan-in space", () => {
    const symbols = makeSymbols({
      defIndex: {
        "src/types.ts": typeDef("User", "TSInterfaceDeclaration", 10),
      },
      fanInByIdentity: { "src/types.ts::User": 5 },
      fanInByIdentitySpace: {
        "src/types.ts::User": { value: 1, type: 4, broad: 0 },
      },
    });
    const result = collect(symbols);
    const def = result.typeDefsByIdentity.get("src/types.ts::User");
    const md = render(result);

    expect(result.typeDefsByIdentity.size).toBe(1);
    expect(result.identitiesByName.get("User")).toHaveLength(1);
    expect(def).toMatchObject({
      fanIn: 5,
      fanInSpace: { value: 1, type: 4, broad: 0 },
    });
    expect(md).toContain("single-owner-strong");
    expect(md).toContain("`src/types.ts::User`");
    expect(md).toContain("| 5 | value 1, type 4, broad 0 |");
  });

  it("I2/I3/I4. keeps duplicate identities distinct and applies duplicate labels", () => {
    const strong = collect(
      makeSymbols({
        defIndex: {
          "src/a.ts": typeDef("Result", "TSTypeAliasDeclaration", 5),
          "src/b.ts": typeDef("Result", "TSTypeAliasDeclaration", 5),
        },
        fanInByIdentity: {
          "src/a.ts::Result": 18,
          "src/b.ts::Result": 3,
        },
      }),
    );
    expect(strong.identitiesByName.get("Result")).toHaveLength(2);
    expect(render(strong)).toContain("DUPLICATE_STRONG");

    const localCommon = collect(
      makeSymbols({
        defIndex: {
          "src/a.ts": typeDef("Props", "TSInterfaceDeclaration", 5),
          "src/b.ts": typeDef("Props", "TSInterfaceDeclaration", 5),
        },
        fanInByIdentity: {
          "src/a.ts::Props": 1,
          "src/b.ts::Props": 2,
        },
      }),
    );
    expect(render(localCommon)).toContain("LOCAL_COMMON_NAME");

    const crossFile = collect(
      makeSymbols({
        defIndex: {
          "apps/admin/types.ts": typeDef("User", "TSInterfaceDeclaration", 3),
          "apps/blog/types.ts": typeDef("User", "TSInterfaceDeclaration", 3),
        },
        fanInByIdentity: {
          "apps/admin/types.ts::User": 5,
          "apps/blog/types.ts::User": 5,
        },
      }),
    );
    expect(crossFile.typeDefsByIdentity.has("apps/admin/types.ts::User")).toBe(
      true,
    );
    expect(crossFile.typeDefsByIdentity.has("apps/blog/types.ts::User")).toBe(
      true,
    );
    expect(crossFile.identitiesByName.get("User")).toHaveLength(2);
    expect(render(crossFile)).toContain("apps/admin/types.ts::User");
    expect(render(crossFile)).toContain("apps/blog/types.ts::User");
  });

  it("I5/I6. uses exportedName and terminal owner identity through aliases and barrels", () => {
    const aliased = collect(
      makeSymbols({
        defIndex: {
          "src/weird.ts": {
            PublicName: {
              name: "PublicName",
              kind: "TSTypeAliasDeclaration",
              line: 1,
              typeName: "InternalLocal",
            },
          },
        },
        fanInByIdentity: { "src/weird.ts::PublicName": 2 },
      }),
    );
    const md = render(aliased);
    expect(aliased.typeDefsByIdentity.has("src/weird.ts::PublicName")).toBe(
      true,
    );
    expect(aliased.typeDefsByIdentity.has("src/weird.ts::InternalLocal")).toBe(
      false,
    );
    expect(md).toContain("src/weird.ts::PublicName");
    expect(md).not.toContain("src/weird.ts::InternalLocal");

    const reExported = collect(
      makeSymbols({
        defIndex: { "src/y.ts": typeDef("X", "TSInterfaceDeclaration", 1) },
        fanInByIdentity: { "src/y.ts::X": 3 },
        reExportsByFile: { "src/index.ts": [{ source: "./y", line: 1 }] },
      }),
    );
    expect(reExported.typeDefsByIdentity.has("src/y.ts::X")).toBe(true);
    expect(reExported.typeDefsByIdentity.has("src/index.ts::X")).toBe(false);
    expect(
      Array.from(
        reExported.typeUsesByIdentity.get("src/y.ts::X").reExportedThrough,
      ),
    ).toContain("src/index.ts");
  });

  it("I7/I8. filters to type declarations and preserves severe contamination labels", () => {
    const contaminated = collect(
      makeSymbols({
        defIndex: {
          "src/big.ts": typeDef("BigBlob", "TSInterfaceDeclaration", 1),
        },
        fanInByIdentity: { "src/big.ts::BigBlob": 100 },
      }),
    );
    contaminated.typeDefsByIdentity.get(
      "src/big.ts::BigBlob",
    ).anyContamination = {
      label: "severely-any-contaminated",
      labels: ["any-contaminated", "severely-any-contaminated"],
      measurements: {
        totalFields: 3,
        anyFields: 3,
        unknownFields: 0,
        anyFieldRatio: 1,
        indexSignatureAny: false,
      },
    };
    expect(render(contaminated)).toContain("severely-any-contaminated");

    const mixed = collect(
      makeSymbols({
        defIndex: {
          "src/mixed.ts": {
            MyType: {
              name: "MyType",
              kind: "TSTypeAliasDeclaration",
              line: 1,
            },
            myFunc: { name: "myFunc", kind: "FunctionDeclaration", line: 10 },
            myConst: { name: "myConst", kind: "VariableDeclaration", line: 20 },
          },
        },
        fanInByIdentity: {
          "src/mixed.ts::MyType": 2,
          "src/mixed.ts::myFunc": 2,
          "src/mixed.ts::myConst": 2,
        },
      }),
    );
    expect([...mixed.typeDefsByIdentity.keys()]).toEqual([
      "src/mixed.ts::MyType",
    ]);
  });

  it("I9/I10. renders markdown safely for pipes and empty inventories", () => {
    const weird = collect(
      makeSymbols({
        defIndex: {
          "src/with|pipe.ts": typeDef(
            "Weird|Name",
            "TSTypeAliasDeclaration",
            1,
          ),
        },
        fanInByIdentity: { "src/with|pipe.ts::Weird|Name": 2 },
      }),
    );
    const weirdMd = render(weird);
    expect(weirdMd).toContain("Name");
    expect(weirdMd).toContain("Identity");
    expect(weirdMd).toContain("Fan-in");
    expect(weirdMd).toContain("Status");
    expect(weirdMd).toContain("TS/JS including tests");
    expect(weirdMd).toMatch(/\\\||`src\/with\|pipe\.ts::Weird\|Name`/);

    const empty = collect(makeSymbols());
    expect(render(empty)).toMatch(/^# Type ownership draft/);
    expect(empty.typeDefsByIdentity.size).toBe(0);
  });
});

describe("type ownership shape evidence", () => {
  it("I11/I12. adds same-shape and different-shape evidence without replacing labels", () => {
    const sameHash = makeHash("a");
    const duplicateSymbols = makeSymbols({
      defIndex: {
        "src/a.ts": typeDef("Result", "TSTypeAliasDeclaration", 5),
        "src/b.ts": typeDef("Result", "TSTypeAliasDeclaration", 7),
      },
      fanInByIdentity: {
        "src/a.ts::Result": 18,
        "src/b.ts::Result": 3,
      },
    });
    const sameShapeMd = render(collect(duplicateSymbols), {
      shapeIndex: makeShapeIndex([
        { identity: "src/a.ts::Result", hash: sameHash },
        { identity: "src/b.ts::Result", hash: sameHash },
      ]),
    });
    expect(sameShapeMd).toContain("DUPLICATE_STRONG");
    expect(sameShapeMd).toContain("same-shape evidence");
    expect(sameShapeMd).toContain(sameHash);

    const hashA = makeHash("a");
    const hashB = makeHash("b");
    const differentShapeMd = render(
      collect(
        makeSymbols({
          defIndex: {
            "src/a.ts": typeDef("Config", "TSInterfaceDeclaration", 5),
            "src/b.ts": typeDef("Config", "TSInterfaceDeclaration", 7),
          },
          fanInByIdentity: {
            "src/a.ts::Config": 2,
            "src/b.ts::Config": 1,
          },
        }),
      ),
      {
        shapeIndex: makeShapeIndex([
          { identity: "src/a.ts::Config", hash: hashA },
          { identity: "src/b.ts::Config", hash: hashB },
        ]),
      },
    );
    expect(differentShapeMd).toContain("different-shape evidence");
    expect(differentShapeMd).toContain(hashA);
    expect(differentShapeMd).toContain(hashB);
    expect(differentShapeMd).toMatch(/DUPLICATE_REVIEW|LOCAL_COMMON_NAME/);
  });

  it("I13. degrades incomplete shape-index evidence and names missing facts", () => {
    const hash = makeHash("c");
    const md = render(
      collect(
        makeSymbols({
          defIndex: {
            "src/a.ts": typeDef("Model", "TSInterfaceDeclaration", 5),
            "src/b.ts": typeDef("Model", "TSInterfaceDeclaration", 7),
          },
          fanInByIdentity: {
            "src/a.ts::Model": 4,
            "src/b.ts::Model": 1,
          },
        }),
      ),
      {
        shapeIndex: makeShapeIndex([{ identity: "src/a.ts::Model", hash }], {
          complete: false,
        }),
      },
    );

    expect(md).toContain("shape evidence degraded");
    expect(md).toContain("incomplete");
    expect(md).toContain("shape evidence partial");
    expect(md).toContain("Missing shape facts");
    expect(md).toContain("src/b.ts::Model");
  });

  it("I14/I15. summarizes generated-only evidence and fails closed on malformed generated evidence", () => {
    const generatedMd = render(
      collect(
        makeSymbols({
          defIndex: {
            "apps/a/src/routeTree.gen.ts": typeDef(
              "FileRoutesById",
              "TSInterfaceDeclaration",
              5,
            ),
            "apps/b/src/routeTree.gen.ts": typeDef(
              "FileRoutesById",
              "TSInterfaceDeclaration",
              7,
            ),
          },
          fanInByIdentity: {
            "apps/a/src/routeTree.gen.ts::FileRoutesById": 5,
            "apps/b/src/routeTree.gen.ts::FileRoutesById": 3,
          },
        }),
      ),
      {
        shapeIndex: makeShapeIndex([
          {
            identity: "apps/a/src/routeTree.gen.ts::FileRoutesById",
            hash: makeHash("d"),
            generatedFile: {
              kind: "generated-file",
              source: "path",
              evidence: "path:routeTree.gen",
            },
          },
          {
            identity: "apps/b/src/routeTree.gen.ts::FileRoutesById",
            hash: makeHash("e"),
            generatedFile: {
              kind: "generated-file",
              source: "path",
              evidence: "path:routeTree.gen",
            },
          },
        ]),
      },
    );
    expect(generatedMd).toContain("DUPLICATE_STRONG");
    expect(generatedMd).toContain("generated-shape evidence summarized");
    expect(generatedMd).not.toContain(
      "different-shape evidence: `FileRoutesById`",
    );

    const malformedMd = render(
      collect(
        makeSymbols({
          defIndex: {
            "src/a.ts": typeDef("WidgetProps", "TSInterfaceDeclaration", 5),
            "src/b.ts": typeDef("WidgetProps", "TSInterfaceDeclaration", 7),
          },
          fanInByIdentity: {
            "src/a.ts::WidgetProps": 5,
            "src/b.ts::WidgetProps": 3,
          },
        }),
      ),
      {
        shapeIndex: makeShapeIndex([
          {
            identity: "src/a.ts::WidgetProps",
            hash: makeHash("f"),
            generatedFile: true,
          },
          {
            identity: "src/b.ts::WidgetProps",
            hash: makeHash("1"),
            generatedFile: true,
          },
        ]),
      },
    );
    expect(malformedMd).toContain("shape evidence unavailable");
    expect(malformedMd).toContain("malformed-generated-file-evidence");
    expect(malformedMd).not.toContain("generated-shape evidence summarized");
  });
});
