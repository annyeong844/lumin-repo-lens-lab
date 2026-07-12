# Audit Artifact Registry Rust Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a new `lumin-audit-core` Rust crate that owns the audit artifact registry and Rust analyzer manifest summary contracts, then leave JS as a thin compatibility wrapper.

**Architecture:** Add a product-domain Rust crate under `experiments/rust-main/lumin-audit-core`. The crate exposes typed library functions plus a small stdout JSON CLI. `_lib/audit-manifest.mjs` keeps the existing public JS shape while delegating the first migrated contracts to the Rust core after parity is proven.

**Tech Stack:** Rust 2021 workspace crate, `anyhow`, `serde`, `serde_json`, `tempfile`; existing `lumin-rust-common` path helpers; current JS wrappers and Vitest/Node tests only with explicit user approval.

---

## Repository Rule Override

The `writing-plans` skill template mentions TDD. This repository's `AGENTS.md`
overrides that: tests are product behavior proof, not a test-first ritual. Tasks
below say "Add behavior proof" and "Run verification"; they do not require
red/green TDD.

## File Structure

Create:

- `canonical/audit-core.md`
  Canonical owner map for `lumin-audit-core`.
- `experiments/rust-main/lumin-audit-core/Cargo.toml`
  New product-domain crate manifest.
- `experiments/rust-main/lumin-audit-core/src/lib.rs`
  Public library entrypoints.
- `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs`
  Static/dynamic artifact registry and produced-artifact enumeration.
- `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs`
  Typed Rust analyzer artifact summary projection.
- `experiments/rust-main/lumin-audit-core/src/cli.rs`
  CLI argument parsing and JSON command dispatch.
- `experiments/rust-main/lumin-audit-core/src/main.rs`
  Thin binary entrypoint.
- `experiments/rust-main/lumin-audit-core/tests/integration.rs`
  Product behavior tests for registry and Rust analyzer summary.

Modify:

- `experiments/Cargo.toml`
  Add workspace member and workspace dependency for `lumin-audit-core`.
- `scripts/build-skill.mjs`
  Add `audit-core.md` to `RUNTIME_CANON_FILES`.
- `skills/lumin-repo-lens-lab/canonical/audit-core.md`
  Mirror canonical file after the package copy step or direct mirror update.
- `_lib/audit-manifest.mjs`
  Later task only: delegate migrated behavior to Rust core while preserving public exports.
- `skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`
  Mirror `_lib/audit-manifest.mjs`.

Do not modify:

- JS/TS producers such as `build-symbol-graph.mjs`, `build-shape-index.mjs`,
  `build-function-clone-index.mjs`, `classify-dead-exports.mjs`, resolver, SFC,
  or any contamination lanes.
- `audit-repo.mjs` orchestration beyond replacing calls to migrated wrapper
  functions if the wrapper API requires it.

## Task 1: Canonical Owner And Workspace Skeleton

**Files:**
- Create: `canonical/audit-core.md`
- Modify: `scripts/build-skill.mjs`
- Modify: `experiments/Cargo.toml`
- Create: `experiments/rust-main/lumin-audit-core/Cargo.toml`
- Create: `experiments/rust-main/lumin-audit-core/src/lib.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/main.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/cli.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs`
- Create: `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs`

- [ ] **Step 1: Add canonical owner document**

Create `canonical/audit-core.md`:

```markdown
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
```

- [ ] **Step 2: Include canonical file in skill package allowlist**

In `scripts/build-skill.mjs`, insert `audit-core.md` in `RUNTIME_CANON_FILES`
near the other canonical files:

```js
const RUNTIME_CANON_FILES = [
  'any-contamination.md',
  'audit-core.md',
  'canon-drift.md',
  'classification-gates.md',
  'evidence-ladder.md',
  'fact-model.md',
  'identity-and-alias.md',
  'index.md',
  'invariants.md',
  'mode-contract.md',
  'oracle-registry.json',
  'pre-write-gate.md',
];
```

- [ ] **Step 3: Add workspace member**

In `experiments/Cargo.toml`, add the member:

```toml
members = [
    "rust-common",
    "rust-main/lumin-audit-core",
    "rust-main/lumin-rust-analyzer",
    "rust-main/rust-cargo-oracle",
    "rust-sidecar/rust-source-health",
    "rust-sidecar/topology-scanner",
]
```

Add a workspace dependency only if another crate needs to depend on it in this
slice:

```toml
lumin-audit-core = { path = "rust-main/lumin-audit-core" }
```

If no other Rust crate imports it yet, skip the workspace dependency to avoid
unused dependency churn.

- [ ] **Step 4: Create crate manifest**

Create `experiments/rust-main/lumin-audit-core/Cargo.toml`:

```toml
[package]
name = "lumin-audit-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "lumin-audit-core"
path = "src/main.rs"
test = false

[lints]
workspace = true

[dependencies]
anyhow = { workspace = true }
lumin-rust-common = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

- [ ] **Step 5: Create module skeleton**

Create `src/lib.rs`:

```rust
pub mod artifact_registry;
pub mod rust_analysis;
```

Create `src/main.rs`:

```rust
mod cli;

fn main() {
    if let Err(error) = cli::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
```

Create empty module files with compile-safe stubs:

```rust
// artifact_registry.rs
use anyhow::Result;
use std::path::Path;

pub fn collect_produced_artifacts(
    _out_dir: &Path,
    _rust_analysis_usable: bool,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}
```

```rust
// rust_analysis.rs
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisSummary {
    pub artifact: &'static str,
    pub status: String,
    pub available: bool,
}

pub fn summarize_rust_analysis_artifact(
    _root: &Path,
    _artifact: &Value,
) -> Option<RustAnalysisSummary> {
    None
}
```

```rust
// cli.rs
use anyhow::{bail, Result};

pub fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("artifact-registry") | Some("rust-analysis-summary") => {
            bail!("lumin-audit-core command is not implemented yet")
        }
        _ => bail!(
            "usage: lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran]\n       lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>"
        ),
    }
}
```

- [ ] **Step 6: Verify skeleton compiles**

Run:

```powershell
cargo check --manifest-path experiments\Cargo.toml -p lumin-audit-core
```

Expected: exit 0.

- [ ] **Step 7: Commit**

```powershell
git add canonical\audit-core.md scripts\build-skill.mjs experiments\Cargo.toml experiments\rust-main\lumin-audit-core
git commit -m "Add audit core Rust crate skeleton"
```

## Task 2: Artifact Registry Library

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/artifact_registry.rs`
- Create or extend: `experiments/rust-main/lumin-audit-core/tests/integration.rs`

- [ ] **Step 1: Replace artifact registry stub with typed behavior**

Implement static artifacts and dynamic matching in
`src/artifact_registry.rs`:

```rust
use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const RUST_ANALYZER_ARTIFACT: &str = "rust-analyzer-health.latest.json";

const ARTIFACT_CANDIDATES: &[&str] = &[
    "triage.json",
    "topology.json",
    "discipline.json",
    "call-graph.json",
    "barrels.json",
    "shape-index.json",
    "function-clones.json",
    "block-clones.json",
    "framework-resource-surfaces.json",
    "resolver-capabilities.json",
    "resolver-diagnostics.json",
    "symbols.json",
    "unused-deps.json",
    "entry-surface.json",
    "module-reachability.json",
    "dead-classify.json",
    "runtime-evidence.json",
    "staleness.json",
    "fix-plan.json",
    "checklist-facts.json",
    RUST_ANALYZER_ARTIFACT,
    "producer-performance.json",
    "canon-drift.json",
    "topology.mermaid.md",
    "audit-summary.latest.md",
    "audit-review-pack.latest.md",
    "lumin-repo-lens-lab.sarif",
];

pub fn collect_produced_artifacts(
    out_dir: &Path,
    rust_analysis_usable: bool,
) -> Result<Vec<String>> {
    let mut produced = BTreeSet::new();
    for name in ARTIFACT_CANDIDATES {
        if *name == RUST_ANALYZER_ARTIFACT && !rust_analysis_usable {
            continue;
        }
        if out_dir.join(name).is_file() {
            produced.insert((*name).to_string());
        }
    }

    let entries = match fs::read_dir(out_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_dynamic_artifact_name(&name) {
            produced.insert(name);
        }
    }

    Ok(produced.into_iter().collect())
}

fn is_dynamic_artifact_name(name: &str) -> bool {
    is_canon_drift_markdown(name)
        || is_pre_write_advisory(name)
        || is_post_write_delta(name)
        || is_any_inventory(name, "pre")
        || is_any_inventory(name, "post")
}

fn is_canon_drift_markdown(name: &str) -> bool {
    name.starts_with("canon-drift.") && name.ends_with(".md") && name.len() > "canon-drift..md".len()
}

fn is_pre_write_advisory(name: &str) -> bool {
    name == "pre-write-advisory.json"
        || (name.starts_with("pre-write-advisory.") && name.ends_with(".json"))
}

fn is_post_write_delta(name: &str) -> bool {
    name == "post-write-delta.json"
        || (name.starts_with("post-write-delta.") && name.ends_with(".json"))
}

fn is_any_inventory(name: &str, phase: &str) -> bool {
    let prefix = format!("any-inventory.{phase}.");
    name.starts_with(&prefix) && name.ends_with(".json") && name.len() > prefix.len() + ".json".len()
}
```

- [ ] **Step 2: Add artifact registry behavior tests**

Create or extend `tests/integration.rs`:

```rust
use anyhow::Result;
use std::fs;

use lumin_audit_core::artifact_registry::collect_produced_artifacts;

#[test]
fn produced_artifacts_include_static_and_dynamic_names_in_order() -> Result<()> {
    let temp = tempfile::tempdir()?;
    for name in [
        "symbols.json",
        "pre-write-advisory.abc.json",
        "canon-drift.type-ownership.md",
        "post-write-delta.xyz.json",
        "any-inventory.pre.123.json",
        "audit-summary.latest.md",
    ] {
        fs::write(temp.path().join(name), "{}\n")?;
    }

    let artifacts = collect_produced_artifacts(temp.path(), true)?;

    assert_eq!(
        artifacts,
        vec![
            "any-inventory.pre.123.json",
            "audit-summary.latest.md",
            "canon-drift.type-ownership.md",
            "post-write-delta.xyz.json",
            "pre-write-advisory.abc.json",
            "symbols.json",
        ]
    );
    Ok(())
}

#[test]
fn stale_rust_analyzer_artifact_is_not_produced_when_current_run_did_not_use_it() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let artifacts = collect_produced_artifacts(temp.path(), false)?;

    assert!(!artifacts.contains(&"rust-analyzer-health.latest.json".to_string()));
    Ok(())
}

#[test]
fn current_rust_analyzer_artifact_is_produced_when_current_run_used_it() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("rust-analyzer-health.latest.json"), "{}\n")?;

    let artifacts = collect_produced_artifacts(temp.path(), true)?;

    assert_eq!(artifacts, vec!["rust-analyzer-health.latest.json"]);
    Ok(())
}
```

- [ ] **Step 3: Verify artifact registry behavior**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core produced_artifacts
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core stale_rust_analyzer
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core current_rust_analyzer
```

Expected: all selected tests pass.

- [ ] **Step 4: Commit**

```powershell
git add experiments\rust-main\lumin-audit-core
git commit -m "Move artifact registry into audit core"
```

## Task 3: Rust Analyzer Summary Library

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/rust_analysis.rs`
- Extend: `experiments/rust-main/lumin-audit-core/tests/integration.rs`

- [ ] **Step 1: Define typed summary structs**

Replace `src/rust_analysis.rs` with typed subset projection:

```rust
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

const ARTIFACT_NAME: &str = "rust-analyzer-health.latest.json";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisSummary {
    pub artifact: &'static str,
    pub status: RustAnalysisStatus,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_health_profile: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_mode: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_scope: Option<ScanScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_review_signals: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_review_opaque_surfaces: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_function_clone_exact_body_groups: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_function_clone_structure_groups: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_function_clone_signature_groups: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_function_clone_near_candidates: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_tier_summary: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle_bridge_status: Option<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RustAnalysisStatus {
    RootMismatch,
    InvalidShape,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tests: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_policy: Option<Value>,
}

pub fn summarize_rust_analysis_artifact(
    root: &Path,
    artifact: &Value,
) -> Option<RustAnalysisSummary> {
    if !artifact.is_object() {
        return None;
    }

    let artifact_root = artifact.pointer("/meta/input/root").and_then(Value::as_str);
    if !same_resolved_path(artifact_root, root) {
        return Some(RustAnalysisSummary {
            artifact: ARTIFACT_NAME,
            status: RustAnalysisStatus::RootMismatch,
            available: false,
            root: artifact_root.map(ToOwned::to_owned),
            schema_version: None,
            policy_version: None,
            producer: None,
            mode: None,
            source_health_profile: None,
            semantic_mode: None,
            scan_scope: None,
            files: None,
            syntax_review_signals: None,
            syntax_review_opaque_surfaces: None,
            syntax_function_clone_exact_body_groups: None,
            syntax_function_clone_structure_groups: None,
            syntax_function_clone_signature_groups: None,
            syntax_function_clone_near_candidates: None,
            action_tier_summary: None,
            oracle_bridge_status: None,
        });
    }

    let summary = artifact.get("summary").and_then(Value::as_object);
    let valid = artifact.get("schemaVersion").and_then(Value::as_str).is_some()
        && artifact.get("policyVersion").and_then(Value::as_str).is_some()
        && artifact.pointer("/meta/producer").and_then(Value::as_str) == Some("lumin-rust-analyzer")
        && artifact.pointer("/meta/mode").and_then(Value::as_str) == Some("rust-main")
        && summary
            .and_then(|summary| summary.get("files"))
            .and_then(Value::as_u64)
            .is_some();

    if !valid {
        return Some(RustAnalysisSummary {
            artifact: ARTIFACT_NAME,
            status: RustAnalysisStatus::InvalidShape,
            available: false,
            root: artifact_root.map(ToOwned::to_owned),
            schema_version: None,
            policy_version: None,
            producer: None,
            mode: None,
            source_health_profile: None,
            semantic_mode: None,
            scan_scope: None,
            files: None,
            syntax_review_signals: None,
            syntax_review_opaque_surfaces: None,
            syntax_function_clone_exact_body_groups: None,
            syntax_function_clone_structure_groups: None,
            syntax_function_clone_signature_groups: None,
            syntax_function_clone_near_candidates: None,
            action_tier_summary: None,
            oracle_bridge_status: None,
        });
    }

    let Some(summary) = summary else {
        return Some(RustAnalysisSummary {
            artifact: ARTIFACT_NAME,
            status: RustAnalysisStatus::InvalidShape,
            available: false,
            root: artifact_root.map(ToOwned::to_owned),
            schema_version: None,
            policy_version: None,
            producer: None,
            mode: None,
            source_health_profile: None,
            semantic_mode: None,
            scan_scope: None,
            files: None,
            syntax_review_signals: None,
            syntax_review_opaque_surfaces: None,
            syntax_function_clone_exact_body_groups: None,
            syntax_function_clone_structure_groups: None,
            syntax_function_clone_signature_groups: None,
            syntax_function_clone_near_candidates: None,
            action_tier_summary: None,
            oracle_bridge_status: None,
        });
    };
    Some(RustAnalysisSummary {
        artifact: ARTIFACT_NAME,
        status: RustAnalysisStatus::Complete,
        available: true,
        root: None,
        schema_version: string_field(artifact, "/schemaVersion"),
        policy_version: string_field(artifact, "/policyVersion"),
        producer: string_field(artifact, "/meta/producer"),
        mode: string_field(artifact, "/meta/mode"),
        source_health_profile: artifact
            .pointer("/meta/input/effectiveSourceHealthProfile")
            .or_else(|| artifact.pointer("/meta/input/sourceHealthProfile"))
            .cloned(),
        semantic_mode: artifact.pointer("/meta/input/semanticMode").cloned(),
        scan_scope: scan_scope_from_artifact(artifact),
        files: u64_summary(summary, "files"),
        syntax_review_signals: u64_summary(summary, "syntaxReviewSignals"),
        syntax_review_opaque_surfaces: u64_summary(summary, "syntaxReviewOpaqueSurfaces"),
        syntax_function_clone_exact_body_groups: u64_summary(summary, "syntaxFunctionCloneExactBodyGroups"),
        syntax_function_clone_structure_groups: u64_summary(summary, "syntaxFunctionCloneStructureGroups"),
        syntax_function_clone_signature_groups: u64_summary(summary, "syntaxFunctionCloneSignatureGroups"),
        syntax_function_clone_near_candidates: u64_summary(summary, "syntaxFunctionCloneNearCandidates"),
        action_tier_summary: summary.get("actionTierSummary").cloned(),
        oracle_bridge_status: summary.get("oracleBridgeStatus").cloned(),
    })
}

fn same_resolved_path(artifact_root: Option<&str>, root: &Path) -> bool {
    let Some(artifact_root) = artifact_root else {
        return false;
    };
    let left = std::fs::canonicalize(PathBuf::from(artifact_root));
    let right = std::fs::canonicalize(root);
    match (left, right) {
        (Ok(left), Ok(right)) => left == right,
        _ => PathBuf::from(artifact_root) == root,
    }
}

fn string_field(value: &Value, pointer: &str) -> Option<String> {
    value.pointer(pointer).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn u64_summary(summary: &serde_json::Map<String, Value>, field: &str) -> Option<u64> {
    summary.get(field).and_then(Value::as_u64)
}

fn scan_scope_from_artifact(artifact: &Value) -> Option<ScanScope> {
    let input = artifact.pointer("/meta/input");
    let syntax_input = artifact.pointer("/phases/syntax/meta/input");
    let include_tests = input
        .and_then(|input| input.get("includeTests"))
        .and_then(Value::as_bool)
        .or_else(|| {
            syntax_input
                .and_then(|input| input.get("includeTests"))
                .and_then(Value::as_bool)
        });
    let exclude = string_array_from(input.and_then(|input| input.get("exclude")))
        .or_else(|| string_array_from(syntax_input.and_then(|input| input.get("exclude"))));
    let path_policy = syntax_input
        .and_then(|input| input.get("pathPolicy"))
        .or_else(|| input.and_then(|input| input.get("pathPolicy")))
        .cloned();
    if include_tests.is_none() && exclude.is_none() && path_policy.is_none() {
        return None;
    }
    Some(ScanScope {
        include_tests,
        exclude,
        path_policy,
    })
}

fn string_array_from(value: Option<&Value>) -> Option<Vec<String>> {
    let values = value?.as_array()?;
    let out = values
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    Some(out)
}
```

- [ ] **Step 2: Add Rust analyzer summary behavior tests**

Extend `tests/integration.rs`:

```rust
use lumin_audit_core::rust_analysis::{
    summarize_rust_analysis_artifact, RustAnalysisStatus,
};
use serde_json::json;

#[test]
fn rust_analysis_summary_reports_root_mismatch() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let other = tempfile::tempdir()?;
    let other_root = other.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "policyVersion": "lumin-rust-analyzer-policy.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": { "root": other_root }
        },
        "summary": { "files": 1 }
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::RootMismatch);
    assert!(!summary.available);
    Ok(())
}

#[test]
fn rust_analysis_summary_reports_invalid_shape() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let root_text = root.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": { "root": root_text }
        },
        "summary": {}
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::InvalidShape);
    assert!(!summary.available);
    Ok(())
}

#[test]
fn rust_analysis_summary_preserves_complete_scope_and_counts() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let root_text = root.path().to_string_lossy().to_string();
    let artifact = json!({
        "schemaVersion": "lumin-rust-analyzer.v1",
        "policyVersion": "lumin-rust-analyzer-policy.v1",
        "meta": {
            "producer": "lumin-rust-analyzer",
            "mode": "rust-main",
            "input": {
                "root": root_text,
                "effectiveSourceHealthProfile": "compact",
                "semanticMode": "metadata-only",
                "includeTests": false,
                "exclude": ["generated"]
            }
        },
        "phases": {
            "syntax": {
                "meta": {
                    "input": {
                        "pathPolicy": {
                            "exclude": ["**/target/**", "**/vendor/**", "generated"]
                        }
                    }
                }
            }
        },
        "summary": {
            "files": 2,
            "syntaxReviewSignals": 3,
            "syntaxReviewOpaqueSurfaces": 4,
            "syntaxFunctionCloneExactBodyGroups": 5,
            "syntaxFunctionCloneStructureGroups": 6,
            "syntaxFunctionCloneSignatureGroups": 7,
            "syntaxFunctionCloneNearCandidates": 8,
            "actionTierSummary": { "safeFix": 1 },
            "oracleBridgeStatus": "metadata-only"
        }
    });

    let summary = summarize_rust_analysis_artifact(root.path(), &artifact)
        .ok_or_else(|| anyhow::anyhow!("object artifact should yield a summary"))?;

    assert_eq!(summary.status, RustAnalysisStatus::Complete);
    assert!(summary.available);
    assert_eq!(summary.files, Some(2));
    assert_eq!(summary.syntax_review_signals, Some(3));
    assert_eq!(summary.scan_scope.as_ref().and_then(|scope| scope.include_tests), Some(false));
    assert_eq!(
        summary.scan_scope.as_ref().and_then(|scope| scope.exclude.clone()),
        Some(vec!["generated".to_string()])
    );
    Ok(())
}
```

- [ ] **Step 3: Verify Rust analyzer summary behavior**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core rust_analysis_summary
```

Expected: selected tests pass.

- [ ] **Step 4: Commit**

```powershell
git add experiments\rust-main\lumin-audit-core
git commit -m "Summarize Rust analyzer artifacts in audit core"
```

## Task 4: CLI JSON Compatibility

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/cli.rs`
- Extend: `experiments/rust-main/lumin-audit-core/tests/integration.rs`

- [ ] **Step 1: Implement CLI dispatch**

Implement `cli.rs` so these commands work:

```text
lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran]
lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>
```

Use `serde_json::to_writer(std::io::stdout(), &value)` and write a trailing
newline. For `rust-analysis-summary`, read the artifact file, parse JSON, and
emit either `null` or the typed summary object.

Use this parsing shape:

```rust
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::artifact_registry::collect_produced_artifacts;
use crate::rust_analysis::summarize_rust_analysis_artifact;

pub fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("artifact-registry") => run_artifact_registry(args.collect()),
        Some("rust-analysis-summary") => run_rust_analysis_summary(args.collect()),
        _ => bail!(
            "usage: lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran]\n       lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>"
        ),
    }
}
```

Implement local helpers:

- `take_path(args, "--output")`
- `take_path(args, "--root")`
- `take_path(args, "--artifact")`
- `write_stdout_json<T: Serialize>(value: &T)`

Do not add a dependency on a CLI framework in this slice.

- [ ] **Step 2: Verify CLI manually through cargo run**

Run:

```powershell
cargo run --manifest-path experiments\Cargo.toml -p lumin-audit-core -- artifact-registry --output .\does-not-exist
```

Expected stdout:

```json
[]
```

Run a temp fixture manually or through a focused test to confirm
`rust-analysis-summary` emits a JSON object with `"status":"complete"` for a
valid artifact.

- [ ] **Step 3: Add CLI product behavior tests if practical**

If invoking the compiled binary from Rust integration tests is already simple in
this workspace, add tests for stdout JSON. If not, rely on library tests plus
manual cargo run in this slice and document that JS wrapper tests cover the
process boundary in Task 5.

- [ ] **Step 4: Verify crate**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core
cargo fmt --manifest-path experiments\Cargo.toml --all -- --check
```

Expected: all `lumin-audit-core` tests pass and formatting is clean.

- [ ] **Step 5: Commit**

```powershell
git add experiments\rust-main\lumin-audit-core
git commit -m "Add audit core JSON CLI"
```

## Task 5: JS Wrapper Wiring

**Files:**
- Modify: `_lib/audit-manifest.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`
- Modify or add focused JS tests only when user explicitly approves Node execution.

- [ ] **Step 1: Add Rust core invocation helper**

In `_lib/audit-manifest.mjs`, import child-process and URL/path helpers if not
already present:

```js
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
```

Add a local helper near artifact functions:

```js
function auditCoreBinary() {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const root = path.resolve(here, '..');
  const exe = process.platform === 'win32' ? 'lumin-audit-core.exe' : 'lumin-audit-core';
  return path.join(root, 'experiments', 'target', 'debug', exe);
}

function runAuditCoreJson(args, label) {
  const command = auditCoreBinary();
  if (!existsSync(command)) {
    throw new Error(`${label}: lumin-audit-core binary missing at ${command}; run cargo build --manifest-path experiments/Cargo.toml -p lumin-audit-core`);
  }
  const stdout = execFileSync(command, args, {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
  return JSON.parse(stdout);
}
```

For package builds, adjust the binary lookup if the generated skill installs
the binary somewhere else. Do not silently fall back to the JS implementation
without a documented transition flag.

- [ ] **Step 2: Delegate `collectProducedArtifacts`**

Change exported `collectProducedArtifacts(outDir)` to accept an optional
options object:

```js
export function collectProducedArtifacts(outDir, options = {}) {
  const rustAnalysisUsable = options.rustAnalysisUsable ?? true;
  return runAuditCoreJson([
    'artifact-registry',
    '--output', outDir,
    ...(rustAnalysisUsable ? ['--rust-analysis-ran'] : []),
  ], 'collectProducedArtifacts');
}
```

Then update `audit-repo.mjs` `collectManifestProducedArtifacts` to call:

```js
const artifacts = collectProducedArtifacts(OUT, {
  rustAnalysisUsable: rustAnalysisArtifactUsable(rustAnalysis),
});
```

This preserves the stale Rust artifact product behavior.

- [ ] **Step 3: Delegate Rust analysis summary**

Keep `buildManifestEvidence` public shape unchanged. Replace only the inner Rust
summary computation:

```js
function buildRustAnalysisSummaryFromFile(root, outDir) {
  const artifactPath = path.join(outDir, 'rust-analyzer-health.latest.json');
  if (!existsSync(artifactPath)) return null;
  return runAuditCoreJson([
    'rust-analysis-summary',
    '--root', root,
    '--artifact', artifactPath,
  ], 'buildRustAnalysisSummary');
}
```

In `buildManifestEvidence`, use:

```js
const rustAnalysis = buildRustAnalysisSummaryFromFile(root, outDir);
```

Remove the now-duplicated JS-only helpers after parity is proven in the same
diff:

- `sameResolvedPath`
- `stringArrayOrNull`
- `objectOrNull`
- `rustScanScopeFromArtifact`
- `buildRustAnalysisSummary`

Do not remove unrelated JS manifest summary helpers in this slice.

- [ ] **Step 4: Mirror skill package source**

Apply the same `_lib/audit-manifest.mjs` changes to:

`skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`

If using `scripts/build-skill.mjs` is approved, rebuild the skill package
instead. Without Node approval, mirror by exact patch and verify with
`Compare-Object`.

- [ ] **Step 5: Verify wrapper parity**

Without Node approval, run only static checks:

```powershell
Compare-Object (Get-Content _lib\audit-manifest.mjs) (Get-Content skills\lumin-repo-lens-lab\_engine\lib\audit-manifest.mjs)
git diff --check
```

With Node approval, also run:

```powershell
npm run test:vitest:audit-manifest-export-surface
node tests/test-audit-repo.mjs
```

Expected:

- audit-manifest public exports stay the same
- stale Rust analyzer artifact is not listed in `manifest.artifactsProduced`
- malformed/root-mismatch Rust analyzer artifacts keep Rust blind zones
- complete Rust analyzer artifact suppresses Rust blind zones

- [ ] **Step 6: Commit**

```powershell
git add _lib\audit-manifest.mjs skills\lumin-repo-lens-lab\_engine\lib\audit-manifest.mjs audit-repo.mjs
git commit -m "Use audit core for manifest artifact summaries"
```

## Task 6: Final Verification And Handoff

**Files:**
- Modify only if verification reveals missing docs or mirror drift.

- [ ] **Step 1: Run Rust verification**

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core
cargo check --manifest-path experiments\Cargo.toml -p lumin-audit-core -p lumin-rust-analyzer -p lumin-rust-source-health
cargo fmt --manifest-path experiments\Cargo.toml --all -- --check
```

Expected: all commands exit 0.

- [ ] **Step 2: Run static no-Node verification**

```powershell
Compare-Object (Get-Content _lib\audit-manifest.mjs) (Get-Content skills\lumin-repo-lens-lab\_engine\lib\audit-manifest.mjs)
Select-String -Path scripts\build-skill.mjs -Pattern "'audit-core.md'"
git diff --check
git status --short --branch
```

Expected:

- mirror compare has no output
- `audit-core.md` appears in `RUNTIME_CANON_FILES`
- no whitespace errors
- only intentional files are changed before the final commit

- [ ] **Step 3: Node verification only with explicit approval**

If the user approves Node execution, run:

```powershell
npm run test:vitest:audit-manifest-export-surface
npm run test:vitest:audit-repo-blind-zones
node tests/test-audit-repo.mjs
```

Expected: existing JS behavior remains compatible.

- [ ] **Step 4: Summarize remaining goal scope**

Report that this slice moves only artifact registry and Rust analyzer manifest
summary into Rust. The full active goal remains open because the broader audit
orchestrator and JS/TS producer lanes are not yet Rust-owned.

Do not call `update_goal complete` after this slice.

## Plan Self-Review

Spec coverage:

- New crate owner: Task 1.
- Artifact registry: Task 2.
- Rust analyzer summary: Task 3.
- CLI shape: Task 4.
- Thin JS wrapper: Task 5.
- Existing JS/TS lane preservation: Non-goals and Task 5/6 verification.
- No timeout/cap: Non-goals and canonical rules.
- Canonical owner: Task 1.

Placeholder scan:

- No placeholder markers or copy-forward shortcuts are intentionally left.
- Where implementation details depend on package binary placement, Task 5
  requires explicit adjustment and forbids silent fallback.

Type consistency:

- Library function names match the design:
  `collect_produced_artifacts` and `summarize_rust_analysis_artifact`.
- JS wrapper names preserve existing public exports:
  `collectProducedArtifacts`, `buildManifestEvidence`, and
  `refreshManifestEvidence`.
