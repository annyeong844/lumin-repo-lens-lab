# Vitest Call Graph Evidence Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:** `tests/test-call-graph-bounded.mjs`,
> `tests/test-call-graph-parse-errors.mjs`,
> `tests/test-call-graph-truncation-defense.mjs`.

---

## Purpose

This review decides whether three call-graph evidence guard suites can move as
one narrow Lane H Vitest mirror batch. It does not add Vitest suites. The goal
is to preserve `call-graph.json` evidence semantics without turning the mirror
into a ranking, deadness, resolver, full audit, performance, or incremental
cache test.

The candidates are acceptable as one batch because all three suites run
`build-call-graph.mjs` against controlled temporary fixtures and inspect only
the produced call graph artifact:

- `tests/test-call-graph-bounded.mjs` validates bounded imported object member
  call evidence and the bounded-out counters that prevent overclaiming.
- `tests/test-call-graph-parse-errors.mjs` validates incomplete-artifact
  diagnostics when one scanned file fails to parse.
- `tests/test-call-graph-truncation-defense.mjs` validates that display
  truncation in `topCallees` does not delete full fan-in evidence.

The future mirrors should keep those contracts local. They must not expand into
dead-export classification, ranking policy selection, action-safety proof,
resolver expansion, module reachability, full audit orchestration, or
performance measurement.

## Reviewed Evidence

| Suite                                          | Preserved Node Command                              | Proposed Focused Vitest Command                     | Surface Under Review                                      |
| ---------------------------------------------- | --------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------- |
| `tests/test-call-graph-bounded.mjs`            | `node tests/test-call-graph-bounded.mjs`            | `npm run test:vitest:call-graph-bounded`            | bounded imported object member-call fan-in evidence       |
| `tests/test-call-graph-parse-errors.mjs`       | `node tests/test-call-graph-parse-errors.mjs`       | `npm run test:vitest:call-graph-parse-errors`       | call graph parse-error completeness warnings              |
| `tests/test-call-graph-truncation-defense.mjs` | `node tests/test-call-graph-truncation-defense.mjs` | `npm run test:vitest:call-graph-truncation-defense` | full fan-in evidence preserved outside display truncation |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane H, call-graph evidence guard.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should preserve the same temporary-fixture
subprocess boundary and artifact assertions without changing
`build-call-graph.mjs`, `call-graph.json` fan-in semantics, bounded member-call
support, parse-error warning semantics, or display truncation behavior. The
Node entrypoints must remain runnable.

## Protected Invariants

The future Vitest mirrors must preserve these contracts:

- `call-graph.json.meta.supports.boundedMemberCallResolution` stays `true`;
- default exported object member calls map only to mechanically known function
  properties;
- named exported object member calls map function properties only;
- non-function imported object member calls are counted as bounded out and do
  not create fake fan-in;
- depth-2 imported object member calls are counted as bounded out and do not
  create fake fan-in;
- `memberCallsByFile` and `boundedOutMemberCallsByFile` keep the observed total
  versus bounded-out split;
- `call-graph.json.meta.complete` becomes `false` when any scanned file fails
  to parse;
- `call-graph.json.meta.parseErrors` records the parse-error count;
- parse-error warnings retain the warning code, count, malformed file, and a
  human-readable parser message;
- `topCallees` remains a display slice capped at 100 entries;
- `callFanInByIdentity` remains the full evidence map and retains identities
  outside the `topCallees` display slice.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- treating unknown, non-function, or depth-2 member calls as real fan-in must
  fail;
- dropping bounded-out counters must fail;
- letting a parse-error artifact look complete must fail;
- dropping parse-error file names or parser messages must fail;
- using `topCallees` as the full fan-in source must fail;
- truncating `callFanInByIdentity` together with the display slice must fail;
- promoting call-graph evidence directly into deadness, ranking, or action
  proof must stay out of scope.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is a temporary repo plus direct
  `build-call-graph.mjs` subprocess execution.
- A future mirror may use setup-only temp helpers, but helper code must not
  decide call-graph fan-in, bounded-out, parse-error, ranking, deadness,
  action-safety, resolver, or performance meaning.
- The mirror must not run the full audit pipeline.
- The mirror must not change ranking, classifier, resolver, deadness,
  performance, incremental cache, or public package behavior.
- The mirror must not absorb `tests/test-rank-fixes.mjs`,
  `tests/test-module-reachability.mjs`, `tests/test-export-action-safety.mjs`,
  resolver suites, broader audit-repo suites, or performance/incremental
  suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/call-graph-bounded.test.mjs`,
2. `tests/call-graph-parse-errors.test.mjs`,
3. `tests/call-graph-truncation-defense.test.mjs`,
4. `npm run test:vitest:call-graph-bounded`,
5. `npm run test:vitest:call-graph-parse-errors`,
6. `npm run test:vitest:call-graph-truncation-defense`,
7. candidate-board updates moving the three suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current Node assertion as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.
