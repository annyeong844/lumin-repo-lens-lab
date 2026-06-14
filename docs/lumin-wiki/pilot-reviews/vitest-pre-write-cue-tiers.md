# Vitest Pre-Write Cue Tiers Split-Track Review

> **Status:** REVIEWED - SPLIT TRACKS ONLY.
> **Date:** 2026-05-18.
> **Pilot candidate:** `tests/test-pre-write-cue-tiers.mjs`.

---

## Purpose

This review decides whether `tests/test-pre-write-cue-tiers.mjs` can move as one
Lane C Vitest mirror. It does not add a Vitest suite and it does not approve a
direct `pre-write-cue-tiers.test.mjs` mirror.

The suite is still Node-authoritative because it protects the adapter boundary
where lookup results become cue cards, suppressed cues, or unavailable evidence.
That boundary is close to `SAFE_CUE`, `EXISTS`, and `SAFE_FIX` language, so a
single broad mirror could hide whether a future regression came from exact
identity cues, suppressed diagnostics, service-operation policy, local-operation
policy, policy-excluded paths, or unavailable evidence.

This page names the split tracks that future workers may review one at a time
before any focused mirror or helper extraction happens.

## Reviewed Evidence

| Suite                                | Preserved Node Command                    | Proposed Focused Vitest Command | Surface Under Review            |
| ------------------------------------ | ----------------------------------------- | ------------------------------- | ------------------------------- |
| `tests/test-pre-write-cue-tiers.mjs` | `node tests/test-pre-write-cue-tiers.mjs` | _deferred_                      | pre-write cue-tier adapter flow |

Fresh Node evidence checked for this review:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed
```

The suite now includes exact and signature `SAFE_CUE` cases, mixed
`SAFE_CUE`/`AGENT_REVIEW_CUE` cases, class-method review cues, suppressed
near-name and semantic diagnostics, service-operation sibling review cues,
local-operation sibling review cues, unavailable evidence, policy-excluded exact
evidence, file-exists cues, token policy checks, and inline-pattern review cues.

## Result

Do not migrate this suite as one Vitest mirror.

The future migration path is split-track only. Each track must get its own
focused review page and implementation PR, or it must stay in the preserved Node
suite. The exact/signature safe cue review is
[`vitest-pre-write-exact-safe-cues.md`](vitest-pre-write-exact-safe-cues.md),
which covers exact identity and normalized function-signature `SAFE_CUE`
adaptation. The class-method review cue split-track review is
[`vitest-pre-write-class-method-cues.md`](vitest-pre-write-class-method-cues.md),
which covers `classMethodIndex` review-only cue adaptation. The first
muted-evidence split-track review is
[`vitest-pre-write-cue-suppressed-diagnostics.md`](vitest-pre-write-cue-suppressed-diagnostics.md),
which covers the "muted evidence must not become cue cards" failure mode. The
second split-track review is
[`vitest-pre-write-service-operation-cues.md`](vitest-pre-write-service-operation-cues.md),
which covers the service-operation adapter's review-only promotion and muted
candidate guards. The third split-track review is
[`vitest-pre-write-local-operation-cues.md`](vitest-pre-write-local-operation-cues.md),
which covers the local-operation adapter's review-only promotion and mutation
family mute guard. The unavailable/policy-excluded split-track review is
[`vitest-pre-write-unavailable-policy-cues.md`](vitest-pre-write-unavailable-policy-cues.md),
which covers missing-artifact evidence and policy-excluded exact evidence. The
file/token/inline split-track review is
[`vitest-pre-write-file-token-inline-cues.md`](vitest-pre-write-file-token-inline-cues.md),
which covers exact file cues, pre-write token stemming, inline-pattern review
cues, and missing inline-pattern artifacts.

## Split Tracks

| Track                                 | Current Assertions | Protected Contract                                                                                                           | Edge Failure That Must Stay Visible                                                                                  |
| ------------------------------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| exact and signature safe cues         | T1-T3              | Exact identities and function signatures can create `SAFE_CUE` evidence, while mixed candidates render at the review tier.   | Safe evidence loses identity, owner, field, signature, or render-tier priority.                                      |
| class-method review cue lane          | T3c-T3d            | `classMethodIndex` evidence is review-only and cites the class-method lane, not `defIndex`.                                  | Class methods leak into export proof or are cited as top-level definitions.                                          |
| suppressed diagnostics                | T4-T4b             | Suppressed near-name and semantic candidates remain in `suppressedCues[]`, not `cueCards[]`.                                 | A muted search hint becomes an `EXISTS`, `SAFE_CUE`, `SAFE_FIX`, or `AGENT_REVIEW_CUE` card.                         |
| service-operation sibling adapter     | T4c-T4g            | Promoted service-operation siblings are `AGENT_REVIEW_CUE` only; muted, class-method, and generated entries stay suppressed. | Service-operation policy output becomes safe evidence, loses policy metadata, or renders muted/generated candidates. |
| local-operation sibling adapter       | T4h-T4j            | Promoted nested local operations are `AGENT_REVIEW_CUE` only and cite `localOperationSiblingPolicy.promoted`.                | Nested local operations leak into service-operation policy, safe cues, or default cue cards when muted.              |
| unavailable and policy-excluded lanes | T5-T6b             | Unavailable evidence and policy-excluded exact evidence stay distinct from cue cards.                                        | Missing artifacts or generated paths look like clean absence, exact proof, or actionable fix evidence.               |
| file, token, and inline-pattern cues  | T7-T10             | File-exists, token stemming, inline-pattern cues, and inline-pattern unavailability remain separate lanes.                   | Token policy changes or missing inline-pattern evidence are hidden by broad helper setup.                            |

## Protected Invariants

Any future focused Vitest mirror must preserve these contracts:

- `SAFE_CUE`, `AGENT_REVIEW_CUE`, `EXISTS`, and unavailable evidence remain
  separate lanes;
- mixed cue candidates render at the highest required review tier without
  dropping their individual cue records;
- `classMethodIndex` candidates do not become `defIndex` proof;
- suppressed near-name and semantic diagnostics never create `cueCards[]`;
- service-operation sibling promotion is policy-versioned, review-only, and
  copied from `lookups[].serviceOperationSiblingPolicy.promoted`;
- muted service-operation entries stay out of `cueCards[]`;
- service-operation candidates from class-method evidence or generated paths are
  suppressed, not rendered;
- local-operation sibling promotion is policy-versioned, review-only, and
  copied from `lookups[].localOperationSiblingPolicy.promoted`;
- muted local-operation entries stay out of `cueCards[]`;
- local-operation and service-operation identities do not cross-feed between
  policy lanes;
- unavailable evidence carries the lookup reason and artifact instead of
  becoming observed absence;
- policy-excluded exact evidence remains suppressed with its original cue tier
  and exclusion reason;
- inline-pattern cues stay review-only and missing inline-pattern artifacts stay
  unavailable.

## Helper Boundary

A future focused mirror may share only setup-free adapter helpers:

- static lookup fixture object construction;
- candidate identity lookup in `cueCards[]`;
- cue-tier, evidence-lane, policy-id, and policy-version assertions;
- deterministic suppressed-cue filtering;
- disallowed safe-tier checks.

Shared helpers must not decide:

- whether a lookup candidate should be promoted;
- service-operation or local-operation operation-family matching;
- policy exclusion for generated paths;
- class-method eligibility;
- unavailable evidence reason codes;
- token stemming;
- inline-pattern matching;
- renderer wording;
- dead-export ranking or action-safety proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- A future mirror must not absorb `tests/test-pre-write-render.mjs`,
  `tests/test-pre-write-lookup-name.mjs`,
  `tests/test-pre-write-advisory-artifact.mjs`,
  `tests/test-pre-write-integration.mjs`, resolver suites,
  deadness/ranking suites, action-safety suites, or audit-repo orchestration.
- A future mirror must not widen `npm run test:vitest` discovery beyond
  reviewed first-party `tests/*.test.mjs` files.

## Recommendation

Keep `tests/test-pre-write-cue-tiers.mjs` parked as the authoritative Node
umbrella suite.

All current T1-T10 contracts have focused split mirrors: exact/signature safe
cues
([review](vitest-pre-write-exact-safe-cues.md)), class-method review cues
([review](vitest-pre-write-class-method-cues.md)), suppressed diagnostics
([review](vitest-pre-write-cue-suppressed-diagnostics.md)),
service-operation cues
([review](vitest-pre-write-service-operation-cues.md)), local-operation cues
([review](vitest-pre-write-local-operation-cues.md)),
unavailable/policy-excluded cues
([review](vitest-pre-write-unavailable-policy-cues.md)), and
file/token/inline cues
([review](vitest-pre-write-file-token-inline-cues.md)). The parked decision now
applies only to a direct broad umbrella mirror. Future work should continue one
adapter lane at a time, and any new cue-tier behavior needs a fresh review that
names the exact fixture boundary, protected invariants, focused Vitest command,
and preserved Node command.
