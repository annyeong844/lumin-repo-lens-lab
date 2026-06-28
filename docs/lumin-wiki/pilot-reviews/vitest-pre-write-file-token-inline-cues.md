# Vitest Pre-Write File Token And Inline Cue Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T7-T10.

---

## Purpose

This review splits the exact-file, token-policy, and inline-pattern cue lane out
of the parked `tests/test-pre-write-cue-tiers.mjs` suite. It is implemented by
the focused Vitest mirror `tests/pre-write-file-inline-cues.test.mjs`.

The parent cue-tier suite remains parked because it also protects exact safe
cues, class-method review cues, suppressed diagnostics, service-operation
sibling cues, local-operation sibling cues, unavailable evidence, and
policy-excluded evidence. This page covers only the remaining adapter behavior:
exact file existence becomes file-scoped safe cue evidence, pre-write token
stemming preserves important domain stems, inline-pattern matches become
review-only extraction cues, and missing inline-pattern artifacts stay
unavailable.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command             | Surface Under Review          |
| ----------------- | ----------------------------------------- | ------------------------------------------- | ----------------------------- |
| T7-T10            | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-file-inline` | file/token/inline cue adapter |

Current preserved-command evidence:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-file-inline
4 passed, 0 failed
```

The focused mirror extracts only the T7, T8, T9/T9b, and T10 fixture shapes from
the Node suite. The preserved Node command remains the broader authority for the
parked parent cue-tier suite.

## Result

This lane is implemented as a narrow Vitest mirror batch.

The implementation adds one focused mirror for file, token, and inline-pattern
cue adaptation because the fixture boundary is static cue-tier input plus direct
tokenization assertions. The mirror does not run file-system probing,
inline-pattern extraction, renderer logic, resolver behavior, deadness/ranking,
or audit orchestration.

## Protected Invariants

The Vitest mirror preserves these contracts:

- an exact file lookup with `result: FILE_EXISTS` creates a cue card;
- exact file cues use `cueTier: SAFE_CUE`;
- exact file cues use `evidenceLane: exact-file`;
- exact file cues claim `exact file exists`;
- exact file identities use the `::<__file__>` identity convention;
- `tokenizePreWrite()` preserves `class`, `process`, `status`, and `analysis`
  stems;
- inline-pattern matches create cue cards;
- inline-pattern cue cards render at `cueTier: AGENT_REVIEW_CUE`;
- inline-pattern cues use `evidenceLane: inline-extraction`;
- inline-pattern cues claim `repeated inline statement pattern`;
- inline-pattern evidence cites `inline-patterns.json`;
- inline-pattern evidence preserves occurrence count;
- missing inline-pattern artifacts create `unavailableEvidence[]`;
- missing inline-pattern unavailable evidence keeps `evidenceLane:
inline-extraction`;
- missing inline-pattern evidence preserves `status: unavailable` and
  `reason: missing-artifact`;
- missing inline-pattern artifacts do not create `suppressedCues[]` or
  `cueCards[]`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- an exact file result failing to create a cue card must fail;
- an exact file cue losing the `exact-file` evidence lane must fail;
- an exact file cue rendering as review evidence instead of `SAFE_CUE` must
  fail;
- token policy dropping `class`, `process`, `status`, or `analysis` stems must
  fail;
- an inline-pattern match failing to create a review cue must fail;
- an inline-pattern cue becoming `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` proof must
  fail;
- an inline-pattern cue losing `inline-patterns.json` evidence must fail;
- an inline-pattern cue losing occurrence count must fail;
- missing inline-pattern artifacts looking like observed absence must fail;
- missing inline-pattern artifacts entering `suppressedCues[]` must fail.

## Fixture Boundary

The mirror constructs fixed inputs with:

- one file lookup for `src/logger.ts` with `result: FILE_EXISTS`;
- one tokenization assertion set for `className`, `processConfig`,
  `statusCheck`, and `analysisReport`;
- one inline-pattern lookup with `result: INLINE_PATTERN_MATCH`;
- one inline group with `patternHash`, kind, size, owner files, occurrences, and
  review reason;
- one inline-pattern lookup with `result: UNAVAILABLE`,
  `reason: missing-artifact`, and `artifact: inline-patterns.json`;
- minimal intent objects with file or name fields only.

The mirror must not compute file existence, inline-pattern groups, review
reasons, or missing-artifact status. It verifies cue-tier adaptation from
already-computed advisory evidence and direct token policy output.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- `cueCards[]` lookup by file or inline-pattern identity;
- cue lookup by `cueTier` and `evidenceLane`;
- unavailable evidence lookup by artifact and evidence lane;
- token-preservation assertions for a fixed word list;
- disallowed safe-tier checks for inline-pattern cues.

Shared helpers must not decide:

- whether a file exists;
- whether an inline pattern was extracted correctly;
- whether inline extraction should be available;
- whether repeated statements are safe to refactor;
- renderer wording;
- resolver behavior;
- dead-export ranking or action-safety proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb T1-T6b, exact/signature safe cues, class-method
  cues, suppressed diagnostics, service-operation policy, local-operation
  policy, unavailable/policy-excluded fixtures, or broader renderer fixtures
  from `tests/test-pre-write-cue-tiers.mjs`.
- The mirror must not absorb
  [`vitest-pre-write-exact-safe-cues.md`](vitest-pre-write-exact-safe-cues.md),
  [`vitest-pre-write-class-method-cues.md`](vitest-pre-write-class-method-cues.md),
  [`vitest-pre-write-cue-suppressed-diagnostics.md`](vitest-pre-write-cue-suppressed-diagnostics.md),
  [`vitest-pre-write-service-operation-cues.md`](vitest-pre-write-service-operation-cues.md),
  [`vitest-pre-write-local-operation-cues.md`](vitest-pre-write-local-operation-cues.md),
  or
  [`vitest-pre-write-unavailable-policy-cues.md`](vitest-pre-write-unavailable-policy-cues.md).
- The mirror must not absorb `tests/test-pre-write-inline-patterns.mjs`,
  `tests/test-inline-pattern-index.mjs`, `tests/test-pre-write-render.mjs`,
  resolver suites, deadness/ranking suites, or audit-repo orchestration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Keep this mirror narrow. Future work should not absorb renderer, resolver,
inline extraction, deadness/ranking, audit orchestration, or the remaining
parked cue-tier lanes into this file/token/inline mirror.
