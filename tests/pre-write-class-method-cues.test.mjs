import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
} from "../_lib/pre-write-cue-tiers.mjs";

const CLASS_METHOD_IDENTITY =
  "src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete";

function findCard(result, identity) {
  return result.cueCards.find((card) => card.candidate?.identity === identity);
}

function buildClassMethodCueResult() {
  return classifyPreWriteCues({
    lookups: [
      {
        kind: "name",
        intentName: "handleBulkDelete",
        result: "NOT_OBSERVED",
        identities: [],
        nearNames: [
          {
            name: "handleDelete",
            ownerFile: "src/event-dispatcher.ts",
            identity: CLASS_METHOD_IDENTITY,
            className: "TaskControlEventDispatcher",
            distance: 4,
            matchedField: "classMethodIndex",
          },
        ],
        semanticHints: [],
        suppressedSemanticHints: [],
        citations: ["[degraded, class method search hint only]"],
      },
    ],
    intent: {
      names: ["handleBulkDelete"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    },
  });
}

describe("pre-write class-method cue adapter", () => {
  it("T3c. turns class-method near-name evidence into a review cue card", () => {
    const result = buildClassMethodCueResult();
    const card = findCard(result, CLASS_METHOD_IDENTITY);
    const cue = card?.cues.find(
      (entry) =>
        entry.cueTier === CUE_TIERS.AGENT_REVIEW &&
        entry.evidenceLane === "class-method-name",
    );

    expect(card).toBeTruthy();
    expect(card?.renderTier).toBe(CUE_TIERS.AGENT_REVIEW);
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.AGENT_REVIEW,
      evidenceLane: "class-method-name",
      claim: "near class method name",
    });
    expect(card?.candidate).toMatchObject({
      identity: CLASS_METHOD_IDENTITY,
      ownerFile: "src/event-dispatcher.ts",
    });
  });

  it("T3d. cites classMethodIndex instead of defIndex and never emits safe-tier proof", () => {
    const result = buildClassMethodCueResult();
    const card = findCard(result, CLASS_METHOD_IDENTITY);
    const cue = card?.cues.find(
      (entry) => entry.evidenceLane === "class-method-name",
    );
    const cueTiers = new Set(card?.cues.map((entry) => entry.cueTier));

    expect(cue?.evidence?.[0]).toMatchObject({
      matchedField: "classMethodIndex",
      candidateIdentity: CLASS_METHOD_IDENTITY,
    });
    expect(cue?.evidence?.[0]?.matchedField).not.toBe("defIndex");
    expect(cueTiers.has(CUE_TIERS.SAFE)).toBe(false);
    expect(card?.renderTier).not.toBe(CUE_TIERS.SAFE);
  });
});
