use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::lifecycle_exit_policy::{
    apply_lifecycle_exit_policy, LifecycleExitPolicyRequest,
};

fn apply(value: Value) -> Result<Value> {
    let request = serde_json::from_value::<LifecycleExitPolicyRequest>(value)?;
    Ok(serde_json::to_value(apply_lifecycle_exit_policy(request)?)?)
}

#[test]
fn strict_post_write_escalates_only_advisory_non_run_failures() -> Result<()> {
    let result = apply(json!({
        "schemaVersion": "lumin-lifecycle-exit-policy-request.v1",
        "currentExitCode": 0,
        "strictPostWrite": true,
        "postWrite": { "ran": false }
    }))?;

    assert_eq!(result["exitCode"], 2);
    assert!(result["stderr"]
        .as_str()
        .unwrap_or_default()
        .contains("--strict-post-write: post-write did not run"));
    Ok(())
}

#[test]
fn strict_post_write_does_not_override_existing_nonzero_exit() -> Result<()> {
    let result = apply(json!({
        "schemaVersion": "lumin-lifecycle-exit-policy-request.v1",
        "currentExitCode": 1,
        "strictPostWrite": true,
        "postWrite": { "ran": false }
    }))?;

    assert_eq!(result["exitCode"], 1);
    assert!(result.get("stderr").is_none());
    Ok(())
}

#[test]
fn strict_post_write_confidence_preserves_checked_js_limited_branches() -> Result<()> {
    let missing_baseline = apply(json!({
        "schemaVersion": "lumin-lifecycle-exit-policy-request.v1",
        "currentExitCode": 0,
        "strictPostWriteConfidence": true,
        "postWrite": {
            "ran": true,
            "baselineStatus": "missing",
            "scanRangeParity": "ok",
            "typeEscapeDeltaStatus": "computed",
            "afterComplete": true
        }
    }))?;
    assert_eq!(missing_baseline["exitCode"], 2);
    assert!(missing_baseline["stderr"]
        .as_str()
        .unwrap_or_default()
        .contains("--strict-post-write-confidence"));

    let not_applicable_without_file_delta = apply(json!({
        "schemaVersion": "lumin-lifecycle-exit-policy-request.v1",
        "currentExitCode": 0,
        "strictPostWriteConfidence": true,
        "postWrite": {
            "ran": true,
            "typeEscapeDeltaStatus": "not-applicable",
            "fileDeltaStatus": "missing",
            "afterComplete": null
        }
    }))?;
    assert_eq!(not_applicable_without_file_delta["exitCode"], 2);
    Ok(())
}

#[test]
fn strict_post_write_confidence_allows_complete_or_not_applicable_computed_delta() -> Result<()> {
    let complete = apply(json!({
        "schemaVersion": "lumin-lifecycle-exit-policy-request.v1",
        "currentExitCode": 0,
        "strictPostWriteConfidence": true,
        "postWrite": {
            "ran": true,
            "baselineStatus": "available",
            "scanRangeParity": "ok",
            "typeEscapeDeltaStatus": "computed",
            "afterComplete": true
        }
    }))?;
    assert_eq!(complete["exitCode"], 0);
    assert!(complete.get("stderr").is_none());

    let not_applicable = apply(json!({
        "schemaVersion": "lumin-lifecycle-exit-policy-request.v1",
        "currentExitCode": 0,
        "strictPostWriteConfidence": true,
        "postWrite": {
            "ran": true,
            "typeEscapeDeltaStatus": "not-applicable",
            "fileDeltaStatus": "computed",
            "afterComplete": null
        }
    }))?;
    assert_eq!(not_applicable["exitCode"], 0);
    assert!(not_applicable.get("stderr").is_none());
    Ok(())
}

#[test]
fn cli_lifecycle_exit_policy_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("lifecycle-exit-policy")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("lifecycle-exit-policy"));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
