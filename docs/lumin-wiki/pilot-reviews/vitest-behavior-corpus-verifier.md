# Vitest Behavior Corpus Verifier Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-12.
> **Pilot:** `tests/behavior-corpus-verifier.test.mjs`.

---

## Purpose

This review closes the second Vitest pilot gate before any additional
test-runner migration. The pilot tested whether Vitest can mirror a behavior
corpus verifier suite without changing the saved-answer contract, the Node
verifier entrypoint, or the public package runtime.

## Reviewed Evidence

- Focused pilot command: `npm run test:vitest:behavior`.
- All-pilot command: `npm run test:vitest`.
- Node parity command: `node tests/test-behavior-corpus-verifier.mjs`.
- Existing temp fixture Node parity command:
  `node tests/test-temp-repo-fixture-helper.mjs`.
- Existing focused temp fixture pilot command:
  `npm run test:vitest:temp-fixture`.
- Documentation guards: `npm run check:test-doc` and
  `npm run check:doc-script-refs`.
- Full local CI evidence from the implementation PR: `npm run ci`.

## Result

The pilot is acceptable as a second separate runner lane.

Vitest improved this suite's execution mechanics:

- behavior cases are runner-level `it(...)` blocks instead of custom
  pass/fail counters;
- assertion failures would point at the exact user-facing behavior contract;
- `npm run test:vitest:behavior` gives a focused command for the behavior
  verifier;
- `npm run test:vitest` exercises all currently reviewed Vitest pilots;
- the existing Node verifier remains runnable and still checks the saved corpus.

The pilot also found an important runner-boundary issue: unscoped `vitest run`
will treat fixture corpora and generated repo outputs as candidate test files.
`vitest.config.mjs` now scopes discovery to `tests/*.test.mjs` and excludes
known data/output roots so the professional runner does not reinterpret corpus
data as tests.

## Protected Invariants

The Vitest pilot keeps the same behavior corpus verifier contract:

- plain answers with required cues and no internal jargon pass;
- normal chat answers with internal jargon fail;
- review-only dead-export wording can pass when caveated;
- overconfident review-only dead-export wording fails;
- the checked-in behavior corpus verifies 11 cases;
- the CLI exits zero for the checked-in corpus;
- the CLI exits non-zero when an expected-pass case fails;
- read-trace expectations fail when a required artifact is absent.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The Vitest suite calls the verifier through the Node CLI/module path rather
  than making Vite transform verifier internals.
- The all-pilot Vitest command must remain scoped to reviewed pilot files.
  Behavior corpora, generated fixture repositories, audit outputs, and lab
  payloads are test data, not runner-discoverable test suites.

## Recommendation

Do not migrate another suite until the next candidate names:

1. the protected invariant,
2. the exact edge case that must still fail if the migration loses meaning,
3. the Node command that remains runnable,
4. the focused Vitest command that proves the migrated shape,
5. the runner discovery boundary if the suite creates fixture repositories or
   corpus data.

Avoid migrating resolver, deadness, pre-write, performance, ranking, or public
package suites until their suite-specific risks are reviewed in the wiki.
