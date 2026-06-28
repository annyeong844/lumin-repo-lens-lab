# Vitest CJS Surface Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-extract-cjs-consumer.mjs`
> - `tests/test-extract-cjs-export-surface.mjs`
> - `tests/test-cjs-export-surface-artifact.mjs`
> - `tests/test-cjs-classification.mjs`
> - `tests/test-cjs-integration.mjs`

---

## Purpose

This review decides whether the CJS surface suites can move together as one
Lane D Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because all five suites protect the same CommonJS
surface contract:

- mechanically exact CJS consumers may create exact file/symbol evidence;
- side-effect-only `require(...)` must not keep every named export alive;
- namespace escapes, key introspection, non-const namespaces, writes, and
  dynamic requires stay broad or opaque evidence;
- CJS export surfaces distinguish exact export names from opaque export forms;
- symbol artifacts must preserve CJS support metadata instead of guessing from
  file extension alone.

This is analyzer-adjacent, so the future mirror must remain a
behavior-preserving runner mirror. It must not change resolver behavior,
deadness ranking, or action-safety promotion.

## Reviewed Evidence

| Suite                                        | Preserved Node Command                            | Proposed Focused Vitest Command                   | Surface Under Review                       |
| -------------------------------------------- | ------------------------------------------------- | ------------------------------------------------- | ------------------------------------------ |
| `tests/test-extract-cjs-consumer.mjs`        | `node tests/test-extract-cjs-consumer.mjs`        | `npm run test:vitest:extract-cjs-consumer`        | extractor-level CJS consumer facts         |
| `tests/test-extract-cjs-export-surface.mjs`  | `node tests/test-extract-cjs-export-surface.mjs`  | `npm run test:vitest:extract-cjs-export-surface`  | extractor-level CJS export surface facts   |
| `tests/test-cjs-export-surface-artifact.mjs` | `node tests/test-cjs-export-surface-artifact.mjs` | `npm run test:vitest:cjs-export-surface-artifact` | `symbols.json` CJS export surface artifact |
| `tests/test-cjs-classification.mjs`          | `node tests/test-cjs-classification.mjs`          | `npm run test:vitest:cjs-classification`          | fan-in/deadness classification CJS lens    |
| `tests/test-cjs-integration.mjs`             | `node tests/test-cjs-integration.mjs`             | `npm run test:vitest:cjs-integration`             | integrated CJS exact and opacity facts     |

Current Node evidence checked for this review:

```text
node tests/test-extract-cjs-consumer.mjs        # 22 passed, 0 failed
node tests/test-extract-cjs-export-surface.mjs  # 6 passed, 0 failed
node tests/test-cjs-export-surface-artifact.mjs # 3 passed, 0 failed
node tests/test-cjs-classification.mjs          # 11 passed, 0 failed
node tests/test-cjs-integration.mjs             # 3 passed, 0 failed
```

Goal lane: Lane D, resolver/surface. This review covers only the CJS surface
family, not all resolver/surface suites.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all five focused mirrors together because
they share the same CJS exact-vs-opaque evidence boundary. The mirror must keep
every Node entrypoint runnable and must not turn broad CJS evidence into exact
consumer proof.

## Protected Invariants

The future Vitest batch must preserve these CJS contracts:

- Bare `require("./x")` emits side-effect-only use and does not exact-protect
  named exports.
- Destructuring `require(...)`, namespace member reads/calls, direct
  `require(...).member`, static computed members, guarded reads, and namespace
  alias destructuring create exact CJS consumer facts only when the member name
  is mechanically known.
- Rest destructuring, namespace escapes, non-const namespace aliases, key
  introspection, member writes, `delete`, and dynamic require calls stay broad
  or opaque evidence.
- Static package metadata requires such as `require(path.resolve(...,
"package.json"))` do not create dynamic CJS opacity or fake CJS consumers.
- CJS export extraction records obvious `exports.foo`, `module.exports.foo`,
  quoted members, and object-literal `module.exports` properties as exact
  export surface facts.
- Computed export names and non-object `module.exports = ...` assignments stay
  opaque export surface evidence.
- `symbols.json.meta.supports.cjsExportSurface` advertises support, and
  `symbols.cjsExportSurfaceByFile` preserves exact and opaque entries by file.
- Exact CJS consumers affect `fanInByIdentity`; side-effect-only CJS requires do
  not keep every export alive.
- Broad CJS namespace evidence prevents truly-dead confidence without creating
  fake exact fan-in.
- Integrated CJS facts keep exact export surfaces, alias destructuring
  consumers, and dynamic require opacity visible together.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- A broad `require("./exporter")` must not keep all named exports alive.
- A member write such as `mod.foo = 1` must not exact-protect `foo`.
- Key introspection such as `"foo" in mod` must not exact-protect `foo`.
- A shadowed function parameter named like a namespace must not exact-protect an
  outer require namespace.
- Dynamic require opacity must not be reclassified as a clean unresolved import
  or as an exact consumer.
- Opaque CJS export forms must remain visible in artifact output; they must not
  disappear when exact exports are also present.
- Integration fixtures must keep exact CJS export facts, ESM exact fan-in, and
  dynamic require opacity in the same run so cross-stage drift is visible.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is temporary JavaScript/CJS files, direct extractor
  calls, and symbol-graph runs over temporary repos.
- Shared helpers may create temporary roots, write fixture files, run
  `build-symbol-graph.mjs`, read JSON, and clean up directories.
- Shared helpers must not decide exact-vs-broad CJS meaning, opacity
  classification, fan-in identity, deadness classification, or CJS export
  surface semantics.
- The mirror must not absorb TypeScript path resolver suites, workspace export
  suites, generated/framework surface suites, deadness/ranking suites,
  performance/incremental suites, or cue-tier/pre-write policy suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/extract-cjs-consumer.test.mjs`,
2. `tests/extract-cjs-export-surface.test.mjs`,
3. `tests/cjs-export-surface-artifact.test.mjs`,
4. `tests/cjs-classification.test.mjs`,
5. `tests/cjs-integration.test.mjs`,
6. focused `npm run test:vitest:*` commands for each suite,
7. candidate-board updates moving the five suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share setup-only temporary repo and
JSON-read helpers inside test files, but no shared helper should decide CJS
consumer exactness, opacity, export surface exactness, fan-in, or deadness
meaning.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest`, doc-script checks, and formatting checks
so the reviewed runner discovery boundary and wiki references stay current.
