import { describe, expect, it } from "vitest";

import {
  LOW_INFO_NAMES,
  codeCell,
  escapeMdCell,
} from "../_lib/canon-draft-utils.mjs";
import {
  classifySingleIdentity,
  classifyTypeNameGroup,
} from "../_lib/canon-draft-types.mjs";

describe("type ownership classifier constants and markdown helpers", () => {
  it("keeps LOW_INFO_NAMES frozen and populated", () => {
    expect(Array.isArray(LOW_INFO_NAMES)).toBe(true);
    expect(Object.isFrozen(LOW_INFO_NAMES)).toBe(true);
    expect(LOW_INFO_NAMES).toContain("Props");
    expect(LOW_INFO_NAMES).toHaveLength(16);
  });

  it.each([
    ["clean", "clean"],
    [null, ""],
    ["a|b", "a\\|b"],
    ["a\\b", "a\\\\b"],
    ["a\nb", "a b"],
  ])("escapeMdCell(%j) -> %j", (value, expected) => {
    expect(escapeMdCell(value)).toBe(expected);
  });

  it.each([
    ["", ""],
    [null, ""],
    ["x", "`x`"],
    ["a`b", "`` a`b ``"],
  ])("codeCell(%j) -> %j", (value, expected) => {
    expect(codeCell(value)).toBe(expected);
  });
});

describe("type name group classifier", () => {
  function classifyGroup({
    name = "Foo",
    ids = ["a.ts::Foo", "b.ts::Foo"],
    fanInByIdentity = {},
    contaminationByIdentity = {},
  } = {}) {
    return classifyTypeNameGroup({
      name,
      identities: ids,
      fanInByIdentity,
      contaminationByIdentity,
    });
  }

  it("uses universal contamination for ANY_COLLISION before fan-in rules", () => {
    const result = classifyGroup({
      fanInByIdentity: { "a.ts::Foo": 20, "b.ts::Foo": 20 },
      contaminationByIdentity: {
        "a.ts::Foo": { label: "any-contaminated" },
        "b.ts::Foo": { label: "severely-any-contaminated" },
      },
    });

    expect(result).toMatchObject({ label: "ANY_COLLISION", marker: "⚠" });
  });

  it.each([
    [
      "has-any only",
      { "a.ts::X": { label: "has-any" }, "b.ts::X": { label: "has-any" } },
    ],
    [
      "unknown-surface only",
      {
        "a.ts::X": { label: "unknown-surface" },
        "b.ts::X": { label: "unknown-surface" },
      },
    ],
    [
      "mixed contaminated and has-any",
      {
        "a.ts::X": { label: "any-contaminated" },
        "b.ts::X": { label: "has-any" },
      },
    ],
  ])(
    "does not treat %s as ANY_COLLISION",
    (_label, contaminationByIdentity) => {
      const result = classifyGroup({
        name: "X",
        ids: ["a.ts::X", "b.ts::X"],
        fanInByIdentity: { "a.ts::X": 5, "b.ts::X": 5 },
        contaminationByIdentity,
      });

      expect(result.label).toBe("DUPLICATE_STRONG");
    },
  );

  it("promotes high-fan-in duplicate names before low-info fallback", () => {
    const result = classifyGroup({
      name: "Result",
      ids: ["a.ts::Result", "b.ts::Result"],
      fanInByIdentity: { "a.ts::Result": 18, "b.ts::Result": 3 },
    });

    expect(result.label).toBe("DUPLICATE_STRONG");
  });

  it("uses LOCAL_COMMON_NAME only for low-info duplicate names below the fan-in threshold", () => {
    expect(
      classifyGroup({
        name: "Props",
        ids: ["a.ts::Props", "b.ts::Props"],
        fanInByIdentity: { "a.ts::Props": 1, "b.ts::Props": 2 },
      }),
    ).toMatchObject({ label: "LOCAL_COMMON_NAME", marker: "⚠" });

    expect(
      classifyGroup({
        name: "Options",
        ids: ["a.ts::Options", "b.ts::Options"],
        fanInByIdentity: { "a.ts::Options": 0, "b.ts::Options": 0 },
      }).label,
    ).toBe("LOCAL_COMMON_NAME");
  });

  it("falls back to duplicate review for non-low-info low-fan-in duplicates", () => {
    expect(
      classifyGroup({
        name: "Xyz",
        ids: ["a.ts::Xyz", "b.ts::Xyz"],
        fanInByIdentity: { "a.ts::Xyz": 1, "b.ts::Xyz": 1 },
      }),
    ).toMatchObject({ label: "DUPLICATE_REVIEW", marker: "⚠" });
  });

  it("rejects single-identity input so callers use classifySingleIdentity", () => {
    expect(() =>
      classifyTypeNameGroup({
        name: "Foo",
        identities: ["a.ts::Foo"],
        fanInByIdentity: {},
        contaminationByIdentity: {},
      }),
    ).toThrow();
  });
});

describe("single type identity classifier", () => {
  function classifySingle(overrides) {
    return classifySingleIdentity({
      identity: "a.ts::TypeName",
      fanIn: 0,
      kind: "TSInterfaceDeclaration",
      contamination: null,
      ...overrides,
    });
  }

  it("keeps severe contamination stronger than high fan-in", () => {
    expect(
      classifySingle({
        identity: "a.ts::Big",
        fanIn: 100,
        contamination: { label: "severely-any-contaminated" },
      }),
    ).toMatchObject({ label: "severely-any-contaminated" });
  });

  it("does not treat non-severe any contamination as a severe label", () => {
    expect(
      classifySingle({
        identity: "a.ts::Moderate",
        fanIn: 5,
        contamination: { label: "any-contaminated" },
      }).label,
    ).toBe("single-owner-strong");
  });

  it("applies the low-signal alias rule only to one-character low-fan-in type aliases", () => {
    expect(
      classifySingle({
        identity: "a.ts::T",
        fanIn: 2,
        kind: "TSTypeAliasDeclaration",
      }).label,
    ).toBe("low-signal-type-name");

    expect(
      classifySingle({
        identity: "a.ts::T",
        fanIn: 5,
        kind: "TSTypeAliasDeclaration",
      }).label,
    ).toBe("single-owner-strong");

    expect(
      classifySingle({
        identity: "a.ts::X",
        fanIn: 2,
        kind: "TSInterfaceDeclaration",
      }).label,
    ).toBe("single-owner-weak");
  });

  it.each([
    [5, "single-owner-strong", "✅"],
    [2, "single-owner-weak", undefined],
    [1, "single-owner-weak", undefined],
    [0, "zero-internal-fan-in", undefined],
  ])("classifies fanIn=%s as %s", (fanIn, label, marker) => {
    const result = classifySingle({ fanIn, identity: `a.ts::Type${fanIn}` });
    expect(result.label).toBe(label);
    if (marker) expect(result.marker).toBe(marker);
  });
});
