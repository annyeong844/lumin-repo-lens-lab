# Vitest Mirror Goal

> **Status:** GOAL.
> **Date:** 2026-05-14.
> **Scope:** finish the Vitest mirror lane without hand-merging one suite at a
> time.

---

## Purpose

The project has outgrown one-off runner migration PRs. The Vitest pilot proved
that focused runner mirrors are useful, but repeating the same manual
review-page and implementation loop for every remaining Node suite would create
too much process overhead.

This page turns WT-24 into a goal track:

1. inventory every remaining Node `test-*.mjs` suite,
2. group suites by risk lane,
3. generate or batch review pages from stable metadata,
4. implement Vitest mirrors in bounded batches,
5. keep Node entrypoints and analyzer evidence contracts intact until a
   separate cleanup spec says otherwise.

This is not permission to bulk-convert every test in one PR. It is permission
to stop treating each suite as a bespoke project when a lane-level contract can
preserve the same safety.

## Current Counts

Inventory source: local filesystem scan on 2026-05-24.

| Metric                         | Count |
| ------------------------------ | ----: |
| Node `tests/test-*.mjs` suites |   165 |
| Existing Vitest mirror suites  |   176 |
| Remaining parked suites        |     2 |

Reviewed unimplemented suites:

- none currently.

Closure audit:

- [`vitest-mirror-closure-audit.md`](vitest-mirror-closure-audit.md)

Runner shortcut:

- `npm run test:audit-runtime-gate` is the Cargo-backed audit-runtime gate for
  migrated Rust-owned manifest behavior.
- `npm test` remains available as the serial default Node lane while remaining
  JS/TS producers are being retired.
- `npm run test:node:groups` is an opt-in maintainer shortcut implemented by
  [`scripts/run-tests-grouped.mjs`](../../scripts/run-tests-grouped.mjs) and
  covered by [`tests/test-run-tests-grouped.mjs`](../../tests/test-run-tests-grouped.mjs)
  plus [`tests/run-tests-grouped.test.mjs`](../../tests/run-tests-grouped.test.mjs).
  The 2026-05-24 dogfood run passed 165 suites across 12 groups in 362.8
  seconds with `--jobs 3`.

## Parked Lane Inventory

This is the current parked remainder, not a permanent taxonomy. Each future
review may refine suite placement, but the counts force the goal to stay
concrete.

| Lane                         | Count | Parked Suites                  | Batch Rule                                                                            |
| ---------------------------- | ----: | ------------------------------ | ------------------------------------------------------------------------------------- |
| Audit-repo umbrella          |     0 | none                           | Retired from the default Node gate; use Cargo-backed `npm run test:audit-runtime-gate`. |
| Deadness/ranking             |     0 | none                           | Park until graph-lens and action-proof review pages exist.                            |
| Deadness/ranking calibration |     0 | none                           | Require calibration-specific review before mirror work.                               |
| Cue-tier policy              |     1 | `test-pre-write-cue-tiers.mjs` | Known T1-T10 split mirrors are complete; keep the direct umbrella Node-authoritative. |

The parked lane total is 1. `test-pre-write-cue-tiers.mjs` is parked only as a
direct broad umbrella: its current T1-T10 contracts are covered by focused
Vitest mirrors recorded in the
[`vitest-pre-write-cue-tiers.md`](pilot-reviews/vitest-pre-write-cue-tiers.md)
split-track review and the
[`test-migration-candidate-board.md`](test-migration-candidate-board.md)
tracker rows. If a future inventory changes the parked count or adds a new
cue-tier behavior, update this page and `docs/lumin-wiki/log.md` in the same
PR.

## Batch Strategy

### Batch 0: Goal Infrastructure

Create this goal page, keep the JSONC review page open, and document the lane
inventory. No test implementation is added in this batch.

### Batch 1: Low-Risk Core/Parser/Helper Mirrors

Start with JSONC edge cases, then move through small focused helper/parser
suites. These batches may contain multiple mirrors when all suites:

- already have a small Node command,
- do not spawn the full audit pipeline,
- have fixture setup only,
- protect parser/helper contracts rather than analyzer absence claims.

### Batch 2: Canon And Check-Canon Families

Mirror canon/check-canon suites by source family. These suites have repeated
fixture shapes, but renderer and drift semantics must stay local to each source
family until a review page proves shared helpers are setup-only.

### Batch 3: Public Package And Hook Surfaces

Mirror plugin, skill, public-package, and hook suites after their host/public
surface boundaries are named. These suites are not deadness/resolver proof, but
they affect install and user entry behavior, so they should stay separate from
core parser batches.

### Batch 4: Pre/Post-Write Lifecycle

Mirror pre-write and post-write suites only after each lifecycle slice names
whether it protects JSON artifact shape, Markdown wording, evidence
availability, file delta labels, or cue-tier behavior. Cue-tier and service
operation sibling policies remain review-only unless a policy spec says
otherwise.

### Batch 5+: Analyzer-Sensitive Lanes

Resolver, deadness/ranking, performance/incremental, and scanner suites require
lane-specific review pages before implementation. These suites guard false
edges, false absence claims, cache identity, and action-safety proof. They are
the reason this goal is batched instead of a single conversion PR.

## Review Metadata Shape

Future review pages may be generated from metadata, but the generated content
must still answer these fields:

| Field                     | Required Meaning                                                                    |
| ------------------------- | ----------------------------------------------------------------------------------- |
| `suite`                   | Existing `tests/test-*.mjs` Node suite.                                             |
| `nodeCommand`             | Preserved command that must keep running.                                           |
| `vitestCommand`           | Focused command added by the mirror PR.                                             |
| `lane`                    | One of the inventory lanes above.                                                   |
| `protectedInvariant`      | The contract that must fail if the mirror weakens the suite.                        |
| `edgeCaseFailure`         | The non-happy-path regression the mirror preserves.                                 |
| `fixtureBoundary`         | What setup can be shared and what semantic logic must stay suite-local.             |
| `parkedNeighbors`         | Suites or workstreams that must not be absorbed into this mirror.                   |
| `validationCommands`      | Node, focused Vitest, all Vitest, formatting/doc guards as relevant.                |
| `runnerDiscoveryBoundary` | Assurance that Vitest still discovers only reviewed first-party `tests/*.test.mjs`. |

Generated review pages must be treated as drafts until the lane owner checks
that the invariant and edge-case failure are specific enough. Boilerplate is
not a substitute for the protected invariant.

## Implementation Rules

- Keep every Node suite runnable until a later cleanup spec retires it.
- Add focused Vitest commands for every mirrored suite.
- Keep `npm run test:vitest` scoped to reviewed `tests/*.test.mjs` files.
- Do not move analyzer semantics into shared test helpers.
- Shared helpers may create files, directories, JSON, commands, and cleanup
  wrappers, but they must not decide resolver, deadness, ranking, generated,
  public package, or performance meaning.
- A batch PR may include multiple mirrors only when every suite belongs to the
  same lane and has compatible fixture boundaries.
- A failing edge case must be represented as a named `it(...)` block, not hidden
  inside a broad happy-path mirror.
- If a Vitest mirror reveals behavior drift in the Node suite, stop and fix the
  behavior in a separate bugfix PR before continuing the migration.

## Completion Gate

The Vitest mirror goal is complete only when:

- every non-parked Node suite has a focused Vitest mirror,
- parked analyzer-sensitive suites have explicit deferral notes or review pages,
- `npm test` remains available,
- `npm run test:vitest` passes the complete reviewed mirror lane,
- `npm run check:test-doc` and `npm run check:doc-script-refs` pass,
- the candidate board no longer contains stale `REVIEWED` suites,
- the wiki records which Node suites, if any, are intentionally retained as the
  authoritative path.

## Immediate Next Step

The low-risk Lane A queue is complete for now, the Lane B canon batches listed
below are complete, and the Lane D CJS and framework/resource surface batches
are complete. The Lane D public/workspace surface batch is complete. Choose the
next non-parked lane batch before implementing another parked
performance/incremental suite. Do not enter parked analyzer-sensitive suites
directly. The export-surface guard mirror batch is complete and remains covered
by:

```text
node tests/test-definition-id-export.mjs
node tests/test-file-delta-export.mjs
node tests/test-classify-policies-export-surface.mjs
npm run test:vitest:definition-id-export
npm run test:vitest:file-delta-export
npm run test:vitest:classify-policies-export-surface
```

The parser/AST guard mirror batch is complete and remains covered by:

```text
node tests/test-classify-facts-ast.mjs
node tests/test-lang-matrix.mjs
npm run test:vitest:classify-facts-ast
npm run test:vitest:lang-matrix
```

The type escape evidence batch is complete and remains covered by:

```text
node tests/test-extract-ts-escapes.mjs
node tests/test-any-inventory.mjs
npm run test:vitest:extract-ts-escapes
npm run test:vitest:any-inventory
```

The smoke-uncovered artifact smoke mirror batch is complete and remains covered
by:

```text
node tests/test-smoke-uncovered.mjs
npm run test:vitest:smoke-uncovered
```

The hardcoding guard mirror is complete and remains covered by:

```text
node tests/test-hardcoding.mjs
npm run test:vitest:hardcoding
```

The audit-manifest export-surface guard mirror is complete and remains covered
by:

```text
node tests/test-audit-manifest-export-surface.mjs
npm run test:vitest:audit-manifest-export-surface
```

The definition-id canonical guard mirror is complete and remains covered by:

```text
node tests/test-definition-id-canonical.mjs
npm run test:vitest:definition-id-canonical
```

The shell-safety guard mirror is complete and remains covered by:

```text
node tests/test-shell-safety.mjs
npm run test:vitest:shell-safety
```

The evidence-honesty guard mirror is complete and remains covered by:

```text
node tests/test-evidence-honesty.mjs
npm run test:vitest:evidence-honesty
```

The Lane B helper-registry canon batch is complete and remains covered by:

```text
node tests/test-canon-draft-helpers.mjs
node tests/test-canon-draft-helper-registry.mjs
node tests/test-generate-canon-draft-cli-helpers.mjs
node tests/test-check-canon-helpers.mjs
npm run test:vitest:canon-draft-helpers
npm run test:vitest:canon-draft-helper-registry
npm run test:vitest:generate-canon-draft-cli-helpers
npm run test:vitest:check-canon-helpers
```

The Lane B naming canon batch is complete and remains covered by:

```text
node tests/test-canon-draft-naming.mjs
node tests/test-canon-draft-naming-structure.mjs
node tests/test-generate-canon-draft-cli-naming.mjs
node tests/test-check-canon-naming.mjs
npm run test:vitest:canon-draft-naming
npm run test:vitest:canon-draft-naming-structure
npm run test:vitest:generate-canon-draft-cli-naming
npm run test:vitest:check-canon-naming
```

The Lane B type-ownership canon batch is complete and remains covered by:

```text
node tests/test-canon-draft.mjs
node tests/test-canon-draft-type-ownership.mjs
node tests/test-generate-canon-draft-cli.mjs
node tests/test-check-canon-types.mjs
npm run test:vitest:canon-draft
npm run test:vitest:canon-draft-type-ownership
npm run test:vitest:generate-canon-draft-cli
npm run test:vitest:check-canon-types
```

The Lane B topology canon batch is complete and remains covered by:

```text
node tests/test-canon-draft-topology.mjs
node tests/test-canon-draft-topology-structure.mjs
node tests/test-generate-canon-draft-cli-topology.mjs
node tests/test-check-canon-topology.mjs
npm run test:vitest:canon-draft-topology
npm run test:vitest:canon-draft-topology-structure
npm run test:vitest:generate-canon-draft-cli-topology
npm run test:vitest:check-canon-topology
```

The Lane B canon draft integration batch is complete and remains covered by:

```text
node tests/test-canon-draft-integration.mjs
node tests/test-canon-draft-integration-helpers.mjs
node tests/test-canon-draft-integration-topology.mjs
npm run test:vitest:canon-draft-integration
npm run test:vitest:canon-draft-integration-helpers
npm run test:vitest:canon-draft-integration-topology
```

The Lane B canon drift contract batch is complete and remains covered by:

```text
node tests/test-canon-drift-parser-contract.mjs
node tests/test-canonical-fact-model-drift.mjs
npm run test:vitest:canon-drift-parser-contract
npm run test:vitest:canonical-fact-model-drift
```

The Lane B check-canon core batch is complete and remains covered by:

```text
node tests/test-check-canon-utils.mjs
node tests/test-check-canon-artifact.mjs
node tests/test-generate-check-canon-cli.mjs
node tests/test-check-canon-integration.mjs
npm run test:vitest:check-canon-utils
npm run test:vitest:check-canon-artifact
npm run test:vitest:generate-check-canon-cli
npm run test:vitest:check-canon-integration
```

The Lane G public package publish batch is complete and remains covered by:

```text
node tests/test-plugin-package.mjs
node tests/test-publish-public-plugin.mjs
node tests/test-github-actions-ci-policy.mjs
npm run test:vitest:plugin-package
npm run test:vitest:publish-public-plugin
npm run test:vitest:github-actions-ci-policy
```

The Lane G public skill-surface text batch is complete and remains covered by:

```text
node tests/test-skill-surface.mjs
npm run test:vitest:skill-surface
```

The Lane G generated skill-package batch is complete and remains covered by:

```text
node tests/test-skill-package.mjs
npm run test:vitest:skill-package
```

The Lane G host hook runtime batch is complete and remains covered by:

```text
node tests/test-hook-doctor.mjs
node tests/test-hook-runner-scripts.mjs
node tests/test-hook-path-safety.mjs
node tests/test-hook-id-safety.mjs
node tests/test-hook-event-store.mjs
node tests/test-hook-event-drain-renderer.mjs
node tests/test-hook-preimage-store.mjs
node tests/test-hook-ack-observer.mjs
node tests/test-hook-post-write-lite.mjs
npm run test:vitest:hook-doctor
npm run test:vitest:hook-runner-scripts
npm run test:vitest:hook-path-safety
npm run test:vitest:hook-id-safety
npm run test:vitest:hook-event-store
npm run test:vitest:hook-event-drain-renderer
npm run test:vitest:hook-preimage-store
npm run test:vitest:hook-ack-observer
npm run test:vitest:hook-post-write-lite
```

The Lane C post-write lifecycle batch is complete and remains covered by:

```text
node tests/test-post-write-artifact.mjs
node tests/test-post-write-cli.mjs
node tests/test-post-write-delta.mjs
node tests/test-post-write-incremental.mjs
node tests/test-post-write-integration.mjs
node tests/test-post-write-render.mjs
npm run test:vitest:post-write-artifact
npm run test:vitest:post-write-cli
npm run test:vitest:post-write-delta
npm run test:vitest:post-write-incremental
npm run test:vitest:post-write-integration
npm run test:vitest:post-write-render
```

The Lane C class-method pre-write surface batch is complete and remains covered
by:

```text
node tests/test-class-method-index-prototype-names.mjs
node tests/test-class-method-prewrite-surface.mjs
npm run test:vitest:class-method-index-prototype-names
npm run test:vitest:class-method-prewrite-surface
```

The Lane D CJS surface batch is complete and remains covered by:

```text
node tests/test-extract-cjs-consumer.mjs
node tests/test-extract-cjs-export-surface.mjs
node tests/test-cjs-export-surface-artifact.mjs
node tests/test-cjs-classification.mjs
node tests/test-cjs-integration.mjs
npm run test:vitest:extract-cjs-consumer
npm run test:vitest:extract-cjs-export-surface
npm run test:vitest:cjs-export-surface-artifact
npm run test:vitest:cjs-classification
npm run test:vitest:cjs-integration
```

The Lane D framework/resource surface batch is complete and remains covered by:

```text
node tests/test-framework-resource-surfaces.mjs
node tests/test-build-framework-resource-surfaces.mjs
node tests/test-framework-policy-facts.mjs
node tests/test-framework-policy-matrix.mjs
npm run test:vitest:framework-resource-surfaces
npm run test:vitest:build-framework-resource-surfaces
npm run test:vitest:framework-policy-facts
npm run test:vitest:framework-policy-matrix
```

The Lane D public/workspace surface batch is complete and remains covered by:

```text
node tests/test-public-surface.mjs
node tests/test-public-deep-import-risk.mjs
node tests/test-workspace-no-exports.mjs
node tests/test-mdx-consumers.mjs
npm run test:vitest:public-surface
npm run test:vitest:public-deep-import-risk
npm run test:vitest:workspace-no-exports
npm run test:vitest:mdx-consumers
```

The Lane H artifact-output presentation batch is complete and remains covered
by:

```text
node tests/test-topology-mermaid.mjs
node tests/test-sarif-fix-plan.mjs
npm run test:vitest:topology-mermaid
npm run test:vitest:sarif-fix-plan
```

The Lane H call-graph evidence batch is complete and remains covered by:

```text
node tests/test-call-graph-bounded.mjs
node tests/test-call-graph-parse-errors.mjs
node tests/test-call-graph-truncation-defense.mjs
npm run test:vitest:call-graph-bounded
npm run test:vitest:call-graph-parse-errors
npm run test:vitest:call-graph-truncation-defense
```

The threshold metadata batch is complete and remains covered by:

```text
node tests/test-threshold-policies.mjs
node tests/test-calibration-corpora.mjs
npm run test:vitest:threshold-policies
npm run test:vitest:calibration-corpora
```

The pre-write inventory hook batch is complete and remains covered by:

```text
node tests/test-pre-write-inventory-hook.mjs
npm run test:vitest:pre-write-inventory-hook
```

The pre-write lookup contracts batch is complete and remains covered by:

```text
node tests/test-pre-write-lookup-dep.mjs
node tests/test-pre-write-lookup-file.mjs
node tests/test-pre-write-lookup-shape.mjs
node tests/test-pre-write-shape-index.mjs
npm run test:vitest:pre-write-lookup-dep
npm run test:vitest:pre-write-lookup-file
npm run test:vitest:pre-write-lookup-shape
npm run test:vitest:pre-write-shape-index
```

The pre-write input contracts batch is complete and remains covered by:

```text
node tests/test-pre-write-intent.mjs
node tests/test-pre-write-canonical-parser.mjs
npm run test:vitest:pre-write-intent
npm run test:vitest:pre-write-canonical-parser
```

The pre-write inline extraction batch is complete and remains covered by:

```text
node tests/test-inline-pattern-index.mjs
node tests/test-pre-write-inline-patterns.mjs
npm run test:vitest:inline-pattern-index
npm run test:vitest:pre-write-inline-patterns
```

The direct pre-write advisory lifecycle batch is complete and remains covered
by:

```text
node tests/test-pre-write-advisory-artifact.mjs
node tests/test-pre-write-bootstrap.mjs
node tests/test-pre-write-cli.mjs
node tests/test-pre-write-drift.mjs
node tests/test-pre-write-integration.mjs
npm run test:vitest:pre-write-advisory-artifact
npm run test:vitest:pre-write-bootstrap
npm run test:vitest:pre-write-cli
npm run test:vitest:pre-write-drift
npm run test:vitest:pre-write-integration
```

The audit-repo command lifecycle wrapper batch is complete and remains covered
by:

```text
node tests/test-audit-repo-canon-draft.mjs
node tests/test-audit-repo-check-canon.mjs
node tests/test-audit-repo-pre-write.mjs
node tests/test-audit-repo-post-write.mjs
npm run test:vitest:audit-repo-canon-draft
npm run test:vitest:audit-repo-check-canon
npm run test:vitest:audit-repo-pre-write
npm run test:vitest:audit-repo-post-write
```

The audit-repo incremental forwarding batch is complete and remains covered by:

```text
node tests/test-audit-repo-symbol-incremental.mjs
node tests/test-function-clone-audit-forwarding.mjs
npm run test:vitest:audit-repo-symbol-incremental
npm run test:vitest:function-clone-audit-forwarding
```

The incremental core helpers batch is complete and remains covered by:

```text
node tests/test-incremental-cache-store.mjs
node tests/test-incremental-snapshot.mjs
node tests/test-incremental.mjs
npm run test:vitest:incremental-cache-store
npm run test:vitest:incremental-snapshot
npm run test:vitest:incremental-legacy-cache
```

The JS module edge scanner batch is complete and remains covered by:

```text
node tests/test-js-module-edge-scanner.mjs
npm run test:vitest:js-module-edge-scanner
```

The function clone incremental parked-suite mirror is complete and remains
covered by:

```text
node tests/test-function-clone-incremental.mjs
npm run test:vitest:function-clone-incremental
```

The shape-index incremental parked-suite mirror is complete and remains covered
by:

```text
node tests/test-shape-index-incremental.mjs
npm run test:vitest:shape-index-incremental
```

The symbol-graph incremental parked-suite mirror is complete and remains
covered by:

```text
node tests/test-symbol-graph-incremental.mjs
npm run test:vitest:symbol-graph-incremental
```

The any-inventory incremental parked-suite mirror is complete and remains
covered by:

```text
node tests/test-any-inventory-incremental.mjs
npm run test:vitest:any-inventory-incremental
```

The producer artifact builders batch is complete and remains covered by:

```text
node tests/test-build-shape-index.mjs
node tests/test-build-function-clone-index.mjs
npm run test:vitest:build-shape-index
npm run test:vitest:build-function-clone-index
```

The checklist-facts batch is complete and remains covered by:

```text
node tests/test-checklist-facts.mjs
npm run test:vitest:checklist-facts
```

The mode-dispatch mirror batch is complete and remains covered by:

```text
node tests/test-mode-dispatch.mjs
npm run test:vitest:mode-dispatch
```

The resolver path lookup batch is complete and remains covered by:

```text
node tests/test-resolver-paths.mjs
node tests/test-tsconfig-paths-scoped.mjs
node tests/test-wildcard.mjs
npm run test:vitest:resolver-paths
npm run test:vitest:tsconfig-paths-scoped
npm run test:vitest:wildcard
```

The topology edge lens batch is complete and remains covered by:

```text
node tests/test-dynamic-import.mjs
node tests/test-type-only-reexport.mjs
node tests/test-topology-producer-cross-edges.mjs
npm run test:vitest:dynamic-import
npm run test:vitest:type-only-reexport
npm run test:vitest:topology-producer-cross-edges
```

The entry surface artifact suite is complete and remains covered by:

```text
node tests/test-entry-surface-artifact.mjs
npm run test:vitest:entry-surface-artifact
```

The module reachability graph-lens mirror is complete and remains covered by:

```text
node tests/test-module-reachability.mjs
npm run test:vitest:module-reachability
```

The rank-fixes action-proof mirror is complete and remains covered by:

```text
node tests/test-rank-fixes.mjs
npm run test:vitest:rank-fixes
```

The precision corpus mirror is complete and remains covered by:

```text
node tests/test-corpus.mjs
npm run test:vitest:corpus
```

The P6 measurement calibration mirror is complete and remains covered by:

```text
node tests/test-p6-measurement.mjs
npm run test:vitest:p6-measurement
```

The P6 member precision mirror is complete and remains covered by:

```text
node tests/test-p6-member-precision.mjs
npm run test:vitest:p6-member-precision
```

The P6 SAFE_FIX calibration mirror is complete and remains covered by:

```text
node tests/test-p6-safe-fix-calibration.mjs
npm run test:vitest:p6-safe-fix-calibration
```

Keep package publishing, skill package/surface tests, behavior tests for the
underlying algorithms, `test-audit-repo.mjs`, broader orchestrator behavior,
resolver expansion, reachability ranking, SAFE_FIX action proof,
generated/framework resource packs, symlink resolution, and
performance/incremental cache identity out of the next PR.

The Python conventions suite is complete and remains covered by:

```text
node tests/test-python-conventions.mjs
npm run test:vitest:python-conventions
```

The `test-audit-repo.mjs` umbrella suite has a split/park review and is retired
from the default `npm test` gate. Do not add a direct `test:vitest:audit-repo`
mirror. The known focused split mirrors are complete, and
Cargo-backed `npm run test:audit-runtime-gate` is the migrated audit-repo runtime
gate; focused Vitest mirrors remain reference coverage while JS/TS producers are
being retired. Any future audit-repo product-pass behavior needs a fresh split
review before a new mirror.

The audit-repo blind-zone/confidence split track is complete and remains
covered by:

```text
npm run test:node:legacy-audit-repo
npm run test:vitest:audit-repo-blind-zones
```

The audit-repo scan range/self-audit exclusions split track is complete and
remains covered by:

```text
npm run test:node:legacy-audit-repo
npm run test:vitest:audit-repo-scan-range
```

The audit-repo lifecycle artifact collection split track is complete and
remains covered by:

```text
npm run test:node:legacy-audit-repo
npm run test:vitest:audit-repo-lifecycle-artifacts
```

The audit-repo full-profile staleness/artifacts split track is complete and
remains covered by:

```text
npm run test:node:legacy-audit-repo
npm run test:vitest:audit-repo-full-profile-staleness
```

The classification gates suite is complete and remains covered by:

```text
node tests/test-classification-gates.mjs
npm run test:vitest:classification-gates
```

The symlink aliasing suite is complete and remains covered by:

```text
node tests/test-symlink-aliasing.mjs
npm run test:vitest:symlink-aliasing
```

The remaining Node-authoritative suites are parked and mapped in
[`vitest-mirror-closure-audit.md`](vitest-mirror-closure-audit.md). Do not add
more direct Vitest mirrors from that remainder until the target suite receives a
suite-specific review page.
