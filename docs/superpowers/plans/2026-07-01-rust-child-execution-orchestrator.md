# Rust Child-Execution Orchestrator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move base audit child-process execution from `audit-repo.mjs` into `lumin-audit-core` while keeping JS/MJS producers, lifecycle helpers, `blindZones`, human renderers, and final manifest writing outside this slice.

**Architecture:** Add a typed Rust executor that consumes the existing Rust orchestration plan, runs only the base audit pipeline, and emits one execution result containing `commandsRun`, `skipped`, `rustAnalysisRun`, and a `lumin-audit-orchestration-ledger.v1` ledger built from the same observed events. The JS runner becomes a wrapper around this executor for base pipeline execution and keeps the JS-owned phases.

**Tech Stack:** Rust 2021, `lumin-audit-core`, `serde`, `serde_json`, `anyhow`, `std::process::Command`, current JS wrapper in `audit-repo.mjs` and `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`.

**Repo rule override:** This repo does not use TDD. Do not write red/green scaffolding tests. Each test added below must prove product behavior, realistic edge cases, or hard-stop behavior.

---

## File Structure

- Create `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs`
  - Owns executor request/result protocol, base step precondition evaluation, argv construction, child execution observation, memory snapshots, stderr snippets, and conversion to runtime-log/ledger shapes.
  - Must not parse producer artifacts, interpret `blindZones`, render human markdown, or write `manifest.json`.
- Modify `experiments/rust-main/lumin-audit-core/src/lib.rs`
  - Export `orchestration_executor`.
- Modify `experiments/rust-main/lumin-audit-core/src/cli.rs`
  - Add `execute-base-plan --input <path|->`.
- Modify `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`
  - After executor is active, change base step `executionOwner` from `audit-repo.mjs` to `lumin-audit-core`; keep lifecycle execution owners as `audit-repo.mjs`.
- Modify `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs`
  - Reuse ledger structs. Add `Serialize` derives only where the executor must emit the typed ledger; do not duplicate ledger shapes in the executor.
- Create `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`
  - Product behavior tests for request validation, precondition skips, optional failure continuation, required failure halt, stderr snippet cap, planned skip copying, and no timeout/cap fields.
- Modify `_lib/audit-manifest.mjs` and `skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`
  - Add `executeBasePlan(...)` wrapper around the Rust CLI.
- Modify `audit-repo.mjs` and `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`
  - Replace base `runStep` / `runRustAnalyzerStep` execution with the Rust executor result.
  - Keep lifecycle helpers, artifact reads, phase timing reads, `blindZones`, human renderers, and final manifest write JS-owned.
- Modify `canonical/audit-core.md`
  - Add `orchestration_executor.rs` as owner of base child execution and typed runtime observations.
  - Keep lifecycle child execution, artifact-read timing, phase timing reads, renderers, `blindZones`, and final manifest write outside scope.

---

### Task 1: Add Typed Executor Protocol

**Files:**
- Create: `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/lib.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`

- [ ] **Step 1: Add protocol structs**

Create `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs` with these public shapes:

```rust
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::orchestration_events::{
    ArtifactReadSummary, ArtifactSizeSummary, LedgerCache, LedgerEvent,
    LedgerGeneratedArtifacts, LedgerScanRange, OrchestrationLedger,
    ORCHESTRATION_LEDGER_SCHEMA_VERSION,
};
use crate::orchestration_plan::ORCHESTRATION_PLAN_SCHEMA_VERSION;

pub const EXECUTOR_REQUEST_SCHEMA_VERSION: &str = "lumin-audit-executor-request.v1";
pub const EXECUTOR_RESULT_SCHEMA_VERSION: &str = "lumin-audit-executor-result.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorRequest {
    pub schema_version: String,
    pub plan: ExecutorPlanInput,
    pub generated: String,
    pub root: PathBuf,
    pub output: PathBuf,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    #[serde(default)]
    pub verbose: bool,
    pub scan_range: LedgerScanRange,
    pub cache: LedgerCache,
    pub generated_artifacts: LedgerGeneratedArtifacts,
    pub artifact_reads: ArtifactReadSummary,
    pub artifacts: ArtifactSizeSummary,
    #[serde(default)]
    pub rust_analyzer: RustAnalyzerExecutorRequest,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorPlanInput {
    pub schema_version: String,
    pub profile: String,
    #[serde(default)]
    pub emit_sarif: bool,
    pub base_pipeline: ExecutorBasePipelineInput,
    #[serde(default)]
    pub steps: Vec<ExecutorStepInput>,
    #[serde(default)]
    pub skipped: Vec<ExecutorPlannedSkipInput>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorBasePipelineInput {
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorStepInput {
    pub step: String,
    pub script: String,
    #[serde(default)]
    pub required: bool,
    pub producer_owner: String,
    pub execution_owner: String,
    #[serde(default)]
    pub skip_reason_when_unmet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorPlannedSkipInput {
    pub step: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalyzerExecutorRequest {
    #[serde(default)]
    pub requested: bool,
    #[serde(default)]
    pub rust_files: u64,
    #[serde(default)]
    pub source_commit: Option<String>,
    #[serde(default)]
    pub invocation: Option<RustAnalyzerInvocation>,
    #[serde(default)]
    pub forwarded_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalyzerInvocation {
    pub command: String,
    #[serde(default)]
    pub prefix_args: Vec<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorResult {
    pub schema_version: &'static str,
    pub ledger: OrchestrationLedger,
    pub commands_run: Vec<CommandRun>,
    pub skipped: Vec<SkippedRun>,
    pub rust_analysis_run: RustAnalysisRunResult,
    pub exit_policy: ExecutorExitPolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRun {
    pub step: String,
    pub status: String,
    pub ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<RustAnalyzerInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    pub memory: ExecutorMemoryObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedRun {
    pub step: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisRunResult {
    pub requested: bool,
    pub ran: bool,
    pub status: String,
    pub rust_files: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<RustAnalyzerInvocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorExitPolicy {
    pub base_pipeline_failed_required: bool,
    pub recommended_exit_code: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutorMemoryObservation {
    pub before: crate::orchestration_events::MemorySnapshot,
    pub after: crate::orchestration_events::MemorySnapshot,
    pub delta: crate::orchestration_events::MemorySnapshot,
}
```

Do not force the existing `OrchestrationPlan` builder structs to become deserialization structs in this slice. They intentionally contain static strings for the producer-side plan builder. `ExecutorPlanInput` is the owned typed input shape for the JSON plan after it has crossed through the JS wrapper.

- [ ] **Step 2: Export the module**

Add this line to `experiments/rust-main/lumin-audit-core/src/lib.rs`:

```rust
pub mod orchestration_executor;
```

- [ ] **Step 3: Add request validation**

In `orchestration_executor.rs`, add:

```rust
pub fn validate_executor_request(request: &ExecutorRequest) -> Result<()> {
    if request.schema_version != EXECUTOR_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-base-plan: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.plan.schema_version != ORCHESTRATION_PLAN_SCHEMA_VERSION {
        bail!(
            "execute-base-plan: unsupported plan.schemaVersion '{}'",
            request.plan.schema_version
        );
    }
    validate_non_empty("generated", &request.generated)?;
    validate_non_empty("nodeExecutable", &request.node_executable)?;
    if request.root.as_os_str().is_empty() {
        bail!("execute-base-plan: root must be provided");
    }
    if request.output.as_os_str().is_empty() {
        bail!("execute-base-plan: output must be provided");
    }
    if request.scripts_dir.as_os_str().is_empty() {
        bail!("execute-base-plan: scriptsDir must be provided");
    }
    Ok(())
}

fn validate_non_empty(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("execute-base-plan: {field} must be a non-empty string");
    }
    Ok(())
}
```

- [ ] **Step 4: Add behavior tests for malformed request and unsupported plan schema**

Create `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs` with tests that deserialize real JSON into `ExecutorRequest` and call `validate_executor_request`.

Use this helper:

```rust
use anyhow::Result;
use lumin_audit_core::orchestration_executor::{
    validate_executor_request, ExecutorRequest,
};
use lumin_audit_core::orchestration_plan::{
    build_orchestration_plan, OrchestrationPlanOptions,
};
use serde_json::{json, Value};

fn base_request() -> Value {
    json!({
        "schemaVersion": "lumin-audit-executor-request.v1",
        "plan": build_orchestration_plan(OrchestrationPlanOptions::default()),
        "generated": "2026-07-01T00:00:00.000Z",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "scriptsDir": "C:/repo",
        "nodeExecutable": "node",
        "verbose": false,
        "scanRange": {
            "includeTests": true,
            "production": false,
            "excludes": [],
            "autoExcludes": []
        },
        "cache": {
            "noIncremental": false,
            "cacheRoot": "C:/repo/.audit/.cache",
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "default" },
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 0,
            "totalReadBytes": 0,
            "totalReadMs": 0,
            "totalJsonParseMs": 0,
            "parseFailureCount": 0
        },
        "artifacts": { "producedCount": 0, "totalBytes": 0 },
        "rustAnalyzer": { "requested": false, "rustFiles": 0 }
    })
}

fn request(value: Value) -> Result<ExecutorRequest> {
    Ok(serde_json::from_value(value)?)
}
```

Add these tests:

```rust
#[test]
fn executor_request_accepts_current_plan_shape() -> Result<()> {
    let request = request(base_request())?;
    validate_executor_request(&request)?;
    Ok(())
}

#[test]
fn executor_request_rejects_wrong_schema() -> Result<()> {
    let mut value = base_request();
    value["schemaVersion"] = json!("old");
    let request = request(value)?;
    let error = validate_executor_request(&request).unwrap_err();
    assert!(error.to_string().contains("unsupported schemaVersion"));
    Ok(())
}

#[test]
fn executor_request_rejects_empty_node_executable() -> Result<()> {
    let mut value = base_request();
    value["nodeExecutable"] = json!(" ");
    let request = request(value)?;
    let error = validate_executor_request(&request).unwrap_err();
    assert!(error.to_string().contains("nodeExecutable must be a non-empty string"));
    Ok(())
}
```

- [ ] **Step 5: Run focused tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_executor
```

Expected: the new request validation tests pass.

- [ ] **Step 6: Commit protocol slice**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs experiments/rust-main/lumin-audit-core/src/lib.rs experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs
git commit -m "Add typed audit executor protocol"
```

---

### Task 2: Port Base-Step Preconditions And Planned Skips

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`

- [ ] **Step 1: Add precondition evaluation**

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreconditionOutcome {
    Met,
    Unmet,
}

fn precondition_outcome(request: &ExecutorRequest, step: &str) -> Result<PreconditionOutcome> {
    let output = &request.output;
    let root = &request.root;
    let exists_in_output = |name: &str| output.join(name).is_file();

    let met = match step {
        "build-resolver-diagnostics.mjs" | "build-entry-surface.mjs" => {
            exists_in_output("symbols.json")
        }
        "build-module-reachability.mjs" => {
            exists_in_output("symbols.json") && exists_in_output("entry-surface.json")
        }
        "export-action-safety.mjs" | "rank-fixes.mjs" => {
            exists_in_output("dead-classify.json")
        }
        "merge-runtime-evidence.mjs" => {
            root.join("coverage").join("coverage-final.json").is_file()
                || root.join(".nyc_output").join("coverage-final.json").is_file()
        }
        "measure-staleness.mjs" => is_git_work_tree(root)?,
        _ => true,
    };

    Ok(if met {
        PreconditionOutcome::Met
    } else {
        PreconditionOutcome::Unmet
    })
}
```

Add:

```rust
fn is_git_work_tree(root: &std::path::Path) -> Result<bool> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(root)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim() == "true")
        }
        Ok(_) => Ok(false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
```

- [ ] **Step 2: Add planned skip conversion**

Add:

```rust
fn planned_skips(request: &ExecutorRequest) -> Vec<SkippedRun> {
    request
        .plan
        .skipped
        .iter()
        .map(|skip| SkippedRun {
            step: skip.step.to_string(),
            reason: skip.reason.to_string(),
        })
        .collect()
}

fn push_skip(skipped: &mut Vec<SkippedRun>, step: &str, reason: &str) {
    skipped.push(SkippedRun {
        step: step.to_string(),
        reason: reason.to_string(),
    });
}
```

- [ ] **Step 3: Add behavior tests for precondition skips**

Add tests:

```rust
#[test]
fn resolver_diagnostics_skip_uses_plan_reason_when_symbols_missing() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let mut value = base_request();
    value["output"] = json!(temp.path());
    let request = request(value)?;
    let resolver = request
        .plan
        .steps
        .iter()
        .find(|step| step.step == "build-resolver-diagnostics.mjs")
        .expect("resolver step exists");
    assert_eq!(
        resolver.skip_reason_when_unmet,
        Some("symbols.json missing (symbol graph step failed or was skipped)")
    );
    assert_eq!(
        lumin_audit_core::orchestration_executor::precondition_for_test(
            &request,
            resolver.step.as_str(),
        )?,
        "unmet"
    );
    Ok(())
}
```

Expose only a test helper if needed:

```rust
#[cfg(test)]
pub fn precondition_for_test(request: &ExecutorRequest, step: &str) -> Result<&'static str> {
    Ok(match precondition_outcome(request, step)? {
        PreconditionOutcome::Met => "met",
        PreconditionOutcome::Unmet => "unmet",
    })
}
```

In integration tests call `lumin_audit_core::orchestration_executor::precondition_for_test`.

- [ ] **Step 4: Add planned SARIF skip behavior test**

Add:

```rust
#[test]
fn planned_sarif_skip_is_copied_from_plan() -> Result<()> {
    let request = request(base_request())?;
    let skipped = lumin_audit_core::orchestration_executor::planned_skips_for_test(&request);
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].step, "emit-sarif.mjs");
    assert_eq!(skipped[0].reason, "not in --sarif mode");
    Ok(())
}
```

Expose:

```rust
#[cfg(test)]
pub fn planned_skips_for_test(request: &ExecutorRequest) -> Vec<SkippedRun> {
    planned_skips(request)
}
```

- [ ] **Step 5: Run tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_executor
```

Expected: request validation, precondition, and planned skip tests pass.

- [ ] **Step 6: Commit precondition slice**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs
git commit -m "Port audit executor precondition skips"
```

---

### Task 3: Add Argv Construction For JS/MJS Base Steps

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`

- [ ] **Step 1: Add argv builder**

Add:

```rust
const INCREMENTAL_PRODUCER_STEPS: &[&str] = &[
    "measure-topology.mjs",
    "measure-staleness.mjs",
    "build-block-clone-index.mjs",
    "build-symbol-graph.mjs",
    "build-shape-index.mjs",
    "build-function-clone-index.mjs",
];

fn is_incremental_step(step: &str) -> bool {
    INCREMENTAL_PRODUCER_STEPS.contains(&step)
}

fn argv_for_js_step(request: &ExecutorRequest, script: &str) -> Vec<String> {
    let mut argv = vec![
        request
            .scripts_dir
            .join(script)
            .to_string_lossy()
            .to_string(),
        "--root".to_string(),
        request.root.to_string_lossy().to_string(),
        "--output".to_string(),
        request.output.to_string_lossy().to_string(),
    ];

    if !request.scan_range.include_tests {
        argv.push("--production".to_string());
    }
    for exclude in &request.scan_range.excludes {
        argv.push("--exclude".to_string());
        argv.push(exclude.clone());
    }
    if is_incremental_step(script) {
        if request.cache.no_incremental {
            argv.push("--no-incremental".to_string());
        }
        if !request.cache.cache_root.trim().is_empty() {
            argv.push("--cache-root".to_string());
            argv.push(request.cache.cache_root.clone());
        }
    }
    if script == "build-symbol-graph.mjs" {
        argv.push("--generated-artifacts".to_string());
        argv.push(request.generated_artifacts.mode.clone());
    }
    argv
}
```

- [ ] **Step 2: Add tests for argv parity**

Add:

```rust
#[test]
fn js_step_argv_preserves_scan_incremental_and_generated_args() -> Result<()> {
    let mut value = base_request();
    value["scanRange"]["includeTests"] = json!(false);
    value["scanRange"]["excludes"] = json!(["dist", "vendor"]);
    value["cache"]["noIncremental"] = json!(true);
    value["cache"]["cacheRoot"] = json!("C:/repo/.audit/.cache");
    value["generatedArtifacts"]["mode"] = json!("prepared");
    let request = request(value)?;
    let argv = lumin_audit_core::orchestration_executor::argv_for_js_step_for_test(
        &request,
        "build-symbol-graph.mjs",
    );

    assert!(argv.iter().any(|arg| arg == "--production"));
    assert!(argv.windows(2).any(|pair| pair == ["--exclude", "dist"]));
    assert!(argv.windows(2).any(|pair| pair == ["--exclude", "vendor"]));
    assert!(argv.iter().any(|arg| arg == "--no-incremental"));
    assert!(argv.windows(2).any(|pair| pair == ["--cache-root", "C:/repo/.audit/.cache"]));
    assert!(argv.windows(2).any(|pair| pair == ["--generated-artifacts", "prepared"]));
    Ok(())
}
```

Expose:

```rust
#[cfg(test)]
pub fn argv_for_js_step_for_test(request: &ExecutorRequest, script: &str) -> Vec<String> {
    argv_for_js_step(request, script)
}
```

- [ ] **Step 3: Run tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_executor
```

Expected: argv test passes without executing Node.

- [ ] **Step 4: Commit argv slice**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs
git commit -m "Build audit executor producer argv in Rust"
```

---

### Task 4: Add Child Process Observation

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`

- [ ] **Step 1: Add memory snapshot helpers**

Add:

```rust
fn memory_snapshot() -> crate::orchestration_events::MemorySnapshot {
    crate::orchestration_events::MemorySnapshot {
        rss_bytes: current_process_rss_bytes(),
        heap_total_bytes: 0,
        heap_used_bytes: 0,
        external_bytes: 0,
        array_buffers_bytes: 0,
    }
}

#[cfg(windows)]
fn current_process_rss_bytes() -> i64 {
    0
}

#[cfg(not(windows))]
fn current_process_rss_bytes() -> i64 {
    0
}

fn memory_delta(
    before: &crate::orchestration_events::MemorySnapshot,
    after: &crate::orchestration_events::MemorySnapshot,
) -> crate::orchestration_events::MemorySnapshot {
    crate::orchestration_events::MemorySnapshot {
        rss_bytes: after.rss_bytes - before.rss_bytes,
        heap_total_bytes: after.heap_total_bytes - before.heap_total_bytes,
        heap_used_bytes: after.heap_used_bytes - before.heap_used_bytes,
        external_bytes: after.external_bytes - before.external_bytes,
        array_buffers_bytes: after.array_buffers_bytes - before.array_buffers_bytes,
    }
}
```

This intentionally does not claim child peak RSS. If native RSS is later added, keep `childPeakRssAvailable=false` in producer-performance until a child-process RSS owner exists.

- [ ] **Step 2: Add child output runner**

Add:

```rust
struct ChildObservation {
    status: String,
    ms: u64,
    stderr_snippet: Option<String>,
    memory: ExecutorMemoryObservation,
}

fn run_child(command: &str, args: &[String], verbose: bool) -> Result<ChildObservation> {
    let before = memory_snapshot();
    let started = std::time::Instant::now();
    let output = if verbose {
        std::process::Command::new(command).args(args).status().map(|status| {
            std::process::Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        })
    } else {
        std::process::Command::new(command).args(args).output()
    }?;
    let ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let after = memory_snapshot();
    let status = if output.status.success() { "ok" } else { "failed" }.to_string();
    let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
    let stderr_snippet = (!stderr_text.trim().is_empty())
        .then(|| stderr_text.chars().take(500).collect::<String>());
    Ok(ChildObservation {
        status,
        ms,
        stderr_snippet,
        memory: ExecutorMemoryObservation {
            delta: memory_delta(&before, &after),
            before,
            after,
        },
    })
}
```

- [ ] **Step 3: Add real child process tests without Node**

In `orchestration_executor.rs`, expose a test helper:

```rust
#[cfg(test)]
pub fn run_child_for_test(command: &str, args: &[String]) -> Result<(String, Option<String>)> {
    let observed = run_child(command, args, false)?;
    Ok((observed.status, observed.stderr_snippet))
}
```

In `tests/orchestration_executor.rs`, add cross-platform helpers:

```rust
#[cfg(windows)]
fn failing_child_command() -> (String, Vec<String>) {
    (
        "cmd".to_string(),
        vec![
            "/C".to_string(),
            "echo fixture failure 1>&2 & exit /b 7".to_string(),
        ],
    )
}

#[cfg(not(windows))]
fn failing_child_command() -> (String, Vec<String>) {
    (
        "sh".to_string(),
        vec![
            "-c".to_string(),
            "printf 'fixture failure' 1>&2; exit 7".to_string(),
        ],
    )
}

#[test]
fn child_runner_captures_nonzero_stderr_without_node() -> Result<()> {
    let (command, args) = failing_child_command();
    let (status, stderr) =
        lumin_audit_core::orchestration_executor::run_child_for_test(&command, &args)?;
    assert_eq!(status, "failed");
    assert!(stderr.unwrap_or_default().contains("fixture failure"));
    Ok(())
}
```

- [ ] **Step 4: Run tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_executor
```

Expected: the real child process test passes without Node.

- [ ] **Step 5: Commit child runner slice**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs
git commit -m "Observe audit executor child process results"
```

---

### Task 5: Execute Base Plan Into One Typed Event Source

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs`
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_events.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`

- [ ] **Step 1: Make ledger event structs serializable**

In `orchestration_events.rs`, change derives:

```rust
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum LedgerEvent {
    Producer(Box<ProducerLedgerEvent>),
    Skipped(Box<SkippedLedgerEvent>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerLedgerEvent { /* existing fields */ }

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedLedgerEvent { /* existing fields */ }
```

- [ ] **Step 2: Add execution event builder**

In `orchestration_executor.rs`, add:

```rust
pub fn execute_base_plan(request: ExecutorRequest) -> Result<ExecutorResult> {
    validate_executor_request(&request)?;
    let mut commands_run = Vec::new();
    let mut skipped = planned_skips(&request);
    let mut events = Vec::new();
    let mut failed_required = false;
    let mut rust_analysis_run = not_requested_rust_analysis(&request);

    if request.plan.base_pipeline.status != "planned" {
        for skip in &skipped {
            events.push(LedgerEvent::Skipped(Box::new(
                crate::orchestration_events::SkippedLedgerEvent {
                    name: skip.step.clone(),
                    reason: skip.reason.clone(),
                },
            )));
        }
        return Ok(result_from_parts(request, commands_run, skipped, events, rust_analysis_run, false));
    }

    for step in request.plan.steps.clone() {
        if step.step == "lumin-rust-analyzer" {
            let observed = execute_rust_analyzer_step(&request)?;
            rust_analysis_run = observed.rust_analysis_run;
            commands_run.extend(observed.commands_run);
            events.extend(observed.events);
            skipped.extend(observed.skipped);
            continue;
        }
        if precondition_outcome(&request, &step.step)? == PreconditionOutcome::Unmet {
            let reason = step
                .skip_reason_when_unmet
                .as_deref()
                .unwrap_or("precondition unmet");
            push_skip(&mut skipped, &step.step, reason);
            events.push(LedgerEvent::Skipped(Box::new(
                crate::orchestration_events::SkippedLedgerEvent {
                    name: step.step.clone(),
                    reason: reason.to_string(),
                },
            )));
            continue;
        }

        let argv = argv_for_js_step(&request, &step.script);
        let observed = run_child(&request.node_executable, &argv, request.verbose)?;
        let status = if observed.status == "ok" {
            "ok".to_string()
        } else if step.required {
            failed_required = true;
            "failed-required".to_string()
        } else {
            "failed-optional".to_string()
        };
        commands_run.push(CommandRun {
            step: step.step.clone(),
            status: status.clone(),
            ms: observed.ms,
            artifact: None,
            rust_files: None,
            analyzer_invocation: None,
            stderr: observed.stderr_snippet.clone(),
            memory: observed.memory.clone(),
        });
        events.push(LedgerEvent::Producer(Box::new(
            crate::orchestration_events::ProducerLedgerEvent {
                name: step.step.clone(),
                status,
                wall_ms: Some(observed.ms),
                phases: None,
                counters: None,
                memory: Some(crate::orchestration_events::ProducerMemory {
                    before: observed.memory.before,
                    after: observed.memory.after,
                    delta: observed.memory.delta,
                }),
                stderr_snippet: observed.stderr_snippet,
            },
        )));
        if failed_required {
            break;
        }
    }

    Ok(result_from_parts(
        request,
        commands_run,
        skipped,
        events,
        rust_analysis_run,
        failed_required,
    ))
}
```

Add `result_from_parts(...)` to build `OrchestrationLedger` from request fields and the observed events:

```rust
fn result_from_parts(
    request: ExecutorRequest,
    commands_run: Vec<CommandRun>,
    skipped: Vec<SkippedRun>,
    events: Vec<LedgerEvent>,
    rust_analysis_run: RustAnalysisRunResult,
    failed_required: bool,
) -> ExecutorResult {
    ExecutorResult {
        schema_version: EXECUTOR_RESULT_SCHEMA_VERSION,
        ledger: OrchestrationLedger {
            schema_version: ORCHESTRATION_LEDGER_SCHEMA_VERSION.to_string(),
            generated: request.generated,
            root: request.root.to_string_lossy().to_string(),
            output: request.output.to_string_lossy().to_string(),
            profile: request.plan.profile.clone(),
            scan_range: request.scan_range,
            cache: request.cache,
            generated_artifacts: request.generated_artifacts,
            artifact_reads: request.artifact_reads,
            artifacts: request.artifacts,
            events,
        },
        commands_run,
        skipped,
        rust_analysis_run,
        exit_policy: ExecutorExitPolicy {
            base_pipeline_failed_required: failed_required,
            recommended_exit_code: if failed_required { 1 } else { 0 },
        },
    }
}
```

- [ ] **Step 3: Add rust analyzer request-state behavior**

For this implementation slice, preserve current JS behavior by receiving invocation from the request:

```rust
struct RustAnalyzerObserved {
    commands_run: Vec<CommandRun>,
    skipped: Vec<SkippedRun>,
    events: Vec<LedgerEvent>,
    rust_analysis_run: RustAnalysisRunResult,
}

fn not_requested_rust_analysis(request: &ExecutorRequest) -> RustAnalysisRunResult {
    RustAnalysisRunResult {
        requested: request.rust_analyzer.requested,
        ran: false,
        status: "not-requested".to_string(),
        rust_files: request.rust_analyzer.rust_files,
        reason: None,
        artifact: None,
        path: None,
        source_commit: None,
        producer: None,
        analyzer_invocation: None,
    }
}

fn execute_rust_analyzer_step(request: &ExecutorRequest) -> Result<RustAnalyzerObserved> {
    if !request.rust_analyzer.requested {
        return Ok(RustAnalyzerObserved {
            commands_run: Vec::new(),
            skipped: Vec::new(),
            events: Vec::new(),
            rust_analysis_run: not_requested_rust_analysis(request),
        });
    }
    if request.rust_analyzer.rust_files == 0 {
        let reason = "no Rust files counted by triage".to_string();
        let skipped = vec![SkippedRun {
            step: "lumin-rust-analyzer".to_string(),
            reason: reason.clone(),
        }];
        return Ok(RustAnalyzerObserved {
            commands_run: Vec::new(),
            events: vec![LedgerEvent::Skipped(Box::new(
                crate::orchestration_events::SkippedLedgerEvent {
                    name: "lumin-rust-analyzer".to_string(),
                    reason: reason.clone(),
                },
            ))],
            skipped,
            rust_analysis_run: RustAnalysisRunResult {
                requested: true,
                ran: false,
                status: "skipped".to_string(),
                rust_files: 0,
                reason: Some(reason),
                artifact: None,
                path: None,
                source_commit: None,
                producer: None,
                analyzer_invocation: None,
            },
        });
    }
    let Some(invocation) = request.rust_analyzer.invocation.clone() else {
        let reason = "rust analyzer requested but no Rust analyzer invocation was supplied".to_string();
        let skipped = vec![SkippedRun {
            step: "lumin-rust-analyzer".to_string(),
            reason: reason.clone(),
        }];
        return Ok(RustAnalyzerObserved {
            commands_run: Vec::new(),
            events: vec![LedgerEvent::Skipped(Box::new(
                crate::orchestration_events::SkippedLedgerEvent {
                    name: "lumin-rust-analyzer".to_string(),
                    reason: reason.clone(),
                },
            ))],
            skipped,
            rust_analysis_run: RustAnalysisRunResult {
                requested: true,
                ran: false,
                status: "unavailable".to_string(),
                rust_files: request.rust_analyzer.rust_files,
                reason: Some(reason),
                artifact: None,
                path: None,
                source_commit: None,
                producer: None,
                analyzer_invocation: None,
            },
        });
    };
    let artifact = "rust-analyzer-health.latest.json".to_string();
    let artifact_path = request.output.join(&artifact);
    let mut args = invocation.prefix_args.clone();
    args.extend([
        "--root".to_string(),
        request.root.to_string_lossy().to_string(),
        "--source-commit".to_string(),
        request.rust_analyzer.source_commit.clone().unwrap_or_else(|| "unknown".to_string()),
        "--output".to_string(),
        artifact_path.to_string_lossy().to_string(),
        "--source-health-profile".to_string(),
        "compact".to_string(),
        "--semantic-mode".to_string(),
        "metadata-only".to_string(),
    ]);
    args.extend(request.rust_analyzer.forwarded_args.clone());

    let observed = run_child(&invocation.command, &args, request.verbose)?;
    if observed.status == "ok" {
        let command = CommandRun {
            step: "lumin-rust-analyzer".to_string(),
            status: "ok".to_string(),
            ms: observed.ms,
            artifact: Some(artifact.clone()),
            rust_files: Some(request.rust_analyzer.rust_files),
            analyzer_invocation: Some(invocation.clone()),
            stderr: None,
            memory: observed.memory.clone(),
        };
        return Ok(RustAnalyzerObserved {
            commands_run: vec![command],
            skipped: Vec::new(),
            events: vec![LedgerEvent::Producer(Box::new(
                crate::orchestration_events::ProducerLedgerEvent {
                    name: "lumin-rust-analyzer".to_string(),
                    status: "ok".to_string(),
                    wall_ms: Some(observed.ms),
                    phases: None,
                    counters: None,
                    memory: Some(crate::orchestration_events::ProducerMemory {
                        before: observed.memory.before.clone(),
                        after: observed.memory.after.clone(),
                        delta: observed.memory.delta.clone(),
                    }),
                    stderr_snippet: None,
                },
            ))],
            rust_analysis_run: RustAnalysisRunResult {
                requested: true,
                ran: true,
                status: "complete".to_string(),
                rust_files: request.rust_analyzer.rust_files,
                reason: None,
                artifact: Some(artifact),
                path: Some(artifact_path.to_string_lossy().to_string()),
                source_commit: request.rust_analyzer.source_commit.clone(),
                producer: Some("lumin-rust-analyzer".to_string()),
                analyzer_invocation: Some(invocation),
            },
        });
    }
    let reason = "lumin-rust-analyzer exited non-zero".to_string();
    Ok(RustAnalyzerObserved {
        commands_run: vec![CommandRun {
            step: "lumin-rust-analyzer".to_string(),
            status: "failed-optional".to_string(),
            ms: observed.ms,
            artifact: None,
            rust_files: Some(request.rust_analyzer.rust_files),
            analyzer_invocation: None,
            stderr: observed.stderr_snippet.clone(),
            memory: observed.memory,
        }],
        skipped: Vec::new(),
        events: Vec::new(),
        rust_analysis_run: RustAnalysisRunResult {
            requested: true,
            ran: false,
            status: "failed-optional".to_string(),
            rust_files: request.rust_analyzer.rust_files,
            reason: Some(reason),
            artifact: None,
            path: None,
            source_commit: None,
            producer: None,
            analyzer_invocation: None,
        },
    })
}
```

- [ ] **Step 4: Add behavior tests for optional failure and required halt**

Use a test-only plan constructed in the test by taking `base_request()` and replacing `plan.steps` with two fixture steps whose scripts are interpreted by a test child runner. If direct `execute_base_plan` cannot inject a runner, add a private `execute_base_plan_with_runner` function and keep `execute_base_plan` using the real runner.

The test must assert:

```rust
assert_eq!(result.commands_run[0].status, "failed-optional");
assert_eq!(result.commands_run[1].status, "ok");
assert_eq!(result.exit_policy.recommended_exit_code, 0);
assert!(result.ledger.events.iter().any(|event| matches!(event, LedgerEvent::Producer(_))));
```

For required halt:

```rust
assert_eq!(result.commands_run[0].status, "failed-required");
assert_eq!(result.commands_run.len(), 1);
assert_eq!(result.exit_policy.base_pipeline_failed_required, true);
assert_eq!(result.exit_policy.recommended_exit_code, 1);
```

Do not use mock-only assertions as the only coverage. Keep the `run_child_for_test` real process test from Task 4.

- [ ] **Step 5: Run tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_executor --test orchestration_events
```

Expected: executor behavior tests and existing ledger projection tests pass.

- [ ] **Step 6: Commit execution result slice**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs experiments/rust-main/lumin-audit-core/src/orchestration_events.rs experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs
git commit -m "Execute audit base plan through typed events"
```

---

### Task 6: Add `execute-base-plan` CLI

**Files:**
- Modify: `experiments/rust-main/lumin-audit-core/src/cli.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs`

- [ ] **Step 1: Wire CLI command**

Import:

```rust
use lumin_audit_core::orchestration_executor::{execute_base_plan, ExecutorRequest};
```

Add to `USAGE`:

```text
lumin-audit-core execute-base-plan --input <path|->
```

Add match arm:

```rust
Some("execute-base-plan") => run_execute_base_plan(args.collect()),
```

Add function:

```rust
fn run_execute_base_plan(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-base-plan: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-base-plan: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-base-plan")?;
    let request = serde_json::from_value::<ExecutorRequest>(json)
        .context("execute-base-plan: invalid request shape")?;
    let result = execute_base_plan(request)?;
    write_stdout_json(&result)
}
```

- [ ] **Step 2: Add CLI malformed request test**

Add:

```rust
#[test]
fn cli_execute_base_plan_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    std::fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("execute-base-plan")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("execute-base-plan")
    );
    Ok(())
}
```

- [ ] **Step 3: Run CLI tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_executor
```

Expected: CLI hard-stop behavior is covered.

- [ ] **Step 4: Commit CLI slice**

Run:

```powershell
git add experiments/rust-main/lumin-audit-core/src/cli.rs experiments/rust-main/lumin-audit-core/tests/orchestration_executor.rs
git commit -m "Expose audit base executor CLI"
```

---

### Task 7: Thin JS Wrapper Around Rust Executor

**Files:**
- Modify: `_lib/audit-manifest.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`
- Modify: `audit-repo.mjs`
- Modify: `skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs`

- [ ] **Step 1: Add wrapper**

Add to both `_lib/audit-manifest.mjs` files:

```js
export function executeBasePlan(request) {
  return runAuditCoreJson([
    'execute-base-plan',
    '--input', '-',
  ], 'executeBasePlan', {
    input: JSON.stringify(request ?? {}),
  });
}
```

- [ ] **Step 2: Import wrapper**

In both `audit-repo.mjs` files, add `executeBasePlan` to the import from audit manifest helpers.

- [ ] **Step 3: Build executor request in JS**

Add a small function near `buildProducerPerformanceArtifact`:

```js
function buildExecutorRequest(generated, artifactsProduced) {
  return {
    schemaVersion: 'lumin-audit-executor-request.v1',
    plan: ORCHESTRATION_PLAN,
    generated,
    root: ROOT,
    output: OUT,
    scriptsDir: __dirname,
    nodeExecutable: process.execPath,
    verbose: values.verbose === true,
    scanRange: {
      includeTests: INCLUDE_TESTS,
      production: PRODUCTION,
      excludes: EFFECTIVE_EXCLUDES,
      autoExcludes: AUTO_EXCLUDES,
    },
    cache: {
      noIncremental: values['no-incremental'] === true,
      cacheRoot: performanceCacheRoot(),
      clearIncrementalCache: values['clear-incremental-cache'] === true,
    },
    generatedArtifacts: {
      mode: GENERATED_ARTIFACTS_MODE,
    },
    artifactReads: artifactReadMetrics.summary(),
    artifacts: buildArtifactSizeSummary(OUT, artifactsProduced),
    rustAnalyzer: {
      requested: values['rust-analyzer'] === true,
      rustFiles: rustFileCountFromTriage(loadIfExists('triage.json')),
      sourceCommit: gitHeadCommit(ROOT),
      invocation: values['rust-analyzer'] === true ? rustAnalyzerInvocationOrNull() : null,
      forwardedArgs: forwardedRustAnalyzerArgs(),
    },
  };
}

function rustAnalyzerInvocationOrNull() {
  try {
    const invocation = rustAnalyzerInvocation();
    return {
      command: invocation.command,
      prefixArgs: invocation.prefixArgs,
      source: invocation.source,
      ...(invocation.manifestPath ? { manifestPath: invocation.manifestPath } : {}),
    };
  } catch {
    return null;
  }
}
```

This keeps invocation resolution in JS for the first slice, matching the spec.

- [ ] **Step 4: Replace base pipeline execution**

Replace:

```js
if (!RUN_BASE_PIPELINE) {
  recordPlannedSkip('base-audit-profile', 'base audit profile skipped by Rust orchestration plan');
} else {
  runBasePipelineFromPlan(ORCHESTRATION_PLAN);
  if (!EMIT_SARIF && plannedSkip('emit-sarif.mjs')) {
    recordPlannedSkip('emit-sarif.mjs', 'not in --sarif mode');
  }
}
```

with:

```js
const baseExecutorGenerated = new Date().toISOString();
const initialArtifactsProduced = collectProducedArtifacts(OUT);
const baseExecution = executeBasePlan(buildExecutorRequest(
  baseExecutorGenerated,
  initialArtifactsProduced
));
commandsRun.push(...(baseExecution.commandsRun ?? []));
skipped.push(...(baseExecution.skipped ?? []));
rustAnalysisRun = baseExecution.rustAnalysisRun ?? rustAnalysisRun;
let basePipelineExitCode = Number(baseExecution.exitPolicy?.recommendedExitCode ?? 0);
```

Then include `basePipelineExitCode` in final exit policy:

```js
let finalExitCode = basePipelineExitCode;
```

Do not remove lifecycle execution paths in this task.

- [ ] **Step 5: Delete base JS execution helpers after wiring**

Remove from both `audit-repo.mjs` files only after Step 4 compiles:

- `runStep`
- `runRustAnalyzerStep`
- `plannedStepPrecondition`
- `runPlannedBaseStep`
- `runBasePipelineFromPlan`
- `recordPlannedSkip` if no longer used outside base execution
- `hasCoverage` and `isGitWorkTree` if no longer used

Keep:

- `rustAnalyzerInvocation`
- `rustFileCountFromTriage`
- `gitHeadCommit`
- `forwardedRustAnalyzerArgs`
- `forwardedScanArgs` only if still used by lifecycle helpers; otherwise remove
- `performanceCacheRoot`
- `artifactReadMetrics`
- human renderer and final manifest write code

- [ ] **Step 6: Static verification**

Run:

```powershell
rg -n "function runStep|function runRustAnalyzerStep|runBasePipelineFromPlan|plannedStepPrecondition" audit-repo.mjs skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs
```

Expected: no matches.

- [ ] **Step 7: Commit JS wrapper slice**

Run:

```powershell
git add _lib/audit-manifest.mjs skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs audit-repo.mjs skills/lumin-repo-lens-lab/_engine/producers/audit-repo.mjs
git commit -m "Route base audit execution through audit core"
```

---

### Task 8: Canonical Owner Update And Plan Execution Owner Flip

**Files:**
- Modify: `canonical/audit-core.md`
- Modify: `experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs`
- Test: `experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs`

- [ ] **Step 1: Update canonical owner table**

In `canonical/audit-core.md`, add:

```markdown
| `experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs` | Base audit child-process execution for planned base pipeline steps, typed `commandsRun` / `skipped` value production for those steps, child status/wall/stderr observation, and ledger event production from the same observed events | JS/TS producer internals, lifecycle child execution, artifact-read timing, phase timing reads, human renderers, `blindZones`, final `manifest.json` writing |
```

Update the remaining JS-owned row for child execution to say lifecycle child helpers remain JS-owned, while base pipeline execution is Rust-owned.

- [ ] **Step 2: Flip base step execution owner**

In `orchestration_plan.rs`, change base step `execution_owner` from `"audit-repo.mjs"` to `"lumin-audit-core"` for base pipeline steps only.

Keep lifecycle plans unchanged:

```rust
execution_owner: "audit-repo.mjs",
```

for `pre_write`, `post_write`, `canon_draft`, and `check_canon`.

- [ ] **Step 3: Update orchestration plan tests**

In `tests/orchestration_plan.rs`, update assertions:

```rust
assert_eq!(plan["executionOwner"], "lumin-audit-core");
```

For lifecycle assertions add:

```rust
assert_eq!(plan["lifecycle"]["preWrite"]["executionOwner"], "audit-repo.mjs");
assert_eq!(plan["lifecycle"]["postWrite"]["executionOwner"], "audit-repo.mjs");
```

For serialized step precondition expected JSON, change:

```json
"executionOwner": "lumin-audit-core"
```

- [ ] **Step 4: Run tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core --test orchestration_plan --test orchestration_executor
```

Expected: plan owner assertions and executor tests pass.

- [ ] **Step 5: Commit owner flip**

Run:

```powershell
git add canonical/audit-core.md experiments/rust-main/lumin-audit-core/src/orchestration_plan.rs experiments/rust-main/lumin-audit-core/tests/orchestration_plan.rs
git commit -m "Mark base audit execution rust owned"
```

---

### Task 9: Final Rust Verification

**Files:**
- No code changes expected.

- [ ] **Step 1: Run full audit-core formatting**

Run:

```powershell
cargo fmt --manifest-path experiments\Cargo.toml --all -- --check
```

Expected: exit 0.

- [ ] **Step 2: Run full audit-core tests**

Run:

```powershell
cargo test --manifest-path experiments\Cargo.toml -p lumin-audit-core
```

Expected: exit 0, all `lumin-audit-core` tests pass.

- [ ] **Step 3: Run clippy**

Run:

```powershell
cargo clippy --manifest-path experiments\Cargo.toml -p lumin-audit-core --all-targets -- -D warnings
```

Expected: exit 0.

- [ ] **Step 4: Run diff whitespace check**

Run:

```powershell
git diff --check
```

Expected: exit 0.

- [ ] **Step 5: Do not run Node**

Do not run `node`, `npm`, `pnpm`, `vitest`, or `audit-repo.mjs` unless the user explicitly approves it for this execution pass.

- [ ] **Step 6: Report remaining JS owners**

Run:

```powershell
rg -n "function runStep|function runRustAnalyzerStep|blindZones|renderAuditSummary|renderAuditReviewPack|writeFileSync\\(manifestPath|preWriteBlock|postWriteBlock|canonDraft|checkCanon" audit-repo.mjs canonical/audit-core.md
```

Expected:

- no base `runStep` / `runRustAnalyzerStep` implementation;
- `blindZones`, lifecycle blocks, human renderers, and final manifest write still present and documented as JS-owned.

---

## Self-Review

- Spec coverage:
  - Base child execution: Tasks 4, 5, 7, 8.
  - JS/MJS producer internals remain JS-owned: Tasks 3 and 7 keep script argv and do not port producer logic.
  - Lifecycle helpers remain JS-owned: Tasks 7 and 8 explicitly preserve them.
  - `blindZones`, human renderers, final manifest write remain out of scope: Tasks 7, 8, 9.
  - No timeout/caps/quotas: Tasks 4, 5, 9.
  - Artifact-visible omissions: Tasks 2, 5.
- Completion-marker scan:
  - No incomplete-marker strings remain in task bodies.
  - Every task names files, concrete code direction, commands, and expected outcomes.
- Type consistency:
  - Request/result names match the spec: `ExecutorRequest`, `ExecutorResult`, `execute-base-plan`.
  - Ledger types are reused from `orchestration_events.rs`; no duplicate product artifact shape is introduced.
  - JS wrapper uses `executeBasePlan(...)` consistently in root and skill mirror.
