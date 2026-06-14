# Vitest Pre-Write Service Operation Cue Adapter Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T4c-T4g.

---

## Purpose

This review splits the service-operation sibling cue adapter out of the parked
`tests/test-pre-write-cue-tiers.mjs` suite. The focused Vitest mirror now exists
at `tests/pre-write-service-op-cues.test.mjs`.

The parent cue-tier suite remains parked because it also protects exact safe
cues, class-method cue lanes, suppressed diagnostics, local-operation sibling
cues, unavailable evidence, policy exclusions, file cues, token policy, and
inline-pattern cues. This page covers only the adapter behavior where an already
computed `serviceOperationSiblingPolicy` becomes review-only cue evidence or
muted suppressed evidence.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command                 | Surface Under Review               |
| ----------------- | ----------------------------------------- | ----------------------------------------------- | ---------------------------------- |
| T4c-T4g           | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-service-op-cues` | service-operation cue-tier adapter |

Current implementation evidence on 2026-05-19:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-service-op-cues
3 passed, 0 failed
```

The focused mirror extracts only the T4c-T4g fixture shapes from the Node suite.
The preserved Node command remains the broader authority for the full cue-tier
adapter suite.

## Result

This lane has a narrow Vitest mirror batch.

The implementation adds one focused mirror for service-operation cue adaptation
because the fixture boundary is static policy output. The mirror does not
compute service-operation policy, lookup-name suppression, operation family,
domain tokens, generated-path exclusion, class-method eligibility, Markdown
rendering, deadness/ranking, resolver behavior, or audit orchestration.

## Protected Invariants

The Vitest mirror preserves these contracts:

- promoted `serviceOperationSiblingPolicy.promoted[]` entries create
  `cueCards[]` with `renderTier: AGENT_REVIEW_CUE`;
- service-operation cue entries use `cueTier: AGENT_REVIEW_CUE`;
- service-operation cue entries use `evidenceLane: service-operation-sibling`;
- service-operation cue entries claim `related service operation sibling`;
- service-operation cue entries carry `confidence: heuristic-review`;
- evidence is copied from `pre-write-advisory.json`;
- evidence cites
  `lookups[].serviceOperationSiblingPolicy.promoted`;
- evidence preserves policy id, policy version, candidate identity, operation
  family, shared domain tokens, locality, signature support, and supporting
  reasons when present;
- promoted service-operation candidates never create `SAFE_CUE`, `EXISTS`, or
  `SAFE_FIX` evidence;
- existing suppressed near-name and semantic diagnostics remain in
  `suppressedCues[]`;
- muted service-operation entries stay out of `cueCards[]`;
- muted service-operation entries remain in `suppressedCues[]` with
  `evidenceLane: service-operation-sibling`;
- class-method service-operation candidates are muted with
  `service-sibling-class-method-lane`;
- generated or policy-excluded service-operation candidates are suppressed with
  the policy-exclusion reason, not rendered.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a promoted service-operation sibling becoming `SAFE_CUE`, `EXISTS`, or
  `SAFE_FIX` must fail;
- a promoted service-operation sibling losing policy id or policy version must
  fail;
- a promoted service-operation sibling losing its advisory evidence path must
  fail;
- a promoted service-operation sibling dropping operation family or shared
  domain tokens must fail;
- original suppressed near-name and semantic diagnostics disappearing after
  service cue promotion must fail;
- a muted service-operation entry appearing in `cueCards[]` must fail;
- a class-method service-operation candidate rendering as a cue card must fail;
- a generated path service-operation candidate rendering as a cue card must
  fail;
- a generated path candidate losing `policyReason: path:dist` must fail.

## Fixture Boundary

The mirror constructs fixed `classifyPreWriteCues()` inputs with:

- one `NOT_OBSERVED` `searchUser` lookup containing suppressed near-name and
  semantic diagnostics plus one promoted `fetchUser` service-operation sibling;
- one `NOT_OBSERVED` `createUser` lookup containing a muted `fetchUser`
  service-operation sibling with reason
  `service-sibling-operation-family-mismatch`;
- one `NOT_OBSERVED` `searchUser` lookup containing promoted service-operation
  candidates that must be suppressed because one is from `classMethodIndex` and
  one is under `dist/`;
- minimal intent objects with names only.

The mirror must not compute service-operation candidates by invoking
lookup-name policy. It verifies cue-tier adaptation from already-computed
advisory evidence.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- `cueCards[]` lookup by candidate identity;
- cue/evidence lookup by `evidenceLane: service-operation-sibling`;
- suppressed-cue lookup by reason and identity;
- disallowed safe-tier checks.

Shared helpers must not decide:

- whether a candidate should be promoted or muted;
- operation-family matching;
- domain-token matching;
- locality scoring;
- generated or policy-excluded path classification;
- class-method eligibility;
- lookup-name suppressed diagnostics;
- renderer wording;
- safe-action or dead-export proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb T1-T4b, T4h-T10, or local-operation policy fixtures
  from `tests/test-pre-write-cue-tiers.mjs`.
- The mirror must not absorb
  [`vitest-pre-write-cue-suppressed-diagnostics.md`](vitest-pre-write-cue-suppressed-diagnostics.md)
  implementation scope beyond preserving its already-computed suppressed cues in
  the promoted service-operation fixture.
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
