import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
  UNAVAILABLE_STATUS,
} from "../_lib/pre-write-cue-tiers.mjs";

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

describe("pre-write unavailable and policy-excluded cue adapter", () => {
  it("T5-T5b. keeps unavailable shape evidence separate from cue cards and suppressed cues", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "shape",
          shape: {
            fields: [],
            typeLiteral: "<S>(selector: (state: S) => S) => S",
          },
          result: "UNAVAILABLE",
          reason: "missing-artifact",
          artifact: "function-clones.json",
          citations: ["[확인 불가, function-clones.json absent]"],
        },
      ],
      intent: buildIntent(),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toHaveLength(0);
    expect(result.unavailableEvidence).toHaveLength(1);
    expect(result.unavailableEvidence[0]).toMatchObject({
      status: UNAVAILABLE_STATUS,
      reason: "missing-artifact",
      artifact: "function-clones.json",
    });
  });

  it("T6-T6b. keeps policy-excluded exact evidence suppressed with original safe context", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "generatedHelper",
          result: "EXISTS",
          identities: [
            {
              identity: "dist/generated.ts::generatedHelper",
              ownerFile: "dist/generated.ts",
              exportedName: "generatedHelper",
              policyExcluded: true,
              policyReason: "generated-output",
              citations: ["[grounded, exact symbol exists]"],
            },
          ],
          nearNames: [],
          semanticHints: [],
          suppressedSemanticHints: [],
        },
      ],
      intent: buildIntent({ names: ["generatedHelper"] }),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          reason: "policy-excluded",
          policyReason: "generated-output",
          originalCueTier: CUE_TIERS.SAFE,
          claim: "exact exported symbol exists",
          evidence: expect.arrayContaining([
            expect.objectContaining({ artifact: "symbols.json" }),
          ]),
        }),
      ]),
    );
  });
});
