# Vitest JS Module Edge Scanner Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-js-module-edge-scanner.mjs`.

---

## Purpose

This review decides whether the JS module edge scanner suite can move as a
single Vitest mirror batch. It does not add a Vitest suite. The goal is to
preserve the scanner contract that lets topology-related code avoid full AST
parsing only when module-edge facts are proven equivalent to the Oxc topology
edge lens.

The suite is acceptable as a one-suite Lane F scanner batch because it is
pure-library and fixture-local. It does not run the audit pipeline, mutate cache
state, resolve imports, or decide reachability/deadness. Producer-level
incremental reuse, resolver behavior, graph/SCC outputs, and action-safety
remain outside this mirror.

## Reviewed Evidence

| Suite                                   | Preserved Node Command                       | Proposed Focused Vitest Command              | Surface Under Review                     |
| --------------------------------------- | -------------------------------------------- | -------------------------------------------- | ---------------------------------------- |
| `tests/test-js-module-edge-scanner.mjs` | `node tests/test-js-module-edge-scanner.mjs` | `npm run test:vitest:js-module-edge-scanner` | tokenizer-state module edge fast scanner |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane F, performance/incremental/scanner, limited to scanner
equivalence and fallback-contract behavior.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-js-module-edge-scanner.mjs
16 passed, 0 failed
```

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the current scanner assertions
without changing scanner policy, Oxc fallback behavior, topology resolution,
producer performance counters, or graph outputs. The Node entrypoint must remain
runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- accepted scanner output is equivalent to Oxc-derived normalized topology
  edges for static imports, side-effect imports, re-exports, type-only edges,
  and literal dynamic imports;
- accepted output carries `MODULE_EDGE_SCANNER_POLICY_VERSION` and
  `fast-module-edge` mode;
- fake module syntax inside line comments, strings, regex literals, and
  template literals is ignored rather than emitted as graph edges;
- import/export attributes and assertions with string specifiers are accepted
  only when the module specifier is safely represented;
- accepted edges preserve source line numbers;
- non-literal dynamic imports, template dynamic imports, `require(...)`,
  `import.meta.glob`, TypeScript import-equals, export assignment, ambient
  module declarations, and JSX text fall back with stable reason codes;
- the string-heavy fixture stays linear enough to guard against accidental
  quadratic line scans.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- scanner-accepted files must fail if normalized Oxc topology edges differ;
- fake import/export text in comments, strings, regexes, or templates must not
  create concrete module edges;
- unsupported dynamic or TypeScript module forms must not silently appear as
  accepted fast-path results;
- JSX text must remain fallback-required until the scanner can prove it has
  skipped JSX safely;
- line-number evidence must not be lost when moving from hand-rolled assertions
  to Vitest cases;
- a regression to repeated full-source line scans must be caught by the
  many-string-literal performance guard.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is direct source strings plus the real
  `_lib/js-module-edge-scanner.mjs` and `_lib/parse-oxc.mjs` modules.
- Shared helper code may normalize edge facts, compare JSON-safe edge records,
  and construct fixture source strings.
- Shared helper code must not decide resolver, reachability, topology graph,
  SCC, deadness, generated-artifact, public surface, or performance-counter
  meaning.
- The mirror must not absorb producer-level incremental suites such as
  `tests/test-any-inventory-incremental.mjs`,
  `tests/test-symbol-graph-incremental.mjs`,
  `tests/test-function-clone-incremental.mjs`, or
  `tests/test-shape-index-incremental.mjs`.
- The mirror must not change scanner implementation, resolver behavior,
  deadness/ranking, cache identity, performance counters, or public package
  behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/js-module-edge-scanner.test.mjs`,
2. `npm run test:vitest:js-module-edge-scanner`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves the
current Node assertions as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
