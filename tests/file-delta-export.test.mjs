import { describe, expect, it } from "vitest";

const fileDelta = await import("../_lib/post-write-file-delta.mjs");

describe("file-delta export surface", () => {
  it("FDES1. exposes delta APIs, not path normalizer internals", () => {
    expect(typeof fileDelta.computeFileDelta).toBe("function");
    expect(typeof fileDelta.repoRelativeFileList).toBe("function");
    expect(Object.hasOwn(fileDelta, "normalizeRepoRelativePath")).toBe(false);
  });
});
