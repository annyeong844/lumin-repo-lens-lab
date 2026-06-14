import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
  UNAVAILABLE_STATUS,
} from "../_lib/pre-write-cue-tiers.mjs";
import { tokenizePreWrite } from "../_lib/pre-write-token-policy.mjs";

function buildIntent(fields = {}) {
  return {
    names: [],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
    ...fields,
  };
}

function findCard(result, identity) {
  return result.cueCards.find((card) => card.candidate?.identity === identity);
}

describe("pre-write file token and inline cue adapter", () => {
  it("T7. turns exact file existence into exact-file SAFE_CUE evidence", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "file",
          intentFile: "src/logger.ts",
          result: "FILE_EXISTS",
        },
      ],
      intent: buildIntent({ files: ["src/logger.ts"] }),
    });

    const card = findCard(result, "src/logger.ts::__file__");
    const cue = card?.cues.find(
      (entry) =>
        entry.cueTier === CUE_TIERS.SAFE && entry.evidenceLane === "exact-file",
    );

    expect(card).toBeTruthy();
    expect(card?.candidate).toMatchObject({
      identity: "src/logger.ts::__file__",
      ownerFile: "src/logger.ts",
      exportedName: "__file__",
    });
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.SAFE,
      safeMeaning: "claim-only",
      evidenceLane: "exact-file",
      claim: "exact file exists",
    });
    expect(cue?.evidence?.[0]).toMatchObject({
      artifact: "topology.json",
      matchedField: "nodes",
      file: "src/logger.ts",
      algorithmVersion: "exact-file.v1",
    });
  });

  it("T8. preserves important pre-write token stems", () => {
    expect(tokenizePreWrite("className")).toContain("class");
    expect(tokenizePreWrite("processConfig")).toContain("process");
    expect(tokenizePreWrite("statusCheck")).toContain("status");
    expect(tokenizePreWrite("analysisReport")).toContain("analysis");
  });

  it("T9-T9b. renders inline-pattern matches as review-only extraction evidence", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "inline-pattern",
          result: "INLINE_PATTERN_MATCH",
          groups: [
            {
              patternHash: "sha256:catch-destroy",
              kind: "catch-block",
              size: 4,
              ownerFiles: ["src/server.ts"],
              occurrences: [
                { file: "src/server.ts", line: 498, endLine: 500 },
                { file: "src/server.ts", line: 577, endLine: 579 },
              ],
              reviewReason:
                "same normalized catch block; verify socket ownership before extracting",
            },
          ],
        },
      ],
      intent: buildIntent({ names: ["writeOrDestroyConnection"] }),
    });

    const card = findCard(result, "inline-pattern:sha256:catch-destroy");
    const cue = card?.cues.find(
      (entry) => entry.evidenceLane === "inline-extraction",
    );

    expect(card).toBeTruthy();
    expect(card?.renderTier).toBe(CUE_TIERS.AGENT_REVIEW);
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.AGENT_REVIEW,
      evidenceLane: "inline-extraction",
      claim: "repeated inline statement pattern",
    });
    expect(cue?.cueTier).not.toBe(CUE_TIERS.SAFE);
    expect(cue?.evidence?.[0]).toMatchObject({
      artifact: "inline-patterns.json",
      matchedField: "groups[].patternHash",
      patternHash: "sha256:catch-destroy",
      occurrenceCount: 4,
      reviewReason:
        "same normalized catch block; verify socket ownership before extracting",
    });
  });

  it("T10. keeps missing inline-pattern artifacts as unavailable evidence only", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "inline-pattern",
          result: "UNAVAILABLE",
          reason: "missing-artifact",
          artifact: "inline-patterns.json",
          citations: ["[확인 불가, inline-patterns.json absent]"],
        },
      ],
      intent: buildIntent(),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toHaveLength(0);
    expect(result.unavailableEvidence).toEqual([
      expect.objectContaining({
        evidenceLane: "inline-extraction",
        status: UNAVAILABLE_STATUS,
        reason: "missing-artifact",
        artifact: "inline-patterns.json",
      }),
    ]);
  });
});
