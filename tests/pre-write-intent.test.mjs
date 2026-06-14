import { describe, expect, it } from "vitest";

import { ESCAPE_KINDS, validateIntent } from "../_lib/pre-write-intent.mjs";

function expectInvalid(input, errorPath) {
  const result = validateIntent(input);
  expect(result.ok).toBe(false);
  expect(result.errorPath).toBe(errorPath);
  return result;
}

describe("pre-write intent schema and normalization", () => {
  it("normalizes the five top-level arrays and warns when keys default", () => {
    const complete = validateIntent({
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    });
    expect(complete.ok).toBe(true);
    expect(complete.intent.names).toEqual([]);
    expect(complete.intent.shapes).toEqual([]);
    expect(complete.intent.files).toEqual([]);
    expect(complete.intent.dependencies).toEqual([]);
    expect(complete.intent.plannedTypeEscapes).toEqual([]);
    expect(complete.warnings).toEqual([]);

    const missingOne = validateIntent({
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
    });
    expect(missingOne.ok).toBe(true);
    expect(missingOne.intent.plannedTypeEscapes).toEqual([]);
    expect(missingOne.warnings).toContainEqual(
      expect.objectContaining({
        kind: "missing-intent-key-defaulted",
        key: "plannedTypeEscapes",
        action: "defaulted-to-empty-array",
      }),
    );

    const empty = validateIntent({});
    expect(empty.ok).toBe(true);
    expect(empty.warnings).toHaveLength(5);
    expect(
      empty.warnings.every(
        (warning) => warning.kind === "missing-intent-key-defaulted",
      ),
    ).toBe(true);
  });

  it("rejects invalid top-level name shapes with precise error paths", () => {
    expectInvalid(
      {
        names: "formatDate",
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      "names",
    );

    const badElement = validateIntent({
      names: [42, "valid"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    });
    expect(badElement.ok).toBe(false);
    expect(badElement.errorPath).toMatch(/^names\[\d+\]/);

    expect(validateIntent(null).ok).toBe(false);
    expect(validateIntent("not an object").ok).toBe(false);
  });

  it("accepts structured names and dependencies but requires their key fields", () => {
    const names = validateIntent({
      names: [
        "formatDate",
        { name: "formatTimestamp", kind: "function", why: "display helper" },
      ],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    });
    expect(names.ok).toBe(true);
    expect(names.intent.names).toEqual(["formatDate", "formatTimestamp"]);
    expect(names.intent.nameDeclarations[0].why).toBe("display helper");

    const namesWithLocality = validateIntent({
      names: [
        {
          name: "searchMime",
          kind: "function",
          why: "search MIME helpers before adding another helper",
          ownerFile: "src/utils/mime.ts",
        },
        {
          name: "lookupCookie",
          file: "src/helper/cookie/index.ts",
        },
        {
          name: "queryPath",
          targetFile: "src/utils/url.ts",
        },
      ],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    });
    expect(namesWithLocality.ok).toBe(true);
    expect(namesWithLocality.intent.nameDeclarations[0].ownerFile).toBe(
      "src/utils/mime.ts",
    );
    expect(namesWithLocality.intent.nameDeclarations[1]).toMatchObject({
      file: "src/helper/cookie/index.ts",
      ownerFile: "src/helper/cookie/index.ts",
    });
    expect(namesWithLocality.intent.nameDeclarations[2]).toMatchObject({
      targetFile: "src/utils/url.ts",
      ownerFile: "src/utils/url.ts",
    });

    expectInvalid(
      {
        names: [{ kind: "function", why: "missing name" }],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      "names[0].name",
    );

    const deps = validateIntent({
      names: [],
      shapes: [],
      files: [],
      dependencies: [
        "react",
        { specifier: "@scope/pkg", why: "new package boundary" },
      ],
      plannedTypeEscapes: [],
    });
    expect(deps.ok).toBe(true);
    expect(deps.intent.dependencies).toEqual(["react", "@scope/pkg"]);
    expect(deps.intent.dependencyDeclarations[0].why).toBe(
      "new package boundary",
    );

    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [{ why: "missing specifier" }],
        plannedTypeEscapes: [],
      },
      "dependencies[0].specifier",
    );
  });

  it("requires shape fields unless exact hash or supported typeLiteral is present", () => {
    expect(
      validateIntent({
        names: [],
        shapes: [{ fields: ["a", "b"] }],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      }).ok,
    ).toBe(true);

    const hash = `sha256:${"a".repeat(64)}`;
    const exactHash = validateIntent({
      names: [],
      shapes: [{ fields: [], hash }],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    });
    expect(exactHash.ok).toBe(true);
    expect(exactHash.intent.shapes[0].hash).toBe(hash);

    const literal = validateIntent({
      names: [],
      shapes: [
        {
          name: "TimestampViewModel",
          typeLiteral: "{ label: string; iso: string; timezone: string }",
          why: "view model contract",
        },
      ],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    });
    expect(literal.ok).toBe(true);
    expect(literal.intent.shapes[0]).toMatchObject({
      fields: [],
      name: "TimestampViewModel",
      why: "view model contract",
    });

    expectInvalid(
      {
        names: [],
        shapes: [{}],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      "shapes[0].fields",
    );
    expectInvalid(
      {
        names: [],
        shapes: [{ fields: "a,b" }],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      "shapes[0].fields",
    );
    expectInvalid(
      {
        names: [],
        shapes: [{ fields: [], hash: "sha256:nothex" }],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      "shapes[0].hash",
    );
    expectInvalid(
      {
        names: [],
        shapes: [{ fields: [], typeLiteral: "   " }],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
      "shapes[0].typeLiteral",
    );
  });

  it("accepts only canonical planned type escape kinds with reason and locationHint", () => {
    expect(ESCAPE_KINDS).toEqual([
      "explicit-any",
      "as-any",
      "angle-any",
      "as-unknown-as-T",
      "rest-any-args",
      "index-sig-any",
      "generic-default-any",
      "ts-ignore",
      "ts-expect-error",
      "no-explicit-any-disable",
      "jsdoc-any",
    ]);

    for (const escapeKind of ESCAPE_KINDS) {
      const result = validateIntent({
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [
          { escapeKind, reason: "legacy parser", locationHint: "unknown" },
        ],
      });
      expect(result.ok, escapeKind).toBe(true);
    }

    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [
          { escapeKind: "any", reason: "bad", locationHint: "src/a.ts" },
        ],
      },
      "plannedTypeEscapes[0].escapeKind",
    );
    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [
          { escapeKind: "explicit-any", locationHint: "src/a.ts" },
        ],
      },
      "plannedTypeEscapes[0].reason",
    );
    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [{ escapeKind: "explicit-any", reason: "legacy" }],
      },
      "plannedTypeEscapes[0].locationHint",
    );
    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [
          { escapeKind: "explicit-any", reason: "legacy", locationHint: "" },
        ],
      },
      "plannedTypeEscapes[0].locationHint",
    );
  });

  it("preserves optional metadata and reports indexed failures", () => {
    const indexed = validateIntent({
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [
        {
          escapeKind: "explicit-any",
          reason: "legacy",
          locationHint: "src/a.ts",
        },
        { escapeKind: "bad", reason: "legacy", locationHint: "src/b.ts" },
      ],
    });
    expect(indexed.ok).toBe(false);
    expect(indexed.errorPath).toBe("plannedTypeEscapes[1].escapeKind");

    const optional = validateIntent({
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [
        {
          escapeKind: "explicit-any",
          reason: "legacy",
          locationHint: "unknown",
          codeShape: "function parse(x: any)",
          alternativeConsidered: "unknown first",
        },
      ],
      taskId: "T-1",
    });
    expect(optional.ok).toBe(true);
    expect(optional.intent.taskId).toBe("T-1");
    expect(optional.intent.plannedTypeEscapes[0]).toMatchObject({
      codeShape: "function parse(x: any)",
      alternativeConsidered: "unknown first",
    });
  });

  it("validates refactor source paths and positive line numbers", () => {
    const valid = validateIntent({
      names: ["writeOrDestroyConnection"],
      shapes: [],
      files: ["src/connection-write.ts"],
      dependencies: [],
      plannedTypeEscapes: [],
      refactorSources: [
        {
          file: "src/server.ts",
          lines: [498, 577, 661, 689],
          why: "extract repeated catch-destroy handling",
        },
      ],
    });
    expect(valid.ok).toBe(true);
    expect(valid.intent.refactorSources).toEqual([
      {
        file: "src/server.ts",
        lines: [498, 577, 661, 689],
        why: "extract repeated catch-destroy handling",
      },
    ]);

    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
        refactorSources: [{ file: "../server.ts", lines: [1] }],
      },
      "refactorSources[0].file",
    );
    expectInvalid(
      {
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
        refactorSources: [{ file: "src/server.ts", lines: [0] }],
      },
      "refactorSources[0].lines[0]",
    );
  });
});
