# Vitest Module Reachability Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-20.
> **Pilot candidate:** `tests/test-module-reachability.mjs`

---

## Purpose

This review decided whether `tests/test-module-reachability.mjs` may move to a
focused Lane E Vitest mirror. It is implemented by
`tests/module-reachability.test.mjs`.

The suite protects `module-reachability.json`, the entry-rooted runtime/type
graph lenses, bounded traversal behavior, and the audit summary/review-pack
wording for entry-unreachable strongly connected components. It is safe to
review as a single-suite batch because the fixture is intentionally narrow: one
package export entry, one runtime import chain, one type-only import, one
isolated file, and one entry-unreachable runtime cycle.

This review must stay separate from export ranking, `SAFE_FIX` proof,
namespace re-export precision, resolver blind-zone relevance, generated or
framework blockers, and the broad `test-audit-repo.mjs` product-pass suite.

## Reviewed Evidence

| Suite                                | Preserved Node Command                    | Proposed Focused Vitest Command           | Surface Under Review                                        |
| ------------------------------------ | ----------------------------------------- | ----------------------------------------- | ----------------------------------------------------------- |
| `tests/test-module-reachability.mjs` | `node tests/test-module-reachability.mjs` | `npm run test:vitest:module-reachability` | `module-reachability.json` and audit reachability surfacing |
| focused mirror                       | _implemented_                             | `tests/module-reachability.test.mjs`      | module reachability mirror file                             |

Current Node evidence checked for this review:

```text
node tests/test-module-reachability.mjs # 19 passed, 0 failed
npm run test:vitest:module-reachability # 4 passed, 0 failed
```

Goal lane: Lane E, deadness/ranking/calibration. This review covers only file
reachability evidence and the summary/review-pack cue that describes
entry-unreachable SCCs as review evidence.

## Result

This suite is implemented as one focused Vitest mirror.

The implementation adds one mirror file and one focused script while keeping
the Node entrypoint runnable. Every mirrored assertion stays tied to
`module-reachability.json`, `manifest.json.commandsRun`, `manifest.json`
artifact listing, `audit-summary.latest.md`, or `audit-review-pack.latest.md`.
The mirror does not treat unreachable files or entry-unreachable SCCs as
export-level `SAFE_FIX` proof.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `module-reachability.json.meta.tool` remains
  `build-module-reachability.mjs`;
- `module-reachability.json.meta.entrySurfaceFile` points to
  `entry-surface.json`;
- runtime BFS reaches the entry file and transitive runtime/value imports;
- runtime BFS excludes type-only dependencies;
- type reachability includes type-only dependencies;
- `reachableFiles` is the union of runtime and type reachability;
- isolated files become `unreachableFiles` only when traversal completed;
- clean runs do not produce `boundedOutFiles`;
- `completenessBySubmodule` is copied from entry-surface evidence;
- `meta.supports.unreachableStronglyConnectedComponents` is true;
- entry-unreachable runtime SCCs are recorded under
  `unreachableStronglyConnectedComponents[]` with
  `kind: "entry-unreachable-scc"` and `graph: "runtime"`;
- unreachable SCC summary counts both groups and files;
- emergency traversal caps record `boundedOutReason`;
- capped traversal sends unvisited files to `boundedOutFiles`, not
  `unreachableFiles`;
- quick `audit-repo.mjs` runs `build-module-reachability.mjs`;
- quick audit artifacts list `module-reachability.json`;
- pipeline-produced reachability keeps the isolated file unreachable;
- `audit-summary.latest.md` surfaces unreachable SCCs as review evidence;
- `audit-review-pack.latest.md` mirrors unreachable SCC evidence in the
  dead-surface lane.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- mixing type-only imports into runtime reachability must fail;
- dropping type reachability for type-only imports must fail;
- marking unvisited files as unreachable when traversal was bounded out must
  fail;
- losing `boundedOutReason` must fail;
- losing entry-surface submodule completeness must fail;
- dropping entry-unreachable SCC detection must fail;
- blanket-treating intra-cycle imports as liveness from an entry must fail;
- wording that turns SCC evidence into export `SAFE_FIX` proof must fail;
- dropping the quick-audit producer hook must fail;
- dropping the audit artifact listing must fail;
- hiding the isolated file from pipeline reachability must fail;
- dropping the summary or review-pack review cue must fail.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The Node suite remains runnable and authoritative until a later cleanup spec
  retires it.
- The fixture boundary is temporary repo creation, package exports, direct
  producer invocation, quick/full audit invocation, and JSON/Markdown artifact
  reads.
- Shared setup may write fixture files, run `build-symbol-graph.mjs`, run
  `build-entry-surface.mjs`, run `build-module-reachability.mjs`, run
  `audit-repo.mjs`, read `module-reachability.json`, read `manifest.json`, and
  read audit summary/review-pack Markdown.
- Shared helpers must not decide resolver semantics, symbol consumer precision,
  dead-export classification, action-safety proof, ranking tiers, framework
  blockers, generated blockers, namespace fan-in, or CJS opacity.
- The mirror must not change `build-module-reachability.mjs`,
  `audit-repo.mjs`, summary/review-pack wording, ranking, action safety,
  resolver behavior, entry-surface behavior, or public package behavior.
- The mirror must not absorb `tests/test-export-action-safety.mjs`,
  `tests/test-rank-fixes.mjs`, `tests/test-namespace-reexport-deadness.mjs`,
  P6 calibration/member precision suites, resolver blind-zone suites,
  generated/framework blocker suites, CJS opacity suites, or
  `tests/test-audit-repo.mjs`.

## Implementation Notes

- Prefer one Vitest file: `tests/module-reachability.test.mjs`.
- Add one focused script: `test:vitest:module-reachability`.
- Keep assertion labels close to the Node labels `E1` through `E19`.
- Keep the `App.ts` / `Modal.ts` cycle explicit; it is the proof that
  entry-unreachable SCCs are review evidence and that intra-cycle imports are
  not enough to prove entry reachability.
- Keep the type-only edge explicit; it is the proof that runtime and type graph
  lenses stay separate.
- Keep the `--max-files-visited 1` fixture explicit; it is the proof that
  bounded traversal creates uncertainty rather than false unreachable claims.

## Validation Commands

The implementation PR must run:

```text
node tests/test-module-reachability.mjs
npm run test:vitest:module-reachability
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-module-reachability.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
```

Before merge, the implementation should also keep the broader runner lane
green:

```text
npm run check
npm run lint
npm run test:vitest
npm test
```

## Non-Goals

- Do not change module reachability logic.
- Do not change audit summary or review-pack wording.
- Do not add export ranking or action-safety proof.
- Do not promote entry-unreachable SCCs to `SAFE_FIX`.
- Do not merge runtime and type graph lenses.
- Do not expand resolver, CJS, generated, public, or framework blocker
  behavior.
- Do not convert the broad deadness/ranking lane in this PR.
