# Vitest Aliased Export Classification Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-alias.mjs`.

---

## Purpose

This review decides whether `tests/test-alias.mjs` can move as one narrow
Vitest mirror batch. It does not add a Vitest suite.

Despite the filename, this suite does not protect resolver alias lookup. It
protects a dead-export/action-safety edge case for aliased export specifiers:

```ts
function foo() {}
export { foo as publicThing };
```

The suite is acceptable as a single-suite batch because it uses a small
temporary fixture and only checks that symbol graph and dead-classification
artifacts preserve local/exported name distinction. The future mirror must not
generalize into broader deadness/ranking, namespace reachability, public API,
resolver, or SAFE_FIX calibration behavior.

## Reviewed Evidence

| Suite                  | Preserved Node Command      | Proposed Focused Vitest Command | Surface Under Review                         |
| ---------------------- | --------------------------- | ------------------------------- | -------------------------------------------- |
| `tests/test-alias.mjs` | `node tests/test-alias.mjs` | `npm run test:vitest:alias`     | aliased export classification/action wording |

Current preserved-command evidence on 2026-05-16:

```text
node tests/test-alias.mjs
6 passed, 0 failed
```

Goal lane: Lane E, deadness/ranking/calibration. This review covers only
aliased export-specifier action safety.

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR may add one focused mirror for this suite because
the fixture is small and the protected behavior is specific: the analyzer must
not confuse the exported name with the local implementation name when proposing
dead-export actions.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `symbols.deadProdList` records `localName` for aliased export specifiers;
- non-aliased export specifiers do not record a distinct `localName`;
- an aliased export whose local implementation is still used in the same file
  must not be described as "definition removal";
- the safe/review action surface for an aliased export carries the local name
  so a reviewer knows which implementation symbol to inspect;
- an aliased export whose local implementation is also dead carries a
  `localAlsoDead` or equivalent zero-local-use signal;
- internal-use counts for aliased exports are measured against the local name,
  not the public exported alias.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- using the exported alias as the only identity for local reference counting
  must fail;
- omitting `localName` from aliased export symbol facts must fail;
- adding `localName` noise to non-aliased exports must fail;
- placing an aliased-but-locally-used export in a definition-removal action
  bucket without specifier-aware wording must fail;
- hiding the fact that the local implementation is also dead must fail;
- treating aliased export-specifier removal as proof that the local definition
  can be deleted must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is a temporary TypeScript package with aliased,
  non-aliased, locally-used, locally-dead, and consumer-control files.
- The mirror may share local helpers for temp directory setup, file writes,
  producer subprocess execution, JSON reads, and cleanup.
- Shared helpers must not decide deadness buckets, action wording,
  local-vs-exported identity semantics, local reference counts, public API
  status, resolver behavior, namespace reachability, or SAFE_FIX calibration.
- The mirror must not absorb `tests/test-export-action-safety.mjs`,
  `tests/test-rank-fixes.mjs`, `tests/test-module-reachability.mjs`,
  `tests/test-namespace-reexport-deadness.mjs`, P6 calibration/member
  precision suites, resolver suites, public/framework/generated blocker
  suites, or performance/incremental suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/alias.test.mjs`,
2. `npm run test:vitest:alias`,
3. candidate-board updates moving `tests/test-alias.mjs` from `REVIEWED` to
   `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves the six
current Node assertions as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, `npm run test:vitest`, doc-script checks,
formatting checks, and `npm test` before completion.
