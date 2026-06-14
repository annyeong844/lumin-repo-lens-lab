import { describe, expect, it } from "vitest";

import {
  extractTypeEscapes,
  normalizeCodeShape,
} from "../_lib/extract-ts-escapes.mjs";

const CANON_ESCAPE_KINDS = [
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
];

function extract(src, filePath = "/fake/test.ts") {
  return extractTypeEscapes(src, filePath);
}

function kinds(result) {
  return (result.typeEscapes ?? []).map((entry) => entry.escapeKind);
}

function byKind(result, kind) {
  return (result.typeEscapes ?? []).filter(
    (entry) => entry.escapeKind === kind,
  );
}

describe("type escape extraction emits canonical escape kinds", () => {
  it("emits explicit any facts with exported type alias identity", () => {
    const result = extract("export type X = any;\n");
    const hits = byKind(result, "explicit-any");

    expect(hits).toHaveLength(1);
    expect(hits[0]).toMatchObject({
      file: "/fake/test.ts",
      line: 1,
      insideExportedIdentity: "/fake/test.ts::X",
    });
  });

  it("emits explicit any facts with exported interface identity", () => {
    const result = extract("export interface Foo { a: any; b: string }\n");
    const hits = byKind(result, "explicit-any");

    expect(hits).toHaveLength(1);
    expect(hits[0]?.insideExportedIdentity).toBe("/fake/test.ts::Foo");
  });

  it("detects as-any and angle-any assertion forms", () => {
    expect(
      byKind(extract("const x = (foo as any).bar;\n"), "as-any"),
    ).toHaveLength(1);
    expect(byKind(extract("const x = <any>foo;\n"), "angle-any")).toHaveLength(
      1,
    );
  });

  it("keeps chained unknown assertions distinct from generic as-any", () => {
    const result = extract("const x = foo as unknown as Bar;\n");

    expect(byKind(result, "as-unknown-as-T")).toHaveLength(1);
    expect(byKind(result, "as-any")).toHaveLength(0);
  });

  it("keeps rest-any-args distinct from explicit-any", () => {
    const result = extract("export function f(...args: any[]) {}\n");

    expect(byKind(result, "rest-any-args")).toHaveLength(1);
    expect(byKind(result, "explicit-any")).toHaveLength(0);
  });

  it("keeps index-sig-any distinct from explicit-any", () => {
    const result = extract("type Dict = { [key: string]: any };\n");

    expect(byKind(result, "index-sig-any")).toHaveLength(1);
    expect(byKind(result, "explicit-any")).toHaveLength(0);
  });

  it("keeps generic-default-any distinct from explicit-any", () => {
    const result = extract("type Box<T = any> = { value: T };\n");

    expect(byKind(result, "generic-default-any")).toHaveLength(1);
    expect(byKind(result, "explicit-any")).toHaveLength(0);
  });

  it("preserves ts-ignore and ts-expect-error comment text", () => {
    const ignore = byKind(
      extract("// @ts-ignore reason text\nconst x = 1;\n"),
      "ts-ignore",
    );
    const expectError = byKind(
      extract("// @ts-expect-error upstream type bug\nconst x = 1;\n"),
      "ts-expect-error",
    );

    expect(ignore).toHaveLength(1);
    expect(ignore[0]?.codeShape).toContain("@ts-ignore");
    expect(ignore[0]?.codeShape).toContain("reason text");
    expect(expectError).toHaveLength(1);
    expect(expectError[0]?.codeShape).toContain("upstream type bug");
  });

  it("detects all no-explicit-any eslint disable forms", () => {
    const result = extract(
      "// eslint-disable-next-line no-explicit-any\nconst a = 1;\n" +
        "// eslint-disable-next-line @typescript-eslint/no-explicit-any\nconst b = 2;\n" +
        "// eslint-disable-line @typescript-eslint/no-explicit-any\nconst c = 3;\n" +
        "/* eslint-disable @typescript-eslint/no-explicit-any */\nconst d = 4;\n",
    );

    expect(byKind(result, "no-explicit-any-disable")).toHaveLength(4);
  });

  it("detects jsdoc-any on the comment line", () => {
    const hits = byKind(
      extract(
        "/** @type {any} */\nconst fromJsdoc = readValue();\n",
        "/fake/t.mjs",
      ),
      "jsdoc-any",
    );

    expect(hits).toHaveLength(1);
    expect(hits[0]?.codeShape).toContain("@type");
    expect(hits[0]?.codeShape).toContain("{any}");
    expect(hits[0]?.line).toBe(1);
  });

  it("emits all canonical escape kinds from the full-coverage fixture", () => {
    const result = extract(
      "type A = any;\n" +
        "const b = (x as any);\n" +
        "const c = (<any>x);\n" +
        "const d = (x as unknown as Foo);\n" +
        "function e(...args: any[]) {}\n" +
        "type F = { [k: string]: any };\n" +
        "type G<T = any> = T;\n" +
        "// @ts-ignore reason\nconst h = 1;\n" +
        "// @ts-expect-error reason\nconst i = 1;\n" +
        "// eslint-disable-next-line no-explicit-any\nconst j = 1;\n" +
        "/** @type {any} */\nconst k = readValue();\n",
    );
    const seen = new Set(kinds(result));

    for (const kind of CANON_ESCAPE_KINDS) {
      expect(seen, kind).toContain(kind);
    }
  });
});

describe("type escape extraction normalizes and keys evidence", () => {
  it("collapses outer whitespace without changing string-literal interior whitespace", () => {
    const outer = byKind(extract("const x = foo   as    any ;\n"), "as-any")[0];
    const stringLiteral = byKind(
      extract('const x = ("a   b" as any);\n'),
      "as-any",
    )[0];

    expect(outer?.normalizedCodeShape).toBe("foo as any");
    expect(stringLiteral?.normalizedCodeShape).toMatch(/a   b/);
  });

  it("keeps occurrence keys distinct across files and stable across line shifts", () => {
    const firstFile = byKind(
      extract("const x = foo as any;\n", "/a.ts"),
      "as-any",
    )[0]?.occurrenceKey;
    const secondFile = byKind(
      extract("const x = foo as any;\n", "/b.ts"),
      "as-any",
    )[0]?.occurrenceKey;
    const shifted = byKind(
      extract("\n\n\nconst x = foo as any;\n", "/a.ts"),
      "as-any",
    )[0]?.occurrenceKey;

    expect(firstFile).toMatch(/^sha256:[a-f0-9]{64}$/);
    expect(secondFile).toMatch(/^sha256:[a-f0-9]{64}$/);
    expect(firstFile).not.toBe(secondFile);
    expect(shifted).toBe(firstFile);
  });

  it("returns structured parse-error markers without type escapes", () => {
    const result = extract("const x = ;;;broken syntax\n");

    expect(result.parseError).toEqual(expect.any(String));
    expect(result.typeEscapes ?? []).toHaveLength(0);
  });

  it("exports normalizeCodeShape and keeps it in lockstep with extraction", () => {
    const raw = 'foo   as   "a   b"   as   any';
    const result = extract('const x = foo   as   "a   b"   as   any;\n');
    const hit = byKind(result, "as-any")[0];

    expect(normalizeCodeShape(raw)).toBe('foo as "a   b" as any');
    expect(hit?.normalizedCodeShape).toBe(
      normalizeCodeShape(hit?.codeShape ?? ""),
    );
  });
});

describe("type escape extraction assigns exported identities", () => {
  it("assigns export function and exported const identities", () => {
    expect(
      byKind(
        extract("export function fetchUser() { return x as any; }\n"),
        "as-any",
      )[0]?.insideExportedIdentity,
    ).toBe("/fake/test.ts::fetchUser");
    expect(
      byKind(extract("export const fetchUser = () => x as any;\n"), "as-any")[0]
        ?.insideExportedIdentity,
    ).toBe("/fake/test.ts::fetchUser");
  });

  it("uses exported alias names and default export identities", () => {
    expect(
      byKind(
        extract(
          "function foo() { return x as any; }\nexport { foo as bar };\n",
        ),
        "as-any",
      )[0]?.insideExportedIdentity,
    ).toBe("/fake/test.ts::bar");
    expect(
      byKind(
        extract("export default function fetchUser() { return x as any; }\n"),
        "as-any",
      )[0]?.insideExportedIdentity,
    ).toBe("/fake/test.ts::default");
    expect(
      byKind(extract("export default () => (x as any);\n"), "as-any")[0]
        ?.insideExportedIdentity,
    ).toBe("/fake/test.ts::default");
  });

  it("keeps local and top-level escapes unowned while nested escapes inherit exported parents", () => {
    expect(
      byKind(extract("function unused() { return x as any; }\n"), "as-any")[0]
        ?.insideExportedIdentity,
    ).toBeNull();
    expect(
      byKind(extract("const x = foo as any;\n"), "as-any")[0]
        ?.insideExportedIdentity,
    ).toBeNull();
    expect(
      byKind(
        extract(
          "export function outer() {\n" +
            "  function inner() { return x as any; }\n" +
            "  return inner();\n" +
            "}\n",
        ),
        "as-any",
      )[0]?.insideExportedIdentity,
    ).toBe("/fake/test.ts::outer");
  });
});
