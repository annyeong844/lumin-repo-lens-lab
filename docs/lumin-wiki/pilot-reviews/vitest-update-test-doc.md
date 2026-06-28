# Vitest Generated Test README Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-12.
> **Pilot:** `tests/update-test-doc.test.mjs`.

---

## Purpose

This review closes the generated test README Vitest pilot gate before any
additional test-runner migration. The pilot tested whether Vitest can mirror the
generated `tests/README.md` drift guard without changing the generator
contract, the existing Node test entrypoint, or the repository-level doc checks.

## Reviewed Evidence

- Focused pilot command: `npm run test:vitest:update-test-doc`.
- All-pilot command: `npm run test:vitest`.
- Node parity command: `node tests/test-update-test-doc.mjs`.
- Repository doc drift guard: `npm run check:test-doc`.
- Documentation reference guard: `npm run check:doc-script-refs`.
- Full local CI evidence from the implementation PR: `npm run ci`.

## Result

The pilot is acceptable as a third separate runner lane.

Vitest improved this suite's execution mechanics:

- generated README contracts are runner-level `it(...)` blocks instead of
  custom pass/fail counters;
- drift, regeneration, marker, count-leak, maintainer-note, and hermeticity
  failures would point at the exact contract that failed;
- `npm run test:vitest:update-test-doc` gives a focused command for this pilot;
- `npm run test:vitest` exercises all reviewed pilot suites;
- the existing Node test remains runnable and keeps the original generator
  regression guard;
- `npm run check:test-doc` remains the repository-level generated README drift
  gate.

The pilot does not prove that generated docs or Markdown tests should all move
to Vitest. It proves that this setup-heavy drift guard can be mirrored while the
Node entrypoint and repo-level check stay authoritative.

## Protected Invariants

The Vitest pilot keeps the same generated README contract:

- `--check` passes when the fixture README is already in sync;
- `--check` fails when the README drifts;
- the drift report points maintainers at the regeneration command;
- regeneration rewrites drifted fixture content and makes `--check` pass again;
- the generated README keeps the do-not-edit marker;
- the generated README does not present an authoritative assertion count;
- a suite without a registered description surfaces a maintainer note;
- fixture regeneration never mutates the real repository `tests/README.md`.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-update-test-doc.mjs` remains runnable.
- `npm run check:test-doc` remains the repository-level drift check.
- The Vitest suite uses a helper-managed temporary repository and copies only
  the files needed by the generator: `CHANGELOG.md`, `scripts/`, and `tests/`.
- The shared temp fixture helper supplies setup and cleanup only; README
  generator meaning, drift assertions, and maintainer-note assertions stay local
  to the suite.
- The all-pilot Vitest command must remain scoped to reviewed `tests/*.test.mjs`
  files so generated fixture repositories and audit outputs are not collected as
  tests.

## Recommendation

The current Vitest pilot lane is healthy enough for reviewed, low-risk suites,
but broad migration is still not justified.

Before another suite migrates, require:

1. the protected invariant,
2. the edge case that must fail if the migration loses meaning,
3. the Node command that remains runnable,
4. the focused Vitest command,
5. the fixture boundary if the suite creates temporary repositories or generated
   files.

Avoid migrating resolver, deadness, pre-write, performance, ranking, or public
package suites until their suite-specific risks are reviewed in the wiki.
