import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { lookupShape } from "../_lib/pre-write-lookup-shape.mjs";
import { extractShapeHashFactsFromSource } from "../_lib/shape-hash.mjs";
import { functionSignatureFromTypeLiteral } from "../_lib/function-signature-hash.mjs";

const HASH_A = `sha256:${"a".repeat(64)}`;
const HASH_B = `sha256:${"b".repeat(64)}`;
const TYPE_LITERAL = "{ year: number }";
const TYPE_LITERAL_HASH = extractShapeHashFactsFromSource(
  `export type __IntentShape = ${TYPE_LITERAL};\n`,
  "__intent_shape.ts",
).facts[0].hash;
const UNION_LITERAL = '"open" | "closed"';
const UNION_LITERAL_HASH = extractShapeHashFactsFromSource(
  `export type __IntentShape = ${UNION_LITERAL};\n`,
  "__intent_shape.ts",
).facts[0].hash;
const FUNCTION_TYPE_LITERAL = "(raw: string) => string";
const FUNCTION_SIGNATURE = functionSignatureFromTypeLiteral(FUNCTION_TYPE_LITERAL);
const FUNCTION_SIGNATURE_HASH = FUNCTION_SIGNATURE.hash;

function shapeIndex({ complete = true } = {}) {
  return {
    schemaVersion: "shape-index.v1",
    meta: { complete },
    groupsByHash: {
      [HASH_A]: ["src/a.ts::CalendarA", "src/b.ts::CalendarB"],
      [TYPE_LITERAL_HASH]: ["src/c.ts::CalendarC"],
      [UNION_LITERAL_HASH]: ["src/status.ts::Status"],
    },
    facts: [
      {
        kind: "shape-hash",
        hash: HASH_A,
        identities: ["src/a.ts::CalendarA"],
        identity: "src/a.ts::CalendarA",
        ownerFile: "src/a.ts",
        exportedName: "CalendarA",
        fields: [{ name: "year", type: "number" }],
        confidence: "high",
      },
      {
        kind: "shape-hash",
        hash: HASH_A,
        identities: ["src/b.ts::CalendarB"],
        identity: "src/b.ts::CalendarB",
        ownerFile: "src/b.ts",
        exportedName: "CalendarB",
        fields: [{ name: "year", type: "number" }],
        confidence: "high",
      },
      {
        kind: "shape-hash",
        hash: TYPE_LITERAL_HASH,
        identities: ["src/c.ts::CalendarC"],
        identity: "src/c.ts::CalendarC",
        ownerFile: "src/c.ts",
        exportedName: "CalendarC",
        fields: [{ name: "year", type: "number" }],
        confidence: "high",
      },
      {
        kind: "shape-hash",
        hash: UNION_LITERAL_HASH,
        identities: ["src/status.ts::Status"],
        identity: "src/status.ts::Status",
        ownerFile: "src/status.ts",
        exportedName: "Status",
        shapeKind: "literal-union",
        fields: [],
        literals: [
          { kind: "string", value: "closed" },
          { kind: "string", value: "open" },
        ],
        confidence: "high",
      },
    ],
  };
}

function functionCloneIndex({ complete = true } = {}) {
  return {
    schemaVersion: "function-clones.v3",
    meta: { complete },
    facts: [
      {
        kind: "function-body-fingerprint",
        identity: "src/user-a.ts::normalizeUserName",
        ownerFile: "src/user-a.ts",
        exportedName: "normalizeUserName",
        localName: "normalizeUserName",
        visibility: "file-local",
        exported: false,
        normalizedSignatureHash: FUNCTION_SIGNATURE_HASH,
        signature: FUNCTION_SIGNATURE.signature,
        confidence: "high",
      },
    ],
  };
}

describe("pre-write shape lookup availability and exact evidence", () => {
  it("returns UNAVAILABLE for missing index and legacy fields-only shapes", () => {
    const missing = lookupShape({ fields: [], hash: HASH_A }, {});
    expect(missing).toMatchObject({
      kind: "shape",
      result: "UNAVAILABLE",
      shape: { fields: [], hash: HASH_A },
    });
    expect(missing.citations.join(" ")).toContain("build-shape-index.mjs");

    const fieldsOnly = lookupShape(
      { fields: ["year"] },
      { shapeIndex: shapeIndex() },
    );
    expect(fieldsOnly.result).toBe("UNAVAILABLE");
    expect(fieldsOnly.citations.join(" ")).toContain("field names alone");

    const citation = lookupShape({ fields: ["x"] }, {}).citations.join(" ");
    expect(citation).toMatch(/shape-hash/i);
    expect(citation).toContain("P4");
    expect(citation).toContain("[확인 불가");
  });

  it("matches exact hashes and supported typeLiteral shapes", () => {
    const exact = lookupShape(
      { fields: [], hash: HASH_A },
      { shapeIndex: shapeIndex() },
    );
    expect(exact.result).toBe("SHAPE_MATCH");
    expect(exact.matches.map((match) => match.identity)).toEqual([
      "src/a.ts::CalendarA",
      "src/b.ts::CalendarB",
    ]);
    expect(exact.citations.join(" ")).toContain("shape-index.json facts[]");

    const typeLiteral = lookupShape(
      { fields: [], typeLiteral: TYPE_LITERAL },
      { shapeIndex: shapeIndex() },
    );
    expect(typeLiteral.result).toBe("SHAPE_MATCH");
    expect(typeLiteral.shapeHash).toBe(TYPE_LITERAL_HASH);
    expect(typeLiteral.shapeHashSource).toBe("typeLiteral");
    expect(typeLiteral.citations.join(" ")).toContain("typeLiteral normalized");

    const unionLiteral = lookupShape(
      { fields: [], typeLiteral: UNION_LITERAL },
      { shapeIndex: shapeIndex() },
    );
    expect(unionLiteral.result).toBe("SHAPE_MATCH");
    expect(unionLiteral.matches[0].shapeKind).toBe("literal-union");
    expect(unionLiteral.matches[0].literals).toHaveLength(2);
  });

  it("keeps mismatches, unsupported literals, incomplete indexes, and malformed facts unavailable", () => {
    const mismatch = lookupShape(
      { fields: [], hash: HASH_A, typeLiteral: TYPE_LITERAL },
      { shapeIndex: shapeIndex() },
    );
    expect(mismatch.result).toBe("UNAVAILABLE");
    expect(mismatch.citations.join(" ")).toContain("does not match");

    const unsupported = lookupShape(
      { fields: [], typeLiteral: "{ [K in keyof T]: T[K] }" },
      { shapeIndex: shapeIndex() },
    );
    expect(unsupported.result).toBe("UNAVAILABLE");

    expect(
      lookupShape({ fields: [], hash: HASH_B }, { shapeIndex: shapeIndex() })
        .result,
    ).toBe("NOT_OBSERVED");
    expect(
      lookupShape(
        { fields: [], hash: HASH_B },
        { shapeIndex: shapeIndex({ complete: false }) },
      ).result,
    ).toBe("UNAVAILABLE");

    expect(
      lookupShape(
        { fields: [], hash: HASH_A },
        { shapeIndex: { schemaVersion: "wrong" } },
      ).result,
    ).toBe("UNAVAILABLE");
    expect(
      lookupShape({ fields: [], hash: "abc" }, { shapeIndex: shapeIndex() })
        .result,
    ).toBe("UNAVAILABLE");

    const ghost = lookupShape(
      { fields: [], hash: HASH_A },
      {
        shapeIndex: {
          ...shapeIndex(),
          facts: [],
        },
      },
    );
    expect(ghost.result).toBe("UNAVAILABLE");
    expect(ghost.citations.join(" ")).toMatch(/malformed|inconsistent/i);
  });

  it("preserves file-local helper visibility on function signature matches", () => {
    const result = lookupShape(
      { fields: [], typeLiteral: FUNCTION_TYPE_LITERAL },
      { functionClones: functionCloneIndex() },
    );

    expect(result.result).toBe("SIGNATURE_MATCH");
    expect(result.matches[0]).toMatchObject({
      identity: "src/user-a.ts::normalizeUserName",
      ownerFile: "src/user-a.ts",
      exportedName: "normalizeUserName",
      localName: "normalizeUserName",
      visibility: "file-local",
      exported: false,
    });
  });

  it("does not fall back to defIndex, symbols.uses, or source-string heuristics", () => {
    const source = readFileSync(
      path.resolve("_lib/pre-write-lookup-shape.mjs"),
      "utf8",
    );

    expect(source).not.toMatch(/defIndex/);
    expect(source).not.toMatch(/symbols\.uses/);
    expect(source).not.toMatch(/interface/);

    const richContext = {
      shapeIndex: null,
      symbols: {
        defIndex: { "src/a.ts": { CalendarA: { fields: ["year"] } } },
        uses: [{ exportedName: "CalendarA" }],
      },
    };
    expect(lookupShape({ fields: ["year"] }, richContext).result).toBe(
      "UNAVAILABLE",
    );
  });

  it("returns a deterministic result shape for unavailable lookups", () => {
    const result = lookupShape({ fields: ["year"] }, {});

    expect(Object.keys(result).sort()).toEqual([
      "citations",
      "kind",
      "result",
      "shape",
    ]);
  });
});
