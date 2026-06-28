# Vitest Vocab Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-vocab.mjs`.

---

## Purpose

This review decides whether `tests/test-vocab.mjs` is a reasonable Lane A
low-risk core/parser/helper Vitest pilot candidate. It does not add the Vitest
suite. The goal is to preserve the stringly-typed vocabulary contracts that
cross artifact producers, rankers, post-write delta rendering, and downstream
tooling.

The suite is a good next candidate because it imports small pure modules,
creates no repository fixtures, does not run producers, and pins exactly the
kind of drift that can silently corrupt evidence interpretation: renamed
evidence labels, taint labels, delta labels, severity group membership, frozen
constant objects, and provenance forwarding shape.

## Reviewed Evidence

- Preserved Node command: `node tests/test-vocab.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:vocab`.
- Helper modules under review:
  - `_lib/vocab.mjs`,
  - `_lib/post-write-delta.mjs` only for
    `requiredAcknowledgements(...)` integration with `DELTA_LABELS`.
- Current suite description: `tests/README.md`.
- Goal lane: Lane A, low-risk core/parser/helper.

## Result

The suite is acceptable as the next narrow Vitest pilot candidate.

The future mirror should keep this as a vocabulary-contract suite, not a
classifier, ranking, deadness, post-write, or renderer behavior suite. The old
Node entrypoint must remain runnable, and the Vitest mirror should express each
current vocabulary pin as named assertions grouped by contract family.

## Protected Invariants

The future Vitest pilot must preserve these vocabulary contracts:

- `EVIDENCE` literal labels stay stable:
  - `ast-ident-ref-count`,
  - `text-zero-ident-ref-count`,
  - `regex-text-fallback-parse-error`,
  - `parse-error`;
- `EVIDENCE_VALUES` mirrors every `EVIDENCE` value exactly;
- `TAINT` literal labels stay stable for unresolved specifier matches,
  resolver blind zones, generated artifact relevance, defining-file parse
  errors, and parse errors elsewhere;
- `BLOCKING_TAINTS` and `SOFT_TAINTS` preserve their current severity
  membership without overlap;
- every `TAINT` value appears in either the blocking or soft severity set;
- `EVIDENCE`, `TAINT`, `BLOCKING_TAINTS`, and `SOFT_TAINTS` remain frozen;
- `provenanceFields(...)` forwards only the known provenance fields and does
  not leak unrelated candidate fields such as `symbol`, `file`, or `line`;
- `provenanceFields(...)` omits keys whose value is `undefined`;
- `getProvenanceFieldNames()` returns a fresh copy, not the mutable internal
  list;
- `DELTA_LABELS` remains the canonical six-label union:
  `planned`, `planned-not-observed`, `silent-new`, `pre-existing`, `removed`,
  and `observed-unbaselined`;
- `DELTA_LABEL_VALUES` mirrors `DELTA_LABELS` exactly;
- `requiredAcknowledgements(...)` continues to acknowledge only
  `DELTA_LABELS.SILENT_NEW`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- renaming a public evidence, taint, or delta label must fail the focused suite;
- adding a new taint without severity membership must fail;
- accidentally overlapping blocking and soft taint groups must fail;
- replacing frozen vocab exports with mutable objects must fail;
- leaking unrelated classified-candidate fields through `provenanceFields(...)`
  must fail;
- forwarding `undefined` as an own key must fail;
- returning the internal provenance field list by reference must fail;
- adding a new delta label without updating the canonical six-label contract
  and acknowledgement behavior must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-vocab.mjs` remains runnable.
- The fixture boundary is direct module import only; no temp repo helper is
  needed.
- The pilot must not change vocab literal values, severity grouping,
  provenance-forwarding semantics, or post-write acknowledgement semantics.
- The pilot must not absorb classifier, deadness, ranking, renderer,
  post-write delta matching, resolver, or producer suites.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/vocab.test.mjs`,
2. `npm run test:vitest:vocab`,
3. a candidate-board update moving this suite from `REVIEWED` to `DONE`.

The implementation PR should keep every current Node assertion represented as
a named Vitest assertion. It may group assertions by evidence labels, taint
labels, provenance forwarding, and delta labels, but vocabulary meaning must
stay local to this suite.

Run both commands when changing this suite:

- `node tests/test-vocab.mjs`
- `npm run test:vitest:vocab`

Do not migrate `test-collect.mjs`, `test-shape-hash.mjs`, classifier suites,
deadness/ranking suites, post-write matching suites, resolver suites,
performance/incremental suites, or renderer suites as part of the vocab pilot.
