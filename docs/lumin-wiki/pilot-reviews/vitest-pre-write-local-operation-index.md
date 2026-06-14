# Vitest Pre-Write Local Operation Index Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-18.
> **Pilot candidate:** `tests/test-pre-write-local-operation-index.mjs`.

---

## Purpose

This review decides whether `tests/test-pre-write-local-operation-index.mjs`
can move as one narrow WT-24 Vitest mirror batch. It does not change local
operation behavior.

The suite is acceptable as a single-suite batch because it protects the
artifact-only WT-23 local-operation surface and the lookup-only review evidence
policy:

- `symbols.json.preWriteLocalOperationIndex` advertises a complete,
  versioned local-operation surface;
- nested read/query operations inside an exported repository factory are
  indexed with owner file, container, operation-family, domain-token, and
  review-only eligibility metadata;
- mutation and generic helpers stay outside the v1 surface;
- nested local operations do not contaminate `defIndex`, `classMethodIndex`,
  `nearNames[]`, or `semanticHints[]`;
- `lookupName().localOperationSiblingPolicy` remains separate from
  `serviceOperationSiblingPolicy`.

This batch must stay separate from cue-tier rendering, Markdown wording,
service-operation policy calibration, dead-export ranking, resolver behavior,
and performance/incremental cache identity.

## Reviewed Evidence

| Suite                                           | Preserved Node Command                               | Proposed Focused Vitest Command                          | Surface Under Review                               |
| ----------------------------------------------- | ---------------------------------------------------- | -------------------------------------------------------- | -------------------------------------------------- |
| `tests/test-pre-write-local-operation-index.mjs` | `node tests/test-pre-write-local-operation-index.mjs` | `npm run test:vitest:pre-write-local-operation-index` | nested local-operation artifact and lookup policy |

Current preserved-command evidence:

```text
node tests/test-pre-write-local-operation-index.mjs
8 passed, 0 failed
```

Goal lane: WT-24, pre-write lifecycle. This review covers only the
artifact/index and lookup-policy evidence surface.

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The implementation PR may add a focused mirror for this suite because the
current Node test already builds a small temporary repository, runs
`build-symbol-graph.mjs`, and asserts stable artifact/lookup contracts. The
mirror must keep the Node entrypoint runnable and must not promote local
operations into cue-tier or ranking surfaces.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `symbols.meta.supports.nestedLocalOperationIndex === true`;
- `symbols.preWriteLocalOperationIndex.schemaVersion ===
  "pre-write-local-operations.v1"`;
- `symbols.preWriteLocalOperationIndex.status === "complete"`;
- nested read/query operations are indexed with stable identities such as
  `src/repository.ts::createRepository#getWorld`;
- indexed entries preserve `containerName`, closed `containerKind`,
  `matchedField: "preWriteLocalOperationIndex"`, `operationFamily`,
  `domainTokens`, `eligibleForDeadExportRanking: false`, and
  `eligibleForSafeFix: false`;
- const-arrow local operations keep the same exported factory container
  identity;
- mutation and generic helpers stay absent from the v1 local-operation surface;
- local operations do not appear in `defIndex` or `classMethodIndex`;
- local operations do not appear in formal `nearNames[]` or `semanticHints[]`;
- `localOperationSiblingPolicy` emits separate review evidence with policy id,
  policy version, shared domain tokens, support reasons, same-file locality,
  and review-only eligibility flags;
- `serviceOperationSiblingPolicy` does not receive nested local-operation
  identities.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a missing or incomplete `preWriteLocalOperationIndex` advertised as complete
  must fail;
- nested operations mirrored into `defIndex` or `classMethodIndex` must fail;
- a local operation entering `nearNames[]` or `semanticHints[]` must fail;
- mutation helpers such as `deleteWorld` entering the v1 surface must fail;
- generic helpers such as `normalizeInput` entering the v1 surface must fail;
- a local-operation policy entry missing
  `local-operation-same-file-domain-overlap` must fail;
- a nested local operation cross-feeding into `serviceOperationSiblingPolicy`
  must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is a temporary repository with one
  `src/repository.ts` file and direct `lookupName()` calls against generated
  `symbols.json`.
- Shared helpers may write the temporary repository, run
  `build-symbol-graph.mjs`, read `symbols.json`, and construct the
  `searchWorld` lookup input.
- Shared helpers must not decide local-operation eligibility, mutation/generic
  filtering, formal lookup lane membership, service/local policy separation,
  cue-tier promotion, renderer wording, resolver behavior, deadness/ranking,
  or cache identity.
- The mirror must not absorb `tests/test-pre-write-cue-tiers.mjs`,
  `tests/test-pre-write-render.mjs`, `tests/test-pre-write-lookup-name.mjs`,
  service-operation corpus calibration, resolver suites, deadness/ranking
  suites, or performance/incremental suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/pre-write-local-operation-index.test.mjs`,
2. `npm run test:vitest:pre-write-local-operation-index`,
3. candidate-board updates moving `tests/test-pre-write-local-operation-index.mjs`
   to `DONE`.
