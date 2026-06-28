import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
} from "../_lib/pre-write-cue-tiers.mjs";

const SERVICE_POLICY_ID = "prewrite-service-operation-sibling-cue";
const SERVICE_POLICY_VERSION = "prewrite-service-operation-sibling-cue-v1";
const FETCH_USER_ID = "src/services/user.ts::fetchUser";

function buildIntent(names) {
  return {
    names,
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  };
}

function findCard(result, identity) {
  return result.cueCards.find((card) => card.candidate?.identity === identity);
}

describe("pre-write service-operation cue adapter", () => {
  it("T4c-T4e. renders promoted service-operation siblings as review cues while preserving suppressed diagnostics", () => {
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
          serviceOperationSiblingPolicy: {
            policyId: SERVICE_POLICY_ID,
            policyVersion: SERVICE_POLICY_VERSION,
            promoted: [
              {
                identity: FETCH_USER_ID,
                name: "fetchUser",
                ownerFile: "src/services/user.ts",
                operationFamily: "read-query",
                sharedDomainTokens: ["user"],
                supportingReasons: [
                  "near-distance-exceeded",
                  "single-non-weak-token-only",
                ],
                locality: { sameDir: true, sameFile: false },
                signatureSupport: {
                  status: "unavailable",
                  reason: "no-signature-facts",
                },
              },
            ],
            muted: [],
          },
        },
      ],
      intent: buildIntent(["searchUser"]),
    });

    const card = findCard(result, FETCH_USER_ID);
    const cue = card?.cues.find(
      (entry) => entry.evidenceLane === "service-operation-sibling",
    );

    expect(card?.renderTier).toBe(CUE_TIERS.AGENT_REVIEW);
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.AGENT_REVIEW,
      evidenceLane: "service-operation-sibling",
      claim: "related service operation sibling",
      confidence: "heuristic-review",
    });
    expect(cue?.evidence?.[0]).toMatchObject({
      artifact: "pre-write-advisory.json",
      matchedField: "lookups[].serviceOperationSiblingPolicy.promoted",
      policyId: SERVICE_POLICY_ID,
      policyVersion: SERVICE_POLICY_VERSION,
      candidateIdentity: FETCH_USER_ID,
      operationFamily: "read-query",
      sharedDomainTokens: ["user"],
    });
    expect(card?.cues.some((entry) => entry.cueTier === CUE_TIERS.SAFE)).toBe(
      false,
    );
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ reason: "near-distance-exceeded" }),
        expect.objectContaining({ reason: "single-non-weak-token-only" }),
      ]),
    );
  });

  it("T4f. keeps muted service-operation siblings out of cue cards", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "createUser",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [],
          semanticHints: [],
          suppressedNearNames: [],
          suppressedSemanticHints: [],
          serviceOperationSiblingPolicy: {
            policyId: SERVICE_POLICY_ID,
            policyVersion: SERVICE_POLICY_VERSION,
            promoted: [],
            muted: [
              {
                identity: FETCH_USER_ID,
                name: "fetchUser",
                ownerFile: "src/services/user.ts",
                reason: "service-sibling-operation-family-mismatch",
                operationFamily: "read-query",
                sharedDomainTokens: ["user"],
              },
            ],
          },
        },
      ],
      intent: buildIntent(["createUser"]),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          identity: FETCH_USER_ID,
          evidenceLane: "service-operation-sibling",
          reason: "service-sibling-operation-family-mismatch",
        }),
      ]),
    );
  });

  it("T4g. suppresses class-method and generated service-operation candidates", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "searchUser",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [],
          semanticHints: [],
          suppressedNearNames: [],
          suppressedSemanticHints: [],
          serviceOperationSiblingPolicy: {
            policyId: SERVICE_POLICY_ID,
            policyVersion: SERVICE_POLICY_VERSION,
            promoted: [
              {
                identity:
                  "src/event-dispatcher.ts::TaskControlEventDispatcher#searchUser",
                name: "searchUser",
                ownerFile: "src/event-dispatcher.ts",
                matchedField: "classMethodIndex",
                operationFamily: "read-query",
                sharedDomainTokens: ["user"],
              },
              {
                identity: "dist/generated/user.ts::fetchUser",
                name: "fetchUser",
                ownerFile: "dist/generated/user.ts",
                operationFamily: "read-query",
                sharedDomainTokens: ["user"],
              },
            ],
            muted: [],
          },
        },
      ],
      intent: buildIntent(["searchUser"]),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          evidenceLane: "service-operation-sibling",
          reason: "service-sibling-class-method-lane",
        }),
        expect.objectContaining({
          reason: "policy-excluded",
          policyReason: "path:dist",
        }),
      ]),
    );
  });
});
