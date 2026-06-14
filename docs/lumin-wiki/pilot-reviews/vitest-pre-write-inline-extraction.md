# Vitest Pre-Write Inline Extraction Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidates:** `tests/test-inline-pattern-index.mjs` and
> `tests/test-pre-write-inline-patterns.mjs`.

---

## Purpose

This review decides whether the inline extraction evidence suites can move as
one narrow Lane C Vitest mirror batch. It does not add Vitest suites.

The batch is acceptable because both suites protect the same conservative
pre-write evidence lane:

- `build-inline-pattern-index.mjs` records repeated inline catch-block patterns
  in `inline-patterns.json`;
- `pre-write.mjs` consumes that artifact only as an agent review cue when the
  user intent includes explicit `refactorSources`;
- missing inline evidence remains unavailable evidence, not a grounded absence
  or a silent zero.

This batch must stay separate from cue-tier promotion, service-operation
sibling policy, name lookup thresholds, deadness/ranking, resolver behavior,
function clone semantics, performance/incremental cache identity, and full
audit orchestration.

## Reviewed Evidence

| Suite                                      | Preserved Node Command                          | Proposed Focused Vitest Command                 | Surface Under Review                     |
| ------------------------------------------ | ----------------------------------------------- | ----------------------------------------------- | ---------------------------------------- |
| `tests/test-inline-pattern-index.mjs`      | `node tests/test-inline-pattern-index.mjs`      | `npm run test:vitest:inline-pattern-index`      | `inline-patterns.json` producer artifact |
| `tests/test-pre-write-inline-patterns.mjs` | `node tests/test-pre-write-inline-patterns.mjs` | `npm run test:vitest:pre-write-inline-patterns` | pre-write inline extraction review cue   |

Current preserved-command evidence on 2026-05-16:

```text
node tests/test-inline-pattern-index.mjs
8 passed, 0 failed

node tests/test-pre-write-inline-patterns.mjs
6 passed, 0 failed
```

Goal lane: Lane C, pre-write lifecycle. This review covers only inline
extraction artifact evidence and its pre-write advisory consumption.

## Result

These suites are acceptable as one bounded Vitest mirror batch.

The future implementation PR may add two focused mirrors because the suites
share the same fixture boundary and evidence contract. The implementation must
mirror the existing edge-case assertions as named Vitest cases and must keep
both Node entrypoints runnable.

## Protected Invariants

The future Vitest mirrors must preserve these contracts:

- `build-inline-pattern-index.mjs` writes `inline-patterns.json`;
- `inline-patterns.json.meta.schemaVersion` remains
  `inline-patterns.v1`;
- `inline-patterns.json.meta.supports.catchBlockPatterns` remains `true`;
- `inline-patterns.json.meta.supports.statementSequencePatterns` remains
  `false` until that surface is implemented;
- inline pattern threshold metadata remains present as
  `inline-pattern-policy` / `inline-pattern-policy-v1`;
- four repeated catch-destroy blocks group into one `catch-block` review group;
- grouped occurrences cite file, start line, end line, and enclosing function;
- generic logging catches and control-flow-only catches stay out of the default
  review groups;
- group and occurrence ordering remains deterministic across repeated runs;
- cold pre-write can create the inline-pattern artifact and surface an
  `AGENT_REVIEW_CUE` in the `inline-extraction` lane;
- the cue claim remains `repeated inline statement pattern`;
- Markdown renders the review cue without claiming safe extraction, semantic
  duplicate behavior, or an automated action;
- `--no-fresh-audit` without `inline-patterns.json` records unavailable
  inline evidence;
- missing inline evidence does not invent an inline extraction review cue.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- catch-block grouping must fail if repeated catch-destroy blocks no longer
  group together;
- group metadata must fail if source ranges or enclosing functions disappear;
- noisy `console.error` or `return` catch bodies must not create default
  extraction groups;
- repeated runs must not reorder group keys or occurrences nondeterministically;
- pre-write must not emit inline extraction cues without explicit evidence;
- `--no-fresh-audit` must not hide missing `inline-patterns.json`;
- unavailable inline evidence must not look like a grounded zero;
- Markdown wording must not say or imply safe extraction, semantic duplicate
  behavior, or automatic helper creation.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is temporary filesystem roots plus real producer and
  pre-write entrypoints.
- Shared helper code may create temp roots, write repeated catch fixtures, run
  `build-inline-pattern-index.mjs` or `pre-write.mjs`, read
  `inline-patterns.json`, read `pre-write-advisory.latest.json`, and clean up.
- Shared helper code must not decide pattern grouping semantics, cue-tier
  promotion, name lookup thresholds, deadness/ranking, resolver behavior,
  function clone semantics, performance counters, cache identity, or full audit
  orchestration.
- The mirror must not absorb `tests/test-pre-write-cue-tiers.mjs`,
  `tests/test-pre-write-render.mjs`, `tests/test-pre-write-lookup-name.mjs`,
  `tests/test-pre-write-integration.mjs`, `tests/test-function-clone-*.mjs`,
  resolver suites, deadness/ranking suites, or incremental/performance suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/inline-pattern-index.test.mjs`,
2. `tests/pre-write-inline-patterns.test.mjs`,
3. `npm run test:vitest:inline-pattern-index`,
4. `npm run test:vitest:pre-write-inline-patterns`,
5. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current edge-case assertion as named Vitest cases. It should run both preserved
Node commands, both focused Vitest commands, `npm run test:vitest`, doc-script
checks, formatting checks, and `npm test` before completion.
