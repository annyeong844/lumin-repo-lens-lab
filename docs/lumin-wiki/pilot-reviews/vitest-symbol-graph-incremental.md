# Vitest Symbol Graph Incremental Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-20.
> **Pilot candidate:** `tests/test-symbol-graph-incremental.mjs`.

---

## Purpose

This review dogfoods the parked performance/incremental lane before a focused
Vitest mirror. The suite protects strict incremental cache identity for
`build-symbol-graph.mjs`, including legacy cache invalidation paths that are
easy to hide in broad setup helpers.

The key risk is that a mirror could preserve only cold/warm equivalence while
dropping the cache-version and extractor-identity failures that protect CJS
surface and dynamic require evidence. Any future mirror must keep those
invalidation assertions visible as local edge-case tests.

## Reviewed Evidence

| Suite                                     | Preserved Node Command                         | Proposed Focused Vitest Command                | Surface Under Review                           |
| ----------------------------------------- | ---------------------------------------------- | ---------------------------------------------- | ---------------------------------------------- |
| `tests/test-symbol-graph-incremental.mjs` | `node tests/test-symbol-graph-incremental.mjs` | `npm run test:vitest:symbol-graph-incremental` | symbol graph strict incremental cache identity |

Goal lane: Lane F, performance/incremental. This is a suite-specific review for
one parked cache-identity suite, not permission to migrate the whole parked
performance lane.

Fresh preserved-command evidence on 2026-05-20:

```text
node tests/test-symbol-graph-incremental.mjs
13 passed, 0 failed
```

## Result

This suite has a focused Vitest mirror in
`tests/symbol-graph-incremental.test.mjs`, provided the mirror stays local to
`build-symbol-graph.mjs` and does not absorb the remaining
performance/incremental parked suites.

The future mirror may share setup-only helpers for temporary repo creation,
source writing, command execution, JSON artifact reads, and direct cache file
mutation. It must not extract helper logic that decides fan-in correctness,
CJS export surface meaning, dynamic require opacity, stale JSON require
opacity, parser identity, or incremental invalidation semantics.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- first incremental output is equivalent to a cold `--no-incremental` artifact
  after stable public-fact normalization;
- a warm run is equivalent to the cold artifact after the same stable
  normalization;
- warm runs report strict incremental mode with
  `identityMode: "strict-content-hash"`;
- warm runs reuse unchanged file facts;
- changing a consumer file updates fan-in and refreshes at least one changed
  file while reusing at least one unchanged file;
- deleting a definition file removes its `defIndex` facts and increments
  dropped-file evidence;
- `--no-incremental` reports disabled cache metadata with
  `reason: "disabled-by-flag"`;
- legacy caches without `cjsExportSurface` are invalidated and rebuild exact and
  opaque CommonJS export evidence;
- legacy caches without `cjsRequireOpacity` are invalidated and rebuild dynamic
  require opacity evidence;
- stale JSON require opacity from an older schema is invalidated and removed;
- old CJS extractor identities are invalidated so bracket member fan-in remains
  precise.

## Edge-Case Failures To Preserve

The mirror must fail if:

- stable public-fact comparison includes volatile metadata and hides actual
  fact drift;
- warm runs stop reporting strict incremental identity;
- changed consumer files leave stale fan-in;
- changed-file runs report no changed files or no reused files;
- deleted-file facts remain in `symbols.json`;
- deleted-file runs do not increment dropped-file evidence;
- `--no-incremental` silently enables cache behavior;
- old cache payloads without CJS export surface are reused;
- old cache payloads without dynamic CJS require opacity are reused;
- stale JSON package require opacity survives a current run;
- old CJS extractor identities keep stale namespace/member precision.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- write small TypeScript and JavaScript/CommonJS fixtures;
- run the real `build-symbol-graph.mjs` command with explicit stdout and stderr
  capture;
- read `symbols.json`;
- locate and mutate `symbols.cache.json` entries for legacy-cache fixtures;
- remove volatile run metadata before comparing cold and incremental artifacts.

Forbidden helper behavior:

- deciding whether fan-in should be present or absent;
- deciding whether a legacy cache entry is stale;
- hiding `meta.incremental` counters behind broad booleans;
- rewriting identities, owner files, or `fanInByIdentity`;
- swallowing command failures or stale artifact reads;
- sharing fixture semantics with shape-index, function-clone, any-inventory, or
  classify-performance incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb
  `tests/test-any-inventory-incremental.mjs`,
  `tests/test-classify-performance-metadata.mjs`,
  `tests/test-shape-index-incremental.mjs`, or
  `tests/test-function-clone-incremental.mjs`.
- The mirror must not change cache policy, producer output schema, symbol graph
  extraction, CJS extraction, fan-in logic, resolver behavior, deadness/ranking,
  or artifact renderer behavior.

## Recommendation

The narrow implementation PR adds:

1. `tests/symbol-graph-incremental.test.mjs`;
2. `npm run test:vitest:symbol-graph-incremental`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.
