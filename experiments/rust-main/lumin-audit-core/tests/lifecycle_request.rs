use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::lifecycle_request::{
    evaluate_lifecycle_request_guard, LifecycleRequestGuardInput,
};

fn evaluate(value: Value) -> Result<Value> {
    let request = serde_json::from_value::<LifecycleRequestGuardInput>(value)?;
    Ok(serde_json::to_value(evaluate_lifecycle_request_guard(
        request,
    )?)?)
}

#[test]
fn pre_post_mutex_projects_both_blocks_and_checked_stderr() -> Result<()> {
    let result = evaluate(json!({
        "schemaVersion": "lumin-lifecycle-request-guard.v1",
        "preWriteRequested": true,
        "postWriteRequested": true,
        "preWriteIntentPresent": false,
        "requestedPreWriteEngine": "auto"
    }))?;

    assert_eq!(result["status"], "blocked");
    assert_eq!(result["exitCode"], 2);
    assert_eq!(
        result["stderr"],
        "[audit-repo] --pre-write and --post-write are mutually exclusive\n"
    );
    assert_eq!(result["preWrite"]["requested"], true);
    assert_eq!(result["preWrite"]["ran"], false);
    assert_eq!(
        result["preWrite"]["reason"],
        "--pre-write and --post-write are mutually exclusive"
    );
    assert!(result["preWrite"].get("engine").is_none());
    assert_eq!(result["postWrite"]["requested"], true);
    assert_eq!(result["postWrite"]["ran"], false);
    assert_eq!(
        result["postWrite"]["reason"],
        "--pre-write and --post-write are mutually exclusive"
    );
    Ok(())
}

#[test]
fn missing_pre_write_intent_preserves_explicit_rust_owner() -> Result<()> {
    let result = evaluate(json!({
        "schemaVersion": "lumin-lifecycle-request-guard.v1",
        "preWriteRequested": true,
        "postWriteRequested": false,
        "preWriteIntentPresent": false,
        "requestedPreWriteEngine": "rust"
    }))?;

    assert_eq!(result["status"], "blocked");
    assert_eq!(result["exitCode"], 2);
    assert_eq!(
        result["stderr"],
        "[audit-repo] --pre-write requested but skipped: --intent <file|-> missing\n"
    );
    assert_eq!(result["preWrite"]["requested"], true);
    assert_eq!(result["preWrite"]["ran"], false);
    assert_eq!(result["preWrite"]["engine"], "rust");
    assert_eq!(result["preWrite"]["language"], "rust");
    assert_eq!(result["preWrite"]["producer"], "lumin-rust-analyzer");
    assert_eq!(result["preWrite"]["reason"], "--intent missing");
    assert!(result.get("postWrite").is_none());
    Ok(())
}

#[test]
fn clear_request_does_not_invent_lifecycle_blocks() -> Result<()> {
    let result = evaluate(json!({
        "schemaVersion": "lumin-lifecycle-request-guard.v1",
        "preWriteRequested": true,
        "postWriteRequested": false,
        "preWriteIntentPresent": true,
        "requestedPreWriteEngine": "auto"
    }))?;

    assert_eq!(result["status"], "clear");
    assert_eq!(result["exitCode"], 0);
    assert!(result.get("stderr").is_none());
    assert!(result.get("preWrite").is_none());
    assert!(result.get("postWrite").is_none());
    Ok(())
}

#[test]
fn cli_lifecycle_request_guard_emits_json_and_rejects_bad_schema() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-lifecycle-request-guard.v1",
            "preWriteRequested": true,
            "postWriteRequested": false,
            "preWriteIntentPresent": false,
            "requestedPreWriteEngine": "js"
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("lifecycle-request-guard")
        .arg("--input")
        .arg(&input_path)
        .output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["status"], "blocked");
    assert_eq!(result["preWrite"]["engine"], "js");
    assert_eq!(result["preWrite"]["language"], "js-ts");
    assert_eq!(result["preWrite"]["producer"], "pre-write.mjs");

    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("lifecycle-request-guard")
        .arg("--input")
        .arg(&input_path)
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("lifecycle-request-guard"));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
