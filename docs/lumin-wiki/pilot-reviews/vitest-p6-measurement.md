# Vitest P6 Measurement Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-22.
> **Pilot candidate:** `tests/test-p6-measurement.mjs`.

---

## Purpose

This review decides whether `tests/test-p6-measurement.mjs` may move to a
focused Vitest mirror. The suite protects the P6 measurement artifact contract:
candidate counts, adjudication denominators, readiness gates, schema
round-trip evidence, dirty-worktree safety, multi-corpus merges, and the
`p6-measurement.mjs` CLI.

The key risk is that a broad mirror could preserve only "measurement exists"
while dropping why a readiness gate is Red, Yellow, or Green. That would make
calibration evidence look stronger than it is, especially when counts are
missing, adjudication is incomplete, corpus state is dirty, or `SAFE_FIX`
population is empty.

## Reviewed Evidence

| Suite                           | Preserved Node Command               | Proposed Focused Vitest Command      | Surface Under Review                                       |
| ------------------------------- | ------------------------------------ | ------------------------------------ | ---------------------------------------------------------- |
| `tests/test-p6-measurement.mjs` | `node tests/test-p6-measurement.mjs` | `npm run test:vitest:p6-measurement` | P6 measurement artifact helpers, readiness, merge, and CLI |

Goal lane: deadness/ranking calibration. This is a suite-specific review for
P6 measurement evidence, not permission to migrate P6 member precision,
P6 safe-fix calibration, rank-fixes, corpus precision, cue-tier policy, or
audit-repo umbrella behavior.

Fresh preserved-command evidence on 2026-05-22:

```text
node tests/test-p6-measurement.mjs
26 passed, 0 failed
```

## Result

This suite now has a narrow Vitest mirror at
`tests/p6-measurement.test.mjs`, and the preserved Node command remains
runnable. The mirror imports the real `_lib/p6-measurement.mjs` helpers and
runs the real `p6-measurement.mjs` CLI for the CLI smoke cases.

The mirror must not extract helper logic that decides readiness gates,
candidate-count availability, FP denominators, schema round-trip status,
dirty-worktree safety, merge semantics, or CLI artifact shape.

## Protected Invariants

The future Vitest mirror must preserve these 26 contracts:

### Candidate Counts

- P6-1a: missing `fix-plan.json` makes `candidateCounts.available` false.
- P6-1b: missing cleanup counts remain `null`, not zero.
- P6-1c: missing `dead-classify.json` keeps `rawTierC` null.
- P6-1d: missing canon drift keeps canon drift unavailable with null totals.
- P6-2a: review-visible cleanup equals `safeFixes + reviewFixes`.
- P6-2b: degraded, muted, raw Tier C, and canon drift counts remain separate.

### Adjudication Denominators

- P6-3a: `inconclusive` and `not_applicable` entries are excluded from the
  `SAFE_FIX` FP denominator while still counted as separate context.
- P6-3b: review-visible cleanup denominator includes both `SAFE_FIX` and
  `REVIEW_FIX` entries.

### Readiness Blockers

- P6-4a: unavailable candidate counts force a Red readiness gate.
- P6-4b: missing adjudication records `fp-rate-unknown`.
- P6-4c: `schemaRoundTrip.attempted === false` blocks Green.
- P6-5a: dirty worktrees without snapshot or content hash block Green.
- P6-5b: unknown dirty state blocks Green.
- P6-6: clean corpus, low FP, and attempted schema round-trip can reach Green.
- P6-6b: measured-zero `SAFE_FIX` population blocks Green without becoming Red
  or falsely reporting unknown FP rate.

### Schema Round-Trip

- P6-7a: checked canon-drift sources mark round-trip attempted.
- P6-7b: parse-error canon sources record known schema drift bugs.
- P6-7c: direct `canon-drift.json` overrides stale
  `manifest.checkCanon` data.

### Multi-Corpus Merge

- P6-8a: merges sum cleanup counts and preserve per-corpus totals.
- P6-8b: merged schema and canon sources are prefixed by corpus.
- P6-8c: a merged two-corpus low-FP baseline can reach Green.
- P6-8d: a merged corpus with review-visible candidates but no adjudication is
  Red.

### CLI Smoke

- P6-9a: CLI writes `p6-measurement.json`.
- P6-9b: CLI preserves candidate counts and adjudication entries.
- P6-10a: CLI `--merge` writes a merged `p6-measurement.json`.
- P6-10b: CLI `--merge` recomputes aggregate readiness.

## Edge-Case Failures To Preserve

The mirror must fail if:

- missing input artifacts are converted from `null` or unavailable into zero;
- degraded, muted, raw Tier C, or canon drift counts are folded into
  review-visible cleanup;
- inconclusive or not-applicable adjudications affect FP denominators;
- Red readiness blockers disappear under missing counts, missing adjudication,
  unattempted schema round-trip, dirty worktrees, or unknown dirty state;
- measured-zero `SAFE_FIX` population is treated as Green;
- direct canon-drift evidence loses precedence over stale manifest data;
- merged corpus data loses per-corpus prefixes or silently drops unreviewed
  corpus candidates;
- CLI smoke tests validate only process exit without reading the artifact;
- merge mode reuses stale readiness instead of recomputing it.

## Fixture Boundary

Allowed shared helper behavior:

- construct in-memory candidate-count, corpus, adjudication, runtime, and
  schema-round-trip payloads;
- create and remove temporary directories for CLI smoke cases;
- write synthesized `fix-plan.json`, `dead-classify.json`, `canon-drift.json`,
  `manifest.json`, adjudication input, and merge input artifacts;
- run `p6-measurement.mjs` as a child process for CLI cases;
- read `p6-measurement.json` and assert schema, counts, readiness, merge, and
  adjudication fields.

Forbidden helper behavior:

- deciding whether missing artifacts mean zero;
- deciding readiness gate or reason codes;
- deciding FP denominator membership;
- deciding dirty-worktree safety;
- deciding schema round-trip source precedence;
- deciding merge prefixing or aggregate readiness;
- sharing calibration helper logic with P6 member precision,
  P6 safe-fix calibration, corpus, rank-fixes, export-action-safety, or
  audit-repo umbrella suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change `_lib/p6-measurement.mjs`, `p6-measurement.mjs`,
  ranking policy, dead-export classification, canon drift production,
  `fix-plan.json`, or audit orchestration.
- The mirror must not absorb `tests/test-p6-member-precision.mjs`,
  `tests/test-p6-safe-fix-calibration.mjs`, `tests/test-rank-fixes.mjs`,
  `tests/test-corpus.mjs`, cue-tier policy, or `tests/test-audit-repo.mjs`.
- The mirror must not turn readiness evidence into permission to promote
  `SAFE_FIX` action proof.

## Recommendation

The narrow implementation PR added:

1. `tests/p6-measurement.test.mjs`;
2. `npm run test:vitest:p6-measurement`;
3. candidate-board and goal updates moving this suite from `REVIEWED` to
   `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 26 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-p6-measurement.mjs
npm run test:vitest:p6-measurement
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-p6-measurement.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/vitest-mirror-closure-audit.md
git diff --check
```

Keep `docs/lumin-wiki/test-migration-candidate-board.md` as a targeted wide
table edit; do not run Prettier write on that file unless a separate table
normalization PR owns the churn.
