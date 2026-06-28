# Vitest Checklist Facts Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-checklist-facts.mjs`.

---

## Purpose

This review decides whether `tests/test-checklist-facts.mjs` can move as one
narrow Lane H Vitest mirror batch. It does not add a Vitest suite. The goal is
to preserve the `checklist-facts.json` pre-compute artifact contract for
`templates/REVIEW_CHECKLIST.md` without changing analyzer producers,
thresholds, ranking, deadness, resolver behavior, or full audit orchestration.

This suite is acceptable as a single-suite batch because it uses temporary
fixtures, invokes producer entrypoints through subprocesses, and inspects the
produced checklist artifact shape. It is not acceptable to absorb deadness,
resolver, performance, or broader orchestrator suites into this mirror. The
future implementation should keep the mirror focused on graceful degradation,
fresh AST-backed checklist facts, artifact-backed checklist facts, citation
hints, context-check flags, and review-only duplicate/shape/function evidence.

## Reviewed Evidence

| Suite                            | Preserved Node Command                | Proposed Focused Vitest Command       | Surface Under Review            |
| -------------------------------- | ------------------------------------- | ------------------------------------- | ------------------------------- |
| `tests/test-checklist-facts.mjs` | `node tests/test-checklist-facts.mjs` | `npm run test:vitest:checklist-facts` | `checklist-facts.json` artifact |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane H, checklist artifact pre-compute evidence.

Fresh preserved-command evidence on 2026-05-16:

```text
node tests/test-checklist-facts.mjs
59 passed, 0 failed
```

## Result

This suite is acceptable as one narrow Vitest mirror batch.

The future implementation PR should mirror the current artifact assertions
without changing `checklist-facts.mjs`, `checklist-facts.json` gate semantics,
shape/function clone producer semantics, topology semantics, triage semantics,
deadness/ranking behavior, resolver behavior, or full audit orchestration. The
Node entrypoint must remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `checklist-facts.json.meta.schemaVersion` remains present and at least `2`;
- A2 function-size facts and E2 catch facts can be computed from a fresh AST
  pass even when upstream pipeline artifacts are absent;
- artifact-backed checklist items degrade cleanly to `available: false` and
  `gate: unknown` when `topology.json`, `fix-plan.json`, `triage.json`,
  `barrels.json`, or `shape-index.json` are missing;
- `_not_computed` explicitly lists skipped checklist items so the checklist
  walker cannot silently omit them;
- citation hints identify grounded facts and unavailable scan-range evidence;
- `_context_check_required` stays `false` for structural gates such as cycles
  and `true` for threshold/judgment gates such as function size;
- oversized function evidence records production/test/script roles and
  role-specific buckets;
- A5 decoupling ratio uses the full `crossSubmoduleEdges` list when present,
  downgrades healthy entry/test/script-to-engine flow, and keeps inverted
  engine-to-root flow as a structural smell;
- B1/B2 shape drift records exact duplicate groups, near shape candidates,
  concrete identities, shared fields, citation hints, and judgment-only
  `_not_computed` entries;
- B1 duplicate implementation records function-clone structure groups and
  near-function candidates as review-only evidence, not semantic merge proof;
- E2 silent catch evidence distinguishes empty catches, documented empty
  catches, non-empty anonymous catches, unused catch parameters, and used catch
  parameters;
- C5 lint-enforcement evidence can be grounded by a `no-restricted-imports`
  rule passed through `triage-repo.mjs`;
- pipeline-backed mode records `inputsPresent` bits for `topology.json`,
  `fix-plan.json`, `triage.json`, `barrels.json`, and `shape-index.json`.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- missing upstream artifacts must not crash checklist generation;
- missing upstream artifacts must not look like clean `ok` evidence;
- `_not_computed` items must not disappear from the artifact;
- exact shape drift must not become a broader semantic type-merge verdict;
- near shape candidates must remain review cues, not proof;
- function clone structure groups and near-function candidates must not imply
  semantic equivalence;
- healthy layered cross-submodule flow must not be treated as an automatic fix
  gate;
- inverted cross-submodule flow must not be downgraded by the healthy-flow
  exception;
- documented catches must not inflate the silent-catch gate;
- non-empty anonymous catches and unused catch parameters must stay visible as
  watch evidence;
- checklist pipeline evidence must not hide which upstream artifacts were
  actually present.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is temporary filesystem roots plus real producer
  entrypoints such as `checklist-facts.mjs`, `triage-repo.mjs`,
  `measure-topology.mjs`, `build-shape-index.mjs`,
  `build-function-clone-index.mjs`, `classify-dead-exports.mjs`,
  `rank-fixes.mjs`, and `check-barrel-discipline.mjs`.
- Shared helper code may create temp roots, write source files, run producers,
  read `checklist-facts.json`, and clean up.
- Shared helper code must not decide checklist gate semantics, shape drift,
  function clone review policy, topology decoupling, deadness/ranking, resolver
  behavior, cache identity, performance counters, or full audit orchestration.
- The mirror must not absorb `tests/test-module-reachability.mjs`,
  `tests/test-rank-fixes.mjs`, `tests/test-export-action-safety.mjs`,
  resolver suites, incremental/performance suites, pre-write cue-tier suites,
  or `tests/test-audit-repo.mjs`.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/checklist-facts.test.mjs`,
2. `npm run test:vitest:checklist-facts`,
3. candidate-board updates moving `tests/test-checklist-facts.mjs` from
   `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current Node assertion as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.
