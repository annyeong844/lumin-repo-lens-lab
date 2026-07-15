import { describe, expect, it } from "vitest";

import {
  THRESHOLD_POLICY_SCHEMA_VERSION,
  getThresholdPolicy,
  thresholdPolicySummary,
} from "../_lib/threshold-policies.mjs";

describe("threshold policy metadata", () => {
  it("records function clone near-policy ids, class, thresholds, and hashes", () => {
    const functionPolicy = getThresholdPolicy("function-clone-near-policy");

    expect(functionPolicy.schemaVersion).toBe(THRESHOLD_POLICY_SCHEMA_VERSION);
    expect(functionPolicy.policyId).toBe("function-clone-near-policy");
    expect(functionPolicy.policyVersion).toBe("function-clone-near-policy-v1");
    expect(functionPolicy.policyClass).toBe("review");
    expect(functionPolicy.thresholds.minSingleTokenIdf).toBe(3);
    expect(functionPolicy.thresholds.minCallTokenIdfScore).toBe(0.5);
    expect(functionPolicy.thresholds.weights.callTokenIdfScore).toBe(0.45);
    expect(functionPolicy.thresholds.minNearScore).toBe(0.62);
    expect(functionPolicy.thresholds.maxNearCandidates).toBe(50);
    expect(functionPolicy.candidateGenerationMode).toBe("bounded-retrieval");
    expect(functionPolicy.projection).toBe("streaming-top-n");
    expect(functionPolicy.policyHash).toMatch(/^sha256:[a-f0-9]{64}$/);
    expect(functionPolicy.thresholdHash).toMatch(/^sha256:[a-f0-9]{64}$/);
  });

  it("records inline-pattern and resolver blind-zone policy thresholds", () => {
    const inlinePolicy = getThresholdPolicy("inline-pattern-policy");
    const resolverPolicy = getThresholdPolicy("resolver-blind-zone-policy");

    expect(inlinePolicy.policyClass).toBe("review");
    expect(inlinePolicy.thresholds.minOccurrences).toBe(3);
    expect(inlinePolicy.thresholds.maxCatchStatements).toBe(2);
    expect(resolverPolicy.policyClass).toBe("confidence");
    expect(resolverPolicy.thresholds.unresolvedRatio).toBe(0.15);
    expect(resolverPolicy.thresholds.absoluteUnresolvedCount).toBe(1000);
    expect(resolverPolicy.thresholds.prefixConcentrationMinUnresolved).toBe(
      100,
    );
    expect(resolverPolicy.thresholds.prefixConcentrationShare).toBe(0.8);
  });

  it("returns compact ordered policy summaries with calibration corpus metadata", () => {
    const functionPolicy = getThresholdPolicy("function-clone-near-policy");
    const summary = thresholdPolicySummary([
      "function-clone-near-policy",
      "inline-pattern-policy",
    ]);

    expect(summary.map((p) => p.policyId)).toEqual([
      "function-clone-near-policy",
      "inline-pattern-policy",
    ]);
    expect(summary[0].thresholds.minNearScore).toBe(0.62);
    expect(summary[0].candidateGenerationMode).toBe("bounded-retrieval");
    expect(summary[0].thresholdHash).toBe(functionPolicy.thresholdHash);
    expect(summary[0].calibrationCorpus?.corpusId).toBe(
      "calibration-2026-05-prewrite-v1",
    );
    expect(summary[0].calibrationCorpus?.entryCount).toBe(3);
    expect(summary[0].calibrationCorpus?.metrics).toContain("precisionProxy");
    expect(summary[0]).not.toHaveProperty("notes");
  });
});
