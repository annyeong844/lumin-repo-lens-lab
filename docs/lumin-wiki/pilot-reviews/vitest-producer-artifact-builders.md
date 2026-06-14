# Vitest Producer Artifact Builders Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidates:** `tests/test-build-shape-index.mjs`,
> `tests/test-build-function-clone-index.mjs`.

---

## Purpose

This review decides whether two producer artifact builder suites can move
together as one Vitest mirror batch. It does not add Vitest suites. The goal is
to preserve the public artifact contracts for `shape-index.json` and
`function-clones.json` without changing producer behavior, shared normalization
logic, or downstream analyzer claims.

The suites are acceptable as a paired batch because both are fixture-local CLI
producer tests. They write temporary TS projects, run one producer entrypoint,
and inspect the produced artifact shape. They do not run the full audit
pipeline, invoke ranking, decide deadness, resolve workspace imports, or change
incremental cache identity. Producer-level incremental suites remain parked
until their own cache identity review pages name cold/warm boundaries.

## Reviewed Evidence

| Suite                                       | Preserved Node Command                           | Proposed Focused Vitest Command                  | Surface Under Review                     |
| ------------------------------------------- | ------------------------------------------------ | ------------------------------------------------ | ---------------------------------------- |
| `tests/test-build-shape-index.mjs`          | `node tests/test-build-shape-index.mjs`          | `npm run test:vitest:build-shape-index`          | `shape-index.json` producer artifact     |
| `tests/test-build-function-clone-index.mjs` | `node tests/test-build-function-clone-index.mjs` | `npm run test:vitest:build-function-clone-index` | `function-clones.json` producer artifact |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane F/H boundary, limited to standalone producer artifact shape and
review-cue wording contracts.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-build-shape-index.mjs
28 passed, 0 failed

node tests/test-build-function-clone-index.mjs
20 passed, 0 failed
```

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the current artifact assertions
without changing shape hashing, function clone grouping, exact body grouping,
near-function policy, signature grouping, parse-error handling, generated-file
evidence, or production/test scan policy. The Node entrypoints must remain
runnable.

## Protected Invariants

The future Vitest mirror must preserve these shape-index contracts:

- the CLI writes `shape-index.json` and stdout includes the shape-index summary;
- `schemaVersion`, `meta.tool`, support flags, `complete`, scope, and canonical
  fact metadata remain stable;
- structurally equivalent exported type/interface shapes group by hash with
  deterministic `groupsByHash` identities;
- unsupported mapped/generic declarations emit diagnostics instead of fake
  facts and do not make the run incomplete;
- parse errors make the artifact incomplete while preserving good-file facts
  and structured diagnostics;
- `--production` excludes test files and records production scope;
- paths containing spaces and `$` remain shell-safe;
- declaration merging emits an unsupported diagnostic instead of partial facts;
- generated-file facts stay present while generated evidence counts remain
  visible;
- literal union aliases participate in exact normalized grouping.

The future Vitest mirror must preserve these function-clone contracts:

- the CLI writes `function-clones.json` and stdout includes the function-clone
  summary;
- `schemaVersion`, support flags, and `semanticEquivalence=false` remain
  visible;
- structural groups are review cues only and refuse semantic-equivalence proof;
- exact normalized body groups include aliased exports and small exact-body
  clones;
- parse errors make the artifact incomplete while preserving good facts;
- `--production` excludes test helpers and records production scope;
- near-function candidates expose review threshold policy metadata and do not
  promote structurally different helpers to exact/structure groups;
- near candidate text keeps source review required wording;
- same-signature helpers are surfaced as review-only signature groups without
  requiring body clone lanes.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- unsupported shape declarations must not become partial facts;
- parse-error files must not erase facts from valid files;
- generated files must not disappear from facts just because they carry
  generated evidence;
- declaration merging must not emit one misleading partial shape;
- exact body clones must not disappear solely because the function bodies are
  small;
- near-function and signature groups must not imply automatic semantic
  equivalence;
- production mode must not leak test fixtures into producer artifacts;
- shell-sensitive temp roots with spaces and `$` must keep working.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is temporary filesystem roots plus the real
  `build-shape-index.mjs` and `build-function-clone-index.mjs` producer
  entrypoints.
- Shared helper code may create temp roots, write source files, run a producer,
  read the output artifact, and clean up.
- Shared helper code must not decide shape hashing, clone grouping, near
  thresholds, semantic equivalence, generated policy, resolver behavior,
  deadness/ranking, or cache identity.
- The mirror must not absorb producer-level incremental suites such as
  `tests/test-shape-index-incremental.mjs` or
  `tests/test-function-clone-incremental.mjs`.
- The mirror must not change scanner behavior, resolver behavior,
  deadness/ranking, cache identity, performance counters, or public package
  behavior.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/build-shape-index.test.mjs`,
2. `tests/build-function-clone-index.test.mjs`,
3. `npm run test:vitest:build-shape-index`,
4. `npm run test:vitest:build-function-clone-index`,
5. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve the
current Node assertions as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.
