# Vitest Classification Label Emission Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-classification-label-emission-corpus.mjs`.

---

## Purpose

This review decides whether `tests/test-classification-label-emission-corpus.mjs`
can move as one narrow Lane B Vitest mirror batch. It does not add a Vitest
suite. The goal is to preserve the end-to-end canon label emission contract:
tiny TS fixture repo, `build-symbol-graph.mjs`, `generate-canon-draft.mjs`, and
the emitted `canonical-draft/type-ownership.md` table.

The suite is acceptable as a single-suite batch because it does not test the
full classifier matrix directly. Unit-style classifier predicate coverage stays
in `tests/test-classification-gates.mjs`. This suite proves that the public
producer path can still surface the canonical labels in real artifacts.

## Reviewed Evidence

| Suite                                                 | Preserved Node Command                                     | Proposed Focused Vitest Command                     | Surface Under Review                                         |
| ----------------------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------- | ------------------------------------------------------------ |
| `tests/test-classification-label-emission-corpus.mjs` | `node tests/test-classification-label-emission-corpus.mjs` | `npm run test:vitest:classification-label-emission` | generated type-ownership Markdown label emission from corpus |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane B, canon/check-canon artifact emission guard.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-classification-label-emission-corpus.mjs
15 passed, 0 failed
```

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the existing corpus and Markdown row
assertions without changing symbol graph extraction, canon draft generation,
type-ownership table schema, classifier thresholds, label selection, or
Markdown renderer behavior. The Node entrypoint must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `build-symbol-graph.mjs` declares `meta.supports.anyContamination === true`
  for the corpus fixture;
- `generate-canon-draft.mjs --source type-ownership` writes a parseable
  `canonical-draft/type-ownership.md` table;
- duplicate `Result` type owners with real import fan-in emit
  `DUPLICATE_STRONG`;
- the `Result` rows carry fan-in grounded in real consumer files;
- low-info local `Props` owners with low fan-in emit `LOCAL_COMMON_NAME`;
- duplicate non-low-info `Envelope` owners with low fan-in emit
  `DUPLICATE_REVIEW`;
- contaminated same-name `Opaque` owners emit `ANY_COLLISION`;
- `ANY_COLLISION` rows carry contamination tags from real `any` type escapes;
- a single-owner `Session` type with three real consumers emits
  `single-owner-strong`;
- the `Session` row fan-in remains grounded in exactly three consumer files.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- dropping `anyContamination` support from `symbols.json.meta.supports` must
  fail;
- failing to write or parse the type-ownership Markdown table must fail;
- low-info name handling must not override high fan-in duplicate strength;
- low-info local names must not become duplicate-review labels when fan-in is
  low;
- duplicate non-low-info names must not be silently treated as local/common;
- severely contaminated same-name owners must not lose `ANY_COLLISION`;
- contamination tags must not disappear from emitted Markdown rows;
- single-owner fan-in must not be inferred from constants or fixture shape
  rather than real consumer imports.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is the current synthetic TS corpus plus the real symbol
  graph and canon draft CLIs.
- A future mirror may use setup-only temp helpers, but helper code must not
  decide canonical labels, fan-in, contamination, Markdown row parsing, or
  type-ownership meaning.
- The mirror must not absorb the larger
  `tests/test-classification-gates.mjs` classifier matrix.
- The mirror must not change classifier predicates, canon markdown schemas,
  symbol graph extraction, any-contamination evidence, deadness/ranking,
  resolver behavior, performance, or public package behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/classification-label-emission-corpus.test.mjs`,
2. `npm run test:vitest:classification-label-emission`,
3. candidate-board updates moving
   `tests/test-classification-label-emission-corpus.mjs` from `REVIEWED` to
   `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves the
current Node assertions as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
