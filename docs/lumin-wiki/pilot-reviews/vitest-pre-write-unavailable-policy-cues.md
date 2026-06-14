# Vitest Pre-Write Unavailable And Policy Cue Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T5-T6b.

---

## Purpose

This review splits the unavailable-evidence and policy-excluded exact-evidence
lane out of the parked `tests/test-pre-write-cue-tiers.mjs` suite. The focused
Vitest mirror now exists at `tests/pre-write-evidence-gaps.test.mjs`.

The parent cue-tier suite remains parked because it also protects exact safe
cues, class-method review cues, suppressed diagnostics, service-operation
sibling cues, local-operation sibling cues, file cues, token policy, and
inline-pattern cues. This page covers only the narrow adapter behavior where
missing lookup evidence stays in `unavailableEvidence[]`, and exact evidence
from policy-excluded paths stays suppressed instead of becoming a cue card.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command               | Surface Under Review                  |
| ----------------- | ----------------------------------------- | --------------------------------------------- | ------------------------------------- |
| T5-T6b            | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-evidence-gaps` | unavailable/policy-excluded cue lanes |

Current implementation evidence on 2026-05-19:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-evidence-gaps
2 passed, 0 failed
```

The focused mirror extracts only the T5/T5b and T6/T6b fixture shapes from the
Node suite. The preserved Node command remains the broader authority for the
full cue-tier adapter suite.

## Result

This lane has a narrow Vitest mirror batch.

The implementation adds one focused mirror for unavailable and policy-excluded
cue adaptation because the fixture boundary is static lookup output. The mirror
does not run function-clone extraction, symbol graph, generated-artifact policy
detection, Markdown rendering, resolver behavior, deadness/ranking, or audit
orchestration.

## Protected Invariants

The Vitest mirror preserves these contracts:

- lookup results with `result: UNAVAILABLE` create `unavailableEvidence[]`;
- unavailable evidence preserves `status: unavailable`;
- unavailable evidence preserves the lookup `reason`;
- unavailable evidence preserves the missing or unavailable artifact name;
- unavailable evidence does not create `suppressedCues[]`;
- unavailable evidence does not create `cueCards[]`;
- policy-excluded exact identities do not create `cueCards[]`;
- policy-excluded exact identities become `suppressedCues[]`;
- policy-excluded suppressed cues preserve `reason: policy-excluded`;
- policy-excluded suppressed cues preserve `policyReason`;
- policy-excluded suppressed cues preserve `originalCueTier: SAFE_CUE`;
- policy-excluded suppressed cues preserve the original exact-symbol claim and
  `symbols.json` evidence.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- missing `function-clones.json` looking like observed absence must fail;
- unavailable lookup evidence entering `suppressedCues[]` must fail;
- unavailable lookup evidence creating a cue card must fail;
- unavailable evidence losing `reason` or `artifact` must fail;
- generated output exact evidence creating a `SAFE_CUE` card must fail;
- policy-excluded exact evidence disappearing entirely must fail;
- policy-excluded exact evidence losing its original safe tier must fail;
- policy-excluded exact evidence losing `policyReason` must fail;
- policy-excluded exact evidence losing the `symbols.json` citation lane must
  fail.

## Fixture Boundary

The mirror constructs fixed `classifyPreWriteCues()` inputs with:

- one shape lookup with `result: UNAVAILABLE`;
- `reason: missing-artifact`;
- `artifact: function-clones.json`;
- one exact-name lookup for `generatedHelper`;
- one identity from `dist/generated.ts`;
- `policyExcluded: true`;
- `policyReason: generated-output`;
- a grounded exact-symbol citation;
- minimal intent objects with shapes or names only.

The mirror must not compute why an artifact is missing or why a path is
policy-excluded. It verifies cue-tier adaptation from already-computed advisory
evidence.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- unavailable evidence lookup by reason and artifact;
- suppressed cue lookup by reason and policy reason;
- `cueCards[]` empty assertion;
- original cue-tier assertion.

Shared helpers must not decide:

- whether an artifact should exist;
- whether function-clone facts are complete;
- whether a generated path is policy-excluded;
- whether an exact symbol exists;
- renderer wording;
- resolver behavior;
- generated-artifact blind-zone policy;
- dead-export ranking or action-safety proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb T1-T4j, T7-T10, exact/signature safe cues,
  class-method cues, suppressed diagnostics, service-operation policy,
  local-operation policy, file cues, token policy, or inline-pattern fixtures
  from `tests/test-pre-write-cue-tiers.mjs`.
- The mirror must not absorb
  [`vitest-pre-write-exact-safe-cues.md`](vitest-pre-write-exact-safe-cues.md),
  [`vitest-pre-write-class-method-cues.md`](vitest-pre-write-class-method-cues.md),
  [`vitest-pre-write-cue-suppressed-diagnostics.md`](vitest-pre-write-cue-suppressed-diagnostics.md),
  [`vitest-pre-write-service-operation-cues.md`](vitest-pre-write-service-operation-cues.md),
  or
  [`vitest-pre-write-local-operation-cues.md`](vitest-pre-write-local-operation-cues.md).
- The mirror must not absorb `tests/test-pre-write-lookup-name.mjs`,
  `tests/test-pre-write-render.mjs`, resolver suites, generated-artifact suites,
  deadness/ranking suites, or audit-repo orchestration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Keep this mirror narrow and do not fold broader cue-tier lanes into it.

Future work on the remaining cue-tier lanes should follow the same pattern:
start from the focused review page, preserve the Node command, add one Vitest
mirror at a time, and keep setup-free adapter fixtures separate from lookup,
renderer, resolver, deadness, and audit orchestration semantics.
