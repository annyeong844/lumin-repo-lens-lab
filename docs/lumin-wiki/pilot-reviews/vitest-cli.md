# Vitest CLI Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-cli.mjs`.

---

## Purpose

This review decides whether `tests/test-cli.mjs` is a reasonable Lane A
low-risk core/parser/helper Vitest pilot candidate. It does not add the Vitest
suite. The goal is to preserve the focused CLI helper contracts around
`parseCliArgs(...)` and `isTestLikePath(...)` without widening the suite into
audit orchestrator behavior.

The suite is a good next candidate because it imports small helper modules,
does not run the audit pipeline, and protects concrete edge cases that have
failed before: boolean negation flags, string-valued include-tests flags,
production precedence, test-path convention coverage, and substring false
positives.

## Reviewed Evidence

- Preserved Node command: `node tests/test-cli.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:cli`.
- Helper modules under review:
  - `_lib/cli.mjs`,
  - `_lib/test-paths.mjs`.
- Current suite description: `tests/README.md`.
- Goal lane: Lane A, low-risk core/parser/helper.

## Result

The suite is acceptable as the next narrow Vitest pilot candidate.

The future mirror should keep the current direct-module probing shape. It
should not spawn the public CLI, run `audit-repo.mjs`, test mode-dispatch
behavior, or expand into broader command lifecycle semantics. The old Node
entrypoint must remain runnable, and the Vitest mirror should express each CLI
flag and test-path convention as named assertions.

## Protected Invariants

The future Vitest pilot must preserve these CLI helper contracts:

- default `parseCliArgs(...)` returns `includeTests === true` as a boolean;
- default output remains `<root>/.audit`;
- `--include-tests` keeps `includeTests === true`;
- `--no-include-tests`, `--no-tests`, `--exclude-tests`, and `--production`
  set `includeTests === false`;
- `--include-tests=false` becomes boolean `false`, not truthy string
  `"false"`;
- `--include-tests=true` becomes boolean `true`, not string `"true"`;
- `--production` wins when combined with `--include-tests`;
- unrelated flags such as `--verbose` do not perturb `includeTests`;
- `isTestLikePath(...)` recognizes `.test`, `.spec`, pytest `test_*.py`, Go
  `*_test.go`, `tests/`, `runtime-tests/`, `test-utils/`, and
  `*-test-support` conventions;
- `isTestLikePath(...)` does not match substring-only names such as
  `contest.ts`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- `--include-tests=false` must fail if it regresses to a truthy string value;
- `--no-include-tests` and alias flags must fail if negation silently remains
  true;
- `--production` must fail if it no longer overrides `--include-tests`;
- test-path convention coverage must fail if pytest, Go, or path-segment test
  naming is dropped;
- `contest.ts` must fail if substring matching reappears;
- Windows absolute paths must continue to use `file://` dynamic import URLs
  rather than raw drive-letter paths.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-cli.mjs` remains runnable.
- The fixture boundary is process argument mutation plus direct dynamic imports
  of `_lib/cli.mjs` and `_lib/test-paths.mjs`.
- The mirror may use cache-busted file URLs to isolate repeated
  `process.argv` probes.
- The pilot must not spawn `audit-repo.mjs` or test full command lifecycle
  routing.
- The pilot must not absorb `test-mode-dispatch.mjs`, `test-audit-repo.mjs`,
  `test-collect.mjs`, or other orchestrator/file-walk suites.
- The pilot must not change CLI parsing behavior.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/cli.test.mjs`,
2. `npm run test:vitest:cli`,
3. a candidate-board update moving this suite from `REVIEWED` to `DONE`.

The implementation PR should keep every current Node case represented as a
named Vitest assertion. It may factor a tiny local helper for cache-busted
module imports and temporary `process.argv` mutation, but CLI/test-path meaning
must stay local to this suite.

Run both commands when changing this suite:

- `node tests/test-cli.mjs`
- `npm run test:vitest:cli`

Do not migrate `test-mode-dispatch.mjs`, `test-audit-repo.mjs`,
`test-collect.mjs`, resolver suites, deadness/ranking suites,
performance/incremental suites, or cue-tier suites as part of the CLI pilot.
