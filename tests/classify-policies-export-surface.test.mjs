import { describe, expect, it } from "vitest";

const classifyPolicies = await import("../_lib/classify-policies.mjs");

describe("classify-policies export surface", () => {
  it("CPES1. does not expose legacy framework sentinel helpers", () => {
    for (const symbol of [
      "isCoreSentinel",
      "detectNuxtNitro",
      "isNuxtNitroSentinel",
    ]) {
      expect(Object.hasOwn(classifyPolicies, symbol), symbol).toBe(false);
    }
  });

  it("CPES2. does not re-export non-public policy actions", () => {
    for (const symbol of ["ACTION_NONE", "ACTION_REVIEW_HINT"]) {
      expect(Object.hasOwn(classifyPolicies, symbol), symbol).toBe(false);
    }
  });
});
