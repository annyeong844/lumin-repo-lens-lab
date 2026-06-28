# Vitest Audit Repo Incremental Forwarding Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidates:** `tests/test-audit-repo-symbol-incremental.mjs`,
> `tests/test-function-clone-audit-forwarding.mjs`.

---

## Purpose

This review decides whether two small `audit-repo.mjs` incremental-forwarding
suites can move together as one narrow Vitest mirror batch. It does not add a
Vitest suite. The goal is to preserve the orchestrator contract that
incremental flags and cache roots are forwarded to supported producers without
changing symbol graph extraction, function clone grouping, cache identity, or
the broader full-audit pipeline.

The suites are acceptable as a paired batch because both protect command
forwarding and metadata visibility. They do not validate the underlying
incremental cache algorithms themselves. Those deeper cache identity contracts
remain parked in the dedicated performance/incremental suites.

## Reviewed Evidence

| Suite                                            | Preserved Node Command                                | Proposed Focused Vitest Command                       | Surface Under Review                               |
| ------------------------------------------------ | ----------------------------------------------------- | ----------------------------------------------------- | -------------------------------------------------- |
| `tests/test-audit-repo-symbol-incremental.mjs`   | `node tests/test-audit-repo-symbol-incremental.mjs`   | `npm run test:vitest:audit-repo-symbol-incremental`   | quick audit forwarding to `build-symbol-graph.mjs` |
| `tests/test-function-clone-audit-forwarding.mjs` | `node tests/test-function-clone-audit-forwarding.mjs` | `npm run test:vitest:function-clone-audit-forwarding` | full audit forwarding to function clone producer   |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane H, orchestrator/artifact pipelines, with a narrow Lane F
incremental-metadata boundary.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-audit-repo-symbol-incremental.mjs
2 passed, 0 failed

node tests/test-function-clone-audit-forwarding.mjs
3 passed, 0 failed
```

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the existing temp-repo assertions
without changing `audit-repo.mjs`, incremental cache stores, symbol graph
extraction, function clone normalization, producer ordering, or full-audit
artifact semantics. The Node entrypoints must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `audit-repo.mjs --no-incremental` forwards the disabled incremental mode to
  `build-symbol-graph.mjs`;
- `audit-repo.mjs --cache-root <path>` forwards the explicit cache root to
  `build-symbol-graph.mjs`;
- full-profile `audit-repo.mjs --no-incremental` forwards the disabled
  incremental mode to the function clone producer;
- full-profile `audit-repo.mjs --cache-root <path with spaces>` forwards the
  explicit cache root to the function clone producer;
- full-profile `audit-repo.mjs --clear-incremental-cache` clears the shared
  incremental cache before supported producers run;
- forwarded metadata remains visible in `symbols.json.meta.incremental` and
  `function-clones.json.meta.incremental`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- dropping `--no-incremental` forwarding must fail;
- dropping custom `--cache-root` forwarding must fail;
- paths with spaces in the cache root must still round-trip through the
  orchestrator and producer metadata;
- clearing the shared incremental cache must reset reuse counts for the next
  supported producer run;
- a mirror must not pass by reading stale artifact output from an earlier run;
- a helper must not hide whether the observed metadata came from `symbols.json`
  or `function-clones.json`.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is a tiny synthetic package plus the real
  `audit-repo.mjs` entrypoint.
- A future mirror may use setup-only temp helpers for directory creation,
  source writing, command execution, JSON reads, and cleanup.
- Shared helper code must not decide cache correctness, producer reuse,
  function clone grouping, symbol graph extraction, or incremental identity.
- The mirror must not absorb the broader
  `tests/test-any-inventory-incremental.mjs`,
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-function-clone-incremental.mjs`,
  `tests/test-incremental-cache-store.mjs`,
  `tests/test-incremental-snapshot.mjs`, or `tests/test-incremental.mjs`
  suites.
- The mirror must not change resolver behavior, deadness/ranking,
  generated-artifact policy, cache identity, performance counters, or public
  package behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/audit-repo-symbol-incremental.test.mjs`,
2. `tests/function-clone-audit-forwarding.test.mjs`,
3. `npm run test:vitest:audit-repo-symbol-incremental`,
4. `npm run test:vitest:function-clone-audit-forwarding`,
5. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve the
current Node assertions as named Vitest cases. It should run both preserved
Node commands, both focused Vitest commands, and `npm run test:vitest`.
