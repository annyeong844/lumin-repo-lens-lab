use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::producer_performance::summarize_producer_performance;

#[test]
fn producer_performance_summary_projects_manifest_fields() -> Result<()> {
    let artifact = json!({
        "schemaVersion": "producer-performance.v1",
        "summary": {
            "producerCount": 13,
            "okCount": 10,
            "failedCount": 1,
            "skippedCount": 2,
            "totalWallMs": 1234,
            "artifactCount": 8,
            "totalArtifactBytes": 4096,
            "artifactReadCount": 6,
            "totalArtifactReadBytes": 2048,
            "totalJsonParseMs": 17,
            "phaseSupportCount": 4,
            "maxObservedOrchestratorRssBytes": 123456789
        },
        "artifacts": {
            "largest": [
                { "name": "symbols.json", "bytes": 3000 },
                { "name": "triage.json", "bytes": 1000 }
            ]
        },
        "producers": [
            { "name": "build-symbol-graph.mjs", "status": "ok" }
        ]
    });

    let summary = serde_json::to_value(summarize_producer_performance(&artifact))?;

    assert_eq!(summary["artifact"], "producer-performance.json");
    assert_eq!(summary["schemaVersion"], "producer-performance.v1");
    assert_eq!(summary["producerCount"], 13);
    assert_eq!(summary["okCount"], 10);
    assert_eq!(summary["failedCount"], 1);
    assert_eq!(summary["skippedCount"], 2);
    assert_eq!(summary["totalWallMs"], 1234);
    assert_eq!(summary["artifactCount"], 8);
    assert_eq!(summary["totalArtifactBytes"], 4096);
    assert_eq!(summary["artifactReadCount"], 6);
    assert_eq!(summary["totalArtifactReadBytes"], 2048);
    assert_eq!(summary["totalJsonParseMs"], 17);
    assert_eq!(summary["phaseSupportCount"], 4);
    assert_eq!(summary["largestArtifacts"][0]["name"], "symbols.json");
    assert_eq!(summary["maxObservedOrchestratorRssBytes"], 123456789);
    Ok(())
}

#[test]
fn producer_performance_summary_defaults_missing_optional_counts_to_zero() -> Result<()> {
    let summary = serde_json::to_value(summarize_producer_performance(&json!({
        "schemaVersion": "producer-performance.v1"
    })))?;

    assert_eq!(summary["schemaVersion"], "producer-performance.v1");
    assert_eq!(summary["producerCount"], 0);
    assert_eq!(summary["totalArtifactBytes"], 0);
    assert_eq!(summary["largestArtifacts"], json!([]));
    assert_eq!(summary["maxObservedOrchestratorRssBytes"], 0);
    Ok(())
}

#[test]
fn cli_producer_performance_summary_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let artifact_path = temp.path().join("producer-performance.json");
    fs::write(
        &artifact_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "producer-performance.v1",
            "summary": {
                "producerCount": 1,
                "okCount": 1,
                "artifactCount": 1,
                "totalArtifactBytes": 32
            },
            "artifacts": {
                "largest": [{ "name": "manifest.json", "bytes": 32 }]
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-summary")
        .arg("--artifact")
        .arg(&artifact_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["producerCount"], 1);
    assert_eq!(stdout["largestArtifacts"][0]["name"], "manifest.json");
    Ok(())
}

#[test]
fn cli_producer_performance_summary_hard_stops_on_malformed_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let artifact_path = temp.path().join("producer-performance.json");
    fs::write(&artifact_path, "{not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-summary")
        .arg("--artifact")
        .arg(&artifact_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("producer-performance-summary: invalid JSON"));
    assert!(stderr.contains("producer-performance.json"));
    Ok(())
}
