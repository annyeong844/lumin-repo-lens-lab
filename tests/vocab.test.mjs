import { describe, expect, it } from "vitest";

import { requiredAcknowledgements } from "../_lib/post-write-delta.mjs";
import {
  BLOCKING_TAINTS,
  DELTA_LABELS,
  DELTA_LABEL_VALUES,
  EVIDENCE,
  EVIDENCE_VALUES,
  SOFT_TAINTS,
  TAINT,
  getProvenanceFieldNames,
  provenanceFields,
} from "../_lib/vocab.mjs";

describe("vocab evidence labels", () => {
  it("pins evidence label literals", () => {
    expect(EVIDENCE.AST_REF_COUNT).toBe("ast-ident-ref-count");
    expect(EVIDENCE.TEXT_ZERO_REF_COUNT).toBe("text-zero-ident-ref-count");
    expect(EVIDENCE.REGEX_FALLBACK).toBe("regex-text-fallback-parse-error");
    expect(EVIDENCE.PARSE_ERROR).toBe("parse-error");
  });

  it("keeps EVIDENCE_VALUES exactly mirrored to EVIDENCE", () => {
    expect(
      Object.values(EVIDENCE).every((value) => EVIDENCE_VALUES.has(value)),
    ).toBe(true);
    expect(EVIDENCE_VALUES.size).toBe(Object.values(EVIDENCE).length);
  });
});

describe("vocab taint labels and severity groups", () => {
  it("pins taint label literals", () => {
    expect(TAINT.UNRESOLVED_SPEC_MATCH).toBe(
      "unresolved-specifier-could-match",
    );
    expect(TAINT.UNRESOLVED_SPEC_MATCH_UNKNOWN).toBe(
      "unresolved-specifier-could-match-unknown",
    );
    expect(TAINT.RESOLVER_BLIND_ZONE_RELEVANT).toBe(
      "resolver-blind-zone-relevant",
    );
    expect(TAINT.GENERATED_ARTIFACT_MISSING_RELEVANT).toBe(
      "generated-artifact-missing-relevant",
    );
    expect(TAINT.DEFINING_FILE_PARSE_ERROR).toBe("defining-file-parse-error");
    expect(TAINT.PARSE_ERRORS_ELSEWHERE).toBe("parse-errors-present");
  });

  it("keeps blocking and soft taint groups complete and disjoint", () => {
    expect(BLOCKING_TAINTS.has(TAINT.UNRESOLVED_SPEC_MATCH)).toBe(true);
    expect(BLOCKING_TAINTS.has(TAINT.DEFINING_FILE_PARSE_ERROR)).toBe(true);
    expect(SOFT_TAINTS.has(TAINT.PARSE_ERRORS_ELSEWHERE)).toBe(true);
    expect(SOFT_TAINTS.has(TAINT.RESOLVER_BLIND_ZONE_RELEVANT)).toBe(true);

    expect([...BLOCKING_TAINTS].some((key) => SOFT_TAINTS.has(key))).toBe(
      false,
    );
    expect(
      Object.values(TAINT).every(
        (value) => BLOCKING_TAINTS.has(value) || SOFT_TAINTS.has(value),
      ),
    ).toBe(true);
  });

  it("keeps exported vocab containers frozen", () => {
    expect(Object.isFrozen(EVIDENCE)).toBe(true);
    expect(Object.isFrozen(TAINT)).toBe(true);
    expect(Object.isFrozen(BLOCKING_TAINTS)).toBe(true);
    expect(Object.isFrozen(SOFT_TAINTS)).toBe(true);
  });
});

describe("provenanceFields", () => {
  it("forwards only known provenance fields from classified candidates", () => {
    const candidate = {
      symbol: "x",
      file: "src/x.ts",
      line: 1,
      kind: "VariableDeclaration",
      fileInternalUses: 0,
      fileInternalUsesEvidence: "ast-ident-ref-count",
      fileInternalRefs: { typeRefs: 0, valueRefs: 0 },
      supportedBy: [{ kind: "ast-ident-ref-count", count: 0 }],
      taintedBy: [],
      resolverConfidence: "high",
      parseStatus: "ok",
      declarationExportDependency: true,
      declarationExportRefs: { count: 1, lines: [2] },
    };

    const out = provenanceFields(candidate);

    expect(out.fileInternalUsesEvidence).toBe(
      candidate.fileInternalUsesEvidence,
    );
    expect(out.fileInternalRefs).toBe(candidate.fileInternalRefs);
    expect(out.supportedBy).toBe(candidate.supportedBy);
    expect(out.taintedBy).toBe(candidate.taintedBy);
    expect(out.resolverConfidence).toBe(candidate.resolverConfidence);
    expect(out.parseStatus).toBe(candidate.parseStatus);
    expect(out.declarationExportDependency).toBe(
      candidate.declarationExportDependency,
    );
    expect(out.declarationExportRefs).toBe(candidate.declarationExportRefs);

    expect(out.symbol).toBeUndefined();
    expect(out.file).toBeUndefined();
    expect(out.line).toBeUndefined();
  });

  it("omits undefined values instead of forwarding undefined own keys", () => {
    const out = provenanceFields({
      fileInternalUsesEvidence: undefined,
      parseStatus: "ok",
    });

    expect("fileInternalUsesEvidence" in out).toBe(false);
    expect(out.parseStatus).toBe("ok");
  });

  it("returns a fresh provenance field name copy each time", () => {
    const names = getProvenanceFieldNames();

    expect(Array.isArray(names)).toBe(true);
    expect(names.length).toBeGreaterThan(0);

    names.push("hackedField");

    expect(getProvenanceFieldNames()).not.toContain("hackedField");
  });
});

describe("delta labels", () => {
  it("pins the canonical six delta label literals", () => {
    expect(DELTA_LABELS.PLANNED).toBe("planned");
    expect(DELTA_LABELS.PLANNED_NOT_OBSERVED).toBe("planned-not-observed");
    expect(DELTA_LABELS.SILENT_NEW).toBe("silent-new");
    expect(DELTA_LABELS.PRE_EXISTING).toBe("pre-existing");
    expect(DELTA_LABELS.REMOVED).toBe("removed");
    expect(DELTA_LABELS.OBSERVED_UNBASELINED).toBe("observed-unbaselined");
  });

  it("keeps delta label values exact, mirrored, and frozen", () => {
    const expected = new Set([
      "planned",
      "planned-not-observed",
      "silent-new",
      "pre-existing",
      "removed",
      "observed-unbaselined",
    ]);
    const actual = new Set(Object.values(DELTA_LABELS));

    expect(Object.keys(DELTA_LABELS)).toHaveLength(6);
    expect(DELTA_LABEL_VALUES.size).toBe(6);
    expect(
      Object.values(DELTA_LABELS).every((value) =>
        DELTA_LABEL_VALUES.has(value),
      ),
    ).toBe(true);
    expect(Object.isFrozen(DELTA_LABELS)).toBe(true);
    expect(actual).toEqual(expected);
  });

  it("requires acknowledgements only for silent-new delta entries", () => {
    const fakeDelta = {
      entries: Object.values(DELTA_LABELS).map((label) => ({
        label,
        diagnostics: [],
      })),
    };

    const required = requiredAcknowledgements(fakeDelta);

    expect(required).toHaveLength(1);
    expect(required[0].label).toBe(DELTA_LABELS.SILENT_NEW);
  });
});
