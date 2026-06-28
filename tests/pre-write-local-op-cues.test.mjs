import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
} from "../_lib/pre-write-cue-tiers.mjs";

const LOCAL_POLICY_ID = "prewrite-local-operation-sibling";
const LOCAL_POLICY_VERSION = "prewrite-local-operation-sibling-v1";
const GET_WORLD_ID = "src/repository.ts::createRepository#getWorld";

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

describe("pre-write local-operation cue adapter", () => {
  it("T4h-T4i. renders promoted local operations as review cues with copied policy evidence", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "searchWorld",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [],
          semanticHints: [],
          suppressedNearNames: [],
          suppressedSemanticHints: [],
          serviceOperationSiblingPolicy: {
            policyId: "prewrite-service-operation-sibling-cue",
            policyVersion: "prewrite-service-operation-sibling-cue-v1",
            promoted: [],
            muted: [],
          },
          localOperationSiblingPolicy: {
            policyId: LOCAL_POLICY_ID,
            policyVersion: LOCAL_POLICY_VERSION,
            status: "complete",
            promoted: [
              {
                identity: GET_WORLD_ID,
                name: "getWorld",
                ownerFile: "src/repository.ts",
                matchedField: "preWriteLocalOperationIndex",
                surfaceKind: "nested-local-operation",
                containerName: "createRepository",
                containerKind: "function-declaration",
                operationFamily: "read-query",
                sharedDomainTokens: ["world"],
                supportingReasons: ["local-operation-same-file-domain-overlap"],
                locality: { sameDir: true, sameFile: true },
                eligibleForDeadExportRanking: false,
                eligibleForSafeFix: false,
              },
            ],
            muted: [],
          },
        },
      ],
      intent: buildIntent(["searchWorld"]),
    });

    const card = findCard(result, GET_WORLD_ID);
    const cue = card?.cues.find(
      (entry) => entry.evidenceLane === "local-operation-sibling",
    );

    expect(card?.renderTier).toBe(CUE_TIERS.AGENT_REVIEW);
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.AGENT_REVIEW,
      evidenceLane: "local-operation-sibling",
      claim: "related local service operation",
      confidence: "heuristic-review",
    });
    expect(cue?.evidence?.[0]).toMatchObject({
      artifact: "pre-write-advisory.json",
      matchedField: "lookups[].localOperationSiblingPolicy.promoted",
      policyId: LOCAL_POLICY_ID,
      policyVersion: LOCAL_POLICY_VERSION,
      candidateIdentity: GET_WORLD_ID,
      containerName: "createRepository",
      surfaceKind: "nested-local-operation",
      operationFamily: "read-query",
      sharedDomainTokens: ["world"],
      supportingReasons: ["local-operation-same-file-domain-overlap"],
      locality: { sameDir: true, sameFile: true },
    });
    expect(card?.cues.some((entry) => entry.cueTier === CUE_TIERS.SAFE)).toBe(
      false,
    );
    expect(cue?.evidence?.[0]?.eligibleForSafeFix).not.toBe(true);
  });

  it("T4j. keeps muted local-operation siblings out of cue cards", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "deleteWorld",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [],
          semanticHints: [],
          suppressedNearNames: [],
          suppressedSemanticHints: [],
          localOperationSiblingPolicy: {
            policyId: LOCAL_POLICY_ID,
            policyVersion: LOCAL_POLICY_VERSION,
            status: "complete",
            promoted: [],
            muted: [
              {
                identity: GET_WORLD_ID,
                name: "getWorld",
                ownerFile: "src/repository.ts",
                reason: "local-operation-operation-family-mismatch",
                matchedField: "preWriteLocalOperationIndex",
                surfaceKind: "nested-local-operation",
                containerName: "createRepository",
                containerKind: "function-declaration",
                operationFamily: "read-query",
                sharedDomainTokens: ["world"],
                locality: { sameDir: true, sameFile: true },
              },
            ],
          },
        },
      ],
      intent: buildIntent(["deleteWorld"]),
    });

    expect(result.cueCards).toHaveLength(0);
    expect(result.suppressedCues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          identity: GET_WORLD_ID,
          evidenceLane: "local-operation-sibling",
          reason: "local-operation-operation-family-mismatch",
        }),
      ]),
    );
  });
});
