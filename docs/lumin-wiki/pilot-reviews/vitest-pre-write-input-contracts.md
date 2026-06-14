# Vitest Pre-Write Input Contracts Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-pre-write-intent.mjs`
> - `tests/test-pre-write-canonical-parser.mjs`

---

## Purpose

This review decides whether the pre-write input contract suites can move
together as one Lane C Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because both candidates protect deterministic
component-level inputs used by pre-write before advisory lookup, cue-tier
promotion, Markdown rendering, resolver expansion, dead-export ranking, or
performance cache behavior can run:

- `test-pre-write-intent.mjs` protects user intent normalization,
  validation errors, planned type-escape declarations, structured names and
  dependencies, exact shape inputs, and refactor source safety;
- `test-pre-write-canonical-parser.mjs` protects canonical type-ownership
  parsing, recognized-schema detection, owner-claim extraction, and the guard
  against treating free-form or group-level canonical prose as owner evidence.

This batch must stay separate from `tests/test-pre-write-bootstrap.mjs`,
`tests/test-mode-dispatch.mjs`, broader pre-write CLI/advisory orchestration,
lookup-name service-operation policy, cue-tier policy, renderer wording,
resolver behavior, deadness/ranking, and performance/incremental cache
identity.

## Reviewed Evidence

| Suite                                       | Preserved Node Command                           | Proposed Focused Vitest Command                  | Surface Under Review                   |
| ------------------------------------------- | ------------------------------------------------ | ------------------------------------------------ | -------------------------------------- |
| `tests/test-pre-write-intent.mjs`           | `node tests/test-pre-write-intent.mjs`           | `npm run test:vitest:pre-write-intent`           | intent schema and normalization        |
| `tests/test-pre-write-canonical-parser.mjs` | `node tests/test-pre-write-canonical-parser.mjs` | `npm run test:vitest:pre-write-canonical-parser` | canonical owner-claim parser contracts |

Current Node evidence checked for this review:

```text
node tests/test-pre-write-intent.mjs           # 54 passed, 0 failed
node tests/test-pre-write-canonical-parser.mjs # 24 passed, 0 failed
```

Goal lane: Lane C, pre/post-write lifecycle. This review covers only the
pre-write component input parsing subset of that lane.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add both focused mirrors together because
they share a pure input-parser boundary and temporary fixture style. The
mirror must keep every Node entrypoint runnable and must not relax intent
schema validation, canonical owner-claim parsing, free-form prose rejection, or
group-level canonical row exclusion.

## Protected Invariants

The future Vitest batch must preserve these input contracts:

- intent validation normalizes the five top-level arrays `names`, `shapes`,
  `files`, `dependencies`, and `plannedTypeEscapes`;
- missing top-level intent arrays default to empty arrays with structured
  warnings rather than silently disappearing;
- invalid top-level types and invalid array elements return `{ ok: false }`
  with an `errorPath` pointing at the failing key or indexed element;
- structured name declarations normalize to names while preserving declaration
  metadata and requiring `name`;
- shape inputs require fields unless an exact `hash` or supported
  `typeLiteral` is present;
- malformed shape hashes and empty `typeLiteral` values fail validation;
- planned type escapes accept only the canonical 11 escape kinds;
- planned type escapes require `reason` and non-empty `locationHint`, while the
  literal `unknown` location remains valid;
- optional `codeShape` and `alternativeConsidered` metadata remain preserved;
- structured dependency declarations normalize to specifiers while requiring
  `specifier`;
- invalid entries preserve indexed `errorPath` values such as
  `plannedTypeEscapes[1].escapeKind`;
- `refactorSources` entries validate safe relative paths and positive lines;
- missing canonical files return `recognized:false` with an absent reason;
- free-form canonical markdown without a generated or source header returns
  `recognized:false` and no owner tables;
- recognized `Status`, `Generated`, or `Source` header schemas can produce
  owner tables;
- only single-owner and severely-any-contaminated owner-level sections produce
  owner rows;
- group-level canonical sections such as `DUPLICATE_STRONG`,
  `LOCAL_COMMON_NAME`, `ANY_COLLISION`, and low-signal type-name sections do
  not produce owner claims;
- recognized files with extra free-form prose parse only recognized owner
  tables, not prose bullets;
- current flat type-ownership drafts extract owner files from identity owner
  cells while excluding duplicate/common rows;
- `findCanonicalOwnerClaim` returns `null` on misses instead of producing a
  weak inferred owner.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- defaulting missing intent keys without emitting warnings must fail;
- accepting malformed shape hashes, empty `typeLiteral`, or non-canonical
  escape kinds must fail;
- planned type escapes without `reason` or `locationHint` must fail;
- losing indexed `errorPath` precision must fail;
- accepting parent traversal in `refactorSources` must fail;
- parsing free-form canonical prose as owner evidence must fail;
- parsing duplicate/common/group-level canonical rows as single-owner claims
  must fail;
- ignoring severely-any-contaminated owner sections must fail;
- current flat type-ownership drafts losing owner-file extraction or
  duplicate/common exclusion must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is direct helper imports, in-memory intent objects,
  temporary canonical markdown files, and setup-only temp directory helpers.
- The mirror may share helpers for temp directories, fixture file writes, and
  direct helper invocation.
- Shared helpers must not decide intent validity, `errorPath` selection,
  escape-kind membership, path safety, recognized-canonical status, owner-table
  section eligibility, or owner-claim extraction.
- The mirror must not absorb `tests/test-pre-write-bootstrap.mjs`,
  `tests/test-mode-dispatch.mjs`, `tests/test-pre-write-cli.mjs`,
  `tests/test-pre-write-advisory-artifact.mjs`,
  `tests/test-pre-write-render.mjs`, lookup-name policy suites, cue-tier
  policy suites, resolver behavior, deadness/ranking, or
  performance/incremental cache identity suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/pre-write-intent.test.mjs`,
2. `tests/pre-write-canonical-parser.test.mjs`,
3. focused `npm run test:vitest:*` commands for each suite,
4. candidate-board updates moving the two suites from `REVIEWED` to `DONE`.

The implementation PR should first watch at least one focused Vitest command
fail because the script or file is missing, then add mirrors that preserve the
current Node assertion groups as named Vitest cases. It should run the
preserved Node commands, the focused Vitest commands, `npm run test:vitest`,
doc-script checks, and formatting checks.
