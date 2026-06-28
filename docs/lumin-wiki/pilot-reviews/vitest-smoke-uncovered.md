# Vitest Smoke Uncovered Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-smoke-uncovered.mjs`.

---

## Purpose

This review decides whether `tests/test-smoke-uncovered.mjs` can move as one
narrow Lane H Vitest mirror batch. It does not add a Vitest suite. The goal is
to preserve the shallow smoke coverage for script entrypoints that historically
had no dedicated coverage without turning the mirror into a deeper call graph,
barrel, discipline, SARIF, drift, runtime evidence, or full audit pipeline
suite.

The candidate is acceptable as a single-suite batch because it deliberately
checks only whether each entrypoint can run on a minimal fixture and emit a
recognizable artifact shape. Its current comments define the suite as a total
breakage guard for wrong imports, crashed parsers, missing writes, and schema
regressions. Deeper producer semantics must remain in dedicated suites.

## Reviewed Evidence

| Suite                            | Preserved Node Command                | Proposed Focused Vitest Command       | Surface Under Review                                      |
| -------------------------------- | ------------------------------------- | ------------------------------------- | --------------------------------------------------------- |
| `tests/test-smoke-uncovered.mjs` | `node tests/test-smoke-uncovered.mjs` | `npm run test:vitest:smoke-uncovered` | shallow script-entrypoint and artifact-shape smoke guards |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane H, orchestrator/artifact pipeline smoke guard.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-smoke-uncovered.mjs
30 passed, 0 failed
```

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the existing named smoke assertions
without changing producer behavior, artifact schemas, SARIF semantics,
drift-check behavior, or runtime evidence merge semantics. The Node entrypoint
must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `build-call-graph.mjs` completes on the minimal TS fixture and writes a
  parseable `call-graph.json` with a recognizable top-level shape;
- `check-barrel-discipline.mjs` completes on the fixture barrel and writes a
  parseable `barrels.json` with a recognizable top-level shape;
- `measure-discipline.mjs` completes on the fixture and writes a parseable
  `discipline.json` with a recognizable top-level shape;
- `emit-sarif.mjs` accepts an empty upstream artifact directory and writes
  parseable SARIF 2.1.0 with at least one run and a tool driver name/version;
- SARIF prefers classifier-backed dead-classify evidence over raw symbol output
  for the controlled policy fixture;
- classifier-policy-filtered config defaults do not leak into SARIF results;
- structured symbol graph parse warnings are retained in `symbols.json`;
- clean symbol graph scans produce an empty warnings array;
- SARIF propagates symbol graph parse warnings through upstream warning
  metadata;
- `scripts/check-drift.mjs` accepts matching package-lock versions, including
  prerelease versions, and reports `package-lock.json` drift when the lockfile
  version diverges;
- `merge-runtime-evidence.mjs` accepts a minimal symbols-plus-Istanbul coverage
  fixture and writes parseable `runtime-evidence.json` with a recognizable
  top-level shape.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- an entrypoint crash must fail the focused mirror;
- a missing or unparseable artifact must fail;
- losing the minimal top-level artifact shape must fail;
- emitting SARIF without version `2.1.0`, without runs, or without tool
  name/version must fail;
- reading raw symbols instead of classifier output must fail when it leaks the
  `eslint.config.mjs` default export;
- dropping structured parse warnings or SARIF upstream warnings must fail;
- accepting a drifted `package-lock.json` must fail;
- hiding the lockfile offender from drift output must fail;
- failing to merge runtime coverage into a parseable artifact must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is setup-only temporary repos plus current script
  entrypoints.
- A future mirror may use setup-only temp helpers, but helper code must not
  decide call graph, barrel, discipline, SARIF, classifier, drift, or runtime
  evidence semantics.
- The mirror must not deepen shape checks into full producer semantic
  assertions.
- The mirror must not change `build-call-graph.mjs`,
  `check-barrel-discipline.mjs`, `measure-discipline.mjs`, `emit-sarif.mjs`,
  `merge-runtime-evidence.mjs`, `scripts/check-drift.mjs`, classifier policy,
  ranking, resolver, deadness, performance, or public package behavior.
- The mirror must not absorb dedicated call graph, barrel, discipline, SARIF,
  drift, runtime evidence, audit-repo, ranking, resolver, or incremental suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/smoke-uncovered.test.mjs`,
2. `npm run test:vitest:smoke-uncovered`,
3. candidate-board updates moving `tests/test-smoke-uncovered.mjs` from
   `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves the
current Node assertions as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
