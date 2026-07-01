# canonical/audit-core.md

> **Role:** canonical owner map for Rust audit orchestration and manifest evidence migration.
> **Owner:** this file.
> **Status:** first Rust artifact-registry slice.
> **Last updated:** 2026-07-01

## Scope

`lumin-audit-core` owns typed audit artifact registry and manifest evidence
summary contracts that are not source-language analysis.

It does not own JS/TS producer behavior, Rust source-health syntax analysis,
Cargo semantic oracle behavior, or final `audit-repo.mjs` orchestration yet.

## Canonical Rust Modules

| File | Owns | Must not own |
|---|---|---|
| `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs` | Known artifact names, dynamic artifact filename matching, deterministic produced-artifact enumeration | child process execution, JSON artifact parsing beyond filenames |
| `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs` | `rust-analyzer-health.latest.json` manifest summary projection, root mismatch, invalid-shape, complete/available status | Rust source parsing, source-health analysis, Cargo oracle execution |
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
