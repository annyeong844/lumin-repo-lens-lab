# Vitest Threshold Policy Drift Guard Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-13.
> **Pilot candidate:** `tests/test-threshold-policy-drift-guard.mjs`.

---

## Purpose

This review decides whether the threshold policy drift guard is a reasonable
next Vitest pilot candidate. It does not add the Vitest suite. The goal is to
confirm that a future runner migration can improve execution mechanics without
weakening the explicit policy-review gate around numeric threshold changes.

Threshold drift is product-sensitive. A small numeric change can affect ranking,
suppression, confidence wording, review lanes, or future promotion behavior.
The pilot must therefore preserve the exact snapshot contract instead of
rewriting the suite into loose shape checks.

## Reviewed Evidence

- Preserved Node command: `node tests/test-threshold-policy-drift-guard.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:threshold-policy-drift-guard`.
- Policy source under review: `_lib/threshold-policies.mjs`.
- Companion metadata suite: `node tests/test-threshold-policies.mjs`.
- Current policy ids guarded by the snapshot:
  - `function-clone-near-policy`,
  - `inline-pattern-policy`,
  - `resolver-blind-zone-policy`.
- Existing reviewed all-pilot command: `npm run test:vitest`.
- Documentation guards: `npm run check:test-doc` and
  `npm run check:doc-script-refs`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as an exact
snapshot mirror.

The current Node suite protects the review workflow for threshold changes. It
does not merely check that policies exist. It pins the ordered policy ids,
policy versions, policy classes, threshold hashes, calibration corpora, and
calibration notes for the threshold policies that affect clone, inline-pattern,
and resolver blind-zone behavior.

That strictness is intentional. If a maintainer changes a numeric threshold but
does not also update the version/hash/calibration snapshot with review intent,
the suite should fail. A future Vitest mirror may improve failure localization,
but it must not relax the snapshot into partial assertions that would let silent
threshold tuning pass.

## Protected Invariants

The future Vitest pilot must preserve these threshold drift contracts:

- `thresholdPolicyDriftSnapshot(...)` returns the same ordered policy ids for
  the guarded threshold policies;
- each guarded policy keeps an explicit `policyVersion`;
- each guarded policy keeps an explicit `policyClass`;
- each guarded policy keeps a deterministic `thresholdHash`;
- each guarded policy keeps a named `calibrationCorpus`;
- each guarded policy keeps a human-readable `calibrationNote`;
- changing a numeric threshold without the corresponding reviewed snapshot
  update fails;
- removing a policy from the guarded snapshot fails;
- reordering the guarded snapshot fails unless the review intentionally updates
  the expected order.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-threshold-policy-drift-guard.mjs` remains runnable.
- The pilot must not introduce a temporary repo fixture. This suite validates
  policy metadata and exact snapshot semantics, not repo-shaped analyzer
  behavior.
- The pilot must not replace exact expected values with loose presence checks.
- The pilot must not combine this suite with broader threshold metadata tests;
  `tests/test-threshold-policies.mjs` remains a separate policy metadata
  contract.
- `npm run test:vitest` must stay scoped to reviewed `tests/*.test.mjs` files.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/threshold-policy-drift-guard.test.mjs`,
2. `npm run test:vitest:threshold-policy-drift-guard`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the current Node snapshot represented as
named Vitest `it(...)` blocks without relaxing expected values. It should also
run both:

- `node tests/test-threshold-policy-drift-guard.mjs`
- `npm run test:vitest:threshold-policy-drift-guard`

Do not migrate resolver, deadness, pre-write, ranking, performance, or
public-package install-verification suites as part of the threshold policy drift
guard pilot.
