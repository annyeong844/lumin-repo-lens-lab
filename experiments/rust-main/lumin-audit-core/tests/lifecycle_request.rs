use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

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

#[test]
fn cli_execute_audit_lifecycle_preserves_empty_lifecycle_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-audit-lifecycle-execution-request.v1",
            "baseExitCode": 1,
            "lifecycleRequestGuard": {
                "schemaVersion": "lumin-lifecycle-request-guard.v1",
                "preWriteRequested": false,
                "postWriteRequested": false,
                "preWriteIntentPresent": false,
                "requestedPreWriteEngine": "auto"
            },
            "exitPolicy": {
                "strictPostWrite": false,
                "strictPostWriteConfidence": false
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-audit-lifecycle")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_slice(&fs::read(&result_path)?)?;
    assert_eq!(
        result["schemaVersion"],
        "lumin-audit-lifecycle-execution-result.v1"
    );
    assert!(result["preWrite"].is_null());
    assert!(result["postWrite"].is_null());
    assert!(result["canonDraft"].is_null());
    assert!(result["checkCanon"].is_null());
    assert_eq!(result["finalExitCode"], 1);
    Ok(())
}

#[test]
fn cli_execute_audit_lifecycle_blocks_missing_pre_write_intent_before_route_parse() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-audit-lifecycle-execution-request.v1",
            "baseExitCode": 0,
            "lifecycleRequestGuard": {
                "schemaVersion": "lumin-lifecycle-request-guard.v1",
                "preWriteRequested": true,
                "postWriteRequested": false,
                "preWriteIntentPresent": false,
                "requestedPreWriteEngine": "rust"
            },
            "preWrite": {
                "requested": true,
                "routing": {
                    "schemaVersion": "lumin-pre-write-routing-request.v1",
                    "requestedEngine": "rust",
                    "intentFlag": "unused",
                    "intentText": "not valid JSON because the guard must fire first"
                },
                "rust": {
                    "root": "repo",
                    "output": "out",
                    "invocationId": "INV-1",
                    "rustNativeArtifactPath": "out/rust-pre-write-artifact.INV-1.json",
                    "rustNativeLatestPath": "out/rust-pre-write-artifact.latest.json",
                    "analyzer": null,
                    "includeTests": true,
                    "production": false,
                    "excludes": [],
                    "fileInventory": { "status": "available", "files": [] },
                    "failures": []
                },
                "js": {
                    "root": "repo",
                    "output": "out",
                    "scriptsDir": "scripts",
                    "nodeExecutable": "node",
                    "noFreshAudit": false,
                    "scanArgs": []
                }
            },
            "exitPolicy": {
                "strictPostWrite": false,
                "strictPostWriteConfidence": false
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-audit-lifecycle")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--intent <file|-> missing"));
    let result: Value = serde_json::from_slice(&fs::read(&result_path)?)?;
    assert_eq!(result["preWrite"]["requested"], true);
    assert_eq!(result["preWrite"]["ran"], false);
    assert_eq!(result["preWrite"]["engine"], "rust");
    assert_eq!(result["preWrite"]["producer"], "lumin-rust-analyzer");
    assert_eq!(result["preWrite"]["reason"], "--intent missing");
    assert_eq!(result["finalExitCode"], 2);
    Ok(())
}

#[test]
fn cli_execute_audit_lifecycle_blocks_mutex_before_routing_input_file_read() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    let missing_intent_path = temp.path().join("missing-intent.json");
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-audit-lifecycle-execution-request.v1",
            "baseExitCode": 0,
            "lifecycleRequestGuard": {
                "schemaVersion": "lumin-lifecycle-request-guard.v1",
                "preWriteRequested": true,
                "postWriteRequested": true,
                "preWriteIntentPresent": true,
                "requestedPreWriteEngine": "auto"
            },
            "preWrite": {
                "requested": true,
                "routingInput": {
                    "schemaVersion": "lumin-pre-write-routing-input.v1",
                    "requestedEngine": "auto",
                    "intentFlag": missing_intent_path
                },
                "rust": {
                    "root": "repo",
                    "output": "out",
                    "invocationId": "INV-1",
                    "rustNativeArtifactPath": "out/rust-pre-write-artifact.INV-1.json",
                    "rustNativeLatestPath": "out/rust-pre-write-artifact.latest.json",
                    "analyzer": null,
                    "includeTests": true,
                    "production": false,
                    "excludes": [],
                    "fileInventory": { "status": "available", "files": [] },
                    "failures": []
                },
                "js": {
                    "root": "repo",
                    "output": "out",
                    "scriptsDir": "scripts",
                    "nodeExecutable": "node",
                    "noFreshAudit": false,
                    "scanArgs": []
                }
            },
            "postWrite": {
                "requested": true,
                "request": {
                    "schemaVersion": "lumin-post-write-lifecycle-request.v2",
                    "root": temp.path(),
                    "output": temp.path().join("out"),
                    "advisoryPath": null,
                    "deltaOut": null,
                    "deltaInvocationId": "DELTA-1",
                    "generated": "2026-07-13T00:00:00.000Z",
                    "includeTests": true,
                    "excludes": [],
                    "incremental": { "enabled": false, "clear": false }
                }
            },
            "exitPolicy": {
                "strictPostWrite": false,
                "strictPostWriteConfidence": false
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-audit-lifecycle")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("--pre-write and --post-write are mutually exclusive"));
    let result: Value = serde_json::from_slice(&fs::read(&result_path)?)?;
    assert_eq!(
        result["preWrite"]["reason"],
        "--pre-write and --post-write are mutually exclusive"
    );
    assert_eq!(
        result["postWrite"]["reason"],
        "--pre-write and --post-write are mutually exclusive"
    );
    assert_eq!(result["finalExitCode"], 2);
    Ok(())
}

#[test]
fn cli_execute_audit_lifecycle_reads_routing_input_stdin_after_guard_passes() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-audit-lifecycle-execution-request.v1",
            "baseExitCode": 0,
            "lifecycleRequestGuard": {
                "schemaVersion": "lumin-lifecycle-request-guard.v1",
                "preWriteRequested": true,
                "postWriteRequested": false,
                "preWriteIntentPresent": true,
                "requestedPreWriteEngine": "rust"
            },
            "preWrite": {
                "requested": true,
                "routingInput": {
                    "schemaVersion": "lumin-pre-write-routing-input.v1",
                    "requestedEngine": "rust",
                    "intentFlag": "-"
                },
                "rust": {
                    "root": "repo",
                    "output": "out",
                    "invocationId": "INV-1",
                    "rustNativeArtifactPath": "out/rust-pre-write-artifact.INV-1.json",
                    "rustNativeLatestPath": "out/rust-pre-write-artifact.latest.json",
                    "analyzer": null,
                    "includeTests": true,
                    "production": false,
                    "excludes": [],
                    "fileInventory": { "status": "available", "files": [] },
                    "failures": []
                },
                "js": {
                    "root": "repo",
                    "output": "out",
                    "scriptsDir": "scripts",
                    "nodeExecutable": "node",
                    "noFreshAudit": false,
                    "scanArgs": []
                }
            },
            "exitPolicy": {
                "strictPostWrite": false,
                "strictPostWriteConfidence": false
            }
        }))?,
    )?;

    let mut child = Command::new(audit_core_bin())
        .arg("execute-audit-lifecycle")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .context("stdin was piped")?
        .write_all(br#"{ "language": "python" }"#)?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&fs::read(&result_path)?)?;
    assert_eq!(result["preWrite"]["requested"], true);
    assert_eq!(result["preWrite"]["ran"], false);
    assert_eq!(result["preWrite"]["engine"], "rust");
    assert!(result["preWrite"]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("intent.language must be \"rust\" or \"js-ts\""));
    assert_eq!(result["finalExitCode"], 2);
    Ok(())
}

#[test]
fn cli_execute_audit_lifecycle_replays_post_write_missing_advisory_stderr() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    let output_dir = temp.path().join("out");
    fs::create_dir_all(&output_dir)?;
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-audit-lifecycle-execution-request.v1",
            "baseExitCode": 0,
            "lifecycleRequestGuard": {
                "schemaVersion": "lumin-lifecycle-request-guard.v1",
                "preWriteRequested": false,
                "postWriteRequested": true,
                "preWriteIntentPresent": false,
                "requestedPreWriteEngine": "auto"
            },
            "postWrite": {
                "requested": true,
                "request": {
                    "schemaVersion": "lumin-post-write-lifecycle-request.v2",
                    "root": temp.path(),
                    "output": output_dir,
                    "advisoryPath": null,
                    "deltaOut": null,
                    "deltaInvocationId": "DELTA-1",
                    "generated": "2026-07-13T00:00:00.000Z",
                    "includeTests": true,
                    "excludes": [],
                    "incremental": { "enabled": false, "clear": false }
                }
            },
            "exitPolicy": {
                "strictPostWrite": false,
                "strictPostWriteConfidence": false
            }
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-audit-lifecycle")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--pre-write-advisory <file> missing"));
    let result: Value = serde_json::from_slice(&fs::read(&result_path)?)?;
    assert_eq!(result["postWrite"]["requested"], true);
    assert_eq!(result["postWrite"]["ran"], false);
    assert_eq!(
        result["postWrite"]["reason"],
        "--pre-write-advisory missing"
    );
    assert_eq!(result["finalExitCode"], 2);
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
