import { describe, expect, it } from "vitest";

import {
  detectGeneratedFileEvidence,
  extractShapeHashFactsFromSource,
  groupShapeFactsByHash,
  normalizeTypeText,
} from "../_lib/shape-hash.mjs";

function extract(src, file = "src/types.ts") {
  return extractShapeHashFactsFromSource(src, file, {
    observedAt: "2026-04-22T00:00:00.000Z",
  });
}

function factByName(result, name) {
  return result.facts.find((fact) => fact.exportedName === name);
}

describe("shape hash object normalization", () => {
  it("S1a. extracts two supported exported shapes", () => {
    const result = extract(`
      export interface UserA {
        b: number;
        a: string;
      }
      export type UserB = {
        a: string;
        b: number;
      };
    `);

    expect(result.facts).toHaveLength(2);
  });

  it("S1b. hashes same fields in different order the same", () => {
    const result = extract(`
      export interface UserA {
        b: number;
        a: string;
      }
      export type UserB = {
        a: string;
        b: number;
      };
    `);
    const a = factByName(result, "UserA");
    const b = factByName(result, "UserB");

    expect(a?.hash).toBeTruthy();
    expect(a?.hash).toBe(b?.hash);
  });

  it("S1c. sorts fields by name in fact payloads", () => {
    const result = extract(`
      export interface UserA {
        b: number;
        a: string;
      }
    `);

    expect(
      factByName(result, "UserA")?.fields.map((field) => field.name),
    ).toEqual(["a", "b"]);
  });

  it("S2. changes the hash when a field type changes", () => {
    const result = extract(`
      export type A = { id: string };
      export type B = { id: number };
    `);

    expect(factByName(result, "A")?.hash).not.toBe(
      factByName(result, "B")?.hash,
    );
  });

  it("S3a. treats optional fields as hash-bearing", () => {
    const result = extract(`
      export type A = { id: string; name: string };
      export type B = { id?: string; name: string };
    `);

    expect(factByName(result, "A")?.hash).not.toBe(
      factByName(result, "B")?.hash,
    );
  });

  it("S3b. treats readonly fields as hash-bearing", () => {
    const result = extract(`
      export type A = { id: string; name: string };
      export type C = { readonly id: string; name: string };
    `);

    expect(factByName(result, "A")?.hash).not.toBe(
      factByName(result, "C")?.hash,
    );
  });
});

describe("shape hash type text normalization", () => {
  it("S4a. normalizes punctuation spacing outside literals", () => {
    expect(normalizeTypeText("Array < string | number >")).toBe(
      "Array<string|number>",
    );
  });

  it("S4b. preserves string literal interior spacing", () => {
    expect(normalizeTypeText('"a | b" | string')).toBe('"a | b"|string');
  });

  it("S5. hashes semantically same type text spacing the same", () => {
    const result = extract(`
      export type A = { value: "a | b" | string };
      export type B = { value:"a | b"|string };
    `);

    expect(factByName(result, "A")?.hash).toBe(factByName(result, "B")?.hash);
  });
});

describe("shape hash unsupported shape diagnostics", () => {
  it("S6a. emits no fact for unsupported mapped or generic shapes", () => {
    const result = extract(`
      export type Mapped<T> = { [K in keyof T]: T[K] };
    `);

    expect(result.facts).toEqual([]);
  });

  it("S6b. emits an explicit unsupported type-parameters reason", () => {
    const result = extract(`
      export type Mapped<T> = { [K in keyof T]: T[K] };
    `);

    expect(
      result.diagnostics.some(
        (diagnostic) => diagnostic.code === "unsupported-type-parameters",
      ),
    ).toBe(true);
  });

  it("S7a. emits no fact for index or computed shapes", () => {
    const result = extract(`
      export type Weird = { [key: string]: string };
    `);

    expect(result.facts).toEqual([]);
  });

  it("S7b. emits an unsupported member diagnostic for index or computed shapes", () => {
    const result = extract(`
      export type Weird = { [key: string]: string };
    `);

    expect(
      result.diagnostics.some(
        (diagnostic) => diagnostic.code === "unsupported-member-kind",
      ),
    ).toBe(true);
  });

  it("S9a. emits no facts after parse errors", () => {
    const result = extract(`export interface Broken { id: string `);

    expect(result.facts).toEqual([]);
  });

  it("S9b. emits a parse-error diagnostic after parse errors", () => {
    const result = extract(`export interface Broken { id: string `);

    expect(
      result.diagnostics.some(
        (diagnostic) => diagnostic.code === "parse-error",
      ),
    ).toBe(true);
  });

  it("S12a. emits no partial shape facts for declaration-merged identities", () => {
    const result = extract(`
      export interface Foo { a: string }
      export interface Foo { b: number }
    `);

    expect(result.facts).toEqual([]);
  });

  it("S12b. emits explicit diagnostics for unsupported declaration merges", () => {
    const result = extract(`
      export interface Foo { a: string }
      export interface Foo { b: number }
    `);

    expect(
      result.diagnostics.some(
        (diagnostic) =>
          diagnostic.code === "declaration-merge-unsupported" &&
          diagnostic.identity === "src/types.ts::Foo",
      ),
    ).toBe(true);
  });
});

describe("shape hash fact metadata and grouping", () => {
  it("S8a. emits shape-hash fact kind", () => {
    const result = extract(`export interface User { id: string }\n`);

    expect(factByName(result, "User")?.kind).toBe("shape-hash");
  });

  it("S8b. emits sha256 hashes with lowercase hex payloads", () => {
    const result = extract(`export interface User { id: string }\n`);

    expect(factByName(result, "User")?.hash).toMatch(/^sha256:[a-f0-9]{64}$/);
  });

  it("S8c. emits canonical metadata", () => {
    const result = extract(`export interface User { id: string }\n`);
    const fact = factByName(result, "User");

    expect(fact).toMatchObject({
      source: "fresh-ast-pass",
      scope: "TS/JS production files, exported types only",
      confidence: "high",
      observedAt: "2026-04-22T00:00:00.000Z",
    });
  });

  it("S8d. uses ownerFile::exportedName identities", () => {
    const result = extract(`export interface User { id: string }\n`);
    const fact = factByName(result, "User");

    expect(fact?.identity).toBe("src/types.ts::User");
    expect(fact?.identities).toEqual(["src/types.ts::User"]);
  });

  it("S10a. extracts local declarations exported by specifier", () => {
    const result = extract(`
      interface LocalUser { id: string }
      export { LocalUser as PublicUser };
    `);

    expect(result.facts).toHaveLength(1);
  });

  it("S10b. uses exported aliases rather than local declaration names", () => {
    const result = extract(`
      interface LocalUser { id: string }
      export { LocalUser as PublicUser };
    `);

    expect(factByName(result, "PublicUser")?.identity).toBe(
      "src/types.ts::PublicUser",
    );
  });

  it("S11. groups shape fact identities by hash in sorted order", () => {
    const result = extract(`
      export type B = { x: string };
      export type A = { x: string };
    `);
    const groups = groupShapeFactsByHash(result.facts);

    expect(Object.values(groups)[0]).toEqual([
      "src/types.ts::A",
      "src/types.ts::B",
    ]);
  });
});

describe("shape hash generated-file evidence", () => {
  it("S13a. carries generated-file evidence from path conventions", () => {
    const result = extract(
      `export interface FileRoutesById { id: string }\n`,
      "src/routeTree.gen.ts",
    );

    expect(factByName(result, "FileRoutesById")?.generatedFile).toMatchObject({
      kind: "generated-file",
      source: "path",
    });
  });

  it("S13b. detects generated headers without path conventions", () => {
    const generated = detectGeneratedFileEvidence(
      "src/manual.ts",
      "// @generated by tool\nexport interface A { id: string }",
    );

    expect(generated?.source).toBe("header");
  });
});

describe("shape hash literal unions", () => {
  it("S14a. extracts supported literal union aliases", () => {
    const result = extract(`
      export type StatusA = "open" | 'closed' | null | undefined | true | 1 | 1n;
      export type StatusB = 1n | true | undefined | null | \`closed\` | "open" | 1;
    `);

    expect(factByName(result, "StatusA")?.shapeKind).toBe("literal-union");
    expect(factByName(result, "StatusB")?.shapeKind).toBe("literal-union");
  });

  it("S14b. hashes same literal unions in different order the same", () => {
    const result = extract(`
      export type StatusA = "open" | 'closed' | null | undefined | true | 1 | 1n;
      export type StatusB = 1n | true | undefined | null | \`closed\` | "open" | 1;
      export type StatusC = "open" | "pending";
    `);
    const a = factByName(result, "StatusA");
    const b = factByName(result, "StatusB");
    const c = factByName(result, "StatusC");

    expect(a?.hash).toBeTruthy();
    expect(a?.hash).toBe(b?.hash);
    expect(a?.hash).not.toBe(c?.hash);
  });

  it("S14c. carries normalized literal union evidence", () => {
    const result = extract(`
      export type StatusA = "open" | 'closed' | null | undefined | true | 1 | 1n;
    `);
    const literals = factByName(result, "StatusA")?.literals ?? [];

    expect(
      literals.some(
        (literal) => literal.kind === "string" && literal.value === "open",
      ),
    ).toBe(true);
    expect(literals.some((literal) => literal.kind === "undefined")).toBe(true);
    expect(
      literals.some(
        (literal) => literal.kind === "bigint" && literal.value === "1",
      ),
    ).toBe(true);
  });

  it("S15a. emits no shape facts for broad mixed unions", () => {
    const result = extract(`export type Mixed = "open" | string;\n`);

    expect(result.facts).toEqual([]);
  });

  it("S15b. emits explicit diagnostics for broad mixed unions", () => {
    const result = extract(`export type Mixed = "open" | string;\n`);

    expect(
      result.diagnostics.some(
        (diagnostic) =>
          diagnostic.code === "unsupported-literal-union-member" &&
          diagnostic.identity === "src/types.ts::Mixed",
      ),
    ).toBe(true);
  });
});
