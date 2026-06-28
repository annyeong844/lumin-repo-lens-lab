# Vitest Incremental Core Helpers Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidates:** `tests/test-incremental-cache-store.mjs`,
> `tests/test-incremental-snapshot.mjs`, `tests/test-incremental.mjs`.

---

## Purpose

This review decides whether three small incremental core helper suites can move
together as one Vitest mirror batch. It does not add a Vitest suite. The goal
is to preserve the shared cache-store, repo-snapshot, and legacy file-hash
contracts that producer-level incremental suites depend on.

The suites are acceptable as a paired helper batch because they test pure
library boundaries and temporary filesystem fixtures. They do not run the full
audit pipeline and do not decide producer-specific reuse semantics. Producer
incremental suites, scanner fast paths, and performance counters remain parked
until their own review pages name their cache identity and cold/warm fixture
boundaries.

## Reviewed Evidence

| Suite                                    | Preserved Node Command                        | Proposed Focused Vitest Command                | Surface Under Review                   |
| ---------------------------------------- | --------------------------------------------- | ---------------------------------------------- | -------------------------------------- |
| `tests/test-incremental-cache-store.mjs` | `node tests/test-incremental-cache-store.mjs` | `npm run test:vitest:incremental-cache-store`  | strict shared producer cache store     |
| `tests/test-incremental-snapshot.mjs`    | `node tests/test-incremental-snapshot.mjs`    | `npm run test:vitest:incremental-snapshot`     | repo-relative snapshot identity        |
| `tests/test-incremental.mjs`             | `node tests/test-incremental.mjs`             | `npm run test:vitest:incremental-legacy-cache` | legacy file-hash/stat-first-cut helper |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane F, performance/incremental/scanner, limited to pure helper
contracts.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-incremental-cache-store.mjs
7 passed, 0 failed

node tests/test-incremental-snapshot.mjs
8 passed, 0 failed

node tests/test-incremental.mjs
13 passed, 0 failed
```

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the helper assertions without
changing cache identity, snapshot scanning, stat-first-cut behavior, producer
incremental metadata, or full audit orchestration. The Node entrypoints must
remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- default strict cache roots are stable `.audit/.cache` siblings even when the
  audit output lives under a run directory;
- reusable facts require matching current content hash, producer metadata,
  context fingerprint, and readable current files;
- read-error entries never become clean cache hits;
- malformed producer caches load as empty with an explicit ignored-malformed
  reason;
- producer cache saves write parseable schema-versioned cache files and cache
  clearing removes repo cache entries;
- snapshot repo paths are POSIX repo-relative paths;
- snapshots respect `includeTests=false`;
- snapshot entries carry language, test-like status, nearest package scope,
  content hash, and context fingerprint;
- `hashBytes()` remains deterministic and `sha256:` prefixed;
- unreadable files remain visible as read-error entries when the platform
  honors permissions;
- legacy `pickChangedFiles()` treats first runs as changed, second unchanged
  runs as cached, content edits as changed, missing files as dropped, and stale
  cache versions as invalid;
- legacy stat-first-cut preserves a planted hash when `mtimeMs` and size match,
  proving the hash path was skipped;
- legacy cache banners keep producer name, changed/cached/dropped counts, and
  reuse percentage visible.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- a content-hash mismatch must not become a reusable fact;
- unreadable current files must not reuse stale clean facts;
- malformed cache JSON must not crash the helper or silently appear valid;
- cache clearing must not leave stale producer files behind;
- `includeTests=false` must not leak test files into the snapshot;
- nearest package scope must not collapse nested packages to the root;
- platform-specific unreadable-file behavior must stay explicit rather than
  pretending every platform honors `chmod`;
- stat-first-cut must skip hashing only when both `mtimeMs` and size match;
- touching a file without byte changes must recompute and preserve the content
  hash as unchanged;
- obsolete legacy cache versions must reset to empty instead of partially
  reusing stale entries.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is temporary filesystem roots plus the real
  `_lib/incremental-cache-store.mjs`, `_lib/incremental-snapshot.mjs`, and
  `_lib/incremental.mjs` helpers.
- A future mirror may use setup-only temp helpers for directory creation,
  source writing, JSON reads, permission restore, and cleanup.
- Shared helper code must not decide producer reuse, symbol graph extraction,
  function clone grouping, shape facts, scanner fallback, or performance
  meaning.
- The mirror must not absorb producer-level incremental suites such as
  `tests/test-any-inventory-incremental.mjs`,
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-function-clone-incremental.mjs`, or
  `tests/test-shape-index-incremental.mjs`.
- The mirror must not change resolver behavior, deadness/ranking,
  generated-artifact policy, cache identity, performance counters, scanner
  behavior, or public package behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/incremental-cache-store.test.mjs`,
2. `tests/incremental-snapshot.test.mjs`,
3. `tests/incremental-legacy-cache.test.mjs`,
4. `npm run test:vitest:incremental-cache-store`,
5. `npm run test:vitest:incremental-snapshot`,
6. `npm run test:vitest:incremental-legacy-cache`,
7. candidate-board updates moving all three suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve the
current Node assertions as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.
