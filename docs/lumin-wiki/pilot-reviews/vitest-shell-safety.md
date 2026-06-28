# Vitest Shell Safety Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-shell-safety.mjs`.

---

## Purpose

This review decides whether `tests/test-shell-safety.mjs` can move as a narrow
Lane A Vitest mirror. It does not add a Vitest suite. The goal is to preserve
Issue 7 shell-safety regressions around triage and staleness: file paths with
shell metacharacters must survive tool subprocesses without expansion, and
language file collection must keep the single-pass triage refactor honest.

The candidate is acceptable as a single-suite mirror because it builds small
temporary fixtures with Node filesystem APIs, exercises only `triage-repo.mjs`,
`build-symbol-graph.mjs`, and `measure-staleness.mjs`, and asserts concrete
artifact fields. It does not run the full audit orchestrator or rank analyzer
findings.

The future mirror should keep the shell-safety and file-collection contracts
local. It must not expand into broad git history analysis, deadness/ranking,
resolver behavior, full audit orchestration, performance benchmarking, or
public package behavior.

## Reviewed Evidence

| Suite                         | Preserved Node Command             | Proposed Focused Vitest Command    | Surface Under Review                                        |
| ----------------------------- | ---------------------------------- | ---------------------------------- | ----------------------------------------------------------- |
| `tests/test-shell-safety.mjs` | `node tests/test-shell-safety.mjs` | `npm run test:vitest:shell-safety` | shell-safe paths, triage language counts, staleness records |

Current suite description is in `tests/README.md`.

Goal lane: Lane A, low-risk core/helper command-safety guard.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same fixture-level behavior
without changing `triage-repo.mjs`, `build-symbol-graph.mjs`,
`measure-staleness.mjs`, or their output contracts. The Node entrypoint must
remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `triage-repo.mjs` completes on filenames containing `$`;
- triage counts TypeScript files correctly when one filename contains `$`;
- triage counts Python files correctly when one filename contains `$`;
- triage records single-pass file-collection telemetry with one
  `collectFiles` call;
- a root-only Python repo containing `main.py` is detected without requiring
  `src/` or `tests/`;
- Go files are counted in `triage.json.shape.goFiles`;
- `topDirs.src.files` includes every file in the weird-name fixture;
- `measure-staleness.mjs` emits staleness records on a normal git fixture;
- staleness processing preserves `$`-named files and records non-null
  `fileLastTouchedAt` evidence for them.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- shell expansion of `$` inside tool-forwarded paths must fail;
- accidentally returning to shell-based `find` or gated Python discovery must
  fail for root-only Python fixtures;
- dropping Go counting from triage shape must fail;
- losing single-pass file-collection telemetry must fail;
- staleness rel-file handling that cannot survive `$` paths must fail;
- stale fixture setup that bypasses the actual tool subprocesses must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is temporary directories, git init/commit setup, and
  direct producer subprocesses.
- A future mirror may replace fixed temp paths with the setup-only temp repo
  fixture helper where practical, but helper code must not decide triage,
  staleness, shell-safety, language-count, or git-history meaning.
- The mirror may use argument-safe process calls such as `execFileSync`.
- The mirror must not add broad assertions about dead-export classification,
  resolver behavior, ranking, performance timing, full audit orchestration, or
  public package install behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/shell-safety.test.mjs`,
2. `npm run test:vitest:shell-safety`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current Node assertion as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
