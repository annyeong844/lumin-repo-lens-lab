# Vitest Function Clone Incremental Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-19.
> **Pilot candidate:** `tests/test-function-clone-incremental.mjs`.

---

## Purpose

This review dogfoods the parked performance/incremental lane before the focused
Vitest mirror. The suite protects strict incremental cache identity for
`build-function-clone-index.mjs`; it is not just a happy-path function clone
artifact test.

The key risk is that a broad Vitest helper could hide whether facts were fresh,
reused, changed, dropped, or rebuilt into global clone groups. Any future mirror
must keep those cache and grouping assertions visible as local edge-case tests.

## Reviewed Evidence

| Suite                                       | Preserved Node Command                           | Proposed Focused Vitest Command                  | Surface Under Review                             |
| ------------------------------------------- | ------------------------------------------------ | ------------------------------------------------ | ------------------------------------------------ |
| `tests/test-function-clone-incremental.mjs` | `node tests/test-function-clone-incremental.mjs` | `npm run test:vitest:function-clone-incremental` | function clone strict incremental cache identity |

Goal lane: Lane F, performance/incremental. This is a suite-specific review
for one parked cache-identity suite, not permission to migrate the whole parked
performance lane.

Fresh preserved-command evidence on 2026-05-19:

```text
node tests/test-function-clone-incremental.mjs
15 passed, 0 failed
```

## Result

This suite has a narrow Vitest mirror in
`tests/function-clone-incremental.test.mjs`. The mirror preserves the Node
command and does not extract helper logic that decides function clone cache
correctness or global clone grouping.

The mirror stays local to:

- temporary repository setup;
- direct `build-function-clone-index.mjs` execution;
- `function-clones.json` reads;
- stable run-metadata stripping for cold/warm artifact comparison.

It must not alter producer behavior, cache-store semantics, hash identity,
function clone grouping, or audit-repo orchestration.

## Protected Invariants

The mirror preserves these contracts:

- first incremental output is equivalent to a cold `--no-incremental` artifact
  after removing run metadata;
- a warm run reports strict incremental mode with
  `identityMode: "strict-content-hash"`;
- warm runs reuse unchanged file payloads while stamping reused facts with the
  current artifact `observedAt`;
- changing one file refreshes that file's function clone fact, reuses unchanged
  files, and does not count the changed file as dropped;
- changed incremental output is equivalent to a fresh cold artifact after the
  same source change;
- global `exactBodyGroups[]` rebuild from mixed fresh and reused facts;
- deleting a file removes its function clone facts and increments dropped-file
  evidence;
- moving a file with identical content is treated as changed under relPath
  identity and produces a new identity;
- `--clear-incremental-cache` clears reuse before the next run;
- `--no-incremental` reports disabled cache metadata with
  `reason: "disabled-by-flag"`.

## Edge-Case Failures To Preserve

The mirror must fail if:

- reused facts keep an old `observedAt` stamp;
- a changed file reuses a stale `exactBodyHash`;
- the changed-file path is misreported as dropped;
- `exactBodyGroups[]` fails to include a new exact clone produced from one fresh
  file and one reused file;
- deleted-file facts remain in `function-clones.json`;
- a same-content move is treated as unchanged because only content hash was
  considered;
- `--clear-incremental-cache` leaves reused-file counts above zero;
- `--no-incremental` silently enables cache behavior.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- write fixture source files;
- run the real `build-function-clone-index.mjs` command with explicit stdout and
  stderr capture;
- read `function-clones.json`;
- remove volatile run metadata before comparing cold and incremental artifacts.

Forbidden helper behavior:

- deciding whether a file is changed, reused, dropped, or moved;
- deciding whether clone groups are correct;
- hiding `meta.incremental` counters behind broad booleans;
- rewriting identities or owner files;
- swallowing command failures or stale artifact reads;
- sharing fixture semantics with shape-index or symbol-graph incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb
  `tests/test-shape-index-incremental.mjs`,
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-any-inventory-incremental.mjs`, or
  `tests/test-classify-performance-metadata.mjs`.
- The mirror must not change cache policy, producer output schema, function
  clone fingerprinting, grouping thresholds, or artifact renderer behavior.

## Recommendation

Keep this mirror narrow. `node tests/test-function-clone-incremental.mjs`
remains the authoritative preserved command until a later cleanup spec retires
the Node entrypoint.
