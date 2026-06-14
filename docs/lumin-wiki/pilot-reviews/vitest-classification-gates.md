# Vitest Classification Gates Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-classification-gates.mjs`

---

## Purpose

This review decides whether `tests/test-classification-gates.mjs` may move to a
focused Vitest mirror. It does not add a Vitest suite.

The suite is a large but cohesive Lane B canon contract. It pins
`canonical/classification-gates.md`, `canonical/canon-drift.md`, and the
canon-draft classifier exports against each other. It does not run the audit
pipeline, rank fixes, resolve imports, or classify dead exports. The mirror is
acceptable as one focused suite if it keeps the existing classifier matrix
local and does not turn the matrix into shared semantic helpers.

## Reviewed Evidence

| Suite                                 | Preserved Node Command                     | Proposed Focused Vitest Command            | Surface Under Review                       |
| ------------------------------------- | ------------------------------------------ | ------------------------------------------ | ------------------------------------------ |
| `tests/test-classification-gates.mjs` | `node tests/test-classification-gates.mjs` | `npm run test:vitest:classification-gates` | canonical classifier and drift gate matrix |

Current Node evidence checked for this review:

```text
node tests/test-classification-gates.mjs # 105 passed, 0 failed
```

Goal lane: Lane B, canon/check-canon. This review covers the unit-style
classifier matrix that the classification label emission corpus intentionally
does not absorb.

## Result

This suite is acceptable as one focused Vitest mirror.

The future implementation PR may add one mirror file and one focused script,
provided it preserves the Node entrypoint and keeps the matrix sections visible
as separate `describe` or `it` groups. The implementation must not change the
canonical docs, classifier predicates, or canon-drift parser contract.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- canonical `LOW_INFO_NAMES` parses from §3, has exactly 16 entries, and stays
  byte-equal to the frozen exported mirror;
- canonical type labels parse from §9, contain exactly 9 labels, and bound all
  emitted type classifier labels;
- type classification does not key identity maps on `typeName`;
- canonical `LOW_INFO_HELPER_NAMES` parses from §10.4, has exactly 15 entries,
  and stays byte-equal to the frozen exported mirror;
- canonical helper labels parse from §9, contain exactly 9 labels, and bound all
  emitted helper classifier labels;
- helper contamination-unavailable states do not accidentally emit
  `ANY_COLLISION_HELPER` or `severely-any-contaminated-helper`;
- helper rule precedence stays pinned for low-info names, central helpers, and
  fan-in thresholds;
- helper classification does not key identity maps on `helperName` or
  `calleeName`, and does not treat `topCallees.count` as fan-in;
- topology labels parse from the canonical section, stay frozen in the exported
  mirror, and bound emitted topology classifier labels;
- topology rule precedence distinguishes cyclic, isolated, scoped, oversize,
  and extreme-oversize cases;
- topology uncertainty reasons remain separate from helper uncertainty reasons;
- topology classification does not derive in/out degree from
  `crossSubmoduleTop`;
- naming labels, conventions, and uncertainty reasons stay frozen and
  byte-equal to their canonical sections;
- naming cohort and item classifiers preserve low-info exclusion, convention
  detection, basename normalization, and insufficient-evidence behavior;
- naming cohorts are not keyed by `ownerFile::exportedName`;
- production code imports canon-draft leaf modules instead of the facade;
- `canonical/canon-drift.md` keeps its purpose, drift kind, category, identity,
  parser, JSON shape, and non-goal sections;
- canon-drift categories, families, per-source statuses, and `drifts[]` shape
  stay aligned with the expected contract.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- changing canonical label order or count without updating the mirror must fail;
- widening emitted labels beyond the canonical sets must fail;
- using low-info names as identity keys must fail;
- treating unavailable contamination evidence as actual contamination must fail;
- letting lower-priority helper/topology/naming rules override documented
  precedence must fail;
- collapsing helper and topology uncertainty enums together must fail;
- deriving topology evidence from summary/top lists instead of the full source
  fields must fail;
- failing to strip `.test`, `.stories`, `.d`, or source extensions from naming
  basenames must fail;
- importing classifier implementation through the `canon-draft.mjs` facade must
  fail;
- changing canon-drift category, family, status, or artifact shape without
  updating the parser contract must fail.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The Node suite remains runnable and authoritative until a later cleanup spec
  retires it.
- The fixture boundary is canonical Markdown text, canon-draft leaf exports,
  direct classifier calls, and source-grep guards.
- Shared setup may read Markdown files, import classifier modules, and create
  small sample groups.
- Shared helpers must not decide labels, rule precedence, contamination
  semantics, topology degree semantics, naming convention semantics, or
  canon-drift category/family meaning.
- The mirror must not run the audit pipeline, symbol graph, dead-export
  classification, resolver, rank-fixes, performance, generated-artifact, public
  package, or action-safety code paths.
- The mirror must not absorb the already mirrored
  `tests/test-classification-label-emission-corpus.mjs` suite.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/classification-gates.test.mjs`,
2. `npm run test:vitest:classification-gates`,
3. candidate-board updates moving `tests/test-classification-gates.mjs` from
   `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror with named cases for
the current type, helper, topology, naming, facade, and canon-drift sections.
It should run the preserved Node command, the focused Vitest command, the wiki
guards, `npm run test:vitest`, and `npm test`.
