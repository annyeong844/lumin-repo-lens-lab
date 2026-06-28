# Vitest Evidence Honesty Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-evidence-honesty.mjs`.

---

## Purpose

This review decides whether `tests/test-evidence-honesty.mjs` can move as a
narrow Lane A Vitest mirror. It does not add a Vitest suite. The goal is to
preserve the v1.9.8 evidence-honesty regression guards around two maintainer
surfaces:

1. `compare-repos.mjs` reads existing audit artifacts and reports deltas without
   inventing missing evidence.
2. `scripts/check-doc-script-refs.mjs` fails when live documentation references
   a `.mjs` file that is not present on disk.

The candidate is acceptable as a single-suite mirror because it builds small
temporary artifact directories, copies the real doc-ref guard into a fixture,
and asserts concrete JSON or process-exit behavior. It does not run the full
audit orchestrator, resolver, deadness/ranking, performance, or public package
pipeline.

The future mirror should keep the verifier contracts local. It must not expand
into broad documentation policy, artifact schema redesign, compare heuristics,
audit producer behavior, or generated test README semantics.

## Reviewed Evidence

| Suite                             | Preserved Node Command                 | Proposed Focused Vitest Command        | Surface Under Review                          |
| --------------------------------- | -------------------------------------- | -------------------------------------- | --------------------------------------------- |
| `tests/test-evidence-honesty.mjs` | `node tests/test-evidence-honesty.mjs` | `npm run test:vitest:evidence-honesty` | compare artifacts, doc `.mjs` reference guard |

Current suite description is in `tests/README.md`.

Goal lane: Lane A, low-risk verifier/helper evidence guard.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same fixture-level behavior
without changing `compare-repos.mjs`, `scripts/check-doc-script-refs.mjs`, or
their output contracts. The Node entrypoint must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `compare-repos.mjs` exits zero on valid left/right artifact directories;
- `compare.json.deltas.files` reports `right - left`;
- `compare.json.deltas.safeFixes` reports `right - left`;
- `compare.json.deltas.degraded` reports `right - left`;
- both sides list the expected `fix-plan.json`, `symbols.json`, and
  `triage.json` artifacts when present;
- `missingArtifacts.left` and `missingArtifacts.right` explicitly list absent
  known artifacts such as `runtime-evidence.json` and `staleness.json`;
- a delta depending on an artifact missing from either side is `null`, not an
  invented number;
- `check-doc-script-refs.mjs` exits zero when every referenced `.mjs` file is
  present;
- `check-doc-script-refs.mjs` exits non-zero and names the missing file when a
  live document references a missing `.mjs` file;
- the doc-ref guard error message suggests concrete remediation;
- files under `_lib/` count as present for bare `.mjs` references.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- asymmetric artifact availability must not produce numeric deltas;
- missing runtime/staleness artifacts must remain visible as missing evidence,
  not hidden as zero;
- doc references to missing scripts must fail the guard;
- fixture setup must exercise the real `compare-repos.mjs` and real
  `check-doc-script-refs.mjs`, not reimplement their logic inside the test;
- `_lib/` resolution must remain covered so doc refs to helper modules do not
  become false failures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is temporary artifact directories, synthetic JSON
  artifacts, minimal documentation files, and a copied real doc-ref guard.
- A future mirror may use the setup-only temp repo fixture helper where useful,
  but helper code must not decide compare semantics, missing-artifact semantics,
  doc-reference extraction, or remediation wording.
- The mirror should prefer argument-safe process calls such as `execFileSync`
  over shell command strings where practical.
- The mirror must not broaden into resolver behavior, deadness/ranking,
  generated/framework surfaces, performance timing, full audit orchestration, or
  public package install behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/evidence-honesty.test.mjs`,
2. `npm run test:vitest:evidence-honesty`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current Node assertion as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
