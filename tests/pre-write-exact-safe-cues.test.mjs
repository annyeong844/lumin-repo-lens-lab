import { describe, expect, it } from "vitest";

import {
  classifyPreWriteCues,
  CUE_TIERS,
} from "../_lib/pre-write-cue-tiers.mjs";

function findCard(result, identity) {
  return result.cueCards.find((card) => card.candidate?.identity === identity);
}

describe("pre-write exact and signature safe cue adapter", () => {
  it("T1-T1b. turns exact symbol identity into claim-only SAFE_CUE evidence", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "formatDate",
          result: "EXISTS",
          identities: [
            {
              identity: "src/date.ts::formatDate",
              ownerFile: "src/date.ts",
              exportedName: "formatDate",
              fanIn: 3,
              fanInConfidence: "grounded",
              citations: [
                "[grounded, symbols.json.fanInByIdentity['src/date.ts::formatDate'] = 3]",
              ],
            },
          ],
          nearNames: [],
          semanticHints: [],
          suppressedSemanticHints: [],
        },
      ],
      intent: {
        names: ["formatDate"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
    });

    const card = findCard(result, "src/date.ts::formatDate");
    const cue = card?.cues.find(
      (entry) =>
        entry.cueTier === CUE_TIERS.SAFE &&
        entry.evidenceLane === "exact-symbol",
    );

    expect(card).toBeTruthy();
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.SAFE,
      safeMeaning: "claim-only",
      evidenceLane: "exact-symbol",
      claim: "exact exported symbol exists",
    });
    expect(cue?.notSafeFor).toContain("semantic-equivalence");
    expect(card?.candidate).toMatchObject({
      identity: "src/date.ts::formatDate",
      ownerFile: "src/date.ts",
      exportedName: "formatDate",
    });
    expect(cue?.evidence?.[0]).toMatchObject({
      artifact: "symbols.json",
      matchedField: "defIndex",
      candidateIdentity: "src/date.ts::formatDate",
      algorithmVersion: "exact-symbol.v1",
    });
  });

  it("T2-T2b. turns function signature match into SAFE_CUE evidence", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "shape",
          result: "SIGNATURE_MATCH",
          shapeHash:
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          shapeHashSource: "functionSignature",
          signature: "<S,U>((S)=>U):(S)=>U",
          matches: [
            {
              identity: "src/shallow.ts::useShallow",
              ownerFile: "src/shallow.ts",
              exportedName: "useShallow",
              confidence: "medium",
            },
          ],
          citations: [
            "[grounded, function-clones.json facts[] matched 1 identities]",
          ],
        },
      ],
      intent: {
        names: ["composeProjection"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
    });

    const card = findCard(result, "src/shallow.ts::useShallow");
    const cue = card?.cues.find(
      (entry) =>
        entry.cueTier === CUE_TIERS.SAFE &&
        entry.evidenceLane === "function-signature",
    );

    expect(card).toBeTruthy();
    expect(cue).toMatchObject({
      cueTier: CUE_TIERS.SAFE,
      safeMeaning: "claim-only",
      evidenceLane: "function-signature",
      claim: "same normalized function signature",
    });
    expect(cue?.evidence?.[0]).toMatchObject({
      artifact: "function-clones.json",
      matchedField: "normalizedSignatureHash",
      algorithmVersion: "function-signature.normalized.v1",
      hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    });
  });

  it("keeps file-local function signature matches at review tier", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "shape",
          result: "SIGNATURE_MATCH",
          shapeHash:
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
          shapeHashSource: "functionSignature",
          signature: "(string):string",
          matches: [
            {
              identity: "src/user-a.ts::normalizeUserName",
              ownerFile: "src/user-a.ts",
              exportedName: "normalizeUserName",
              localName: "normalizeUserName",
              visibility: "file-local",
              exported: false,
              confidence: "high",
            },
          ],
          citations: [
            "[grounded, function-clones.json facts[] matched 1 identities]",
          ],
        },
      ],
      intent: {
        names: ["normalizeUserName"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
    });

    const card = findCard(result, "src/user-a.ts::normalizeUserName");
    const reviewCue = card?.cues.find(
      (entry) =>
        entry.cueTier === CUE_TIERS.AGENT_REVIEW &&
        entry.evidenceLane === "function-signature",
    );

    expect(card?.renderTier).toBe(CUE_TIERS.AGENT_REVIEW);
    expect(reviewCue).toBeTruthy();
    expect(reviewCue?.evidence?.[0]).toMatchObject({
      artifact: "function-clones.json",
      matchedField: "normalizedSignatureHash",
      visibility: "file-local",
    });
    expect(
      card?.cues.some((entry) => entry.cueTier === CUE_TIERS.SAFE),
    ).toBe(false);
  });

  it("keeps exported default function signature matches at safe tier", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "shape",
          result: "SIGNATURE_MATCH",
          shapeHash:
            "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
          shapeHashSource: "functionSignature",
          signature: "(string):string",
          matches: [
            {
              identity: "src/default-fn.ts::default",
              ownerFile: "src/default-fn.ts",
              exportedName: "default",
              localName: "normalizePayload",
              visibility: "exported",
              exported: true,
              confidence: "high",
            },
          ],
          citations: [
            "[grounded, function-clones.json facts[] matched 1 identities]",
          ],
        },
      ],
      intent: {
        names: ["normalizePayload"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
    });

    const card = findCard(result, "src/default-fn.ts::default");
    const cue = card?.cues.find(
      (entry) =>
        entry.cueTier === CUE_TIERS.SAFE &&
        entry.evidenceLane === "function-signature",
    );

    expect(card?.renderTier).toBe(CUE_TIERS.SAFE);
    expect(cue).toBeTruthy();
    expect(cue?.evidence?.[0]).toMatchObject({
      visibility: "exported",
      localName: "normalizePayload",
    });
  });

  it("T3-T3b. keeps safe and review cues while rendering mixed candidates at review tier", () => {
    const result = classifyPreWriteCues({
      lookups: [
        {
          kind: "name",
          intentName: "useShallowFromState",
          result: "NOT_OBSERVED",
          identities: [],
          nearNames: [
            { name: "useShallow", ownerFile: "src/shallow.ts", distance: 2 },
          ],
          semanticHints: [],
          suppressedSemanticHints: [],
          citations: ["[degraded, fuzzy-name match; search hint only]"],
        },
        {
          kind: "shape",
          result: "SIGNATURE_MATCH",
          shapeHash:
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
          shapeHashSource: "functionSignature",
          signature: "<S,U>((S)=>U):(S)=>U",
          matches: [
            {
              identity: "src/shallow.ts::useShallow",
              ownerFile: "src/shallow.ts",
              exportedName: "useShallow",
            },
          ],
          citations: [
            "[grounded, function-clones.json facts[] matched 1 identities]",
          ],
        },
      ],
      intent: {
        names: ["useShallowFromState"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      },
    });

    const card = findCard(result, "src/shallow.ts::useShallow");
    const cueTiers = new Set(card?.cues?.map((cue) => cue.cueTier));

    expect(card?.renderTier).toBe(CUE_TIERS.AGENT_REVIEW);
    expect(cueTiers.has(CUE_TIERS.SAFE)).toBe(true);
    expect(cueTiers.has(CUE_TIERS.AGENT_REVIEW)).toBe(true);
    expect(card?.cues).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ evidenceLane: "function-signature" }),
        expect.objectContaining({ evidenceLane: "near-name" }),
      ]),
    );
  });
});
