# Vitest Resolver Path Lookup Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-resolver-paths.mjs`
> - `tests/test-tsconfig-paths-scoped.mjs`
> - `tests/test-wildcard.mjs`

---

## Purpose

This review decides whether three Lane D resolver path lookup suites can move
together as one Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because all three suites protect the same resolver
boundary: a specifier must resolve to the correct file, sentinel, asset marker,
or generated virtual surface without silently converting internal uncertainty
into an external package or a fake concrete edge.

The future mirror must remain behavior-preserving. It must not change resolver
order, package `exports` expansion, scoped `tsconfig` discovery, baseUrl/path
semantics, output-to-source mapping, generated-artifact diagnostics, stage
cache identity, deadness ranking, or action-safety promotion.

## Reviewed Evidence

| Suite                                  | Preserved Node Command                      | Proposed Focused Vitest Command             | Surface Under Review                         |
| -------------------------------------- | ------------------------------------------- | ------------------------------------------- | -------------------------------------------- |
| `tests/test-resolver-paths.mjs`        | `node tests/test-resolver-paths.mjs`        | `npm run test:vitest:resolver-paths`        | core path resolution and resolver sentinels  |
| `tests/test-tsconfig-paths-scoped.mjs` | `node tests/test-tsconfig-paths-scoped.mjs` | `npm run test:vitest:tsconfig-paths-scoped` | scoped `tsconfig` paths/baseUrl resolution   |
| `tests/test-wildcard.mjs`              | `node tests/test-wildcard.mjs`              | `npm run test:vitest:wildcard`              | package `exports` wildcard subpath expansion |

Current Node evidence checked for this review:

```text
node tests/test-resolver-paths.mjs        # 72 passed, 0 failed
node tests/test-tsconfig-paths-scoped.mjs # 40 passed, 0 failed
node tests/test-wildcard.mjs              # 11 passed, 0 failed
```

Goal lane: Lane D, resolver/surface. This review covers path lookup and
resolver-result identity only. It does not cover unsupported-family
diagnostics, resolver diagnostics artifact aggregation, generated blind-zone
relevance, deadness/ranking, performance/incremental cache identity outside
these resolver stages, pre-write cue policy, or topology graph lenses.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The shared invariant is the resolver output contract: callers must be able to
distinguish a resolved source file from `EXTERNAL`, `UNRESOLVED_INTERNAL`,
`null`, non-source assets, and generated virtual surfaces. The future Vitest
batch may share setup-only fixture helpers, but each resolver family assertion
must remain explicit enough to show which input specifier, importer scope, and
expected output shape prove the behavior.

## Protected Invariants

The future Vitest batch must preserve these resolver path lookup contracts:

- Extensionless relative imports resolve through the supported source and
  declaration extension table, including `.cjs`, `.jsx`, `.mts`, `.cts`, and
  `.d.ts` files.
- Directory imports resolve through supported `index.*` files without losing
  CJS or declaration targets.
- Missing relative specifiers return `null` rather than a fake file path.
- Bundler resource-query imports such as `?inline` resolve as non-source asset
  sentinels and do not become source-file edges.
- Missing generated asset targets report generated-artifact reasons with
  target candidates and package-script evidence.
- `isResolvedFile()` distinguishes real paths from `EXTERNAL`,
  `UNRESOLVED_INTERNAL`, `null`, `undefined`, and non-string values.
- Run-local resolver memoization preserves null misses, non-source assets,
  unresolved-internal sentinels, and generated virtual surfaces without
  changing result identity.
- Scoped baseUrl and scoped `tsconfig` probe caches may cross importer files
  only when the resolver context proves the result identity is stable.
- Wildcard alias cache hits preserve resolved paths, no-match fallthroughs,
  unresolved-internal misses, generated virtual surface object identity, and
  frozen mutation protection.
- Per-app `@/*` aliases resolve nearest-scope-first so the same specifier can
  resolve to different targets in different workspace app scopes.
- Missing local aliases stay `UNRESOLVED_INTERNAL`, while genuine external npm
  imports remain `EXTERNAL`.
- JSONC `tsconfig` files with `$schema`, comments, `@/*`, and `**/*.ts` globs
  parse without regex-strip corruption.
- `extends` entries resolved through hoisted `node_modules` keep TypeScript's
  replacement semantics for `paths`.
- Scoped baseUrl type-only and value imports contribute fan-in in the correct
  identity space.
- Unresolved summaries keep ordinary target misses, workspace package subpath
  misses, and generated workspace misses separated by reason.
- Package `exports` wildcard subpaths resolve exact, nested, and most-specific
  matches through supported source/output mappings.
- Matched wildcard package targets with missing files return
  `UNRESOLVED_INTERNAL`, not `EXTERNAL`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- A helper must not collapse resolver sentinels into truthy string paths.
- A helper must not treat non-source asset sentinels or generated virtual
  surfaces as ordinary source files.
- A helper must not infer that a missing internal-looking alias is external
  simply because no target file exists.
- A helper must not flatten scoped `tsconfig` aliases into one repo-global
  alias map.
- A helper must not merge inherited and local `paths` differently than
  TypeScript does.
- A helper must not make `tsconfig` JSONC parsing pass only for plain JSON
  fixtures.
- A helper must not hide generated-artifact reason records behind broad
  unresolved assertions.
- A helper must not reuse cached resolver results when the importer scope,
  package scope, condition profile, or generated virtual surface identity would
  change the answer.
- A helper must not widen wildcard package exports beyond their matching
  subpath.
- The fixture must continue to prove both positive resolution and negative
  sentinel outcomes for every resolver family in the batch.
- The mirror must not combine this batch with `test-alias.mjs`,
  `test-dynamic-import.mjs`, `test-entry-surface-artifact.mjs`,
  unsupported-family diagnostics, generated blind-zone suites,
  deadness/ranking/action-safety suites, topology/type-only graph lenses, or
  performance/incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- Temporary resolver fixtures may be shared for setup, file writes, package
  JSON writes, command execution, import of resolver helpers, and cleanup only.
- Shared helpers must not decide resolver stage order, package `exports`
  matching, tsconfig scope precedence, generated-artifact classification,
  output-to-source mapping, sentinel meaning, cache-hit semantics, deadness
  ranking, or action-safety promotion.
- The mirror must not relax edge-case assertions into broad artifact presence
  checks.
- The mirror must not introduce resolver behavior changes or performance
  optimizations.
- Stage cache counters in these suites are resolver-result identity guards, not
  a general performance/incremental migration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/resolver-paths.test.mjs`,
2. `tests/tsconfig-paths-scoped.test.mjs`,
3. `tests/wildcard.test.mjs`,
4. focused `npm run test:vitest:*` commands for each suite,
5. candidate-board updates moving the three suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share setup-only temporary repo and
resolver-import helpers inside test files, but no shared helper should decide
resolver semantics or hide the expected source/sentinel for each assertion.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest`, doc-script checks, and formatting checks
so the reviewed runner discovery boundary and wiki references stay current.
