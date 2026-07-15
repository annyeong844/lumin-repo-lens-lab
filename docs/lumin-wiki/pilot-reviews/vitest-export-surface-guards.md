# Vitest Export Surface Guards Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-definition-id-export.mjs`
> - `tests/test-file-delta-export.mjs`
> - `tests/test-classify-policies-export-surface.mjs`

---

## Purpose

This review decides whether three tiny export-surface guard suites can move
together as one Lane A Vitest mirror batch. It does not add Vitest suites. The
goal is to preserve module boundary contracts that protect public helper APIs
from leaking internal implementation symbols.

The batch is acceptable because every candidate:

- imports a single helper module directly;
- creates no repository fixture;
- runs no producer or audit pipeline;
- checks only exported symbol presence or absence;
- protects a module boundary rather than analyzer absence, resolver, ranking,
  or performance semantics.

The future mirror should keep the suites as boundary guards. It must not widen
them into behavior tests for definition-id construction, post-write file-delta
normalization or classification policy decisions.

## Reviewed Evidence

| Suite                                             | Preserved Node Command                                 | Proposed Focused Vitest Command                        | Module Under Review                |
| ------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------ | ---------------------------------- |
| `tests/test-definition-id-export.mjs`             | `node tests/test-definition-id-export.mjs`             | `npm run test:vitest:definition-id-export`             | `_lib/definition-id.mjs`           |
| `tests/test-file-delta-export.mjs`                | `node tests/test-file-delta-export.mjs`                | `npm run test:vitest:file-delta-export`                | `_lib/post-write-file-delta.mjs`   |
| `tests/test-classify-policies-export-surface.mjs` | `node tests/test-classify-policies-export-surface.mjs` | `npm run test:vitest:classify-policies-export-surface` | `_lib/classify-policies.mjs`       |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane A, low-risk core/parser/helper export-surface guards.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all three focused mirrors together because
the fixture boundary, assertion style, and risk profile are the same. Each Node
entrypoint must remain runnable, and each suite must stay a module-export
contract rather than expanding into domain behavior.

## Protected Invariants

The future Vitest batch must preserve these export-surface contracts:

- `_lib/definition-id.mjs` exports `definitionIdFromOxcNode`;
- `_lib/definition-id.mjs` does not expose the raw internal
  `makeDefinitionId` builder;
- `_lib/post-write-file-delta.mjs` exports `computeFileDelta`;
- `_lib/post-write-file-delta.mjs` exports `repoRelativeFileList`;
- `_lib/post-write-file-delta.mjs` does not expose
  `normalizeRepoRelativePath`;
- `_lib/classify-policies.mjs` does not expose legacy framework sentinel
  helpers: `isCoreSentinel`, `detectNuxtNitro`, or
  `isNuxtNitroSentinel`;
- `_lib/classify-policies.mjs` does not expose non-public policy action
  constants: `ACTION_NONE` or `ACTION_REVIEW_HINT`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- accidentally exposing a raw ID builder from `definition-id` must fail;
- hiding the public OXC-based definition ID helper must fail;
- exposing file-delta path normalizer internals must fail;
- hiding either public file-delta API must fail;
- re-exporting legacy framework sentinel helpers from classify policies must
  fail;
- exposing non-public classify policy action constants must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is direct dynamic module import only; no temp repo helper
  is needed.
- The mirror must not add behavior coverage for the underlying algorithms.
- The mirror must not change exported symbols.
- The mirror must not absorb resolver, generated, deadness/ranking,
  pre/post-write workflow, or classification behavior suites.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/definition-id-export.test.mjs`,
2. `tests/file-delta-export.test.mjs`,
3. `tests/classify-policies-export-surface.test.mjs`,
4. focused `npm run test:vitest:*` commands for each suite,
5. candidate-board updates moving all three suites from `REVIEWED` to `DONE`.

The implementation PR should keep every current Node assertion represented as
a named Vitest assertion. It may share a tiny assertion helper inside a test
file if useful, but no shared helper should decide which symbols are public.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest` to ensure the reviewed runner discovery
boundary stays scoped to first-party reviewed mirrors.
