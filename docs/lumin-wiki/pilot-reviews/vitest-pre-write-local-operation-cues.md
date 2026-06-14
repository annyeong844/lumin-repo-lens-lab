# Vitest Pre-Write Local Operation Cue Adapter Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-18.
> **Pilot source:** `tests/test-pre-write-cue-tiers.mjs`, assertions T4h-T4j.

---

## Purpose

This review splits the local-operation sibling cue adapter out of the parked
`tests/test-pre-write-cue-tiers.mjs` suite. The focused Vitest mirror now exists
at `tests/pre-write-local-op-cues.test.mjs`.

The parent cue-tier suite remains parked because it also protects exact safe
cues, class-method cue lanes, suppressed diagnostics, service-operation sibling
cues, unavailable evidence, policy exclusions, file cues, token policy, and
inline-pattern cues. This page covers only the adapter behavior where an already
computed `localOperationSiblingPolicy` becomes review-only cue evidence or muted
suppressed evidence.

## Reviewed Evidence

| Source Assertions | Preserved Node Command                    | Proposed Focused Vitest Command               | Surface Under Review             |
| ----------------- | ----------------------------------------- | --------------------------------------------- | -------------------------------- |
| T4h-T4j           | `node tests/test-pre-write-cue-tiers.mjs` | `npm run test:vitest:pre-write-local-op-cues` | local-operation cue-tier adapter |

Current implementation evidence on 2026-05-19:

```text
node tests/test-pre-write-cue-tiers.mjs
27 passed, 0 failed

npm run test:vitest:pre-write-local-op-cues
2 passed, 0 failed
```

The focused mirror extracts only the T4h-T4j fixture shapes from the Node suite.
The preserved Node command remains the broader authority for the full cue-tier
adapter suite.

## Result

This lane has a narrow Vitest mirror batch.

The implementation adds one focused mirror for local-operation cue adaptation
because the fixture boundary is static policy output. The mirror does not
compute local-operation policy, nested operation indexing, operation family,
domain tokens, locality, safe-fix eligibility, Markdown rendering,
deadness/ranking, resolver behavior, or audit orchestration.

## Protected Invariants

The Vitest mirror preserves these contracts:

- promoted `localOperationSiblingPolicy.promoted[]` entries create `cueCards[]`
  with `renderTier: AGENT_REVIEW_CUE`;
- local-operation cue entries use `cueTier: AGENT_REVIEW_CUE`;
- local-operation cue entries use `evidenceLane: local-operation-sibling`;
- local-operation cue entries claim `related local service operation`;
- local-operation cue entries carry `confidence: heuristic-review`;
- evidence is copied from `pre-write-advisory.json`;
- evidence cites
  `lookups[].localOperationSiblingPolicy.promoted`;
- evidence preserves policy id, policy version, candidate identity, container
  name, surface kind, operation family, shared domain tokens, supporting
  reasons, and locality;
- evidence preserves `locality.sameFile === true` for same-file local operation
  cues;
- promoted local-operation candidates never create `SAFE_CUE`, `EXISTS`, or
  `SAFE_FIX` evidence;
- promoted local-operation candidates never become dead-export or safe-fix
  proof;
- muted local-operation entries stay out of `cueCards[]`;
- muted local-operation entries remain in `suppressedCues[]` with
  `evidenceLane: local-operation-sibling`;
- mutation-family mismatches remain muted with
  `local-operation-operation-family-mismatch`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a promoted local-operation sibling becoming `SAFE_CUE`, `EXISTS`, or
  `SAFE_FIX` must fail;
- a promoted local-operation sibling losing policy id or policy version must
  fail;
- a promoted local-operation sibling losing its advisory evidence path must
  fail;
- a promoted local-operation sibling dropping container name, surface kind,
  operation family, shared domain tokens, supporting reasons, or same-file
  locality must fail;
- a promoted local-operation sibling entering the service-operation evidence
  lane must fail;
- a muted local-operation entry appearing in `cueCards[]` must fail;
- a muted local-operation entry losing
  `local-operation-operation-family-mismatch` must fail;
- a local-operation cue with `eligibleForSafeFix: false` creating safe evidence
  must fail.

## Fixture Boundary

The mirror constructs fixed `classifyPreWriteCues()` inputs with:

- one `NOT_OBSERVED` `searchWorld` lookup containing an empty
  `serviceOperationSiblingPolicy` and one promoted `getWorld`
  `localOperationSiblingPolicy` entry from `preWriteLocalOperationIndex`;
- one `NOT_OBSERVED` `deleteWorld` lookup containing a muted `getWorld`
  local-operation sibling with reason
  `local-operation-operation-family-mismatch`;
- metadata for `containerName`, `containerKind`, `surfaceKind`,
  `eligibleForDeadExportRanking: false`, `eligibleForSafeFix: false`,
  `operationFamily`, shared domain tokens, supporting reasons, and same-file
  locality;
- minimal intent objects with names only.

The mirror must not compute local-operation candidates by invoking lookup-name
policy or the nested local operation indexer. It verifies cue-tier adaptation
from already-computed advisory evidence.

## Helper Boundary

The focused mirror shares only these helpers:

- a static `classifyPreWriteCues()` fixture builder;
- `cueCards[]` lookup by candidate identity;
- cue/evidence lookup by `evidenceLane: local-operation-sibling`;
- suppressed-cue lookup by reason and identity;
- disallowed safe-tier checks.

Shared helpers must not decide:

- whether a nested operation should enter `preWriteLocalOperationIndex`;
- whether a local-operation candidate should be promoted or muted;
- operation-family matching;
- domain-token matching;
- locality scoring;
- safe-fix eligibility;
- service-operation policy;
- renderer wording;
- dead-export ranking or action-safety proof.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb T1-T4g, T5-T10, or service-operation policy
  fixtures from `tests/test-pre-write-cue-tiers.mjs`.
- The mirror must not absorb
  [`vitest-pre-write-service-operation-cues.md`](vitest-pre-write-service-operation-cues.md)
  implementation scope beyond asserting lane separation from service-operation
  evidence.
- The mirror must not absorb
  `tests/test-pre-write-local-operation-index.mjs`,
  `tests/test-pre-write-lookup-name.mjs`, `tests/test-pre-write-render.mjs`,
  resolver suites, deadness/ranking suites, or audit-repo orchestration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Keep this mirror narrow and do not fold broader cue-tier lanes into it.

Future work on the remaining cue-tier lanes should follow the same pattern:
start from the focused review page, preserve the Node command, add one Vitest
mirror at a time, and keep setup-free adapter fixtures separate from lookup,
renderer, resolver, deadness, and audit orchestration semantics.
