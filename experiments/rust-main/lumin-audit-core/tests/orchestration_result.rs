use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::orchestration_result::summarize_orchestration_result;

#[test]
fn orchestration_result_summary_preserves_execution_failure_semantics() -> Result<()> {
    let artifact = json!({
        "schemaVersion": "producer-performance.v1",
        "producers": [
            { "name": "triage-repo.mjs", "status": "ok" },
            { "name": "build-symbol-graph.mjs", "status": "failed-required" },
            { "name": "build-unused-deps.mjs", "status": "failed-optional" },
            { "name": "custom-producer.mjs", "status": "failed-custom" }
        ],
        "skipped": [
            { "name": "emit-sarif.mjs", "reason": "not in --sarif mode" },
            { "name": "rank-fixes.mjs", "reason": "dead-classify.json missing" }
        ]
    });

    let summary = serde_json::to_value(summarize_orchestration_result(&artifact))?;

    assert_eq!(summary["artifact"], "producer-performance.json");
    assert_eq!(summary["schemaVersion"], "producer-performance.v1");
    assert_eq!(summary["summaryOwner"], "lumin-audit-core");
    assert_eq!(summary["executionOwner"], "audit-repo.mjs");
    assert_eq!(summary["sourceStatus"], "available");
    assert_eq!(summary["status"], "failed-required");
    assert_eq!(summary["executedStepCount"], 4);
    assert_eq!(summary["okCount"], 1);
    assert_eq!(summary["failedStepCount"], 3);
    assert_eq!(summary["failedRequiredCount"], 1);
    assert_eq!(summary["failedOptionalCount"], 1);
    assert_eq!(summary["failedOtherCount"], 1);
    assert_eq!(summary["skippedStepCount"], 2);
    assert_eq!(summary["observedStatusCounts"]["failed-required"], 1);
    assert_eq!(
        summary["requiredFailureExamples"][0]["name"],
        "build-symbol-graph.mjs"
    );
    assert_eq!(
        summary["optionalFailureExamples"][0]["name"],
        "build-unused-deps.mjs"
    );
    assert_eq!(summary["skippedExamples"][0]["name"], "emit-sarif.mjs");
    assert_eq!(
        summary["skippedExamples"][0]["reason"],
        "not in --sarif mode"
    );
    Ok(())
}

#[test]
fn orchestration_result_summary_reports_degraded_without_required_failure() -> Result<()> {
    let summary = serde_json::to_value(summarize_orchestration_result(&json!({
        "schemaVersion": "producer-performance.v1",
        "producers": [
            { "name": "triage-repo.mjs", "status": "ok" },
            { "name": "build-unused-deps.mjs", "status": "failed-optional" }
        ],
        "skipped": []
    })))?;

    assert_eq!(summary["status"], "degraded");
    assert_eq!(summary["failedRequiredCount"], 0);
    assert_eq!(summary["failedOptionalCount"], 1);
    Ok(())
}

#[test]
fn orchestration_result_summary_makes_malformed_source_unavailable() -> Result<()> {
    let summary = serde_json::to_value(summarize_orchestration_result(&json!({
        "schemaVersion": "producer-performance.v1",
        "producers": { "not": "an array" },
        "skipped": []
    })))?;

    assert_eq!(summary["sourceStatus"], "invalid-shape");
    assert_eq!(summary["status"], "unavailable");
    assert_eq!(summary["executedStepCount"], 0);
    Ok(())
}

#[test]
fn cli_orchestration_result_summary_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let artifact_path = temp.path().join("producer-performance.json");
    fs::write(
        &artifact_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "producer-performance.v1",
            "producers": [{ "name": "triage-repo.mjs", "status": "ok" }],
            "skipped": [{ "name": "emit-sarif.mjs", "reason": "not in --sarif mode" }]
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("orchestration-result-summary")
        .arg("--artifact")
        .arg(&artifact_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["status"], "complete");
    assert_eq!(stdout["executedStepCount"], 1);
    assert_eq!(stdout["skippedStepCount"], 1);
    Ok(())
}

#[test]
fn cli_orchestration_result_summary_hard_stops_on_malformed_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let artifact_path = temp.path().join("producer-performance.json");
    fs::write(&artifact_path, "{not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("orchestration-result-summary")
        .arg("--artifact")
        .arg(&artifact_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("orchestration-result-summary: invalid JSON"));
    assert!(stderr.contains("producer-performance.json"));
    Ok(())
}
