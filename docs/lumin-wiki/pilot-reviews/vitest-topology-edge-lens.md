# Vitest Topology Edge Lens Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:** `tests/test-dynamic-import.mjs`,
> `tests/test-type-only-reexport.mjs`,
> `tests/test-topology-producer-cross-edges.mjs`.

---

## Purpose

This review decides whether three topology edge-lens suites can move as one
narrow Vitest mirror batch. It does not add Vitest suites. The goal is to
preserve `measure-topology.mjs` graph-edge evidence without turning the mirror
into resolver expansion, deadness/ranking, full audit orchestration, or broad
performance/incremental cache coverage.

The candidates are acceptable as one batch because all three suites run
controlled fixture repos through `measure-topology.mjs` and inspect topology
artifact semantics:

- `tests/test-dynamic-import.mjs` validates static plus literal dynamic import
  edge discovery and scanner fallback counters.
- `tests/test-type-only-reexport.mjs` validates that type-only re-export cycles
  are excluded from the runtime SCC lens while mixed/runtime cycles survive.
- `tests/test-topology-producer-cross-edges.mjs` validates the producer-level
  `crossSubmoduleEdges` full-list artifact shape and its legacy display-slice
  companion.

The future mirrors should keep those contracts local. They must not expand into
dead-export classification, action-safety proof, resolver unsupported-family
diagnostics, full audit orchestration, or cache identity beyond the counters
already asserted by the Node suites.

## Reviewed Evidence

| Suite                                          | Preserved Node Command                              | Proposed Focused Vitest Command                     | Surface Under Review                                       |
| ---------------------------------------------- | --------------------------------------------------- | --------------------------------------------------- | ---------------------------------------------------------- |
| `tests/test-dynamic-import.mjs`                | `node tests/test-dynamic-import.mjs`                | `npm run test:vitest:dynamic-import`                | topology literal dynamic import edge discovery             |
| `tests/test-type-only-reexport.mjs`            | `node tests/test-type-only-reexport.mjs`            | `npm run test:vitest:type-only-reexport`            | runtime SCC lens filtering of type-only re-export cycles   |
| `tests/test-topology-producer-cross-edges.mjs` | `node tests/test-topology-producer-cross-edges.mjs` | `npm run test:vitest:topology-producer-cross-edges` | full cross-submodule edge artifact shape and display slice |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane F/H boundary, topology edge-lens evidence guard.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should preserve the same temporary-fixture
subprocess boundary and artifact assertions without changing
`measure-topology.mjs`, runtime/type-only edge semantics, scanner fallback
counter semantics, SCC lens behavior, or `crossSubmoduleEdges` artifact shape.
The Node entrypoints must remain runnable.

## Protected Invariants

The future Vitest mirrors must preserve these contracts:

- literal dynamic imports in `await import(...)`, conditional branches,
  `.then(...)` chains, and object-literal callbacks count as internal topology
  edges;
- regular static imports still contribute to the same topology edge summary;
- unsupported `require(...)` in the dynamic-import fixture falls back to the Oxc
  path and is counted as a scanner fallback, not silently accepted by the fast
  scanner;
- topology performance counters keep `scannerPolicyVersion`,
  `scannerFilesAttempted`, `scannerAcceptedFiles`, `scannerFallbackFiles`,
  `scannerRiskCounts`, `oxcParseCalls`, `resolverMemoHits`,
  `resolverMemoMisses`, and `resolverMemoSize`;
- `topology.json.summary.typeOnlyEdges` remains present;
- type-only re-export forms count as type-only edges;
- the runtime SCC lens reports only the real runtime cycle in the
  type-only-reexport fixture;
- the type-only `a.ts`/`b.ts` cycle never appears in the runtime SCC;
- the mixed/runtime `c.ts`/`d.ts` cycle survives runtime lens filtering;
- `symbols.json.reExportsByFile` still tracks the exact expected re-exporting
  files for the type-only re-export fixture;
- `topology.json.crossSubmoduleEdges` remains present as an array of
  `{ from, to, count }` objects;
- `crossSubmoduleEdges` remains the full list, not the top-30 display slice;
- `crossSubmoduleTop` remains present with the legacy
  `{ edge: "a → b", count }` display shape;
- zero cross-submodule edges produce `crossSubmoduleEdges: []`, not a missing or
  null field.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- dropping literal dynamic import edges must fail;
- treating scanner fallback files as scanner-accepted files must fail;
- losing topology performance counters must fail;
- reporting a purely type-only cycle as a runtime SCC must fail;
- filtering out a mixed/runtime re-export cycle must fail;
- dropping `reExportsByFile` evidence for type-only re-exporting files must
  fail;
- replacing structured `crossSubmoduleEdges` with string labels must fail;
- truncating `crossSubmoduleEdges` together with `crossSubmoduleTop` must fail;
- omitting `crossSubmoduleEdges` when no cross-edges exist must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is a temporary repo plus direct
  `measure-topology.mjs` subprocess execution, with `build-symbol-graph.mjs`
  used only by the existing type-only re-export fixture.
- A future mirror may use setup-only temp helpers, but helper code must not
  decide topology edge, scanner fallback, SCC, type-only, cross-submodule,
  resolver, ranking, deadness, action-safety, or performance meaning.
- The mirror must not run the full audit pipeline.
- The mirror must not change resolver, ranking, classifier, deadness,
  action-safety, incremental cache, public package, or host hook behavior.
- The mirror must not absorb `tests/test-module-reachability.mjs`,
  `tests/test-rank-fixes.mjs`, `tests/test-export-action-safety.mjs`,
  resolver unsupported-family suites, or broader performance/incremental suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/dynamic-import.test.mjs`,
2. `tests/type-only-reexport.test.mjs`,
3. `tests/topology-producer-cross-edges.test.mjs`,
4. `npm run test:vitest:dynamic-import`,
5. `npm run test:vitest:type-only-reexport`,
6. `npm run test:vitest:topology-producer-cross-edges`,
7. candidate-board updates moving the three suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current Node assertion as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.
