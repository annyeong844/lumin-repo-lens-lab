# Vitest Shape Index Incremental Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-20.
> **Pilot candidate:** `tests/test-shape-index-incremental.mjs`.

---

## Purpose

This review dogfoods the parked performance/incremental lane before a focused
Vitest mirror. The suite protects strict incremental cache identity for
`build-shape-index.mjs`; it is not just a happy-path shape artifact smoke test.

The key risk is that a broad Vitest helper could hide whether shape facts were
fresh, reused, changed, dropped, or routed through `audit-repo.mjs` with the
right incremental flags. Any future mirror must keep those cache assertions
visible as local edge-case tests.

## Reviewed Evidence

| Suite                                    | Preserved Node Command                        | Proposed Focused Vitest Command               | Surface Under Review                          |
| ---------------------------------------- | --------------------------------------------- | --------------------------------------------- | --------------------------------------------- |
| `tests/test-shape-index-incremental.mjs` | `node tests/test-shape-index-incremental.mjs` | `npm run test:vitest:shape-index-incremental` | shape-index strict incremental cache identity |

Goal lane: Lane F, performance/incremental. This is a suite-specific review for
one parked cache-identity suite, not permission to migrate the whole parked
performance lane.

Fresh preserved-command evidence on 2026-05-20:

```text
node tests/test-shape-index-incremental.mjs
12 passed, 0 failed
```

## Result

This suite is acceptable as one focused Vitest mirror, provided the mirror stays
local to `build-shape-index.mjs` and the audit-repo forwarding checks already in
the Node suite.

The future mirror may share setup-only helpers for temporary repo creation,
source writing, command execution, JSON artifact reads, and volatile run
metadata stripping. It must not extract helper logic that decides shape hash
correctness, incremental reuse, dropped-file behavior, or audit-repo
orchestration semantics.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- first incremental output is equivalent to a cold `--no-incremental` artifact
  after removing run metadata;
- a warm run is equivalent to the cold artifact after the same stable
  normalization;
- warm runs report strict incremental mode with
  `identityMode: "strict-content-hash"`;
- warm runs reuse unchanged shape facts;
- reused facts are stamped with the current artifact `observedAt`;
- changing one source file changes that file's shape hash while reusing at
  least one unchanged file;
- deleting one source file removes its shape facts and increments dropped-file
  evidence;
- `--no-incremental` reports disabled cache metadata with
  `reason: "disabled-by-flag"`;
- `audit-repo.mjs --profile full --no-incremental` forwards disabled-cache
  metadata to `build-shape-index.mjs`;
- `audit-repo.mjs --profile full --cache-root <dir>` forwards the cache root and
  permits a warm run to reuse shape facts.

## Edge-Case Failures To Preserve

The mirror must fail if:

- reused shape facts keep an old `observedAt` stamp;
- a changed file reuses a stale shape hash;
- a changed-file run reports no changed files or no reused files;
- deleted-file facts remain in `shape-index.json`;
- deleted-file runs do not increment dropped-file evidence;
- `--no-incremental` silently enables cache behavior;
- audit-repo stops forwarding `--no-incremental` to the shape producer;
- audit-repo stops forwarding `--cache-root` to the shape producer;
- stable artifact comparison accidentally includes volatile generated,
  observedAt, or incremental metadata and hides the public-facts contract.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- write small TypeScript interface/type fixtures;
- run the real `build-shape-index.mjs` command with explicit stdout and stderr
  capture;
- run the real `audit-repo.mjs --profile full` command for forwarding checks;
- read `shape-index.json`;
- remove volatile run metadata before comparing cold and incremental artifacts.

Forbidden helper behavior:

- deciding whether a file is changed, reused, or dropped;
- deciding whether two shape hashes should match;
- hiding `meta.incremental` counters behind broad booleans;
- rewriting identities, owner files, or shape hashes;
- swallowing command failures or stale artifact reads;
- sharing fixture semantics with symbol-graph, function-clone, any-inventory, or
  classify-performance incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-any-inventory-incremental.mjs`,
  `tests/test-classify-performance-metadata.mjs`, or
  `tests/test-function-clone-incremental.mjs`.
- The mirror must not change cache policy, producer output schema, shape
  hashing, shape grouping, audit-repo orchestration, or artifact renderer
  behavior.

## Recommendation

Proceed later to one narrow implementation PR that adds:

1. `tests/shape-index-incremental.test.mjs`;
2. `npm run test:vitest:shape-index-incremental`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves the
current Node assertions as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and the doc guards.
