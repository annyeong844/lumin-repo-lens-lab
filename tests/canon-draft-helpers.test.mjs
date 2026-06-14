import { describe, expect, it } from "vitest";

import { LOW_INFO_HELPER_NAMES } from "../_lib/canon-draft-utils.mjs";
import {
  classifyHelperGroup,
  classifyHelperIdentity,
} from "../_lib/canon-draft-helpers.mjs";

function group(overrides) {
  return classifyHelperGroup({
    name: "fetch",
    identities: ["a.ts::fetch", "b.ts::fetch"],
    fanInByIdentity: {
      "a.ts::fetch": 1,
      "b.ts::fetch": 1,
    },
    contaminationByIdentity: {},
    ...overrides,
  });
}

function identity(overrides) {
  return classifyHelperIdentity({
    identity: "a.ts::helper",
    fanIn: 1,
    contamination: null,
    exportedName: "helper",
    ...overrides,
  });
}

describe("helper group classification", () => {
  it("G-R0. emits ANY_COLLISION_HELPER only when every identity is any-contaminated", () => {
    expect(
      group({
        name: "foo",
        contaminationByIdentity: {
          "a.ts::fetch": { label: "severely-any-contaminated" },
          "b.ts::fetch": { label: "any-contaminated" },
        },
      }),
    ).toMatchObject({
      label: "ANY_COLLISION_HELPER",
    });

    expect(
      group({
        contaminationByIdentity: {
          "a.ts::fetch": { label: "severely-any-contaminated" },
        },
      }).label,
    ).not.toBe("ANY_COLLISION_HELPER");
    expect(
      group({
        contaminationByIdentity: {
          "a.ts::fetch": { label: "has-any" },
          "b.ts::fetch": { label: "has-any" },
        },
      }).label,
    ).not.toBe("ANY_COLLISION_HELPER");
  });

  it("G-R1. heavily used duplicate helpers beat low-info local-common classification", () => {
    expect(
      group({
        name: "parse",
        fanInByIdentity: {
          "a.ts::fetch": 4,
          "b.ts::fetch": 3,
        },
      }),
    ).toMatchObject({
      label: "HELPER_DUPLICATE_STRONG",
      marker: "❌",
    });
  });

  it("G-R2/G-R3. low-info names stay local-common while unusual names become duplicate-review", () => {
    expect(
      group({
        name: "get",
        fanInByIdentity: {
          "a.ts::fetch": 1,
          "b.ts::fetch": 2,
        },
      }).label,
    ).toBe("HELPER_LOCAL_COMMON");
    expect(
      group({
        name: "validateThing",
        fanInByIdentity: {
          "a.ts::fetch": 1,
          "b.ts::fetch": 2,
        },
      }).label,
    ).toBe("HELPER_DUPLICATE_REVIEW");
  });

  it("G-edge-size1. refuses to classify a single identity as a group", () => {
    expect(() =>
      group({
        identities: ["a.ts::fetch"],
      }),
    ).toThrow();
  });
});

describe("single helper classification", () => {
  it("S-R0. severe any contamination wins over fan-in and low-info names", () => {
    expect(
      identity({
        fanIn: 10,
        exportedName: "legacyHelper",
        contamination: { label: "severely-any-contaminated" },
      }).label,
    ).toBe("severely-any-contaminated-helper");
    expect(
      identity({
        fanIn: 1,
        exportedName: "get",
        contamination: { label: "severely-any-contaminated" },
      }).label,
    ).toBe("severely-any-contaminated-helper");
  });

  it("S-R1/S-R2. low-info helper names are low-signal only below central threshold", () => {
    expect(
      identity({
        fanIn: 2,
        exportedName: "parse",
      }).label,
    ).toBe("low-signal-helper-name");
    expect(
      identity({
        fanIn: 3,
        exportedName: "get",
      }),
    ).toMatchObject({
      label: "central-helper",
      marker: "✅",
    });
  });

  it("S-R3/S-R4. non-low-info helpers split into shared and zero-internal-fan-in tiers", () => {
    expect(
      identity({
        fanIn: 2,
        exportedName: "renderThing",
      }).label,
    ).toBe("shared-helper");
    expect(
      identity({
        fanIn: 0,
        exportedName: "renderThing",
      }).label,
    ).toBe("zero-internal-fan-in-helper");
  });

  it("S-fallback. derives exportedName from the identity tail when omitted", () => {
    expect(
      identity({
        identity: "a.ts::get",
        exportedName: undefined,
        fanIn: 1,
      }).label,
    ).toBe("low-signal-helper-name");
  });
});

describe("helper classifier constants", () => {
  it("CONST. LOW_INFO_HELPER_NAMES stays frozen with canonical entries", () => {
    expect(Object.isFrozen(LOW_INFO_HELPER_NAMES)).toBe(true);
    expect(LOW_INFO_HELPER_NAMES).toHaveLength(15);
    expect(LOW_INFO_HELPER_NAMES).toContain("get");
    expect(LOW_INFO_HELPER_NAMES).not.toContain("render");
  });

  it("C-SWEEP. fresh-AST mode cannot emit any-contamination helper labels without evidence", () => {
    const names = ["get", "parse", "format", "renderThing", "validateThing"];
    for (const name of names) {
      expect(
        identity({
          identity: `a.ts::${name}`,
          exportedName: name,
          fanIn: 5,
          contamination: null,
        }).label,
      ).not.toBe("severely-any-contaminated-helper");
      expect(
        group({
          name,
          identities: [`a.ts::${name}`, `b.ts::${name}`],
          fanInByIdentity: {
            [`a.ts::${name}`]: 5,
            [`b.ts::${name}`]: 5,
          },
          contaminationByIdentity: {},
        }).label,
      ).not.toBe("ANY_COLLISION_HELPER");
    }
  });
});
