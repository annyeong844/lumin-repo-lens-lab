# Vitest Import Meta Glob Diagnostics Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-import-meta-glob-diagnostics.mjs`.

---

## Purpose

This review decides whether `tests/test-import-meta-glob-diagnostics.mjs` is
ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal is
to name the dynamic-module contracts that a runner migration must preserve
before `import.meta.glob` work moves toward scan-policy-aware expansion.

This suite is analyzer-sensitive. It protects the current honest limitation:
literal `import.meta.glob("./routes/*.ts")` is recognized as an unsupported
dynamic-module surface, not silently expanded, ignored, or treated as a
concrete graph edge.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-import-meta-glob-diagnostics.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:import-meta-glob-diagnostics`.
- Producer under review:
  `skills/lumin-repo-lens-lab/_engine/producers/build-symbol-graph.mjs`.
- Diagnostics producer under review:
  `skills/lumin-repo-lens-lab/_engine/producers/build-resolver-diagnostics.mjs`.
- Companion resolver capability suite:
  `node tests/test-resolver-diagnostics-artifacts.mjs`.
- Companion unsupported-family suite:
  `node tests/test-node-imports-unsupported.mjs`.
- Dynamic module tracker item:
  `docs/spec/lumin-work-tracker.md` WT-17.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite protects one focused dynamic-module shape:

```ts
const routes = import.meta.glob("./routes/*.ts");
```

The fixture also creates `src/routes/home.ts` so the test proves the engine
does not accidentally convert a broad glob into a concrete edge just because a
matching file exists. Until scan-policy-aware glob expansion is designed, this
surface must stay diagnostic-only.

## Protected Invariants

The future Vitest pilot must preserve these dynamic-module contracts:

- `symbols.json.unresolvedInternalSpecifierRecords[]` records
  `specifier: "./routes/*.ts"`;
- the record uses `reason: "import-meta-glob-unsupported"`;
- the record uses `resolverStage: "import-meta-glob"`;
- the record uses `outputLevel: "unsupported"`;
- the record uses `unsupportedFamily: "dynamic-modules"`;
- the record preserves `affectedPackageScope: "src/routes"`;
- no `symbols.json.resolvedInternalEdges[]` entry is created for the glob;
- `resolver-diagnostics.json.unsupportedImports[]` exposes the same family,
  reason, and unsupported output level;
- `resolver-diagnostics.json.blindZones[]` keeps
  `family: "dynamic-modules"`, `blocksAbsenceClaims: true`, and
  `blockingScope: "candidate-relevant"`;
- the blind zone preserves the affected package/surface scope so absence
  claims are limited to plausible route-surface candidates, not repo-global.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not expand the literal glob into `src/routes/home.ts`.
- A helper must not drop the glob record because the source is not a normal
  import declaration.
- A helper must not treat `import.meta.glob` as an external dependency or a
  side-effect-free expression.
- The fixture must keep at least one matching route file present so "no edge"
  is a deliberate diagnostic policy rather than an artifact of no matches.
- The mirror must assert both sides of the contract: unsupported diagnostic
  exists and concrete graph edge does not exist.
- The mirror must not combine this dynamic-module fixture with Node `#imports`,
  output-to-source, generated-artifact, deadness/ranking, or performance
  fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-import-meta-glob-diagnostics.mjs` remains runnable.
- The pilot may use temporary repo fixtures, but the helper boundary is setup
  only. Dynamic-module meaning, glob shape, producer invocation, and artifact
  assertions stay local to this suite.
- The pilot must not add literal glob expansion.
- The pilot must not introduce scan-policy-aware dynamic entry edges.
- The pilot must not introduce shared unsupported-family helpers until at
  least one more dynamic/output resolver family review proves the common
  shape.
- The pilot must not migrate
  `tests/test-output-source-layout-diagnostics.mjs`, generated-artifact suites,
  deadness/ranking suites, or performance/incremental suites.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/import-meta-glob-diagnostics.test.mjs`,
2. `npm run test:vitest:import-meta-glob-diagnostics`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by
dynamic-module output lane:

- `symbols.json` unsupported record;
- no concrete graph edge;
- resolver diagnostics unsupported lane;
- blind-zone scope and affected-surface preservation.

Run both commands when changing this suite:

- `node tests/test-import-meta-glob-diagnostics.mjs`
- `npm run test:vitest:import-meta-glob-diagnostics`

Do not migrate any other resolver or dynamic-module suite as part of the
`import.meta.glob` diagnostics pilot.
