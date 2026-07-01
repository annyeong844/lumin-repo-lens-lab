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
typed `commandsRun` / `skipped` runtime-log shape, the audit orchestration plan
contract, typed orchestration event ledger,
producer-performance artifact construction from completed execution
observations, lifecycle summary projection, and orchestration result summary
projection that are not source-language analysis.

It does not own JS/TS producer behavior, Rust source-health syntax analysis,
Cargo semantic oracle behavior, child-process execution, live telemetry
collection, artifact-size measurement, artifact-read measurement, or final
`manifest.json` writing yet.

## Remaining JS-Owned Manifest Boundaries

These fields are deliberately not Rust-owned yet. They need lane-specific parity
or orchestration ownership before migration.

| Manifest area | Current owner | Why it stays JS-owned for now | Next migration gate |
|---|---|---|---|
| `manifest.json.blindZones` | `_lib/blind-zones.mjs` through `_lib/audit-manifest.mjs` | Blind-zone detection combines TS/JS producer semantics from `triage.json`, `symbols.json`, `dead-classify.json`, `entry-surface.json`, resolver diagnostics, and Rust analysis availability. Rust audit-core must not reinterpret those claims until parity is checked. | Follow `docs/superpowers/specs/2026-07-01-blind-zones-audit-core-parity-design.md`: compare JS blind-zone outputs against a typed Rust port on protected fixtures and real artifacts, including missing/partial producer artifacts, before changing the owner. |
| Producer runtime observation values (`commandsRun`, `skipped`) | `audit-repo.mjs` | JS still executes child processes and observes skip decisions, statuses, stderr snippets, wall-clock measurements, and orchestrator memory snapshots. Audit-core owns the typed manifest runtime-log shape, the typed ledger, and `producer-performance.json` construction from those completed observations, but it does not execute producers yet. | Migrate value production only after the Rust orchestrator can execute the existing producer graph while preserving the ledger contract and manifest summaries. |
| Producer performance measurement inputs | `audit-repo.mjs` plus `_lib/artifacts.mjs` | Artifact size enumeration, artifact read metrics, phase timing reads, and orchestrator memory snapshots are still observed by the JS runner. Rust owns the product artifact shape once those observations are supplied through the ledger. | Move these inputs only with Rust executor or a separate Rust measurement owner. |
| Raw lifecycle blocks (`preWrite`, `postWrite`, `canonDraft`, `checkCanon`) | `audit-repo.mjs` plus lifecycle helpers | These raw blocks describe child lifecycle execution, advisory paths, spawned producer outcomes, and strict-mode exit policy. Audit-core owns only the typed `manifest.json.lifecycle` summary projection from those completed blocks. | Migrate raw blocks only after Rust owns lifecycle child execution or each lifecycle helper has a Rust parity plan. |
| Human companion artifacts (`auditSummary`, `reviewPack`, `topologyMermaid`) | `audit-repo.mjs` plus renderer modules | These are presentation/rendering outputs, not typed manifest evidence summaries. | Migrate only through a separate renderer parity plan. |
| Final `manifest.json` file write | `audit-repo.mjs` | The manifest root still joins Rust summaries with JS producer orchestration and optional pre/post-write lifecycle blocks. | Migrate after all manifest fields have typed Rust owners or an explicit Rust orchestrator owns the final write. |
| Child process execution | `audit-repo.mjs` | Rust audit-core owns the typed orchestration plan and the JS runner follows that plan, but Rust does not spawn JS/MJS producers or observe runtime command results yet. | Migrate only after the Rust orchestrator can execute the existing producer graph while preserving `commandsRun`, `skipped`, and producer-performance semantics. |

## Canonical Rust Modules

| File | Owns | Must not own |
|---|---|---|
| `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs` | Known artifact names, dynamic artifact filename matching, deterministic produced-artifact enumeration | child process execution, JSON artifact parsing beyond filenames |
| `experiments/rust-main/lumin-audit-core/src/artifact_summaries.rs` | `manifest.json.frameworkResourceSurfaces`, `manifest.json.unusedDependencies`, and `manifest.json.blockClones` projections from already-produced artifact JSON | framework/resource scanning, unused-dependency analysis, block-clone detection |
| `experiments/rust-main/lumin-audit-core/src/blind_zones.rs` | Typed `manifest.json.blindZones` parity projection from JS-owned producer artifacts behind a fixture-based CLI gate | JS/TS producer behavior, final manifest wiring before parity, console summary rendering |
| `experiments/rust-main/lumin-audit-core/src/living_audit.rs` | `manifest.json.livingAudit` projection from known living-audit document candidate paths under the audited root | audit document authoring, final answer policy, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/manifest_core.rs` | `manifest.json.scanRange`, `manifest.json.confidence`, and `manifest.json.sfcEvidence` projections from already-produced `triage.json` and `symbols.json` | blind-zone detection, living-audit document discovery, producer execution |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics.rs` | `manifest.json.resolverDiagnostics` projection from already-produced `symbols.json`, `resolver-capabilities.json`, and `resolver-diagnostics.json` | module resolution, blocked-hint production, blind-zone detection |
| `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs` | `rust-analyzer-health.latest.json` manifest summary projection, root mismatch, invalid-shape, complete/available status, and `manifest.json.rustAnalysis` merge projection from JS-observed Rust analyzer run state plus already-produced evidence summary | Rust source parsing, source-health analysis, Cargo oracle execution, child process execution |
| `experiments/rust-main/lumin-audit-core/src/generated_artifacts.rs` | `manifest.json.generatedArtifacts` projection from already-produced `symbols.json`, generated-artifact mode validation, generated miss grouping, blind-zone grouping, and present/prepared out-of-scope evidence | package resolution, generator execution, generated-artifact producer evidence construction |
| `experiments/rust-main/lumin-audit-core/src/lifecycle.rs` | `manifest.json.lifecycle` projection from completed raw `preWrite`, `postWrite`, `canonDraft`, and `checkCanon` manifest blocks | lifecycle child execution, advisory generation, post-write delta production, canon draft/check producer behavior, raw lifecycle block ownership |
| `experiments/rust-main/lumin-audit-core/src/manifest_evidence.rs` | Composition of Rust-owned `manifest.json` evidence fields from already-produced artifacts, excluding `blindZones` | blind-zone detection, producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_final.rs` | Final pre-write `manifest.json` summary patch projection for `performance`, `orchestration`, and `artifactsProduced` from already-produced `producer-performance.json` and output artifact names | producer execution, producer-performance artifact writing, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_meta.rs` | `manifest.json.meta` shape projection from JS-provided run timestamp, profile, root, and output values | clock reading, profile flag parsing before CLI dispatch, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/manifest_root.rs` | Initial `manifest.json` root shell projection and manifest evidence refresh patch projection from Rust-owned summary fields, typed JS-observed `commandsRun` / `skipped` runtime logs, produced-artifact list, and JS-owned pass-through `blindZones` blocks | producer execution, blind-zone interpretation, lifecycle raw block construction, human companion renderers, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs` | Typed `lumin-audit-orchestration-ledger.v1` input contract and `producer-performance.json` construction from completed JS executor observations | child process execution, live telemetry collection, artifact size enumeration, artifact read measurement, phase timing file reads, final manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs` | Typed audit profile command graph, lifecycle request plan, profile/SARIF/base-pipeline skip semantics, and planned precondition metadata consumed by `audit-repo.mjs` | child process execution, filesystem precondition evaluation, command telemetry, producer-performance measurement |
| `experiments/rust-main/lumin-audit-core/src/orchestration_result.rs` | `manifest.json.orchestration` projection from the typed `producer-performance.json` source shape, including execution status counts, required/optional failure counts, skipped counts, and capped examples | child process execution, live telemetry collection, raw `commandsRun`/`skipped` value production, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/producer_performance.rs` | `manifest.json.performance` projection from already-produced `producer-performance.json` | producer execution, memory measurement, artifact read measurement, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/scan_scope.rs` | Audit manifest scan-scope path inclusion policy used by migrated manifest summaries, matching the JS `scanScopeStatusForPath` contract | source walking, parsing, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/cli.rs` | CLI request parsing and stdout JSON dispatch for audit-core commands | producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/lib.rs` | public library exports for audit manifest wrappers | ad hoc JSON shape construction outside owned modules |

## Rules

- Audit-core reads already-produced artifacts. It does not execute producers.
- Audit-core may own an orchestration plan before it owns orchestration
  execution. A plan is declarative profile/lifecycle evidence; it must not
  spawn child processes, read producer outputs, or claim a precondition passed.
- Audit-core may emit JSON to stdout for JS compatibility, but the library owns
  typed Rust structs first.
- JS/TS producer lanes remain JS-owned until a lane-specific Rust parity proof
  exists.
- Do not add elapsed-time caps, repository-size caps, or timeout logic.
- Unknown JSON fields in consumed artifacts must be ignored.
- Missing or malformed migrated inputs must become explicit status, not silent
  zero evidence.
