# Vitest Pre-Write Exact And Signature Safe Cue Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T1-T3.

---

## Purpose

This review splits the exact-symbol and function-signature cue adapter lane out
of the parked `tests/test-pre-write-cue-tiers.mjs` suite. The focused Vitest
mirror now exists at `tests/pre-write-exact-safe-cues.test.mjs`.

The parent cue-tier suite remains parked because it also protects class-method
cue lanes, suppressed diagnostics, service-operation sibling cues,
local-operation sibling cues, unavailable evidence, policy exclusions, file
cues, token policy, and inline-pattern cues. This page covers only the narrow
adapter behavior where already-computed exact identity and signature evidence
becomes `SAFE_CUE` evidence, while a candidate with both safe and review cues
renders at the review tier without losing either cue.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command                 | Surface Under Review             |
| ----------------- | ----------------------------------------- | ----------------------------------------------- | -------------------------------- |
| T1-T3             | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-exact-safe-cues` | exact/signature safe cue adapter |

Current implementation evidence on 2026-05-18:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-exact-safe-cues
3 passed, 0 failed
```

The focused mirror extracts only the T1-T3 fixture shapes from the Node suite.
The preserved Node command remains the broader authority for the full cue-tier
adapter suite.

## Result

This lane has a narrow Vitest mirror batch.

The implementation adds one focused mirror for exact and signature safe cue
adaptation because the fixture boundary is static lookup output. The mirror does
not compute lookup-name policy, function clone grouping, shape hashing, fan-in,
class-method indexing, Markdown rendering, deadness/ranking, resolver behavior,
or audit orchestration.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- exact `EXISTS` name identities create `cueCards[]`;
- exact symbol cues use `cueTier: SAFE_CUE`;
- exact symbol cues use `safeMeaning: claim-only`;
- exact symbol cues use `evidenceLane: exact-symbol`;
- exact symbol cue cards preserve candidate identity, owner file, and exported
  name;
- exact symbol cue evidence preserves `symbols.json`, `defIndex`,
  `candidateIdentity`, and `exact-symbol.v1`;
- exact symbol safe cues explicitly remain not safe for semantic equivalence;
- function signature matches create `cueCards[]`;
- function signature cues use `cueTier: SAFE_CUE`;
- function signature cues use `evidenceLane: function-signature`;
- function signature cues claim `same normalized function signature`;
- function signature cue evidence preserves `function-clones.json`,
  `normalizedSignatureHash`, `function-signature.normalized.v1`, and shape hash;
- one candidate can carry both `SAFE_CUE` and `AGENT_REVIEW_CUE` records;
- a mixed safe/review candidate renders at `AGENT_REVIEW_CUE`;
- mixed render-tier escalation must not drop the safe cue record.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- an exact identity lookup that fails to create a cue card must fail;
- an exact identity cue losing `safeMeaning: claim-only` must fail;
- an exact identity cue becoming a semantic-equivalence claim must fail;
- an exact identity cue losing the exact-symbol evidence lane must fail;
- a function signature match failing to create a cue card must fail;
- a function signature cue losing the `same normalized function signature`
  claim must fail;
- a function signature cue losing the function-signature evidence lane must
  fail;
- a candidate with both safe and review evidence rendering at `SAFE_CUE` must
  fail;
- a mixed candidate dropping either the `SAFE_CUE` record or the
  `AGENT_REVIEW_CUE` record must fail.

## Fixture Boundary

The mirror constructs fixed `classifyPreWriteCues()` inputs with:

- one `EXISTS` name lookup for `formatDate` with a grounded identity,
  fan-in, confidence, and citation;
- one `SIGNATURE_MATCH` shape lookup for `useShallow` with a normalized
  function signature, shape hash, match identity, owner file, confidence, and
  citation;
- one mixed lookup fixture where the same `useShallow` identity has a degraded
  near-name review cue and a grounded signature safe cue;
- minimal intent objects with names and shapes only.

The mirror must not compute exact identities, function signatures, shape hashes,
or near-name review candidates. It verifies cue-tier adaptation from
already-computed advisory evidence.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- `cueCards[]` lookup by candidate identity;
- cue lookup by `cueTier` and `evidenceLane`;
- render-tier assertion for mixed cue cards.

Shared helpers must not decide:

- whether an identity exists;
- whether fan-in is grounded;
- whether two function signatures normalize to the same shape;
- whether a near-name candidate is a review cue;
- class-method eligibility;
- service-operation or local-operation policy;
- renderer wording;
- dead-export ranking or action-safety proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb T3c-T10 from `tests/test-pre-write-cue-tiers.mjs`.
- The mirror must not absorb
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
