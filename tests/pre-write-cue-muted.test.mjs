import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
} from "../_lib/pre-write-cue-tiers.mjs";

function buildIntent(names) {
  return {
    names,
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  };
}

describe("pre-write suppressed cue adapter", () => {
  it("T4. turns domain-token-overlap semantic suppressions into muted cues", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "createLogger",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [],
          semanticHints: [],
          suppressedSemanticHints: [
            {
              name: "createStore",
              ownerFile: "src/store.ts",
              matchedTokens: ["create"],
              reason: "domain-token-overlap",
              candidateCount: 2,
            },
            {
              name: "createJSONStorage",
              ownerFile: "src/storage.ts",
              matchedTokens: ["create"],
              reason: "domain-token-overlap",
              candidateCount: 2,
            },
          ],
        },
      ],
      intent: buildIntent(["createLogger"]),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toHaveLength(2);
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          exportedName: "createStore",
          identity: "src/store.ts::createStore",
          cueTier: CUE_TIERS.MUTED,
          evidenceLane: "intent-token",
          reason: "domain-token-overlap",
          tokenPolicyVersion: "prewrite-token-policy-v1",
          candidateCount: 2,
        }),
        expect.objectContaining({
          exportedName: "createJSONStorage",
          identity: "src/storage.ts::createJSONStorage",
          cueTier: CUE_TIERS.MUTED,
          evidenceLane: "intent-token",
          reason: "domain-token-overlap",
          tokenPolicyVersion: "prewrite-token-policy-v1",
          candidateCount: 2,
        }),
      ]),
    );
  });

  it("T4b. keeps near-name and semantic suppressions muted without cue cards", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "searchUser",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [],
          semanticHints: [],
          suppressedNearNames: [
            {
              name: "fetchUser",
              ownerFile: "src/services/user.ts",
              matchedTokens: ["user"],
              distance: 3,
              reason: "near-distance-exceeded",
              locality: { sameDir: true, sameFile: false },
              candidateCount: 1,
            },
          ],
          suppressedSemanticHints: [
            {
              name: "fetchUser",
              ownerFile: "src/services/user.ts",
              matchedTokens: ["user"],
              score: 1,
              reason: "single-non-weak-token-only",
              locality: { sameDir: true, sameFile: false },
              candidateCount: 1,
            },
          ],
        },
      ],
      intent: buildIntent(["searchUser"]),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toHaveLength(2);
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          exportedName: "fetchUser",
          identity: "src/services/user.ts::fetchUser",
          cueTier: CUE_TIERS.MUTED,
          evidenceLane: "near-name",
          reason: "near-distance-exceeded",
          distance: 3,
          locality: { sameDir: true, sameFile: false },
          candidateCount: 1,
        }),
        expect.objectContaining({
          exportedName: "fetchUser",
          identity: "src/services/user.ts::fetchUser",
          cueTier: CUE_TIERS.MUTED,
          evidenceLane: "intent-token",
          reason: "single-non-weak-token-only",
          score: 1,
          locality: { sameDir: true, sameFile: false },
          candidateCount: 1,
        }),
      ]),
    );
    expect(
      result.suppressedCues.some((cue) => cue.cueTier === CUE_TIERS.SAFE),
    ).toBe(false);
    expect(
      result.suppressedCues.some(
        (cue) => cue.cueTier === CUE_TIERS.AGENT_REVIEW,
      ),
    ).toBe(false);
  });
});
