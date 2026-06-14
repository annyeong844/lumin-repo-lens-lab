# Vitest Collect Files Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-collect.mjs`.

---

## Purpose

This review decides whether `tests/test-collect.mjs` is a reasonable Lane A
low-risk core/parser/helper Vitest pilot candidate. It does not add the Vitest
suite. The goal is to preserve the file collection contract around
`collectFiles(...)` without widening the mirror into audit orchestration,
resolver behavior, or scan-policy ranking.

The suite is a good next candidate because it exercises one helper module with
a self-contained filesystem fixture. It protects real edge cases that have
failed before: language-filter leakage, root-level Python/Go discovery,
cross-language test exclusion, JS/TS root entry preservation, user exclude
semantics, and repo-relative exclude matching.

## Reviewed Evidence

- Preserved Node command: `node tests/test-collect.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:collect`.
- Helper modules under review:
  - `_lib/collect-files.mjs`,
  - `_lib/test-paths.mjs` only through collect's `includeTests=false` filter.
- Current suite description: `tests/README.md`.
- Goal lane: Lane A, low-risk core/parser/helper.

## Result

The suite is acceptable as the next narrow Vitest pilot candidate.

The future mirror should keep the current helper-fixture shape and preserve the
Node entrypoint. It should not spawn the audit pipeline, resolve imports, rank
files, or infer analyzer absence claims. Each language filter, include-tests,
and exclude-path contract should remain a named assertion.

## Protected Invariants

The future Vitest pilot must preserve these file collection contracts:

- `languages: ["py"]` returns only `.py` files and never leaks root `.mjs` or
  `.ts` files;
- `languages: ["go"]` returns only `.go` files and never leaks root `.mjs`
  files;
- root-level `main.py`, `main.go`, and `some_test.go` are discovered when the
  requested language and `includeTests` setting allow them;
- `includeTests=false` drops pytest `*_test.py` and Go `*_test.go` files;
- JS/TS scans still include legitimate root entries such as `root-entry.ts` and
  `build-tool.mjs`;
- JS/TS scans keep `.test.ts` files when `includeTests=true`;
- JS/TS scans drop `.test.ts`, `*-test-support.ts`, `runtime-tests/`, and
  `test-utils/` when `includeTests=false`;
- JS/TS scans do not leak `.py` or `.go` files;
- user `exclude` rules prune root-level directories such as `output/`;
- directory excludes such as `build` match path segments, not filenames such as
  `build-index.ts`;
- basename excludes such as `skip-me.js` exclude the matching file;
- exact file-path excludes such as `src/nested/exact-file.js` exclude only that
  path, not siblings;
- repo-relative excludes such as `vendor` do not match absolute parent
  directory names outside the scanned root.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- root `.mjs` or `.ts` files leaking into Python/Go scans must fail;
- root-level Python/Go files being silently missed must fail;
- Python or Go test files surviving `includeTests=false` must fail;
- JS/TS root entries disappearing while fixing language filters must fail;
- substring-style test-path matching must stay delegated to the shared
  `isTestLikePath(...)` behavior and remain visible through collect's
  production filter;
- directory excludes must not accidentally prune similarly named files such as
  `src/build-index.ts`;
- file-path excludes must not prune sibling files;
- absolute parent directories named `vendor` must not poison repo-relative
  `vendor` excludes.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-collect.mjs` remains runnable.
- The fixture boundary is temporary filesystem setup and cleanup only.
- The mirror may use the setup-only temp repo helper only if it preserves the
  same path shapes, including relative-root and external vendor-root cases.
- The pilot must not change collect semantics, test-path semantics, resolver
  behavior, deadness/ranking behavior, audit orchestration, or scan-policy
  artifact wording.
- The pilot must not absorb `test-cli.mjs`, `test-mode-dispatch.mjs`,
  `test-audit-repo.mjs`, `test-shape-hash.mjs`, or analyzer-sensitive suites.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/collect.test.mjs`,
2. `npm run test:vitest:collect`,
3. a candidate-board update moving this suite from `REVIEWED` to `DONE`.

The implementation PR should keep every current Node case represented as a
named Vitest assertion. It may group assertions by language filtering,
include-tests behavior, JS/TS preservation, cross-language leakage, and exclude
rules, but file-collection meaning must stay local to this suite.

Run both commands when changing this suite:

- `node tests/test-collect.mjs`
- `npm run test:vitest:collect`

Do not migrate `test-shape-hash.mjs`, resolver suites, deadness/ranking suites,
orchestrator suites, performance/incremental suites, renderer suites, or
producer-backed scan-policy suites as part of the collect pilot.
