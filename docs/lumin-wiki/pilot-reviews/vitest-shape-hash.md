# Vitest Shape Hash Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-shape-hash.mjs`.

---

## Purpose

This review decides whether `tests/test-shape-hash.mjs` is a reasonable Lane A
low-risk core/parser/helper Vitest pilot candidate. It does not add the Vitest
suite. The goal is to preserve the pure `_lib/shape-hash.mjs` normalization and
diagnostic contract without widening the mirror into the shape-index producer,
pre-write lookup, deadness, ranking, or audit orchestration.

The suite is a good next candidate because it imports one pure helper module,
uses inline TypeScript source strings, and does not create repository fixtures
or run producers. It protects non-happy-path behavior that matters for the
evidence contract: unsupported shapes must produce diagnostics rather than fake
high-confidence facts, parse errors must suppress shape facts, declaration
merges must not be partially hashed, and generated-file evidence must be tagged
without inventing matches.

## Reviewed Evidence

- Preserved Node command: `node tests/test-shape-hash.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:shape-hash`.
- Helper modules under review:
  - `_lib/shape-hash.mjs`.
- Current suite description: `tests/README.md`.
- Goal lane: Lane A, low-risk core/parser/helper.

## Result

The suite is acceptable as the next narrow Vitest pilot candidate.

The future mirror should keep this as a shape-normalization and diagnostic
suite. It should not build `shape-index.json`, call pre-write shape lookup,
classify duplicates, or make analyzer absence claims. The old Node entrypoint
must remain runnable, and each supported or unsupported shape contract should
stay visible as a named assertion.

## Protected Invariants

The future Vitest pilot must preserve these shape-hash contracts:

- exported interfaces and object type aliases with the same fields hash the
  same even when field order or declaration spelling differs;
- changing a field type changes the hash;
- optional and `readonly` modifiers are hash-bearing;
- `normalizeTypeText(...)` normalizes punctuation spacing outside literals but
  preserves string literal interiors;
- semantically identical type text spacing hashes the same;
- unsupported mapped or generic shapes emit diagnostics, not fake facts;
- index/computed members emit diagnostics rather than fuzzy shape facts;
- emitted facts carry canonical `kind`, `source`, `scope`, `confidence`,
  `observedAt`, `identity`, and `identities` metadata;
- hashes keep the `sha256:<64 lowercase hex>` format;
- parse errors emit source-level diagnostics and no high-confidence facts;
- export specifier aliases use the exported identity name;
- `groupShapeFactsByHash(...)` returns deterministic sorted identity lists;
- declaration merging is unsupported and does not emit partial shape facts;
- generated-file evidence is tagged on facts and can come from either path
  convention or generated header detection;
- literal union aliases are hashable and order-insensitive when every member is
  supported;
- broad mixed unions remain diagnostic-only.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- treating field order as hash-bearing must fail;
- treating optional or `readonly` modifiers as non-hash-bearing must fail;
- normalizing inside string literals must fail;
- emitting facts for mapped/generic/index/computed shapes must fail;
- silently ignoring unsupported-shape diagnostics must fail;
- emitting shape facts after parse errors must fail;
- using local declaration names instead of exported aliases must fail;
- returning nondeterministic hash group identity lists must fail;
- partially hashing declaration-merged interfaces must fail;
- using generated-file evidence as a fake shape match must fail;
- collapsing literal unions into broad fuzzy facts must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-shape-hash.mjs` remains runnable.
- The fixture boundary is inline source strings only; no temp repo helper is
  needed.
- The pilot must not change shape normalization, diagnostic reason codes,
  generated-file tagging, hash algorithm labels, or metadata shape.
- The pilot must not build or consume `shape-index.json`.
- The pilot must not absorb `test-build-shape-index.mjs`,
  `test-pre-write-lookup-shape.mjs`, `test-pre-write-shape-index.mjs`,
  `test-shape-index-incremental.mjs`, resolver suites, deadness/ranking suites,
  pre-write cue suites, or producer-backed scan-policy suites.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/shape-hash.test.mjs`,
2. `npm run test:vitest:shape-hash`,
3. a candidate-board update moving this suite from `REVIEWED` to `DONE`.

The implementation PR should keep every current Node assertion represented as
a named Vitest assertion. It may group assertions by field normalization,
unsupported diagnostics, metadata shape, alias identity, generated evidence,
and literal union handling, but shape-hash meaning must stay local to this
suite.

Run both commands when changing this suite:

- `node tests/test-shape-hash.mjs`
- `npm run test:vitest:shape-hash`

Do not migrate shape-index producers, pre-write shape lookup, resolver suites,
deadness/ranking suites, performance/incremental suites, renderer suites, or
producer-backed scan-policy suites as part of the shape-hash pilot.
