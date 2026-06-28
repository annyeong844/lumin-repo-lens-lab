# Vitest Hash Imports Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-hash-imports.mjs`.

---

## Purpose

This review decides whether `tests/test-hash-imports.mjs` is ready for a narrow
Vitest mirror. It does not add the Vitest suite. The goal is to name the Node
package `#imports` resolver contracts before any runner migration rewrites the
fixture shapes or accidentally broadens output-to-source inference.

This suite is analyzer-sensitive. Unlike
`tests/test-node-imports-unsupported.mjs`, it mostly protects supported
`#imports` resolution paths: exact imports, wildcard imports, suffix-preserving
wildcards, output-to-source mapping parity, authored JS source preservation,
directory index resolution, and malformed workspace package resilience.

## Reviewed Evidence

- Preserved Node command: `node tests/test-hash-imports.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:hash-imports`.
- Resolver code under review: `_lib/resolver-core.mjs`.
- Alias-map code under review: `_lib/alias-map.mjs`.
- Repo-mode code under review: `_lib/repo-mode.mjs`.
- Producer used for graph/deadness checks: `build-symbol-graph.mjs`.
- Companion unsupported-family suite:
  `node tests/test-node-imports-unsupported.mjs`.
- Companion output-layout diagnostic suite:
  `node tests/test-output-source-layout-diagnostics.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving resolver mirror.

The current Node suite protects supported Node `#imports` behavior across
several small fixture shapes. The migration must not turn those fixtures into a
generic resolver helper or use them as a reason to expand alias-map heuristics.
The correct outcome is the same resolved target, unresolved sentinel, graph
edge absence, or deadness protection currently asserted by the Node suite.

## Protected Invariants

The future Vitest pilot must preserve these resolver contracts:

- exact `#imports` entries that point at `.mjs`, `.cjs`, `.js`, or `.jsx`
  output files map back to supported authored source files;
- supported non-`src` source conventions such as `lib/` remain covered by the
  existing output/source mapping policy;
- wildcard `#imports` entries preserve subpath matching for `.mjs` and `.jsx`
  output patterns;
- suffix-preserving wildcard keys such as `#web/request/*.js` resolve to the
  authored TypeScript source and protect both value and type exports from
  deadness;
- missing wildcard targets return `UNRESOLVED_INTERNAL`, not external and not a
  fake resolved edge;
- authored JavaScript source targets remain authored JavaScript targets rather
  than being forced through TypeScript fallbacks;
- wildcard directory targets resolve to an index file when the index file is the
  actual source target;
- `mapOutputPatternToSource(...)` and
  `mapOutputPatternToSourceCandidates(...)` keep their current ordered mapping
  behavior;
- a malformed sibling workspace `package.json` does not abort alias-map
  construction for otherwise valid workspace packages.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not collapse exact and wildcard fixtures into one generic setup
  that hides the difference between exact keys, wildcard keys, suffix wildcards,
  and directory-index targets.
- A helper must not convert missing `#imports` targets into `EXTERNAL`.
- A helper must not create concrete graph edges for unresolved wildcard misses.
- A helper must not add new output/source directory pairs while mirroring this
  suite.
- The deadness checks must still prove that a type-only consumer and a value
  consumer through suffix wildcard imports protect the exported interface and
  function.
- The malformed workspace package fixture must still include one bad package
  and one good package, so the alias-map resilience contract stays meaningful.
- The mirror must not combine this suite with unsupported Node imports,
  output-to-source unsupported diagnostics, generated artifacts,
  `import.meta.glob`, deadness/ranking, or performance fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-hash-imports.mjs` remains runnable.
- The pilot may use the setup-only temporary repo helper, but resolver meaning,
  package `imports` metadata, output/source mapping assertions, direct resolver
  calls, and graph/deadness assertions stay local to this suite.
- The pilot must not change `_lib/alias-map.mjs`,
  `_lib/resolver-core.mjs`, `_lib/repo-mode.mjs`, or
  `build-symbol-graph.mjs`.
- The pilot must not add new resolver capabilities, new output/source pairs, or
  new unsupported-family reason codes.
- The pilot must not migrate broader resolver, deadness/ranking, generated,
  dynamic-module, or performance suites.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/hash-imports.test.mjs`,
2. `npm run test:vitest:hash-imports`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by resolver
shape:

- exact Node `#imports` output-to-source mapping;
- wildcard Node `#imports` output-to-source mapping;
- suffix wildcard graph/deadness protection;
- output-pattern mapping helper behavior;
- authored JS and directory-index target behavior;
- malformed workspace package resilience.

Run both commands when changing this suite:

- `node tests/test-hash-imports.mjs`
- `npm run test:vitest:hash-imports`

Do not migrate unsupported Node imports, output-layout diagnostics, generated
artifact suites, deadness/ranking suites, or performance/incremental suites as
part of the hash-imports pilot.
