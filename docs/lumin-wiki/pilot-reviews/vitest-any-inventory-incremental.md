# Vitest Any Inventory Incremental Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-20.
> **Pilot candidate:** `tests/test-any-inventory-incremental.mjs`.

---

## Purpose

This review dogfoods the parked performance/incremental lane before a focused
Vitest mirror. The suite protects strict incremental cache identity for
`any-inventory.mjs`, including changed, deleted, scan-range, and malformed-cache
cases that are easy to hide behind broad temporary-repo helpers.

The key risk is that a mirror could preserve only cold/warm equivalence while
dropping the type-escape inventory contracts that prove changed files refresh,
deleted files disappear, production-only caches do not leak into default runs,
and unrelated malformed cache payloads do not crash the producer.

## Reviewed Evidence

| Suite                                      | Preserved Node Command                          | Proposed Focused Vitest Command                 | Surface Under Review                            |
| ------------------------------------------ | ----------------------------------------------- | ----------------------------------------------- | ----------------------------------------------- |
| `tests/test-any-inventory-incremental.mjs` | `node tests/test-any-inventory-incremental.mjs` | `npm run test:vitest:any-inventory-incremental` | any-inventory strict incremental cache identity |

Goal lane: Lane F, performance/incremental. This is a suite-specific review for
one parked cache-identity suite, not permission to migrate the whole parked
performance lane.

Fresh preserved-command evidence on 2026-05-20:

```text
node tests/test-any-inventory-incremental.mjs
13 passed, 0 failed
```

## Result

This suite has a focused Vitest mirror in
`tests/any-inventory-incremental.test.mjs`, and the mirror stays local to
`any-inventory.mjs` without absorbing classify-performance, symbol-graph,
shape-index, or function-clone incremental behavior.

The future mirror may share setup-only helpers for temporary repo creation,
source writing, command execution, JSON artifact reads, and stable public-fact
normalization. It must not extract helper logic that decides type escape
meaning, scan-range cache validity, dropped-file evidence, malformed cache
tolerance, or incremental reuse semantics.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- first incremental output is equivalent to a cold public artifact after stable
  public-fact normalization;
- a warm run is equivalent to the cold public artifact after the same stable
  normalization;
- warm runs report incremental mode as enabled;
- warm runs reuse at least one unchanged file fact;
- changing one source file updates that file's type escape facts;
- unchanged file facts remain present after an incremental edit;
- changed-file runs report a positive changed-file count;
- deleted file facts disappear from the public artifact;
- deleted-file runs increment dropped-file evidence;
- changing scan options from production-only back to the default keeps the
  public artifact correct and includes test file facts;
- scan option changes do not reuse stale production-only cache payloads;
- malformed unrelated cache payloads do not crash the producer;
- `--no-incremental` reports disabled cache metadata with
  `reason: "disabled-by-flag"`.

## Edge-Case Failures To Preserve

The mirror must fail if:

- stable public-fact comparison includes volatile metadata and hides actual
  inventory drift;
- warm runs stop reporting incremental mode or reuse;
- changed files leave stale type escape facts;
- unchanged file facts disappear after an incremental edit;
- deleted-file facts remain in `any-inventory.json`;
- deleted-file runs do not increment dropped-file evidence;
- production-only cache entries are reused after the scan range changes back to
  the default include-tests behavior;
- malformed unrelated cache payloads crash the producer or hide current facts;
- `--no-incremental` silently enables cache behavior.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- write small TypeScript fixtures;
- run the real `any-inventory.mjs` command with explicit stdout and stderr
  capture;
- read `any-inventory.json`;
- compare stable public inventory fields after removing volatile metadata;
- create unrelated malformed cache files for tolerance checks.

Forbidden helper behavior:

- deciding whether `as-any` or `as-unknown-as-T` evidence should be present;
- hiding changed, reused, invalidated, or dropped counters behind broad
  booleans;
- deciding scan-range or `includeTests` semantics;
- swallowing malformed cache failures;
- rewriting inventory paths, type escape kinds, or incremental metadata;
- sharing fixture semantics with symbol-graph, shape-index, function-clone, or
  classify-performance incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb
  `tests/test-classify-performance-metadata.mjs`,
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-shape-index-incremental.mjs`, or
  `tests/test-function-clone-incremental.mjs`.
- The mirror must not change cache policy, any-inventory schema, type escape
  extraction, scan-range behavior, artifact renderer behavior, deadness/ranking,
  or producer orchestration.

## Recommendation

The narrow implementation PR adds:

1. `tests/any-inventory-incremental.test.mjs`;
2. `npm run test:vitest:any-inventory-incremental`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.
