# Vitest Threshold Metadata Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:** `tests/test-threshold-policies.mjs`,
> `tests/test-calibration-corpora.mjs`.

---

## Purpose

This review decides whether two threshold metadata suites can move as one
narrow Vitest mirror batch. It does not add Vitest suites. The goal is to
preserve policy and corpus registry contracts without turning the mirror into a
threshold drift, ranking, deadness, resolver, or calibration quality test.

The candidates are acceptable as one batch because both suites import pure
metadata modules and inspect deterministic registry shapes:

- `tests/test-threshold-policies.mjs` validates policy ids, policy classes,
  threshold values, policy hashes, threshold hashes, and compact policy
  summaries.
- `tests/test-calibration-corpora.mjs` validates corpus ids, schema version,
  policy-to-corpus links, corpus metrics, summary compactness, and unknown
  corpus failure behavior.

This batch is intentionally separate from
`tests/test-threshold-policy-drift-guard.mjs`, which already has its own
snapshot-focused Vitest mirror. The metadata batch may check public registry
shape and explicit values, but it must not replace or weaken the stricter drift
snapshot.

## Reviewed Evidence

| Suite                                | Preserved Node Command                    | Proposed Focused Vitest Command           | Surface Under Review                              |
| ------------------------------------ | ----------------------------------------- | ----------------------------------------- | ------------------------------------------------- |
| `tests/test-threshold-policies.mjs`  | `node tests/test-threshold-policies.mjs`  | `npm run test:vitest:threshold-policies`  | threshold policy metadata and compact summaries   |
| `tests/test-calibration-corpora.mjs` | `node tests/test-calibration-corpora.mjs` | `npm run test:vitest:calibration-corpora` | calibration corpus registry and policy references |

Current suite descriptions are in `tests/README.md`.

Goal lane: metadata-only policy registry guard.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should preserve exact metadata expectations and
the unknown-corpus failure path without changing `_lib/threshold-policies.mjs`,
`_lib/calibration-corpora.mjs`, policy thresholds, corpus definitions, drift
snapshot behavior, ranking behavior, resolver behavior, or action-safety proof.
The Node entrypoints must remain runnable.

## Protected Invariants

The future Vitest mirrors must preserve these contracts:

- `getThresholdPolicy('function-clone-near-policy')` returns the expected
  schema version, policy id, policy version, policy class, numeric thresholds,
  policy hash, and threshold hash;
- `getThresholdPolicy('inline-pattern-policy')` returns review-class inline
  threshold metadata;
- `getThresholdPolicy('resolver-blind-zone-policy')` returns confidence-class
  resolver blind-zone threshold metadata;
- `thresholdPolicySummary(...)` returns compact ordered summaries with
  threshold values, threshold hashes, and calibration corpus summaries;
- threshold summaries omit long-form notes;
- the calibration corpus registry lists both pre-write and resolver corpus ids;
- every threshold policy names a known calibration corpus;
- each corpus preserves schema version, corpus id, purpose, and metric names;
- `calibrationCorpusSummary(...)` stays compact and ordered;
- `getCalibrationCorpus(...)` throws a clear error for unknown corpus ids.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- silently dropping a policy hash or threshold hash must fail;
- changing a numeric metadata value without test visibility must fail;
- returning verbose notes from compact summaries must fail;
- letting a threshold policy reference a missing corpus must fail;
- accepting an unknown corpus id without throwing must fail;
- weakening this metadata mirror into loose presence-only assertions must fail;
- replacing the separate threshold drift snapshot with this metadata mirror
  must stay out of scope.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- No temporary repo fixture is required; these suites validate pure metadata
  registries.
- A future mirror may use helper functions for repeated registry lookups, but
  helper code must not decide threshold policy meaning, calibration adequacy,
  ranking, suppression, resolver confidence, or action safety.
- The mirror must not modify threshold values, hashes, policy ids, policy
  versions, corpus ids, or corpus metrics.
- The mirror must not absorb `tests/test-threshold-policy-drift-guard.mjs`,
  cue-tier policy tests, resolver tests, deadness/ranking tests, performance
  tests, or corpus quality/evaluation tests.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/threshold-policies.test.mjs`,
2. `tests/calibration-corpora.test.mjs`,
3. `npm run test:vitest:threshold-policies`,
4. `npm run test:vitest:calibration-corpora`,
5. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current Node assertion as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.
