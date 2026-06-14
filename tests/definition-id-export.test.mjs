import { describe, expect, it } from "vitest";

const definitionId = await import("../_lib/definition-id.mjs");

describe("definition-id export surface", () => {
  it("DIES1. exposes OXC helper, not raw id builder", () => {
    expect(typeof definitionId.definitionIdFromOxcNode).toBe("function");
    expect(Object.hasOwn(definitionId, "makeDefinitionId")).toBe(false);
  });
});
