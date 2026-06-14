import { describe, expect, it } from "vitest";

import {
  CALIBRATION_CORPUS_SCHEMA_VERSION,
  calibrationCorpusSummary,
  getCalibrationCorpus,
  listCalibrationCorpusIds,
} from "../_lib/calibration-corpora.mjs";
import { THRESHOLD_POLICIES } from "../_lib/threshold-policies.mjs";

describe("calibration corpus registry", () => {
  it("lists registered pre-write and resolver corpus ids", () => {
    const ids = listCalibrationCorpusIds();

    expect(ids).toContain("calibration-2026-05-prewrite-v1");
    expect(ids).toContain("calibration-2026-05-resolver-v1");
  });

  it("ensures every threshold policy references a known corpus", () => {
    for (const policy of Object.values(THRESHOLD_POLICIES)) {
      const corpusId = policy.calibration?.corpus;
      expect(corpusId, `${policy.policyId} must name a corpus`).toBeTruthy();

      const corpus = getCalibrationCorpus(corpusId);
      expect(corpus.schemaVersion).toBe(CALIBRATION_CORPUS_SCHEMA_VERSION);
      expect(corpus.corpusId).toBe(corpusId);
      expect(corpus.metrics.length).toBeGreaterThan(0);
    }
  });

  it("preserves pre-write and resolver corpus purposes and metrics", () => {
    const prewrite = getCalibrationCorpus("calibration-2026-05-prewrite-v1");
    const resolver = getCalibrationCorpus("calibration-2026-05-resolver-v1");

    expect(prewrite.purpose).toBe("pre-write cue and threshold calibration");
    expect(prewrite.metrics).toContain("precisionProxy");
    expect(prewrite.metrics).toContain("noiseRate");
    expect(resolver.purpose).toBe(
      "resolver blind-zone and completeness calibration",
    );
    expect(resolver.metrics).toContain("unresolvedInternalRate");
    expect(resolver.metrics).toContain("falseGlobalBlockerCount");
  });

  it("returns compact ordered corpus summaries without notes", () => {
    const summary = calibrationCorpusSummary([
      "calibration-2026-05-prewrite-v1",
      "calibration-2026-05-resolver-v1",
    ]);

    expect(summary.map((item) => item.corpusId)).toEqual([
      "calibration-2026-05-prewrite-v1",
      "calibration-2026-05-resolver-v1",
    ]);
    expect(summary[0]).not.toHaveProperty("notes");
  });

  it("throws a clear error for unknown corpus ids", () => {
    expect(() => getCalibrationCorpus("missing-corpus")).toThrow(
      /Unknown calibration corpus: missing-corpus/,
    );
  });
});
