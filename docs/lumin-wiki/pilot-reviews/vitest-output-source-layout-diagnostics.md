# Vitest Output Source Layout Diagnostics Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-output-source-layout-diagnostics.mjs`.

---

## Purpose

This review decides whether `tests/test-output-source-layout-diagnostics.mjs`
is ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal
is to name the output-to-source mapping contracts that a runner migration must
preserve before any future resolver work attempts broader build-output to source
layout inference.

This suite is analyzer-sensitive. It protects the current honest limitation:
when package `exports` points at an unsupported build output layout, Lumin must
record a named unsupported resolver family rather than inventing a source edge
or treating the unresolved import as deadness proof.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-output-source-layout-diagnostics.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:output-source-layout-diagnostics`.
- Producer under review:
  `skills/lumin-repo-lens-lab/_engine/producers/build-symbol-graph.mjs`.
- Diagnostics producer under review:
  `skills/lumin-repo-lens-lab/_engine/producers/build-resolver-diagnostics.mjs`.
- Resolver code under review:
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-core.mjs`.
- Capability artifact code under review:
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-capabilities.mjs`.
- Companion unsupported-family suite:
  `node tests/test-node-imports-unsupported.mjs`.
- Companion dynamic-module suite:
  `node tests/test-import-meta-glob-diagnostics.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite protects one focused workspace package shape:

```json
{
  "name": "@fixture/weird",
  "exports": {
    "./*": "./compiled/*.js"
  }
}
```

The fixture intentionally provides a source file at
`packages/weird/main/foo.ts`, while the package export points at
`packages/weird/compiled/foo.js`. That non-standard source/output layout is not
supported by the resolver. The correct result is an unsupported diagnostic, not
a fake resolved edge to either path.

## Protected Invariants

The future Vitest pilot must preserve these output-to-source mapping contracts:

- direct resolver lookup for `@fixture/weird/foo` returns
  `UNRESOLVED_INTERNAL`;
- no `symbols.json.resolvedInternalEdges[]` entry is created for
  `@fixture/weird/foo`;
- `symbols.json.unresolvedInternalSpecifierRecords[]` records
  `specifier: "@fixture/weird/foo"`;
- the record uses `reason: "output-source-layout-unsupported"`;
- the record uses `resolverStage: "wildcard-alias"`;
- the record uses `outputLevel: "unsupported"`;
- the record uses `unsupportedFamily: "output-to-source-mapping"`;
- the record preserves `source: "exports"`;
- the record preserves the compiled output candidate
  `packages/weird/compiled/foo.js`;
- `resolver-diagnostics.json.unsupportedImports[]` exposes the same family,
  reason, and unsupported output level;
- `resolver-diagnostics.json.blindZones[]` keeps
  `family: "output-to-source-mapping"`, `blocksAbsenceClaims: true`,
  `blockingScope: "candidate-relevant"`, and
  `affectedPackageScope: "packages/weird"`;
- `resolver-diagnostics.json.blockedCandidateHints[]` points reviewers at the
  affected package surface and the compiled candidate path.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not map `compiled/foo.js` to `main/foo.ts` by guessing a new
  output/source pair.
- A helper must not treat the workspace import as external just because the
  output layout is unsupported.
- A helper must not create a concrete graph edge for either
  `packages/weird/compiled/foo.js` or `packages/weird/main/foo.ts`.
- The fixture must keep both the unsupported package export and the tempting
  source file present so "no edge" is a deliberate resolver policy.
- The mirror must assert both sides of the contract: unsupported diagnostic
  exists and concrete graph edge does not exist.
- The mirror must preserve candidate-scoped blind-zone behavior and blocked
  candidate hints; it must not turn this into a repo-global blocker.
- The mirror must not combine this output-layout fixture with Node `#imports`,
  `import.meta.glob`, generated-artifact, deadness/ranking, or performance
  fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-output-source-layout-diagnostics.mjs` remains runnable.
- The pilot may use temporary repo fixtures, but the helper boundary is setup
  only. Package export shape, unsupported layout meaning, direct resolver
  assertion, producer invocation, and artifact assertions stay local to this
  suite.
- The pilot must not add new output-to-source mapping heuristics.
- The pilot must not change `alias-map` output/source pairs.
- The pilot must not promote output-layout unsupported diagnostics into
  deadness or `SAFE_FIX` evidence.
- The pilot must not migrate generated-artifact suites, dynamic-module suites,
  deadness/ranking suites, or performance/incremental suites.
- Shared unsupported-family helpers remain a separate design question. This
  mirror may use setup-only temporary repo helpers, but any semantic helper that
  abstracts resolver unsupported-family assertions needs its own comparison PR.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/output-source-layout-diagnostics.test.mjs`,
2. `npm run test:vitest:output-source-layout-diagnostics`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by
output-to-source diagnostic lane:

- direct resolver result and no concrete graph edge;
- `symbols.json` unsupported record;
- resolver diagnostics unsupported lane;
- candidate-scoped blind-zone scope;
- blocked candidate hint pointing at the affected compiled output surface.

Run both commands when changing this suite:

- `node tests/test-output-source-layout-diagnostics.mjs`
- `npm run test:vitest:output-source-layout-diagnostics`

Do not migrate any other resolver or output-layout suite as part of the
output-to-source diagnostics pilot.
