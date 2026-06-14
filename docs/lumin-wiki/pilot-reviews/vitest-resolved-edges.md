# Vitest Resolved Edges Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-resolved-edges.mjs`.

---

## Purpose

This review decides whether `tests/test-resolved-edges.mjs` is ready for a
narrow Vitest mirror. It does not add the Vitest suite. The goal is to name the
file-level graph edge contracts that runner migration must preserve before
future reachability or deadness work consumes `symbols.json.resolvedInternalEdges[]`.

This suite is analyzer-sensitive. It protects the split between symbol fan-in
and module reachability. A side-effect import or broad CommonJS escape may
prove that a target file is evaluated, but it must not keep every export in that
target file alive.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-resolved-edges.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:resolved-edges`.
- Symbol graph producer under review:
  `skills/lumin-repo-lens-lab/_engine/producers/build-symbol-graph.mjs`.
- TypeScript/JavaScript extractor under review:
  `skills/lumin-repo-lens-lab/_engine/lib/extract-ts.mjs`.
- Resolver under review:
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-core.mjs`.
- Companion reachability suite:
  `node tests/test-module-reachability.mjs`.
- Companion topology suite:
  `node tests/test-dynamic-import.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.
- Evidence contract guardrail:
  `docs/lumin-wiki/concepts/evidence-contract.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite builds a synthetic source tree, runs
`build-symbol-graph.mjs`, then reads `symbols.json`. The fixture intentionally
mixes named imports, default imports, namespace imports, type-only imports,
side-effect imports, named and star re-exports, dynamic literal imports, CommonJS
exact requires, CommonJS namespace escapes, CommonJS side-effect requires, and a
non-source asset import.

The future Vitest mirror must keep the same lens split:

- `resolvedInternalEdges[]` is file-level reachability evidence;
- `fanInByIdentity` is symbol-level consumer evidence;
- non-source asset imports are neither resolver blindness nor JavaScript module
  reachability edges;
- broad or side-effect file reachability must not become named export liveness.

## Protected Invariants

The future Vitest pilot must preserve these resolved-edge contracts:

- `symbols.json.meta.supports.resolvedInternalEdges === true`;
- `symbols.resolvedInternalEdges` is always an array;
- named imports create `kind: "import-named"` file-level edges;
- default imports create `kind: "import-default"` file-level edges;
- namespace imports create `kind: "import-namespace"` file-level edges;
- type-only named imports keep `typeOnly: true`;
- side-effect imports create `kind: "import-side-effect"` file-level edges;
- named re-exports create `kind: "reexport-named"` file-level edges;
- star re-exports create `kind: "reexport-broad"` file-level edges;
- literal dynamic imports create `kind: "dynamic-literal"` file-level edges;
- exact CommonJS destructuring requires create `kind: "cjs-require-exact"`
  file-level edges;
- broad CommonJS namespace escapes create `kind: "cjs-namespace-escape"`
  file-level edges;
- CommonJS side-effect requires create `kind: "cjs-side-effect"` file-level
  edges;
- side-effect-only ESM imports do not create named fan-in for hidden exports;
- side-effect-only CJS requires do not create named fan-in for hidden exports;
- non-source asset imports do not become unresolved internal records;
- non-source asset imports do not become JavaScript module reachability edges.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not collapse `resolvedInternalEdges[]` into `fanInByIdentity`.
- A helper must not treat side-effect reachability as named export use.
- A helper must not make CJS namespace escapes keep every target export alive.
- A helper must not drop `typeOnly`, because type-only edges have different
  reachability meaning.
- A helper must not treat CSS or other non-source asset imports as unresolved
  internal JavaScript modules.
- A helper must not hide which import form produced which `kind` value.
- The mirror must not combine this graph artifact-shape suite with
  `test-module-reachability.mjs`, `test-dynamic-import.mjs`,
  unsupported-family diagnostics suites, generated-artifact suites,
  deadness/ranking suites, or performance/incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-resolved-edges.mjs` remains runnable.
- The pilot may use temporary repo fixtures, but the fixture content,
  `build-symbol-graph.mjs` invocation, `symbols.json` parsing, and edge/fan-in
  assertions stay local to this suite.
- The pilot must not change resolver behavior.
- The pilot must not change symbol fan-in behavior.
- The pilot must not change module reachability behavior.
- The pilot must not add dynamic import expansion beyond literal imports.
- The pilot must not relax per-edge `kind` and `typeOnly` assertions into broad
  presence checks.
- Shared helpers may extract temporary directory setup later, but they must not
  hide the mapping from source syntax to edge kind.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/resolved-edges.test.mjs`,
2. `npm run test:vitest:resolved-edges`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by graph
lane:

- artifact support and array shape;
- ESM import and re-export edge kinds;
- type-only and dynamic literal edge lenses;
- CommonJS edge kinds;
- fan-in negative guards for side-effect-only edges;
- non-source asset negative guards.

Run both commands when changing this suite:

- `node tests/test-resolved-edges.mjs`
- `npm run test:vitest:resolved-edges`

Do not migrate any other resolver, reachability, generated, deadness, ranking,
cue-tier, or performance suite as part of the resolved edges pilot.
