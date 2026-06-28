# Vitest Classify Performance Metadata Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-21.
> **Pilot candidate:** `tests/test-classify-performance-metadata.mjs`.

---

## Purpose

This review dogfoods the last parked performance/incremental suite before a
focused Vitest mirror. The suite protects `classify-dead-exports.mjs`
performance metadata and bounded-work degradation behavior; it is not a generic
dead-export classification fixture.

The key risk is that a broad mirror could preserve only "classification runs"
while dropping the counters that prove batching, text-zero shortcuts,
candidate-limit incompleteness, time-budget degradation, file-size degradation,
and per-file provenance caching still remain visible in `dead-classify.json`.

## Reviewed Evidence

| Suite                                          | Preserved Node Command                              | Proposed Focused Vitest Command                     | Surface Under Review                                              |
| ---------------------------------------------- | --------------------------------------------------- | --------------------------------------------------- | ----------------------------------------------------------------- |
| `tests/test-classify-performance-metadata.mjs` | `node tests/test-classify-performance-metadata.mjs` | `npm run test:vitest:classify-performance-metadata` | classify-dead-exports performance metadata and degradation guards |

Goal lane: Lane F, performance/incremental. This is a suite-specific review for
one parked performance metadata suite, not permission to migrate the whole
deadness/ranking lane.

Fresh preserved-command evidence on 2026-05-21:

```text
node tests/test-classify-performance-metadata.mjs
13 passed, 0 failed
```

## Result

This suite has a focused Vitest mirror in
`tests/classify-performance-metadata.test.mjs`, and the mirror stays local to
`build-symbol-graph.mjs` plus `classify-dead-exports.mjs` fixture orchestration
without absorbing broader corpus, ranking, action-safety, or P6 calibration
behavior.

The future mirror may share setup-only helpers for temporary repo creation,
source writing, command execution, and artifact reads. It must not extract
helper logic that decides whether a proposal is complete, incomplete, degraded,
AST-counted, text-zero, or provenance-cacheable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `dead-classify.json.summary.performance` exists and remains structured;
- processed dead candidate counts match the fixture's total candidate surface;
- same-file candidates are AST-counted through one file batch rather than one
  parse per symbol;
- candidate limits are absent by default;
- file-size degradation is opt-in and disabled by default;
- text-zero candidates skip AST work without degrading accuracy;
- provenance work is cached per file rather than repeated per symbol;
- all-text-zero batches can complete without parsing candidate files;
- `--classify-candidate-limit` marks the artifact incomplete;
- candidate-limit runs record total versus processed candidate counts;
- `--classify-time-budget-ms` marks the artifact incomplete;
- time-budgeted candidates are materialized as
  `proposal_DEGRADED_unprocessed` evidence;
- `--classify-max-file-bytes` degrades oversized candidate files instead of
  AST-counting them.

## Edge-Case Failures To Preserve

The mirror must fail if:

- performance metadata disappears from the classify summary;
- dead candidate counts drift while proposals still render;
- a same-file batch parses once per symbol instead of once per file;
- default runs silently apply candidate caps or file-size caps;
- text-zero candidates start forcing AST parses;
- provenance cache entries collapse to a broad boolean;
- all-text-zero fixtures parse candidate files unnecessarily;
- candidate-limit or time-budget runs stay marked complete;
- degraded unprocessed proposals disappear under candidate limits, time
  budgets, or file-size caps;
- oversized candidate files are treated as complete AST-counted evidence.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- write small TypeScript fixtures;
- run the real `build-symbol-graph.mjs` command;
- run the real `classify-dead-exports.mjs` command with explicit CLI flags;
- read `dead-classify.json`;
- assert metadata counters and degraded proposal arrays.

Forbidden helper behavior:

- deciding whether a dead candidate should be complete or degraded;
- hiding `summary.performance` counters behind broad booleans;
- deciding whether text-zero evidence is accurate;
- deciding whether a time budget or file-size cap should mark the artifact
  incomplete;
- swallowing command failures or missing artifacts;
- sharing fixture semantics with any-inventory, symbol-graph, shape-index,
  function-clone, corpus, rank-fixes, export-action-safety, or P6 calibration
  suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- The preserved Node command remains runnable.
- The mirror must not absorb
  `tests/test-any-inventory-incremental.mjs`,
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-shape-index-incremental.mjs`, or
  `tests/test-function-clone-incremental.mjs`.
- The mirror must not change classify policy, dead-export ranking, SAFE_FIX
  action proof, corpus budgets, P6 calibration, artifact rendering, or
  performance counter semantics.

## Recommendation

The narrow implementation PR adds:

1. `tests/classify-performance-metadata.test.mjs`;
2. `npm run test:vitest:classify-performance-metadata`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.
