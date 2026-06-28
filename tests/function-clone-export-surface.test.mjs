import { describe, expect, it } from "vitest";

const functionCloneArtifact =
  await import("../_lib/function-clone-artifact.mjs");

describe("function-clone artifact export surface", () => {
  it("FCES1. exposes builder, not version internals", () => {
    expect(typeof functionCloneArtifact.buildFunctionCloneArtifact).toBe(
      "function",
    );

    for (const symbol of [
      "FUNCTION_CLONE_SCHEMA_VERSION",
      "FUNCTION_CLONE_NORMALIZED_VERSION",
    ]) {
      expect(Object.hasOwn(functionCloneArtifact, symbol), symbol).toBe(false);
    }
  });
});
