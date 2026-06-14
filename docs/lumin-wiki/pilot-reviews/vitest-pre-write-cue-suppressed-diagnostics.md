# Vitest Pre-Write Suppressed Cue Diagnostics Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T4-T4b.

---

## Purpose

This review splits the suppressed-diagnostics lane out of the parked
`tests/test-pre-write-cue-tiers.mjs` suite. The focused Vitest mirror now
exists at `tests/pre-write-cue-muted.test.mjs`.

The parent cue-tier suite remains parked because it also protects exact safe
cues, class-method cue lanes, service-operation sibling cues, local-operation
sibling cues, unavailable evidence, policy exclusions, file cues, token policy,
and inline-pattern cues. This page covers only the narrow adapter behavior where
suppressed near-name and semantic candidates become muted evidence without
creating cue cards.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command           | Surface Under Review                 |
| ----------------- | ----------------------------------------- | ----------------------------------------- | ------------------------------------ |
| T4-T4b            | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-cue-muted` | suppressed near/semantic cue adapter |

Current implementation evidence on 2026-05-19:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-cue-muted
2 passed, 0 failed
```

The focused mirror extracts only the T4/T4b fixture shape from the Node suite.
The preserved Node command remains the broader authority for the full cue-tier
adapter suite.

## Result

This lane has a narrow Vitest mirror batch.

The implementation adds one focused mirror for suppressed cue diagnostics
because the fixture boundary is static and does not require running lookup-name
policy, service/local operation policy, Markdown rendering, deadness/ranking,
resolver behavior, or audit orchestration.

## Protected Invariants

The Vitest mirror preserves these contracts:

- `suppressedSemanticHints[]` entries with reason `domain-token-overlap` become
  muted suppressed cues;
- suppressed entries carry `cueTier: MUTED`;
- suppressed semantic entries keep `evidenceLane: intent-token`;
- suppressed near-name entries keep `evidenceLane: near-name`;
- suppressed entries preserve reason codes, matched tokens, score, distance,
  locality, candidate count, and token-policy version when present;
- suppressed near-name and semantic candidates never create `cueCards[]`;
- suppressed candidates never become `SAFE_CUE`, `EXISTS`, `SAFE_FIX`, or
  `AGENT_REVIEW_CUE`;
- the fixture must include at least one semantic-only suppression and one
  combined near-name/semantic suppression for the same candidate.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a muted semantic hint appearing in `cueCards[]` must fail;
- a muted near-name candidate appearing in `cueCards[]` must fail;
- a suppressed entry losing its reason code must fail;
- `domain-token-overlap` dropping the token policy version must fail;
- `near-distance-exceeded` losing distance or locality must fail;
- `single-non-weak-token-only` losing score or locality must fail;
- a suppressed semantic entry using the `near-name` evidence lane must fail;
- a suppressed near-name entry using the `intent-token` evidence lane must fail;
- an empty `cueCards[]` assertion without checking `suppressedCues[]` content
  must fail.

## Fixture Boundary

The mirror constructs fixed `classifyPreWriteCues()` inputs with:

- one `NOT_OBSERVED` lookup for `createLogger` with two
  `suppressedSemanticHints[]` entries using `domain-token-overlap`;
- one `NOT_OBSERVED` lookup for `searchUser` with both
  `suppressedNearNames[]` and `suppressedSemanticHints[]` for `fetchUser`;
- empty `nearNames[]`, `semanticHints[]`, and `identities[]`;
- a minimal intent object with names only.

The mirror must not compute suppressed candidates by invoking lookup-name
policy. It verifies cue-tier adaptation from already-computed advisory evidence.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- `suppressedCues[]` lookup by reason, evidence lane, and name;
- `cueCards[]` empty assertion;
- token-policy version assertion.

Shared helpers must not decide:

- whether a candidate should be suppressed;
- semantic score thresholds;
- near-name distance thresholds;
- weak/common/domain token classification;
- service-operation or local-operation policy;
- renderer wording;
- safe-action or dead-export proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb T1-T3, T3c-T3d, T4c-T4j, T5-T10, or any
  service/local-operation policy fixture from `tests/test-pre-write-cue-tiers.mjs`.
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
