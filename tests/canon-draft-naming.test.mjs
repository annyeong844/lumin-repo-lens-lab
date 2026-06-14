import { describe, expect, it } from "vitest";

import {
  LOW_INFO_HELPER_NAMES,
  LOW_INFO_NAMES,
} from "../_lib/canon-draft-utils.mjs";
import {
  classifyNamingCohort,
  classifyNamingItem,
  detectConvention,
  normalizeFileBasename,
} from "../_lib/canon-draft-naming.mjs";

describe("naming convention classifier", () => {
  it.each([
    ["fooBar", "camelCase"],
    ["fooBarBaz", "camelCase"],
    ["FooBar", "PascalCase"],
    ["foo-bar", "kebab-case"],
    ["foo-bar-baz", "kebab-case"],
    ["foo_bar", "snake_case"],
    ["FOO_BAR", "UPPER_SNAKE"],
    ["MAX_RETRY_COUNT", "UPPER_SNAKE"],
    ["foo", "camelCase"],
    ["Foo", "PascalCase"],
    ["FOO", "UPPER_SNAKE"],
    ["Foo_bar", "mixed"],
    ["foo-Bar", "mixed"],
    ["foo_Bar_baz", "mixed"],
    ["", "mixed"],
    [null, "mixed"],
    ["x", "camelCase"],
    ["X", "UPPER_SNAKE"],
  ])("detectConvention(%j) -> %s", (name, expected) => {
    expect(detectConvention(name)).toBe(expected);
  });

  it.each([
    ["_lib/canon-draft.mjs", "canon-draft"],
    ["src/components/UserCard.tsx", "UserCard"],
    ["tests/user-profile.test.tsx", "user-profile"],
    ["src/api.d.ts", "api"],
    ["src/legacy_module.js", "legacy_module"],
    ["src/FOO.test.mjs", "FOO"],
    ["src/Comp.stories.tsx", "Comp"],
    ["src/a.spec.ts", "a"],
    ["plain.mjs", "plain"],
    ["src\\win\\path.mjs", "path"],
    ["", ""],
    [null, ""],
    ["README", "README"],
  ])("normalizeFileBasename(%j) -> %s", (filePath, expected) => {
    expect(normalizeFileBasename(filePath)).toBe(expected);
  });
});

describe("naming cohort classification", () => {
  function classify(overrides) {
    return classifyNamingCohort({
      cohortId: "x",
      members: [],
      kind: "symbol",
      lowInfoExclusions: new Set(),
      ...overrides,
    });
  }

  it("requires at least three effective members before claiming a dominant convention", () => {
    expect(
      classify({
        members: [{ name: "fooBar" }, { name: "bazQux" }],
      }),
    ).toMatchObject({
      label: "insufficient-evidence",
      dominantConvention: null,
    });

    expect(
      classify({
        members: [{ name: "fooBar" }, { name: "bazQux" }, { name: "zipZap" }],
      }),
    ).toMatchObject({
      label: "camelCase-dominant",
      consistencyRate: 1,
    });
  });

  it("uses the 0.6 dominance threshold and keeps mixed fallback from becoming dominant", () => {
    const dominant = classify({
      members: [
        { name: "fooBar" },
        { name: "bazQux" },
        { name: "zipZap" },
        { name: "barFoo" },
        { name: "FooBar" },
      ],
    });
    expect(dominant).toMatchObject({
      label: "camelCase-dominant",
      dominantConvention: "camelCase",
    });
    expect(dominant.consistencyRate).toBeCloseTo(0.8);

    expect(
      classify({
        members: [
          { name: "fooBar" },
          { name: "bazQux" },
          { name: "FooBar" },
          { name: "BazQux" },
          { name: "foo-bar" },
        ],
      }),
    ).toMatchObject({
      label: "mixed-convention",
      dominantConvention: null,
    });

    expect(
      classify({
        members: [
          { name: "foo_Bar" },
          { name: "Foo-bar" },
          { name: "foo.Bar" },
          { name: "bar:Baz" },
          { name: "alphaBeta" },
        ],
      }),
    ).toMatchObject({
      label: "mixed-convention",
      dominantConvention: null,
    });
  });

  it("normalizes file cohorts before classifying convention dominance", () => {
    expect(
      classify({
        cohortId: "_lib",
        kind: "file",
        members: [
          { name: "_lib/canon-draft.mjs" },
          { name: "_lib/alias-map.mjs" },
          { name: "_lib/extract-ts.mjs" },
        ],
      }),
    ).toMatchObject({
      label: "kebab-case-dominant",
      dominantConvention: "kebab-case",
    });
  });

  it("excludes low-info members from effective size and dominance", () => {
    const lowInfo = new Set(["get", "set", "parse", "format"]);

    expect(
      classify({
        members: [
          { name: "domainHelper" },
          { name: "otherHelper" },
          { name: "get" },
          { name: "set" },
          { name: "parse" },
          { name: "format" },
          { name: "get" },
          { name: "set" },
          { name: "parse" },
          { name: "format" },
        ],
        lowInfoExclusions: lowInfo,
      }),
    ).toMatchObject({
      label: "insufficient-evidence",
      totalMembers: 10,
      effectiveMembers: 2,
    });

    expect(
      classify({
        members: [
          { name: "get" },
          { name: "set" },
          { name: "parse" },
          { name: "format" },
        ],
        lowInfoExclusions: lowInfo,
      }),
    ).toMatchObject({
      label: "insufficient-evidence",
      effectiveMembers: 0,
    });
  });

  it("keeps realistic file cohort dominance after low-info file exclusion", () => {
    const result = classify({
      cohortId: "_lib",
      kind: "file",
      members: [
        { name: "_lib/canon-draft.mjs" },
        { name: "_lib/alias-map.mjs" },
        { name: "_lib/extract-ts.mjs" },
        { name: "_lib/resolver-core.mjs" },
        { name: "_lib/legacy_helper.mjs" },
        { name: "_lib/get.mjs" },
      ],
      lowInfoExclusions: new Set(["get"]),
    });

    expect(result).toMatchObject({
      label: "kebab-case-dominant",
      effectiveMembers: 5,
    });
    expect(result.consistencyRate).toBeCloseTo(0.8);
  });
});

describe("naming item classification", () => {
  it("prioritizes low-info exclusion before match/outlier rules", () => {
    expect(
      classifyNamingItem({
        convention: "camelCase",
        dominantConvention: "camelCase",
        isLowInfo: true,
      }).label,
    ).toBe("low-info-excluded");
    expect(
      classifyNamingItem({
        convention: "camelCase",
        dominantConvention: null,
        isLowInfo: true,
      }).label,
    ).toBe("low-info-excluded");
    expect(
      classifyNamingItem({
        convention: "PascalCase",
        dominantConvention: "camelCase",
        isLowInfo: true,
      }).label,
    ).toBe("low-info-excluded");
  });

  it("labels non-low-info items as match unless they differ from the dominant convention", () => {
    expect(
      classifyNamingItem({
        convention: "camelCase",
        dominantConvention: null,
        isLowInfo: false,
      }).label,
    ).toBe("convention-match");
    expect(
      classifyNamingItem({
        convention: "camelCase",
        dominantConvention: "camelCase",
        isLowInfo: false,
      }).label,
    ).toBe("convention-match");
    expect(
      classifyNamingItem({
        convention: "PascalCase",
        dominantConvention: "camelCase",
        isLowInfo: false,
      }).label,
    ).toBe("convention-outlier");
  });

  it("keeps shared low-info name sets available to naming classifiers", () => {
    expect(LOW_INFO_NAMES).toContain("Props");
    expect(LOW_INFO_HELPER_NAMES).toContain("get");
    const combined = new Set([...LOW_INFO_NAMES, ...LOW_INFO_HELPER_NAMES]);
    expect(combined.has("Props")).toBe(true);
    expect(combined.has("get")).toBe(true);
  });
});
