# Vitest Pre-Write Class Method Cue Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T3c-T3d.

---

## Purpose

This review splits the class-method review cue lane out of the parked
`tests/test-pre-write-cue-tiers.mjs` suite. The focused Vitest mirror now
exists at `tests/pre-write-class-method-cues.test.mjs`.

The parent cue-tier suite remains parked because it also protects exact safe
cues, suppressed diagnostics, service-operation sibling cues, local-operation
sibling cues, unavailable evidence, policy exclusions, file cues, token policy,
and inline-pattern cues. This page covers only the narrow adapter behavior where
already-computed `classMethodIndex` near-name evidence becomes review-only cue
evidence without being cited as top-level `defIndex` proof.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command                   | Surface Under Review         |
| ----------------- | ----------------------------------------- | ------------------------------------------------- | ---------------------------- |
| T3c-T3d           | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-class-method-cues` | class-method review cue lane |

Current implementation evidence on 2026-05-19:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-class-method-cues
2 passed, 0 failed
```

The focused mirror extracts only the T3c/T3d fixture shape from the Node suite.
The preserved Node command remains the broader authority for the full cue-tier
adapter suite.

## Result

This lane has a narrow Vitest mirror batch.

The implementation adds one focused mirror for class-method review cue
adaptation because the fixture boundary is static lookup output. The mirror does
not compute class-method indexes, near-name distance, member fan-in,
deadness/ranking, service/local operation policy, Markdown rendering, resolver
behavior, or audit orchestration.

## Protected Invariants

The Vitest mirror preserves these contracts:

- a `classMethodIndex` near-name candidate creates a `cueCards[]` entry;
- the card renders at `cueTier: AGENT_REVIEW_CUE`;
- the cue uses `evidenceLane: class-method-name`;
- the cue claim is `near class method name`;
- cue evidence preserves `matchedField: classMethodIndex`;
- cue evidence preserves the class-method candidate identity with
  `ClassName#methodName`;
- class-method evidence is review-only and must not become `SAFE_CUE`, `EXISTS`,
  `SAFE_FIX`, or top-level export proof;
- the fixture must prove that the class-method cue cites `classMethodIndex`, not
  `defIndex`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a class-method near-name candidate failing to create a review cue must fail;
- a class-method cue rendering at `SAFE_CUE` or `EXISTS` must fail;
- a class-method cue losing `evidenceLane: class-method-name` must fail;
- a class-method cue losing the `near class method name` claim must fail;
- a class-method cue citing `defIndex` must fail;
- a class-method cue losing the `ClassName#methodName` identity must fail;
- a class-method cue being treated as dead-export or safe-fix proof must fail.

## Fixture Boundary

The mirror constructs fixed `classifyPreWriteCues()` inputs with:

- one `NOT_OBSERVED` lookup for `handleBulkDelete`;
- one `nearNames[]` entry for `handleDelete`;
- `matchedField: classMethodIndex`;
- a candidate identity such as
  `src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete`;
- a class name, owner file, distance, and degraded citation;
- empty `identities[]`, `semanticHints[]`, and `suppressedSemanticHints[]`;
- a minimal intent object with names only.

The mirror must not compute class-method candidates by invoking symbol graph,
near-name, or lookup-name policy. It verifies cue-tier adaptation from
already-computed advisory evidence.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- `cueCards[]` lookup by candidate identity;
- cue lookup by `cueTier` and `evidenceLane`;
- evidence-field assertions for `matchedField` and `candidateIdentity`;
- disallowed safe-tier checks.

Shared helpers must not decide:

- whether a class method exists;
- whether a method name is near enough to an intent name;
- whether class methods should be indexed;
- whether class-method evidence should feed export ranking;
- service-operation or local-operation policy;
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
- The mirror must not absorb T1-T3, T4-T10, suppressed diagnostics,
  service-operation policy, local-operation policy, unavailable evidence,
  policy exclusions, file cues, token policy, or inline-pattern fixtures from
  `tests/test-pre-write-cue-tiers.mjs`.
- The mirror must not absorb
  [`vitest-pre-write-exact-safe-cues.md`](vitest-pre-write-exact-safe-cues.md),
  [`vitest-pre-write-cue-suppressed-diagnostics.md`](vitest-pre-write-cue-suppressed-diagnostics.md),
  [`vitest-pre-write-service-operation-cues.md`](vitest-pre-write-service-operation-cues.md),
  or
  [`vitest-pre-write-local-operation-cues.md`](vitest-pre-write-local-operation-cues.md).
- The mirror must not absorb `tests/test-pre-write-lookup-name.mjs`,
  `tests/test-pre-write-render.mjs`, resolver suites, deadness/ranking suites,
  or audit-repo orchestration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Keep this mirror narrow and do not fold broader cue-tier lanes into it.

Future work on the remaining cue-tier lanes should follow the same pattern:
start from the focused review page, preserve the Node command, add one Vitest
mirror at a time, and keep setup-free adapter fixtures separate from lookup,
renderer, resolver, deadness, and audit orchestration semantics.
