# Parked Suite Dogfooding

> **Date:** 2026-05-16.
> **Last refreshed:** 2026-07-03.
> **Scope:** the remaining Node-authoritative umbrella suite left after the
> audit-repo legacy umbrella retirement.

This page defines how Lumin dogfoods its own structure-review and test-reform
rules before touching the parked remainder. It does not authorize more direct
Vitest mirrors. The remaining parked suites are broad umbrella suites whose
direct mirrors would blur product-pass and cue-tier policy failures. Previously
parked performance, incremental, deadness, ranking, calibration, and scanner
suites now have reviewed focused mirrors; use the closure audit for those
completed lanes instead of reopening this guide.

## Operating Rule

Node remains authoritative for every suite listed in
[`vitest-mirror-closure-audit.md`](../vitest-mirror-closure-audit.md#parked-remainder).
Future work on those suites must start with one of these review artifacts:

1. a split-track review for an umbrella suite;
2. a suite-specific pilot review that names the protected invariant and edge
   failure;
3. a helper-extraction review that proves the helper is setup-only and cannot
   absorb analyzer semantics.

Do not add a direct Vitest mirror from the parked remainder until that review
exists and the preserved Node command still passes.

## Lane Map

| Lane                  | Parked Suites                        | Why Node Stays Authoritative                                                                                                  | Next Safe Dogfood Question                                                                                     |
| --------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `cue-tier policy`     | `tests/test-pre-write-cue-tiers.mjs` | Known T1-T10 split mirrors are complete, but the direct umbrella still protects cross-lane cue policy boundaries.             | Is there new cue-tier behavior with corpus evidence that needs a fresh split review before any focused mirror? |

`tests/test-audit-repo.mjs` is no longer part of the parked Node-authoritative
set or the default `npm test` gate. Its known product-pass contracts are covered
by Rust audit-core cargo tests through `npm run test:audit-runtime-gate`; focused
Vitest mirrors remain reference coverage while JS/TS producers are being retired.
The legacy umbrella remains available only through
`npm run test:node:legacy-audit-repo` for manual archaeology.

## Completed Former Parked Lanes

The following lanes were parked in the original dogfooding map and are now
covered by reviewed focused mirrors. Do not pick work from these lanes through
this guide unless a new behavior appears and receives its own review page.

- Performance and incremental cache identity.
- Deadness and ranking action-proof boundaries.
- P6 calibration and member-precision evidence.
- Scanner and producer artifact builder contracts.

See the
[`Vitest Mirror Lane Closure Audit`](../vitest-mirror-closure-audit.md#removed-from-parked-remainder)
for the suite-by-suite evidence.

## Review Page Contract

A parked-suite review page must name:

- the invariant protected by the Node suite;
- the concrete edge failure that would have caught the original class of bug;
- the preserved Node command;
- the proposed focused Vitest command, if a mirror is being proposed;
- the fixture boundary and cleanup rule;
- the exact helper boundary, including what the helper is forbidden to decide;
- the artifact or Markdown surface that must keep evidence visible;
- the negative assertion that prevents overclaiming.

An implementation without these fields is not a parked-suite migration. It is
test churn.

## Helper Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- normalize paths and line endings;
- read generated JSON artifacts;
- run preserved commands with explicit stdout/stderr capture;
- compare stable fixture snapshots when the owning suite defines the schema.

Forbidden shared helpers:

- decide `SAFE_FIX`, `EXISTS`, or `AGENT_REVIEW_CUE` status;
- classify resolver unsupported families or blind-zone relevance;
- hide cache invalidation, warm-run reuse, or deletion behavior;
- collapse ranking, provenance, or public-surface proof into broad booleans;
- rewrite fixture expectations so the original edge failure disappears.

Every helper extracted from a parked suite needs at least one edge-case contract
test that would fail if the helper masked analyzer semantics.

## First Dogfood Candidates

Start with review work, not runner work:

1. `tests/test-pre-write-cue-tiers.mjs`: keep the broad Node suite
   authoritative. Add another split only when new cue-tier behavior has corpus
   evidence and a review page; do not mirror the umbrella to chase a coverage
   count.

This keeps the next phase aligned with the
[`Structure Review Charter`](review-charter.md), the
[`Test Reform`](test-reform.md) rules, and the
[`Fixture Shapes`](fixture-shapes.md) inventory.
