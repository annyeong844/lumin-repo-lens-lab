# Vitest Hardcoding Guards Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-hardcoding.mjs`.

---

## Purpose

This review decides whether `tests/test-hardcoding.mjs` can move as a narrow
Lane A Vitest mirror. It does not add a Vitest suite. The goal is to preserve
the regression guards that removed repository-specific labels and
repository-specific method-call focus output from generic analyzer behavior.

The candidate is acceptable as a single-suite mirror because it protects two
small Issue 5 contracts:

- dead-export package labels must derive from the actual workspace fixture,
  not from old repo-specific names;
- method-call reporting must only render a class-specific focus block when the
  caller passes `--focus-class`.

The future mirror should keep those two contracts local. It must not expand
into broader deadness/ranking proof, method-call resolution accuracy, call-graph
semantics, or full audit orchestration behavior.

## Reviewed Evidence

| Suite                       | Preserved Node Command           | Proposed Focused Vitest Command  | Surface Under Review                                      |
| --------------------------- | -------------------------------- | -------------------------------- | --------------------------------------------------------- |
| `tests/test-hardcoding.mjs` | `node tests/test-hardcoding.mjs` | `npm run test:vitest:hardcoding` | workspace-derived labels and method focus-class reporting |

Current suite description is in `tests/README.md`.

Goal lane: Lane A, low-risk core/helper regression guard.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same fixture-level behavior
without changing `classify-dead-exports.mjs`, `resolve-method-calls.mjs`, or
their output contracts. The Node entrypoint must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- the synthetic monorepo contains `packages/alpha` and `apps/beta` workspaces;
- `classify-dead-exports.mjs` output includes workspace-derived `alpha` and
  `beta` labels;
- `classify-dead-exports.mjs` output does not fabricate the legacy labels
  `protocol`, `daemon`, `web-shell`, or `shared-utils`;
- `resolve-method-calls.mjs` does not print a `RunChannelClient` focus block
  when `--focus-class` is omitted;
- `resolve-method-calls.mjs --focus-class MyClass` prints a `MyClass`-specific
  focus block;
- `resolve-method-calls.mjs --focus-class MyClass` still does not print a
  `RunChannelClient` block;
- `level2-methods.json.focusClassReport.className` records `MyClass` when the
  flag is supplied;
- `level2-methods.json.focusClassReport` is `null` when the flag is omitted.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- adding repo-specific label fallbacks must fail when the synthetic repo has no
  matching directories;
- losing real workspace label derivation must fail for both `packages/*` and
  `apps/*` fixture scopes;
- restoring a hardcoded `RunChannelClient` method report must fail when
  `--focus-class` is omitted;
- ignoring `--focus-class MyClass` must fail both in console output and in the
  structured `level2-methods.json` artifact;
- stale focus-class JSON must fail after rerunning without the flag.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The mirror may replace fixed `/tmp` paths with the setup-only temp repo
  fixture helper.
- The mirror may use `execFileSync` or equivalent argument-safe process calls
  for harness setup, but it must not change the production scripts.
- The mirror must not add broad assertions about dead-export classification,
  method-call precision, call-graph edges, artifact schemas beyond
  `focusClassReport`, or full audit orchestration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/hardcoding.test.mjs`,
2. `npm run test:vitest:hardcoding`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current Node assertion as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
