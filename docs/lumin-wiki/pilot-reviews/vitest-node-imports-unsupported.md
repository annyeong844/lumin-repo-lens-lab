# Vitest Node Imports Unsupported Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-node-imports-unsupported.mjs`.

---

## Purpose

This review decided whether `tests/test-node-imports-unsupported.mjs` was ready
for a narrow Vitest mirror. The mirror now exists at
`tests/node-imports-unsupported.test.mjs`. The goal remains to name the
resolver contracts that the runner migration must preserve before any other
unsupported-family resolver tests move.

This suite is analyzer-sensitive. It protects the distinction between
unsupported Node package-local `#imports` surfaces and concrete resolved graph
edges. A migration must improve test ergonomics without weakening fake-edge
prevention, output-level labeling, or blind-zone scoping.

## Reviewed Evidence

- Preserved Node command: `node tests/test-node-imports-unsupported.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:node-imports-unsupported`.
- Resolver source under review: `_lib/resolver-core.mjs`.
- Alias map source under review: `_lib/alias-map.mjs`.
- Diagnostics producer under review: `build-resolver-diagnostics.mjs`.
- Companion resolver capability suite:
  `node tests/test-resolver-diagnostics-artifacts.mjs`.
- Companion blind-zone relevance suite:
  `node tests/test-resolver-blind-zone-relevance.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.
- Architecture guardrail:
  `docs/spec/lumin-architecture-realignment.md`.

## Result

The suite was acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror. The implementation kept that boundary: it added a
Vitest mirror for this one Node `#imports` suite while preserving the original
Node command and leaving other resolver unsupported-family suites parked.

The current Node suite protects two Node `#imports` unsupported-family shapes:

1. package-local `#app/config` with no supported imports map;
2. `#env` with an imports condition map that exposes only unsupported
   condition keys.

Both shapes must remain diagnostic-only. The resolver may record candidates,
unsupported imports, and blind zones, but it must not create a concrete graph
edge or silently fall back to external.

## Protected Invariants

The Vitest pilot must preserve these resolver contracts:

- unsupported package-local `#imports` return `UNRESOLVED_INTERNAL` from the
  direct resolver call;
- `symbols.json.unresolvedInternalSpecifierRecords[]` records
  `reason`, `resolverStage`, `outputLevel: "unsupported"`, and
  `unsupportedFamily: "node-imports"`;
- unsupported Node `#imports` create no `resolvedInternalEdges[]` entry;
- `symbols.json.uses.resolvedInternal` does not increase from unsupported
  imports;
- `resolver-diagnostics.json.unresolvedImports[]` preserves
  `family: "node-imports"`, output level, reason, and
  `createsGraphEdge: false`;
- `resolver-diagnostics.json.unsupportedImports[]` contains the unsupported
  record;
- condition-profile ambiguity preserves `targetCandidates[]` as diagnostic
  evidence only;
- blind zones keep their intended scope:
  - missing unsupported imports remain repo-confidence-limited;
  - condition-profile ambiguity remains candidate-relevant when candidates are
    known;
- unsupported-family diagnostics stay paired with the absence of fake graph
  edges.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not convert `#app/config` into an external package miss.
- A helper must not create temporary package metadata that accidentally turns
  the unsupported case into a supported `imports` map.
- The condition-profile fixture must keep unsupported condition keys such as
  `browser` and `react-native`; replacing them with supported defaults would
  stop testing ambiguity.
- The condition-profile fixture must keep both candidate files present so
  candidate preservation is tested.
- The mirror must assert both sides of the contract: unsupported diagnostic
  exists and concrete graph edge does not exist.
- The mirror must not combine Node `#imports` with output-to-source,
  import-meta-glob, generated artifacts, or broader resolver capability
  fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-node-imports-unsupported.mjs` remains runnable.
- The pilot may use temporary repo fixtures, but the helper boundary is setup
  only. Resolver meaning, package imports metadata, condition-profile shape,
  and artifact assertions stay local to this suite.
- The pilot must not introduce a shared resolver unsupported-family assertion
  helper until at least one more resolver family review proves the common
  shape.
- The pilot must not migrate
  `tests/test-output-source-layout-diagnostics.mjs`,
  `tests/test-import-meta-glob-diagnostics.mjs`, generated-artifact suites,
  deadness/ranking suites, or performance/incremental suites.
- The pilot must not relax unsupported output-level assertions into loose
  presence checks.

## Recommendation

The narrow implementation PR added:

1. `tests/node-imports-unsupported.test.mjs`,
2. `npm run test:vitest:node-imports-unsupported`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation keeps the Node suite and represents the current contract as
named Vitest `describe(...)` / `it(...)` blocks grouped by resolver output lane:

- direct resolver result;
- `symbols.json` unsupported record;
- no concrete graph edge;
- resolver diagnostics unresolved/unsupported lanes;
- blind-zone scope and target-candidate preservation.

Run both commands when changing this suite:

- `node tests/test-node-imports-unsupported.mjs`
- `npm run test:vitest:node-imports-unsupported`

Do not migrate any other resolver unsupported-family suite as part of the Node
`#imports` pilot.
