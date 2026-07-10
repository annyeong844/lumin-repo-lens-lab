use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lumin_audit_core::orchestration_executor::{
    execute_base_plan, execute_runtime_request, validate_executor_request, ExecutorRequest,
    RuntimeExecutorRequest,
};
use lumin_audit_core::orchestration_plan::{build_orchestration_plan, OrchestrationPlanOptions};

fn base_request() -> Value {
    json!({
        "schemaVersion": "lumin-audit-executor-request.v2",
        "runId": "test-run",
        "plan": build_orchestration_plan(OrchestrationPlanOptions::default()),
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
        "rustAnalyzer": { "requested": false, "rustFiles": 0 }
    })
}

fn request(value: Value) -> Result<ExecutorRequest> {
    Ok(serde_json::from_value(value)?)
}

fn runtime_request(value: Value) -> Result<RuntimeExecutorRequest> {
    Ok(serde_json::from_value(value)?)
}

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
    let error = validate_executor_request(&request)
        .err()
        .ok_or_else(|| anyhow!("wrong request schema should fail"))?;
    assert!(error.to_string().contains("unsupported schemaVersion"));
    Ok(())
}

#[test]
fn executor_request_rejects_empty_node_executable() -> Result<()> {
    let mut value = base_request();
    value["nodeExecutable"] = json!(" ");
    let request = request(value)?;
    let error = validate_executor_request(&request)
        .err()
        .ok_or_else(|| anyhow!("empty node executable should fail"))?;
    assert!(error
        .to_string()
        .contains("nodeExecutable must be a non-empty string"));
    Ok(())
}

#[test]
fn resolver_diagnostics_skip_uses_plan_reason_when_symbols_missing() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let mut value = base_request();
    value["root"] = json!(path_string(temp.path()));
    value["output"] = json!(path_string(&temp.path().join("out")));
    value["plan"]["steps"] = json!([{
        "step": "build-resolver-diagnostics.mjs",
        "script": "build-resolver-diagnostics.mjs",
        "required": false,
        "producerOwner": "js-mjs",
        "executionOwner": "audit-repo.mjs",
        "skipReasonWhenUnmet": "symbols.json missing (symbol graph step failed or was skipped)"
    }]);
    value["plan"]["skipped"] = json!([]);

    let result = execute_base_plan(request(value)?)?;
    assert!(result.commands_run.is_empty());
    assert_eq!(result.skipped.len(), 1);
    assert_eq!(
        result.skipped[0].reason,
        "symbols.json missing (symbol graph step failed or was skipped)"
    );
    Ok(())
}

#[test]
fn planned_sarif_skip_is_copied_from_plan() -> Result<()> {
    let mut value = base_request();
    value["plan"]["steps"] = json!([]);
    let result = execute_base_plan(request(value)?)?;

    assert_eq!(result.skipped.len(), 1);
    assert_eq!(result.skipped[0].step, "emit-sarif.mjs");
    assert_eq!(result.skipped[0].reason, "not in --sarif mode");
    Ok(())
}

#[test]
fn runtime_executor_builds_plan_inside_rust_before_execution() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let value = runtime_request_json(temp.path());
    let result = execute_runtime_request(runtime_request(value)?)?;

    let serialized = serde_json::to_value(&result)?;
    assert_eq!(
        serialized["schemaVersion"],
        "lumin-audit-runtime-executor-result.v1"
    );
    assert_eq!(serialized["plan"]["planOwner"], "lumin-audit-core");
    assert_eq!(serialized["plan"]["profile"], "quick");
    assert_eq!(serialized["plan"]["basePipeline"]["status"], "skipped");
    assert!(result.commands_run.is_empty());
    assert_eq!(result.skipped.len(), 1);
    assert_eq!(result.skipped[0].step, "base-audit-profile");
    Ok(())
}

#[test]
fn runtime_executor_validation_uses_runtime_command_label() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let mut value = runtime_request_json(temp.path());
    value["nodeExecutable"] = json!(" ");
    let error = execute_runtime_request(runtime_request(value)?)
        .err()
        .ok_or_else(|| anyhow!("empty runtime node executable should fail"))?;

    assert!(error.to_string().contains("execute-base-runtime"));
    assert!(error.to_string().contains("nodeExecutable"));
    Ok(())
}

#[test]
fn cli_execute_base_runtime_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-base-runtime")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-base-runtime"));
    Ok(())
}

#[test]
fn js_step_argv_preserves_scan_incremental_and_generated_args() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(
        temp.path(),
        temp.path(),
        false,
        &["dist", "vendor"],
        &["src/lib.rs", "src/main.ts"],
    )?;
    let mut value = base_request();
    value["root"] = json!(path_string(temp.path()));
    value["output"] = json!(path_string(&temp.path().join("out")));
    value["scriptsDir"] = json!(path_string(temp.path()));
    value["nodeExecutable"] = json!(path_string(&fake_node));
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        fixture_step("build-symbol-graph.mjs", true)
    ]);
    value["plan"]["skipped"] = json!([]);
    value["scanRange"]["includeTests"] = json!(false);
    value["scanRange"]["excludes"] = json!(["dist", "vendor"]);
    value["cache"]["noIncremental"] = json!(true);
    value["cache"]["cacheRoot"] = json!("C:/repo/.audit/.cache");
    value["generatedArtifacts"]["mode"] = json!("prepared");

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run.len(), 2);
    assert_eq!(result.commands_run[1].status, "ok");

    let argv_log = fs::read_to_string(temp.path().join("fake-node-args.txt"))?;
    assert!(argv_log.contains("--production"));
    assert!(argv_log.contains("--exclude dist"));
    assert!(argv_log.contains("--exclude vendor"));
    assert!(argv_log.contains("--no-incremental"));
    assert!(argv_log.contains("--cache-root C:/repo/.audit/.cache"));
    assert!(argv_log.contains("--generated-artifacts prepared"));
    Ok(())
}

#[test]
fn stale_inventory_is_replaced_and_forwarded_to_later_steps() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let output = temp.path().join("out");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(&output)?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(temp.path(), &root, true, &[], &["src/stale.rs"])?;
    fs::rename(
        temp.path().join("source-inventory.template.json"),
        output.join("source-inventory.json"),
    )?;
    write_source_inventory_template(
        temp.path(),
        &root,
        true,
        &[],
        &["src/lib.rs", "src/main.ts"],
    )?;

    let mut value = base_request();
    value["root"] = json!(path_string(&root));
    value["output"] = json!(path_string(&output));
    value["scriptsDir"] = json!(path_string(temp.path()));
    value["nodeExecutable"] = json!(path_string(&fake_node));
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        fixture_step("build-symbol-graph.mjs", true)
    ]);
    value["plan"]["skipped"] = json!([]);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run.len(), 2);
    assert!(result.commands_run.iter().all(|run| run.status == "ok"));

    let lines = fs::read_to_string(temp.path().join("fake-node-args.txt"))?
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(!lines[0].contains("--source-inventory "));
    assert!(lines[0].contains("--source-inventory-run-id test-run"));
    assert!(lines[1].contains("--source-inventory"));
    assert!(lines[1].contains("--source-inventory-run-id test-run"));
    assert!(lines[1].contains("source-inventory.json"));
    Ok(())
}

#[test]
fn successful_triage_without_inventory_hard_stops() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let output = temp.path().join("out");
    fs::create_dir_all(&root)?;
    fs::create_dir_all(&output)?;
    let fake_node = write_fake_node(temp.path())?;

    let mut value = base_request();
    value["root"] = json!(path_string(&root));
    value["output"] = json!(path_string(&output));
    value["scriptsDir"] = json!(path_string(temp.path()));
    value["nodeExecutable"] = json!(path_string(&fake_node));
    value["plan"]["steps"] = json!([fixture_step("triage-repo.mjs", true)]);
    value["plan"]["skipped"] = json!([]);

    let error = execute_base_plan(request(value)?)
        .err()
        .ok_or_else(|| anyhow!("successful triage without inventory must fail closed"))?;
    assert!(error.to_string().contains("did not produce a valid"));
    Ok(())
}

#[test]
fn producer_without_current_run_inventory_hard_stops() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    let mut value = request_with_fake_node(temp.path(), &fake_node);
    value["plan"]["steps"] = json!([fixture_step("build-symbol-graph.mjs", true)]);
    value["plan"]["skipped"] = json!([]);

    let error = execute_base_plan(request(value)?).err().ok_or_else(|| {
        anyhow!("orchestrated producers must not fall back to their own repository walk")
    })?;
    assert!(error
        .to_string()
        .contains("cannot run before current-run source inventory"));
    assert!(!temp.path().join("fake-node-args.txt").exists());
    Ok(())
}

#[test]
fn rust_only_inventory_skips_non_rust_base_producers() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(temp.path(), temp.path(), true, &[], &["src/lib.rs"])?;
    let mut value = request_with_fake_node(temp.path(), &fake_node);
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        fixture_step("build-framework-resource-surfaces.mjs", true),
        fixture_step("build-symbol-graph.mjs", true),
        fixture_step("classify-dead-exports.mjs", true)
    ]);
    value["plan"]["skipped"] = json!([]);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run.len(), 1);
    assert_eq!(result.commands_run[0].step, "triage-repo.mjs");
    assert_eq!(result.skipped.len(), 3);
    assert!(result.skipped.iter().all(|skip| {
        skip.reason
            == "current-run source inventory contains Rust files and no supported non-Rust source files"
    }));
    assert_eq!(
        fs::read_to_string(temp.path().join("fake-node-args.txt"))?
            .lines()
            .count(),
        1
    );
    Ok(())
}

#[test]
fn base_step_removes_stale_phase_sidecar_before_running() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(
        temp.path(),
        temp.path(),
        true,
        &[],
        &["src/lib.rs", "src/main.ts"],
    )?;
    let output = temp.path().join("out");
    let phase_dir = output.join(".producer-phases");
    fs::create_dir_all(&phase_dir)?;
    let stale_phase = phase_dir.join("build-symbol-graph.mjs.json");
    fs::write(
        &stale_phase,
        r#"{"schemaVersion":"producer-phase-timing.v1"}"#,
    )?;

    let mut value = request_with_fake_node(temp.path(), &fake_node);
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        fixture_step("build-symbol-graph.mjs", true)
    ]);
    value["plan"]["skipped"] = json!([]);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run[1].status, "ok");
    assert!(!stale_phase.exists());
    Ok(())
}

#[test]
fn optional_failure_continues_and_emits_typed_event() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(
        temp.path(),
        temp.path(),
        true,
        &[],
        &["src/lib.rs", "src/main.ts"],
    )?;
    let mut value = request_with_fake_node(temp.path(), &fake_node);
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        fixture_step("fail-optional.mjs", false),
        fixture_step("ok-after-optional.mjs", false)
    ]);
    value["plan"]["skipped"] = json!([]);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run.len(), 3);
    assert_eq!(result.commands_run[1].status, "failed-optional");
    assert_eq!(result.commands_run[2].status, "ok");
    assert_eq!(result.exit_policy.recommended_exit_code, 0);
    assert!(serde_json::to_value(&result.events)?
        .as_array()
        .is_some_and(|events| events.iter().any(|event| event["kind"] == "producer")));
    Ok(())
}

#[test]
fn required_failure_halts_and_recommends_nonzero_exit() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(
        temp.path(),
        temp.path(),
        true,
        &[],
        &["src/lib.rs", "src/main.ts"],
    )?;
    let mut value = request_with_fake_node(temp.path(), &fake_node);
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        fixture_step("fail-required.mjs", true),
        fixture_step("must-not-run.mjs", false)
    ]);
    value["plan"]["skipped"] = json!([]);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run.len(), 2);
    assert_eq!(result.commands_run[1].status, "failed-required");
    assert!(result.exit_policy.base_pipeline_failed_required);
    assert_eq!(result.exit_policy.recommended_exit_code, 1);
    Ok(())
}

#[test]
fn rust_analyzer_requested_without_rust_files_is_artifact_visible_skip() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_node(temp.path())?;
    write_source_inventory_template(temp.path(), temp.path(), true, &[], &[])?;
    let mut value = base_request();
    value["root"] = json!(path_string(temp.path()));
    value["output"] = json!(path_string(&temp.path().join("out")));
    value["scriptsDir"] = json!(path_string(temp.path()));
    value["nodeExecutable"] = json!(path_string(&fake_node));
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        {
            "step": "lumin-rust-analyzer",
            "script": "lumin-rust-analyzer",
            "required": false,
            "producerOwner": "rust",
            "executionOwner": "audit-repo.mjs"
        }
    ]);
    value["plan"]["skipped"] = json!([]);
    value["rustAnalyzer"] = json!({ "requested": true, "rustFiles": 0 });

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.rust_analysis_run.status, "skipped");
    assert_eq!(result.skipped.len(), 1);
    assert_eq!(result.skipped[0].step, "lumin-rust-analyzer");
    Ok(())
}

#[test]
fn rust_analyzer_success_preserves_public_invocation_shape_without_command() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let analyzer = write_fake_analyzer(temp.path(), true)?;
    let mut value = rust_analyzer_request(temp.path(), &analyzer)?;
    value["rustAnalyzer"]["invocation"]["prefixArgs"] = json!(["--execution-only"]);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.commands_run.len(), 2);
    assert_eq!(result.commands_run[1].status, "ok");
    assert_eq!(result.rust_analysis_run.status, "complete");

    let command = serde_json::to_value(&result.commands_run[1])?;
    assert_eq!(command["analyzerInvocation"]["source"], "cargo-run");
    assert_eq!(
        command["analyzerInvocation"]["manifestPath"],
        "experiments/Cargo.toml"
    );
    assert!(command["analyzerInvocation"].get("command").is_none());
    assert!(command["analyzerInvocation"].get("prefixArgs").is_none());

    let run = serde_json::to_value(&result.rust_analysis_run)?;
    assert_eq!(run["analyzerInvocation"]["source"], "cargo-run");
    assert!(run["analyzerInvocation"].get("command").is_none());
    assert!(run["analyzerInvocation"].get("prefixArgs").is_none());
    Ok(())
}

#[test]
fn rust_analyzer_fills_missing_source_commit_inside_audit_core() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let analyzer = write_fake_analyzer(temp.path(), true)?;
    let mut value = rust_analyzer_request(temp.path(), &analyzer)?;
    value["rustAnalyzer"]
        .as_object_mut()
        .ok_or_else(|| anyhow!("rustAnalyzer should be an object"))?
        .remove("sourceCommit");

    let result = execute_base_plan(request(value)?)?;

    assert_eq!(result.rust_analysis_run.status, "complete");
    assert_eq!(
        result.rust_analysis_run.source_commit.as_deref(),
        Some("unknown")
    );
    Ok(())
}

#[test]
fn rust_analyzer_uses_current_inventory_over_stale_request_count() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let analyzer = write_fake_analyzer(temp.path(), true)?;
    let output = temp.path().join("out");
    fs::create_dir_all(&output)?;
    fs::write(
        output.join("triage.json"),
        r#"{"byLanguage":{"rs":{"files":4}}}"#,
    )?;
    let mut value = rust_analyzer_request(temp.path(), &analyzer)?;
    write_source_inventory_template(
        temp.path(),
        temp.path(),
        true,
        &[],
        &["a.rs", "b.rs", "c.rs", "d.rs"],
    )?;
    value["rustAnalyzer"]["rustFiles"] = json!(99);

    let result = execute_base_plan(request(value)?)?;
    assert_eq!(result.rust_analysis_run.status, "complete");
    assert_eq!(result.rust_analysis_run.rust_files, 4);
    assert_eq!(result.commands_run[1].rust_files, Some(4));
    Ok(())
}

#[test]
fn rust_analyzer_failure_records_optional_event_without_public_invocation() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let analyzer = write_fake_analyzer(temp.path(), false)?;
    let result = execute_base_plan(request(rust_analyzer_request(temp.path(), &analyzer)?)?)?;

    assert_eq!(result.commands_run.len(), 2);
    assert_eq!(result.commands_run[1].status, "failed-optional");
    assert_eq!(result.rust_analysis_run.status, "failed-optional");

    let command = serde_json::to_value(&result.commands_run[1])?;
    assert!(command.get("analyzerInvocation").is_none());
    assert!(command["stderr"]
        .as_str()
        .unwrap_or_default()
        .contains("analyzer failure"));

    let events = serde_json::to_value(&result.events)?;
    let failed = events
        .as_array()
        .and_then(|events| {
            events
                .iter()
                .find(|event| event["status"] == "failed-optional")
        })
        .ok_or_else(|| anyhow!("failed analyzer event should be present"))?;
    assert_eq!(failed["kind"], "producer");
    assert!(failed["stderrSnippet"]
        .as_str()
        .unwrap_or_default()
        .contains("analyzer failure"));
    Ok(())
}

#[test]
fn rust_analyzer_spawn_failure_records_optional_event() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let missing = temp.path().join("missing-analyzer");
    let result = execute_base_plan(request(rust_analyzer_request(temp.path(), &missing)?)?)?;

    assert_eq!(result.commands_run.len(), 2);
    assert_eq!(result.commands_run[1].status, "failed-optional");
    assert_eq!(result.rust_analysis_run.status, "failed-optional");
    assert_eq!(
        result.rust_analysis_run.reason.as_deref(),
        Some("lumin-rust-analyzer did not complete")
    );
    assert!(result.commands_run[1]
        .stderr
        .as_deref()
        .unwrap_or_default()
        .contains("failed to start child process"));
    assert_eq!(result.exit_policy.recommended_exit_code, 0);
    Ok(())
}

#[test]
fn rust_analyzer_failure_removes_stale_artifact_before_running() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let analyzer = write_fake_analyzer(temp.path(), false)?;
    let output = temp.path().join("out");
    fs::create_dir_all(&output)?;
    let stale_artifact = output.join("rust-analyzer-health.latest.json");
    fs::write(&stale_artifact, r#"{"stale":true}"#)?;

    let result = execute_base_plan(request(rust_analyzer_request(temp.path(), &analyzer)?)?)?;

    assert_eq!(result.rust_analysis_run.status, "failed-optional");
    assert!(!stale_artifact.exists());
    Ok(())
}

#[test]
fn cli_execute_base_plan_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-base-plan")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-base-plan"));
    Ok(())
}

fn runtime_request_json(root: &Path) -> Value {
    let output = root.join("out");
    json!({
        "schemaVersion": "lumin-audit-runtime-executor-request.v2",
        "runId": "test-run",
        "profile": "quick",
        "preWrite": true,
        "root": path_string(root),
        "output": path_string(&output),
        "scriptsDir": path_string(root),
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
            "cacheRoot": path_string(&output.join(".cache")),
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "default" },
        "rustAnalyzer": { "requested": false, "rustFiles": 0 }
    })
}

fn request_with_fake_node(root: &Path, fake_node: &Path) -> Value {
    let output = root.join("out");
    json!({
        "schemaVersion": "lumin-audit-executor-request.v2",
        "runId": "test-run",
        "plan": build_orchestration_plan(OrchestrationPlanOptions::default()),
        "root": path_string(root),
        "output": path_string(&output),
        "scriptsDir": path_string(root),
        "nodeExecutable": path_string(fake_node),
        "verbose": false,
        "scanRange": {
            "includeTests": true,
            "production": false,
            "excludes": [],
            "autoExcludes": []
        },
        "cache": {
            "noIncremental": false,
            "cacheRoot": path_string(&output.join(".cache")),
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "default" },
        "rustAnalyzer": { "requested": false, "rustFiles": 0 }
    })
}

fn fixture_step(script: &str, required: bool) -> Value {
    json!({
        "step": script,
        "script": script,
        "required": required,
        "producerOwner": "js-mjs",
        "executionOwner": "audit-repo.mjs"
    })
}

fn write_source_inventory_template(
    fixture_dir: &Path,
    root: &Path,
    include_tests: bool,
    excludes: &[&str],
    files: &[&str],
) -> Result<()> {
    let rs_files = files.iter().filter(|file| file.ends_with(".rs")).count();
    let ts_files = files.iter().filter(|file| file.ends_with(".ts")).count();
    fs::write(
        fixture_dir.join("source-inventory.template.json"),
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-source-inventory.v2",
            "producer": "triage-repo.mjs",
            "runId": "test-run",
            "root": root,
            "pathMode": "repo-relative",
            "walkScope": {
                "includeTests": true,
                "exclude": excludes,
                "languages": ["rs", "ts"],
                "policyVersion": "lumin-source-walk.v1"
            },
            "analysisScope": {
                "includeTests": include_tests,
                "exclude": excludes
            },
            "fileCount": files.len(),
            "countsByLanguage": { "rs": rs_files, "ts": ts_files },
            "files": files
        }))?,
    )?;
    Ok(())
}

fn rust_analyzer_request(root: &Path, analyzer: &Path) -> Result<Value> {
    let fake_node = write_fake_node(root)?;
    write_source_inventory_template(root, root, true, &[], &["src/lib.rs"])?;
    let mut value = request_with_fake_node(root, &fake_node);
    value["plan"]["steps"] = json!([
        fixture_step("triage-repo.mjs", true),
        {
            "step": "lumin-rust-analyzer",
            "script": "lumin-rust-analyzer",
            "required": false,
            "producerOwner": "rust",
            "executionOwner": "audit-repo.mjs"
        }
    ]);
    value["plan"]["skipped"] = json!([]);
    value["rustAnalyzer"] = json!({
        "requested": true,
        "rustFiles": 3,
        "sourceCommit": "abc123",
        "invocation": {
            "command": path_string(analyzer),
            "prefixArgs": [],
            "source": "cargo-run",
            "manifestPath": "experiments/Cargo.toml"
        },
        "forwardedArgs": ["--fixture-forwarded"]
    });
    Ok(value)
}

#[cfg(windows)]
fn write_fake_node(dir: &Path) -> Result<PathBuf> {
    let path = dir.join("fake-node.cmd");
    fs::write(
        &path,
        "@echo off\r\necho %*>>\"%~dp0fake-node-args.txt\"\r\nif /I not \"%~nx1\"==\"triage-repo.mjs\" goto aftertriage\r\nif not exist \"%~dp0source-inventory.template.json\" goto aftertriage\r\nif not exist \"%~5\" mkdir \"%~5\"\r\ncopy /Y \"%~dp0source-inventory.template.json\" \"%~5\\source-inventory.json\" >nul\r\n:aftertriage\r\nif \"%~nx1\"==\"fail-optional.mjs\" echo fixture failure 1>&2 & exit /b 7\r\nif \"%~nx1\"==\"fail-required.mjs\" echo fixture failure 1>&2 & exit /b 7\r\nexit /b 0\r\n",
    )?;
    Ok(path)
}

#[cfg(windows)]
fn write_fake_analyzer(dir: &Path, success: bool) -> Result<PathBuf> {
    let path = dir.join(if success {
        "fake-analyzer-ok.cmd"
    } else {
        "fake-analyzer-fail.cmd"
    });
    let body = if success {
        "@echo off\r\nexit /b 0\r\n"
    } else {
        "@echo off\r\necho analyzer failure 1>&2\r\nexit /b 9\r\n"
    };
    fs::write(&path, body)?;
    Ok(path)
}

#[cfg(not(windows))]
fn write_fake_node(dir: &Path) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join("fake-node");
    fs::write(
        &path,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$(dirname \"$0\")/fake-node-args.txt\"\nif [ \"$(basename \"$1\")\" = \"triage-repo.mjs\" ] && [ -f \"$(dirname \"$0\")/source-inventory.template.json\" ]; then mkdir -p \"$5\"; cp \"$(dirname \"$0\")/source-inventory.template.json\" \"$5/source-inventory.json\"; fi\ncase \"$(basename \"$1\")\" in\n  fail-optional.mjs|fail-required.mjs) printf 'fixture failure' 1>&2; exit 7 ;;\n  *) exit 0 ;;\nesac\n",
    )?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions)?;
    Ok(path)
}

#[cfg(not(windows))]
fn write_fake_analyzer(dir: &Path, success: bool) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(if success {
        "fake-analyzer-ok"
    } else {
        "fake-analyzer-fail"
    });
    let body = if success {
        "#!/bin/sh\nexit 0\n"
    } else {
        "#!/bin/sh\nprintf 'analyzer failure' 1>&2\nexit 9\n"
    };
    fs::write(&path, body)?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions)?;
    Ok(path)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
