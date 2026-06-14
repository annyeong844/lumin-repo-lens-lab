# Vitest Pre-Write Inventory Hook Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-pre-write-inventory-hook.mjs`.

---

## Purpose

This review decides whether the pre-write inventory hook suite can move as one
narrow Lane C Vitest mirror. It does not add the Vitest suite.

The candidate protects the pre-write artifact-availability boundary: a normal
pre-write run may produce an invocation-specific `any-inventory` snapshot and
stamp that snapshot into advisory artifacts, while `--no-fresh-audit` must leave
that evidence unavailable instead of pretending absence is grounded.

This batch is intentionally separate from the host hook runtime batch,
post-write lifecycle batch, pre-write lookup/advisory shape suites, cue-tier
policy tests, broader `audit-repo.mjs` orchestration, resolver behavior,
deadness/ranking, and performance/incremental cache identity.

## Reviewed Evidence

| Suite                                     | Preserved Node Command                         | Proposed Focused Vitest Command                | Surface Under Review                        |
| ----------------------------------------- | ---------------------------------------------- | ---------------------------------------------- | ------------------------------------------- |
| `tests/test-pre-write-inventory-hook.mjs` | `node tests/test-pre-write-inventory-hook.mjs` | `npm run test:vitest:pre-write-inventory-hook` | pre-write `any-inventory` snapshot stamping |

Current Node evidence checked for this review:

```text
node tests/test-pre-write-inventory-hook.mjs # 21 passed, 0 failed
```

Goal lane: Lane C, pre/post-write lifecycle. This review covers only the
pre-write inventory snapshot hook subset of that lane.

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR should preserve exact artifact and advisory
pointer expectations without changing `pre-write.mjs`, the advisory schema,
`any-inventory` production, lookup behavior, cue tiers, Markdown rendering, or
post-write lifecycle behavior. The Node entrypoint must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- default pre-write runs write exactly one
  `any-inventory.pre.<invocationId>.json` snapshot;
- the snapshot carries a `typeEscapes` array;
- exactly one invocation-specific pre-write advisory is written alongside
  `pre-write-advisory.latest.json`;
- both latest and invocation-specific advisory artifacts stamp
  `preWrite.anyInventoryPath`;
- both advisory artifacts stamp the identical relative snapshot path;
- the stamped `anyInventoryPath` resolves to an existing snapshot file;
- `--no-fresh-audit` writes no pre-write inventory snapshot;
- under `--no-fresh-audit`, `preWrite.anyInventoryPath` is absent, not `null`,
  empty, or stale;
- existing P1 advisory fields remain present: `invocationId`, 64-character
  `intentHash`, `lookups[]`, `drift[]`, `capabilities`, and `failures[]`;
- the snapshot records `meta.supports.typeEscapes === true`;
- the snapshot records `meta.complete === true` for the clean fixture;
- the snapshot records the canonical 11 escape kinds;
- pre-write writes to the invocation-specific snapshot path and leaves an
  existing shared `any-inventory.json` untouched.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a missing snapshot on a default fresh pre-write run must fail;
- latest and invocation-specific advisory artifacts pointing at different
  snapshots must fail;
- `--no-fresh-audit` must not leave a stale or `null` `anyInventoryPath`;
- a pre-write run that clobbers a pre-existing shared `any-inventory.json` must
  fail;
- losing P1 advisory fields while adding snapshot metadata must fail;
- weakening the mirror into loose "file exists" assertions without pointer
  equality and absence checks must fail.

## Known Gap

The Node suite header mentions a hook-failure path, but the current executable
assertions cover T1-T5 only and do not inject an `any-inventory` hook failure.
The Vitest mirror should not claim hook-failure coverage unless a separate
test-strengthening PR first adds that assertion to the preserved Node suite.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is a temporary package root, a temporary output
  directory, an intent JSON file, direct `pre-write.mjs` invocation, and JSON
  reads of the emitted advisory/snapshot artifacts.
- The mirror may share local setup-only helpers for temp roots, fixture file
  writes, intent JSON, advisory discovery, and JSON parsing.
- Shared helpers must not decide evidence availability, absence proof,
  snapshot completeness, advisory schema meaning, lookup semantics, cue-tier
  routing, Markdown wording, or post-write behavior.
- The mirror must not absorb `tests/test-pre-write-advisory-artifact.mjs`,
  `tests/test-pre-write-bootstrap.mjs`, `tests/test-pre-write-cli.mjs`,
  `tests/test-audit-repo-pre-write.mjs`, cue-tier policy suites, host hook
  runtime suites, post-write lifecycle suites, resolver behavior,
  deadness/ranking, or performance/incremental cache identity suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/pre-write-inventory-hook.test.mjs`,
2. `npm run test:vitest:pre-write-inventory-hook`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves the
current Node assertion groups as named Vitest cases. It should run the
preserved Node command, the focused Vitest command, and `npm run test:vitest`.
