# canonical/audit-core.md

> **Role:** canonical owner map for Rust audit orchestration and manifest evidence migration.
> **Owner:** this file.
> **Status:** staged Rust manifest projection migration.
> **Last updated:** 2026-07-01

## Scope

`lumin-audit-core` owns typed audit artifact registry and manifest evidence
summary contracts that are not source-language analysis.

It does not own JS/TS producer behavior, Rust source-health syntax analysis,
Cargo semantic oracle behavior, or final `audit-repo.mjs` orchestration yet.

## Remaining JS-Owned Manifest Boundaries

These fields are deliberately not Rust-owned yet. They need lane-specific parity
or orchestration ownership before migration.

| Manifest area | Current owner | Why it stays JS-owned for now | Next migration gate |
|---|---|---|---|
| `manifest.json.blindZones` | `_lib/blind-zones.mjs` through `_lib/audit-manifest.mjs` | Blind-zone detection combines TS/JS producer semantics from `triage.json`, `symbols.json`, `dead-classify.json`, `entry-surface.json`, resolver diagnostics, and Rust analysis availability. Rust audit-core must not reinterpret those claims until parity is checked. | Compare JS blind-zone outputs against a typed Rust port on real artifacts, including missing/partial producer artifacts, before changing the owner. |
| Producer execution evidence (`commandsRun`, `skipped`) | `audit-repo.mjs` | These fields describe child process orchestration, skip decisions, exit statuses, stderr snippets, and wall-clock observations. Audit-core reads completed artifacts; it does not execute producers. | Only migrate if producer orchestration itself moves to Rust. |
| `producer-performance.json` construction | `audit-repo.mjs` | The artifact records JS orchestrator observations: command timings, memory snapshots, phase timing reads, artifact read metrics, and skipped producer entries. Rust owns only the typed `manifest.json.performance` summary projection from that artifact. | Keep JS-owned unless the measurement source moves to Rust. |
| Human companion artifacts (`auditSummary`, `reviewPack`, `topologyMermaid`) | `audit-repo.mjs` plus renderer modules | These are presentation/rendering outputs, not typed manifest evidence summaries. | Migrate only through a separate renderer parity plan. |
| Final `manifest.json` file write | `audit-repo.mjs` | The manifest root still joins Rust summaries with JS producer orchestration and optional pre/post-write lifecycle blocks. | Migrate after all manifest fields have typed Rust owners or an explicit Rust orchestrator owns the final write. |

## Canonical Rust Modules

| File | Owns | Must not own |
|---|---|---|
| `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs` | Known artifact names, dynamic artifact filename matching, deterministic produced-artifact enumeration | child process execution, JSON artifact parsing beyond filenames |
| `experiments/rust-main/lumin-audit-core/src/artifact_summaries.rs` | `manifest.json.frameworkResourceSurfaces`, `manifest.json.unusedDependencies`, and `manifest.json.blockClones` projections from already-produced artifact JSON | framework/resource scanning, unused-dependency analysis, block-clone detection |
| `experiments/rust-main/lumin-audit-core/src/living_audit.rs` | `manifest.json.livingAudit` projection from known living-audit document candidate paths under the audited root | audit document authoring, final answer policy, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/manifest_core.rs` | `manifest.json.scanRange`, `manifest.json.confidence`, and `manifest.json.sfcEvidence` projections from already-produced `triage.json` and `symbols.json` | blind-zone detection, living-audit document discovery, producer execution |
| `experiments/rust-main/lumin-audit-core/src/resolver_diagnostics.rs` | `manifest.json.resolverDiagnostics` projection from already-produced `symbols.json`, `resolver-capabilities.json`, and `resolver-diagnostics.json` | module resolution, blocked-hint production, blind-zone detection |
| `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs` | `rust-analyzer-health.latest.json` manifest summary projection, root mismatch, invalid-shape, complete/available status | Rust source parsing, source-health analysis, Cargo oracle execution |
| `experiments/rust-main/lumin-audit-core/src/generated_artifacts.rs` | `manifest.json.generatedArtifacts` projection from already-produced `symbols.json`, generated-artifact mode validation, generated miss grouping, blind-zone grouping, and present/prepared out-of-scope evidence | package resolution, generator execution, generated-artifact producer evidence construction |
| `experiments/rust-main/lumin-audit-core/src/manifest_evidence.rs` | Composition of Rust-owned `manifest.json` evidence fields from already-produced artifacts, excluding `blindZones` | blind-zone detection, producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/producer_performance.rs` | `manifest.json.performance` projection from already-produced `producer-performance.json` | producer execution, memory measurement, artifact read measurement, producer-performance artifact writing |
| `experiments/rust-main/lumin-audit-core/src/scan_scope.rs` | Audit manifest scan-scope path inclusion policy used by migrated manifest summaries, matching the JS `scanScopeStatusForPath` contract | source walking, parsing, producer orchestration |
| `experiments/rust-main/lumin-audit-core/src/cli.rs` | CLI request parsing and stdout JSON dispatch for audit-core commands | producer orchestration, manifest file writing |
| `experiments/rust-main/lumin-audit-core/src/lib.rs` | public library exports for audit manifest wrappers | ad hoc JSON shape construction outside owned modules |

## Rules

- Audit-core reads already-produced artifacts. It does not execute producers.
- Audit-core may emit JSON to stdout for JS compatibility, but the library owns
  typed Rust structs first.
- JS/TS producer lanes remain JS-owned until a lane-specific Rust parity proof
  exists.
- Do not add elapsed-time caps, repository-size caps, or timeout logic.
- Unknown JSON fields in consumed artifacts must be ignored.
- Missing or malformed migrated inputs must become explicit status, not silent
  zero evidence.
