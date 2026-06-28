# Vitest P6 SAFE_FIX Calibration Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-23.
> **Pilot candidate:** `tests/test-p6-safe-fix-calibration.mjs`.

---

## Purpose

This review decides whether `tests/test-p6-safe-fix-calibration.mjs` may move
to a focused Vitest mirror. The suite proves that the full production evidence
chain can still produce a real `SAFE_FIX` candidate in a tiny local corpus:
static deadness, staleness, runtime coverage, action safety, ranking, and P6
measurement must agree.

The key risk is obvious and nasty. If this mirror gets too clever, it can fake
the very thing it is supposed to prove: that `SAFE_FIX=0` in large corpora is
corpus evidence, not a broken promotion path.

## Reviewed Evidence

| Suite                                    | Preserved Node Command                        | Proposed Focused Vitest Command               | Surface Under Review                                                   |
| ---------------------------------------- | --------------------------------------------- | --------------------------------------------- | ---------------------------------------------------------------------- |
| `tests/test-p6-safe-fix-calibration.mjs` | `node tests/test-p6-safe-fix-calibration.mjs` | `npm run test:vitest:p6-safe-fix-calibration` | full SAFE_FIX calibration pipeline from symbols through P6 measurement |

Goal lane: deadness/ranking calibration. This is a suite-specific review for
SAFE_FIX calibration evidence, not permission to migrate P6 measurement,
P6 member precision, rank-fixes, corpus precision, cue-tier policy, or
audit-repo umbrella behavior.

Fresh preserved-command evidence on 2026-05-23:

```text
node tests/test-p6-safe-fix-calibration.mjs
15 passed, 0 failed
```

## Result

This suite now has a narrow Vitest mirror at
`tests/p6-safe-fix-calibration.test.mjs`, and the preserved Node command
remains runnable. The mirror keeps the real mini git repo, runs the production
producer scripts as child processes, injects a minimal Istanbul coverage
artifact, and reads the resulting JSON artifacts.

The mirror must not extract helper logic that decides dead-export candidates,
runtime evidence, action safety, ranking, staleness, or P6 readiness. If the
mirror does not run the production chain, it is lying.

## Protected Invariants

The future Vitest mirror must preserve these 15 contracts:

### Calibration Candidate Emission

- P6S-1a: the calibration corpus emits the runtime-dead candidate.
- P6S-1b: the calibration corpus emits the runtime-hit candidate.
- P6S-1c: the calibration corpus emits the uncovered candidate.
- P6S-1d: the calibration corpus emits the type-only candidate as a
  `TSInterfaceDeclaration`.

### Runtime Evidence Merge

- P6S-2a: a covered zero-hit runtime symbol is `dead-confirmed` with grounded
  evidence.
- P6S-2b: a runtime-hit static-dead symbol is marked `executed` with its hit
  count.
- P6S-2c: a file absent from coverage is `uncovered`, not dead-confirmed.
- P6S-2d: an erased interface receives type-only runtime evidence.

### Ranking And Action Proof

- P6S-3a: the `SAFE_FIX` path is reachable when AST, runtime, and stale
  evidence converge.
- P6S-3b: a runtime-hit contradiction is `DEGRADED`, never `SAFE_FIX`.
- P6S-3c: an uncovered runtime range stays `REVIEW_FIX`.
- P6S-3d: a type-only export stays `REVIEW_FIX`.

### P6 Measurement Calibration

- P6S-4a: P6 measurement sees the non-empty `SAFE_FIX` population.
- P6S-4b: the `SAFE_FIX` adjudication denominator is known and zero-FP.
- P6S-4c: one tiny local corpus remains Yellow, not Green, because the
  benchmark is incomplete.

## Edge-Case Failures To Preserve

The mirror must fail if:

- the mini git repo is replaced by synthetic in-memory ranking data;
- zero-hit coverage no longer creates grounded `dead-confirmed` evidence;
- runtime hits are allowed to become `SAFE_FIX`;
- uncovered files are treated as dead-confirmed;
- type-only exports are promoted to automated action;
- `rank-fixes.mjs` cannot produce a non-empty `SAFE_FIX` list from converged
  proof;
- P6 measurement reports `safe-fix-population-empty` despite the calibration
  `SAFE_FIX`;
- zero-FP adjudication is lost or becomes `fp-rate-unknown`;
- the tiny local corpus is incorrectly upgraded to Green.

## Fixture Boundary

Allowed shared helper behavior:

- create and remove the temporary git repo and output directory;
- write small package and source files;
- create deterministic git commits for staleness evidence;
- write a minimal Istanbul coverage payload;
- write `canon-drift.json` and adjudication input artifacts;
- run the production scripts as child processes:
  `build-symbol-graph.mjs`, `classify-dead-exports.mjs`,
  `measure-staleness.mjs`, `merge-runtime-evidence.mjs`,
  `export-action-safety.mjs`, `rank-fixes.mjs`, and `p6-measurement.mjs`;
- read `symbols.json`, `runtime-evidence.json`, `fix-plan.json`, and
  `p6-measurement.json`.

Forbidden helper behavior:

- deciding whether a symbol is statically dead;
- deciding runtime status or grounding;
- deciding `SAFE_FIX`, `REVIEW_FIX`, or `DEGRADED`;
- deciding staleness;
- deciding P6 readiness, FP denominator, or benchmark completeness;
- replacing the production scripts with helper-only fixtures;
- sharing calibration logic with P6 measurement, P6 member precision, corpus,
  rank-fixes, export-action-safety, cue-tier policy, or audit-repo umbrella
  suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change symbol extraction, dead-export classification,
  staleness measurement, runtime merge policy, export action safety,
  rank-fixes, P6 measurement, or audit orchestration.
- The mirror must not absorb `tests/test-p6-measurement.mjs`,
  `tests/test-p6-member-precision.mjs`, `tests/test-rank-fixes.mjs`,
  `tests/test-corpus.mjs`, cue-tier policy, or `tests/test-audit-repo.mjs`.
- The mirror must not turn calibration evidence into broader permission to
  promote real-world candidates.

## Recommendation

The narrow implementation PR added:

1. `tests/p6-safe-fix-calibration.test.mjs`;
2. `npm run test:vitest:p6-safe-fix-calibration`;
3. candidate-board and goal updates moving this suite from `REVIEWED` to
   `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 15 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-p6-safe-fix-calibration.mjs
npm run test:vitest:p6-safe-fix-calibration
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-p6-safe-fix-calibration.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/vitest-mirror-closure-audit.md
git diff --check
```

Keep `docs/lumin-wiki/test-migration-candidate-board.md` as a targeted wide
table edit; do not run Prettier write on that file unless a separate table
normalization PR owns the churn.
