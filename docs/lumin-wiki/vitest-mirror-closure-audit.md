# Vitest Mirror Lane Closure Audit

> **Date:** 2026-05-16.
> **Last refreshed:** 2026-05-24.
> **Scope:** WT-24 Vitest mirror lane plus the grouped Node runner shortcut.

This page records the completion audit for the risk-batched Vitest mirror lane.
It explains why the lane is considered closed even though some Node suites
remain intentionally Node-authoritative.

## Success Criteria

The active lane is complete when:

1. every non-parked Node `tests/test-*.mjs` suite has a focused Vitest mirror;
2. parked analyzer-sensitive suites have explicit deferral notes or review
   pages before any future mirror work;
3. every focused mirror keeps the original Node command runnable;
4. `npm run test:vitest` discovers only reviewed `tests/*.test.mjs` mirrors and
   passes;
5. `npm test` remains runnable and passes;
6. wiki/script reference gates pass.
7. `npm run test:node:groups` remains an opt-in maintainer shortcut, not a
   replacement for the authoritative serial Node lane.

## Current Inventory

| Metric                           | Count | Evidence                         |
| -------------------------------- | ----: | -------------------------------- |
| Node `tests/test-*.mjs` suites   |   165 | 2026-05-24 local filesystem scan |
| Focused Vitest mirror files      |   176 | 2026-05-24 local filesystem scan |
| Node-authoritative parked suites |     2 | refreshed parked remainder below |

`tests/test-incremental.mjs` is already mirrored by
`tests/incremental-legacy-cache.test.mjs`, so it is not part of the parked
remainder even though the mirror file does not share the same stem.

## Parked Remainder

| Node Suite                           | Parked Category     | Why It Stays Node-Authoritative                                                            |
| ------------------------------------ | ------------------- | ------------------------------------------------------------------------------------------ |
| `tests/test-audit-repo.mjs`          | audit-repo umbrella | direct broad product-pass suite stays Node-authoritative; known split mirrors are complete |
| `tests/test-pre-write-cue-tiers.mjs` | cue-tier policy     | direct broad cue adapter suite stays Node-authoritative; known T1-T10 splits are complete  |

`tests/test-pre-write-cue-tiers.mjs` is not parked because known contracts are
unmirrored. Its current T1-T10 contracts are covered by focused mirrors:
[exact/signature safe cues](pilot-reviews/vitest-pre-write-exact-safe-cues.md),
[class-method cues](pilot-reviews/vitest-pre-write-class-method-cues.md),
[suppressed diagnostics](pilot-reviews/vitest-pre-write-cue-suppressed-diagnostics.md),
[service-operation cues](pilot-reviews/vitest-pre-write-service-operation-cues.md),
[local-operation cues](pilot-reviews/vitest-pre-write-local-operation-cues.md),
[unavailable/policy cues](pilot-reviews/vitest-pre-write-unavailable-policy-cues.md),
and
[file/token/inline cues](pilot-reviews/vitest-pre-write-file-token-inline-cues.md).
The tracker row is in
[`test-migration-candidate-board.md`](test-migration-candidate-board.md). The
remaining parked decision applies only to a direct umbrella mirror; any new
cue-tier behavior needs a fresh split review before a new Vitest mirror.

## Removed From Parked Remainder

These suites were parked in the original 2026-05-16 closure audit but now have
reviewed focused mirrors and stay covered by both the preserved Node command and
the focused Vitest command:

| Node Suite                                       | Focused Vitest Mirror                              | Review                                                                      |
| ------------------------------------------------ | -------------------------------------------------- | --------------------------------------------------------------------------- |
| `tests/test-any-inventory-incremental.mjs`       | `tests/any-inventory-incremental.test.mjs`         | `docs/lumin-wiki/pilot-reviews/vitest-any-inventory-incremental.md`         |
| `tests/test-classify-performance-metadata.mjs`   | `tests/classify-performance-metadata.test.mjs`     | `docs/lumin-wiki/pilot-reviews/vitest-classify-performance-metadata.md`     |
| `tests/test-corpus.mjs`                          | `tests/corpus.test.mjs`                            | `docs/lumin-wiki/pilot-reviews/vitest-corpus.md`                            |
| `tests/test-export-action-safety.mjs`            | `tests/export-action-safety.test.mjs`              | `docs/lumin-wiki/pilot-reviews/vitest-export-action-safety.md`              |
| `tests/test-finding-local-provenance.mjs`        | `tests/finding-local-provenance.test.mjs`          | `docs/lumin-wiki/pilot-reviews/vitest-finding-local-provenance.md`          |
| `tests/test-function-clone-incremental.mjs`      | `tests/function-clone-incremental.test.mjs`        | `docs/lumin-wiki/pilot-reviews/vitest-function-clone-incremental.md`        |
| `tests/test-module-reachability.mjs`             | `tests/module-reachability.test.mjs`               | `docs/lumin-wiki/pilot-reviews/vitest-module-reachability.md`               |
| `tests/test-namespace-reexport-deadness.mjs`     | `tests/namespace-reexport-deadness.test.mjs`       | `docs/lumin-wiki/pilot-reviews/vitest-namespace-reexport-deadness.md`       |
| `tests/test-p6-measurement.mjs`                  | `tests/p6-measurement.test.mjs`                    | `docs/lumin-wiki/pilot-reviews/vitest-p6-measurement.md`                    |
| `tests/test-p6-member-precision.mjs`             | `tests/p6-member-precision.test.mjs`               | `docs/lumin-wiki/pilot-reviews/vitest-p6-member-precision.md`               |
| `tests/test-p6-safe-fix-calibration.mjs`         | `tests/p6-safe-fix-calibration.test.mjs`           | `docs/lumin-wiki/pilot-reviews/vitest-p6-safe-fix-calibration.md`           |
| `tests/test-rank-fixes.mjs`                      | `tests/rank-fixes.test.mjs`                        | `docs/lumin-wiki/pilot-reviews/vitest-rank-fixes.md`                        |
| `tests/test-shape-index-incremental.mjs`         | `tests/shape-index-incremental.test.mjs`           | `docs/lumin-wiki/pilot-reviews/vitest-shape-index-incremental.md`           |
| `tests/test-symbol-graph-incremental.mjs`        | `tests/symbol-graph-incremental.test.mjs`          | `docs/lumin-wiki/pilot-reviews/vitest-symbol-graph-incremental.md`          |
| `tests/test-audit-repo.mjs` (blind-zone split)   | `tests/audit-repo-blind-zones.test.mjs`            | `docs/lumin-wiki/pilot-reviews/vitest-audit-repo-blind-zone-confidence.md`  |
| `tests/test-audit-repo.mjs` (scan range split)   | `tests/audit-repo-scan-range.test.mjs`             | `docs/lumin-wiki/pilot-reviews/vitest-audit-repo-scan-range.md`             |
| `tests/test-audit-repo.mjs` (lifecycle split)    | `tests/audit-repo-lifecycle-artifacts.test.mjs`    | `docs/lumin-wiki/pilot-reviews/vitest-audit-repo-lifecycle-artifacts.md`    |
| `tests/test-audit-repo.mjs` (full-profile split) | `tests/audit-repo-full-profile-staleness.test.mjs` | `docs/lumin-wiki/pilot-reviews/vitest-audit-repo-full-profile-staleness.md` |

## Verification Snapshot

The symlink aliasing implementation batch closed with:

```text
node tests/test-symlink-aliasing.mjs
npm run test:vitest:symlink-aliasing
npm run check
npm run check:drift
npm run lint
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check tests/symlink-aliasing.test.mjs package.json docs/lumin-wiki/pilot-reviews/vitest-symlink-aliasing.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
npm run test:vitest
npm test
```

Local Windows symlink creation is unavailable in this shell, so the symlink
Node and Vitest suites report clean skips locally. Linux CI or Windows Developer
Mode provides the positive symlink realpath coverage.

## Grouped Node Shortcut

[`scripts/run-tests-grouped.mjs`](../../scripts/run-tests-grouped.mjs) adds the
opt-in `npm run test:node:groups` shortcut for local maintainer verification.
It preserves fresh Node subprocess isolation, runs suites serially inside each
deterministic group, uses bounded group-level parallelism, and prints failed
group/suite replay commands. Its behavior is covered by
[`tests/test-run-tests-grouped.mjs`](../../tests/test-run-tests-grouped.mjs) and
[`tests/run-tests-grouped.test.mjs`](../../tests/run-tests-grouped.test.mjs).

The latest dogfood run used:

```text
npm run test:node:groups -- --jobs 3
```

It passed 165 suites across 12 groups in 362.8 seconds. This shortcut does not
replace `npm test`; it is a faster first-pass runner for maintainers before the
serial lane or CI gate is needed.

## Closure Rule

Do not add more direct Vitest mirrors from the parked remainder until the
target suite receives a suite-specific review page that names:

- the protected invariant;
- the edge-case failure to preserve;
- the preserved Node command;
- the focused Vitest command;
- the fixture boundary;
- why shared helpers will not hide the original regression.

Use
[`concepts/parked-suite-dogfooding.md`](concepts/parked-suite-dogfooding.md) as
the next operating guide for this remainder. The next action is dogfooding the
parked suites with structure-review and test-reform rules, not adding another
direct mirror.
