# canonical/audit-core.md

> **Role:** canonical owner map for Rust audit orchestration and manifest evidence migration.
> **Owner:** this file.
> **Status:** staged Rust manifest projection migration.
> **Last updated:** 2026-07-01

## Scope

`lumin-audit-core` owns typed audit artifact registry, manifest evidence
summary contracts, manifest metadata projection, initial manifest root-shell
projection, manifest evidence refresh patch projection, final manifest summary
patch projection, `manifest.json.rustAnalysis` run/evidence merge projection,
current Rust-analysis artifact usability projection for produced-artifact lists,
typed `commandsRun` / `skipped` runtime-log shape, the audit orchestration plan
contract, base audit child-process executor core, typed orchestration event
ledger, producer-performance artifact construction from completed execution
observations, artifact-size measurement from JS-supplied produced artifact
names, artifact-read metric summary projection from JS-supplied read
observations, base producer phase timing sidecar reads, lifecycle summary projection,
orchestration result summary projection, lifecycle strict exit-policy
projection, typed pre-write engine routing, and the migrated Rust pre-write /
canon-draft / check-canon / post-write lifecycle child-process wrappers that
are not source-language analysis.

It does not own JS/TS producer behavior, Rust source-health syntax analysis,
Cargo semantic oracle behavior outside the explicitly migrated canon-draft
lifecycle child-process wrapper, check-canon lifecycle child-process wrapper,
post-write lifecycle child-process wrapper, Rust pre-write lifecycle wrapper,
live artifact-read observation,
lifecycle phase timing reads, human
companion rendering, or final `manifest.json` writing yet.

## Remaining JS-Owned Manifest Boundaries

These fields are deliberately not Rust-owned yet. They need lane-specific parity
or orchestration ownership before migration.

| Manifest area | Current owner | Why it stays JS-owned for now | Next migration gate |
|---|---|---|---|
| `manifest.json.blindZones` | `_lib/blind-zones.mjs` through `_lib/audit-manifest.mjs` | Blind-zone detection combines TS/JS producer semantics from `triage.json`, `symbols.json`, `dead-classify.json`, `entry-surface.json`, resolver diagnostics, and Rust analysis availability. Rust audit-core must not reinterpret those claims until parity is checked. | Follow `docs/superpowers/specs/2026-07-01-blind-zones-audit-core-parity-design.md`: compare JS blind-zone outputs against a typed Rust port on protected fixtures and real artifacts, including missing/partial producer artifacts, before changing the owner. |
| Producer runtime observation values (`commandsRun`, `skipped`) | Base audit profile: `orchestration_executor.rs` through the `execute-base-plan` wrapper; pre-write engine selection: `pre_write_routing.rs` through the `pre-write-route` wrapper; Rust pre-write lifecycle raw block: `pre_write_lifecycle.rs` through the `execute-rust-pre-write` wrapper; canon-draft lifecycle raw block: `canon_draft_lifecycle.rs` through the `execute-canon-draft` wrapper; check-canon lifecycle raw block: `check_canon_lifecycle.rs` through the `execute-check-canon` wrapper; post-write lifecycle raw block: `post_write_lifecycle.rs` through the `execute-post-write` wrapper; remaining lifecycle helpers: `audit-repo.mjs` | Rust now owns base-step skip decisions, statuses, stderr snippets, wall-clock measurements, orchestrator memory snapshots, typed pre-write engine routing from requested engine plus intent language, Rust pre-write analyzer child execution plus advisory projection, canon-draft child execution aggregation, check-canon child execution aggregation, and post-write child execution plus delta-summary projection. JS still reads the original pre-write intent file/stdin, builds executor requests, invokes Rust CLIs, owns JS/TS pre-write lifecycle observations, and assembles the final manifest. | Migrate remaining lifecycle helpers only with their own raw-block parity plans; keep JS/TS producer internals outside audit-core. |
| Producer performance measurement inputs | Base audit profile: `orchestration_events.rs` from typed runtime observations; artifact-read summary math: `artifact_read_metrics.rs`; lifecycle helpers and JSON artifact read observation: `audit-repo.mjs` | Rust owns base child status/wall/stderr/memory observations, base producer phase sidecar reads, artifact-read metric summary projection from JS-supplied observations, artifact-size measurement for JS-supplied produced artifact names, and final `producer-performance.json` construction. JS still observes ordinary JSON artifact reads, builds the runtime request, owns JS/TS pre-write lifecycle observations, and assembles the final manifest. | Move remaining lifecycle observations and live artifact-read observation only with explicit Rust owners. |
| Raw lifecycle blocks (`preWrite`, `postWrite`, `canonDraft`, `checkCanon`) | `preWrite` engine selection: `pre_write_routing.rs` through the `pre-write-route` wrapper; `preWrite` Rust engine: `pre_write_lifecycle.rs` through the `execute-rust-pre-write` wrapper; `preWrite` JS/TS engine: `audit-repo.mjs` plus `pre-write.mjs`; `canonDraft`: `canon_draft_lifecycle.rs` through the `execute-canon-draft` wrapper; `checkCanon`: `check_canon_lifecycle.rs` through the `execute-check-canon` wrapper; `postWrite`: `post_write_lifecycle.rs` through the `execute-post-write` wrapper | Rust owns the typed engine route decision from requested engine plus intent JSON `language`, including explicit mismatch hard-stops and route-only `language` stripping before the Rust child. Rust pre-write records `executionOwner: "lumin-audit-core"` and preserves the checked JS helper contract for the Rust engine: analyzer argv/stdin, native artifact latest copy, advisory path projection, JS-supplied file inventory and failure pass-through, rustPreWrite capability fields, child failure block projection, and product-mode streaming stdout/stderr through the Rust CLI result-file bridge. The JS/TS pre-write engine remains JS-owned because `pre-write.mjs` owns JS/TS producer semantics. `canonDraft`, `checkCanon`, and `postWrite` preserve their checked lifecycle contracts. | Migrate the JS/TS pre-write engine only after its producer semantics have a parity plan; do not make audit-core reinterpret JS/TS pre-write artifacts. |
| Lifecycle strict exit policy | `lifecycle_exit_policy.rs` through the `lifecycle-exit-policy` wrapper | Rust owns the typed projection from the current orchestrator exit code plus raw post-write lifecycle block to strict post-write exit-code/stderr decisions. It does not read artifacts or execute producers. | Move additional lifecycle exit policies here only after their raw-block owner is typed. |
| Human companion artifacts (`auditSummary`, `reviewPack`, `topologyMermaid`) | `audit-repo.mjs` plus renderer modules | These are presentation/rendering outputs, not typed manifest evidence summaries. | Migrate only through a separate renderer parity plan. |
| Final `manifest.json` file write | `audit-repo.mjs` | The manifest root still joins Rust summaries with JS producer orchestration and optional pre/post-write lifecycle blocks. | Migrate after all manifest fields have typed Rust owners or an explicit Rust orchestrator owns the final write. |
| Lifecycle child process execution | `preWrite` Rust engine selection: `pre_write_routing.rs`; `preWrite` Rust engine execution: `pre_write_lifecycle.rs`; `canonDraft`: `canon_draft_lifecycle.rs`; `checkCanon`: `check_canon_lifecycle.rs`; `postWrite`: `post_write_lifecycle.rs`; remaining lifecycle helpers: `audit-repo.mjs` | Rust audit-core owns the base audit profile executor, pre-write routing, Rust pre-write analyzer child execution, the canon-draft lifecycle child spawner, the check-canon lifecycle child spawner, and the post-write child spawner. JS still owns the JS/TS pre-write engine and final wrapper assembly. | Migrate the JS/TS pre-write engine only after its producer semantics have a parity plan. |

## Canonical Rust Modules

| File | Owns | Must not own |
|---|---|---|
| `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs` | Known artifact names, dynamic artifact filename matching, deterministic produced-artifact enumeration, and current Rust-analysis artifact usability from `manifest.json.rustAnalysis` blocks | child process execution, JSON artifact parsing beyond artifact usability fields |
| `experiments/rust-main/lumin-audit-core/src/artifact_measurement.rs` | Producer-performance artifact-size measurement for JS-supplied produced artifact names, including best-effort missing/non-file exclusion and largest-artifact projection | produced-artifact discovery, artifact JSON parsing, artifact-read timing, producer execution |
| `experiments/rust-main/lumin-audit-core/src/artifact_read_metrics.rs` | `artifact-read-metrics.v1` summary projection from JS-supplied artifact-read observations and phase sidecar read observations: totals, parse failures, path naming, largest-read projection, slowest-parse projection, and per-artifact aggregation | live artifact-read observation, JSON artifact parsing, producer execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/artifact_summaries.rs` | `manifest.json.frameworkResourceSurfaces`, `manifest.json.unusedDependencies`, and `manifest.json.blockClones` projections from already-produced artifact JSON | framework/resource scanning, unused-dependency analysis, block-clone detection |
| `experiments/rust-main/lumin-audit-core/src/blind_zones.rs` | Typed `manifest.json.blindZones` parity projection from JS-owned producer artifacts behind a fixture-based CLI gate | JS/TS producer behavior, final manifest wiring before parity, console summary rendering |
| `experiments/rust-main/lumin-audit-core/src/canon_draft_lifecycle.rs` | `manifest.canonDraft` raw lifecycle block execution for `--canon-draft`: source selection, unknown-source failure, `generate-canon-draft.mjs` child spawning, per-source exit projection, fallback draft path projection, and advisory exit code result | canon draft source-specific content generation, markdown proposal rendering, check-canon drift reading, pre/post-write lifecycle execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/check_canon_lifecycle.rs` | `manifest.checkCanon` raw lifecycle block execution for `--check-canon`: source selection, unknown-source failure, all/per-source `check-canon.mjs` child spawning, `canon-drift.json` per-source projection, logical per-source exit projection, and advisory/strict exit code result | canon drift detection, canonical parser semantics, drift report rendering, pre/post-write lifecycle execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/pre_write_routing.rs` | Typed pre-write engine routing from requested engine, intent flag, and already-read intent JSON text: `auto`/`js`/`rust` selection, intent language validation, explicit mismatch hard-stops, child intent flag/input projection, and removal of route-only `language` before Rust engine stdin | intent file/stdin reading, JS/TS `pre-write.mjs` producer semantics, Rust analyzer internals, child process execution, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle.rs` | `manifest.preWrite` raw lifecycle block execution for the Rust pre-write engine: `lumin-rust-analyzer pre-write` child spawning, intent stdin forwarding, native artifact latest copy, advisory JSON construction, JS-supplied file inventory and failure pass-through, Rust pre-write capability fields, child failure projection, and product-mode inherited stdout/stderr with result JSON written out-of-band | JS/TS `pre-write.mjs` producer semantics, scan-scope walking or source inventory interpretation, Rust analyzer internals, post-write delta semantics, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/post_write_lifecycle.rs` | `manifest.postWrite` raw lifecycle block execution for `--post-write`: missing-advisory hard-stop, existing `post-write.mjs` child spawning, optional delta-out/no-fresh-audit/scan/incremental argv forwarding, child failure projection, delta path projection, product-mode inherited stdout/stderr with result JSON written out-of-band, and best-effort post-write delta summary projection | post-write delta computation, type-escape/file-delta semantics, markdown rendering, pre-write advisory construction, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/living_audit.rs` | `manifest.json.livingAudit` projection from known living-audit document candidate paths under the audited root | audit document authoring, final answer policy, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/manifest_core.rs` | `manifest.json.scanRange`, `manifest.json.confidence`, and `manifest.json.sfcEvidence` projections from already-produced `triage.json` and `symbols.json` | blind-zone detection, living-audit document discovery, producer execution |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics.rs` | `manifest.json.resolverDiagnostics` projection from already-produced `symbols.json`, `resolver-capabilities.json`, and `resolver-diagnostics.json` | module resolution, blocked-hint production, blind-zone detection |
| `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs` | `rust-analyzer-health.latest.json` manifest summary projection, root mismatch, invalid-shape, complete/available status, and `manifest.json.rustAnalysis` merge projection from JS-observed Rust analyzer run state plus already-produced evidence summary | Rust source parsing, source-health analysis, Cargo oracle execution, child process execution |
| `experiments/rust-main/lumin-audit-core/src/generated_artifacts.rs` | `manifest.json.generatedArtifacts` projection from already-produced `symbols.json`, generated-artifact mode validation, generated miss grouping, blind-zone grouping, and present/prepared out-of-scope evidence | package resolution, generator execution, generated-artifact producer evidence construction |
| `experiments/rust-main/lumin-audit-core/src/lifecycle_exit_policy.rs` | Strict lifecycle exit-code/stderr projection from the current orchestrator exit code, strict post-write flags, and already-built raw `postWrite` block | raw lifecycle block construction, producer execution, post-write delta semantics, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/lifecycle.rs` | `manifest.json.lifecycle` projection from completed raw `preWrite`, `postWrite`, `canonDraft`, and `checkCanon` manifest blocks | lifecycle child execution, advisory generation, post-write delta production, canon draft/check producer behavior, raw lifecycle block ownership |
| `experiments/rust-main/lumin-audit-core/src/manifest_evidence.rs` | Composition of Rust-owned `manifest.json` evidence fields from already-produced artifacts, excluding `blindZones` | blind-zone detection, producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_final.rs` | Final pre-write `manifest.json` summary patch projection for `performance`, `orchestration`, and `artifactsProduced` from already-produced `producer-performance.json`, output artifact names, and the merged Rust analysis block | producer execution, producer-performance artifact writing, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_meta.rs` | `manifest.json.meta` shape projection from JS-provided run timestamp, profile, root, and output values | clock reading, profile flag parsing before CLI dispatch, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_root.rs` | Initial `manifest.json` root shell projection and manifest evidence refresh patch projection from Rust-owned summary fields, typed JS-observed `commandsRun` / `skipped` runtime logs, produced-artifact list, and JS-owned pass-through `blindZones` blocks | producer execution, blind-zone interpretation, lifecycle raw block construction, human companion renderers, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs` | Typed `lumin-audit-orchestration-ledger.v1` input contract, typed runtime-observation-to-ledger projection for base audit runs, base producer phase timing sidecar reads with artifact-read metric merging through `artifact_read_metrics.rs`, and `producer-performance.json` construction from completed execution observations | child process execution, lifecycle telemetry collection, live artifact-read observation, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs` | Base audit child-process execution for planned base pipeline steps, filesystem precondition evaluation using the existing plan reasons, JS/MJS child argv construction, typed `commandsRun` / `skipped` value production, `LedgerEvent` value production from the same observations, child status/wall/stderr observation, and orchestrator memory snapshots before and after base children | JS/TS producer internals, lifecycle child execution, artifact-read timing, phase timing reads, human renderers, `blindZones`, final `manifest.json` writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs` | Typed audit profile command graph, lifecycle request plan, profile/SARIF/base-pipeline skip semantics, base-step `executionOwner` metadata consumed by `orchestration_executor.rs`, and lifecycle `executionOwner` metadata consumed by the JS wrapper | child process execution, filesystem precondition evaluation, command telemetry, producer-performance measurement |
| `experiments/rust-main/lumin-audit-core/src/orchestration_result.rs` | `manifest.json.orchestration` projection from the typed `producer-performance.json` source shape, including execution status counts, required/optional failure counts, skipped counts, and capped examples | child process execution, live telemetry collection, raw `commandsRun`/`skipped` value production, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/producer_performance.rs` | `manifest.json.performance` projection from already-produced `producer-performance.json` | producer execution, memory measurement, artifact read measurement, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/scan_scope.rs` | Audit manifest scan-scope path inclusion policy used by migrated manifest summaries, matching the JS `scanScopeStatusForPath` contract | source walking, parsing, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/cli.rs` | CLI request parsing and stdout JSON dispatch for audit-core commands | producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/lib.rs` | public library exports for audit manifest wrappers | ad hoc JSON shape construction outside owned modules |

## Rules

- Most audit-core modules read already-produced artifacts. `orchestration_executor.rs`
  is the explicit exception for base audit profile child execution, and
  `canon_draft_lifecycle.rs` and `check_canon_lifecycle.rs` are explicit
  exceptions for lifecycle child execution. `pre_write_lifecycle.rs` is the
  explicit Rust pre-write exception: it may run `lumin-rust-analyzer pre-write`
  and project the checked Rust advisory block, but it must not own JS/TS
  `pre-write.mjs` producer semantics. `post_write_lifecycle.rs` is the explicit
  post-write exception: it may run the existing `post-write.mjs` entrypoint and
  project the checked raw lifecycle block, but it must not own post-write delta
  producer semantics. These modules run existing producer entrypoints but do
  not reinterpret source-language producer semantics.
- Audit-core may own orchestration routing separately from execution. The plan
  is declarative profile/lifecycle evidence; `pre_write_routing.rs` owns only
  typed pre-write engine selection from an already-read intent payload; Rust
  child execution must live in a named owner module with an artifact-visible
  owner boundary.
- Audit-core may emit JSON to stdout for JS compatibility, but the library owns
  typed Rust structs first.
- JS/TS producer lanes remain JS-owned until a lane-specific Rust parity proof
  exists.
- Do not add elapsed-time caps, repository-size caps, or timeout logic.
- Unknown JSON fields in consumed artifacts must be ignored.
- Missing or malformed migrated inputs must become explicit status, not silent
  zero evidence.
