use anyhow::Result;
use serde_json::json;
use std::fs;
use std::process::Command;

use lumin_audit_core::manifest_final::build_manifest_final_summary_update;

#[test]
fn manifest_final_summary_update_projects_last_manifest_patch() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("triage.json"), "{}")?;
    fs::write(output_dir.join("producer-performance.json"), "{}")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let producer_performance = json!({
        "schemaVersion": "producer-performance.v1",
        "summary": {
            "producerCount": 2,
            "okCount": 1,
            "failedCount": 1,
            "skippedCount": 1,
            "totalWallMs": 30,
            "artifactCount": 2,
            "totalArtifactBytes": 128,
            "artifactReadCount": 4,
            "totalArtifactReadBytes": 256,
            "totalJsonParseMs": 7,
            "phaseSupportCount": 1,
            "maxObservedOrchestratorRssBytes": 2048
        },
        "artifacts": {
            "largest": [{ "name": "producer-performance.json", "bytes": 90 }]
        },
        "producers": [
            { "name": "triage-repo.mjs", "status": "ok" },
            { "name": "build-symbol-graph.mjs", "status": "failed-optional" }
        ],
        "skipped": [
            { "name": "emit-sarif.mjs", "status": "skipped", "reason": "not in --sarif mode" }
        ]
    });

    let update = serde_json::to_value(build_manifest_final_summary_update(
        &output_dir,
        &producer_performance,
        false,
    )?)?;

    assert_eq!(update["performance"]["producerCount"], 2);
    assert_eq!(update["performance"]["totalWallMs"], 30);
    assert_eq!(update["orchestration"]["status"], "degraded");
    assert_eq!(update["orchestration"]["failedOptionalCount"], 1);
    assert_eq!(
        update["orchestration"]["skippedExamples"][0]["name"],
        "emit-sarif.mjs"
    );
    assert_eq!(
        update["artifactsProduced"],
        json!(["producer-performance.json", "triage.json"])
    );
    Ok(())
}

#[test]
fn manifest_final_summary_update_includes_current_rust_artifact_only_when_usable() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("producer-performance.json"), "{}")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let producer_performance = json!({
        "schemaVersion": "producer-performance.v1",
        "summary": {},
        "producers": [],
        "skipped": []
    });

    let without_rust =
        build_manifest_final_summary_update(&output_dir, &producer_performance, false)?;
    assert_eq!(
        without_rust.artifacts_produced,
        vec!["producer-performance.json".to_string()]
    );

    let with_rust = build_manifest_final_summary_update(&output_dir, &producer_performance, true)?;
    assert_eq!(
        with_rust.artifacts_produced,
        vec![
            "producer-performance.json".to_string(),
            "rust-analyzer-health.latest.json".to_string()
        ]
    );
    Ok(())
}

#[test]
fn cli_manifest_final_summary_update_hard_stops_on_malformed_performance_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    let performance_path = output_dir.join("producer-performance.json");
    fs::write(&performance_path, "{not-json")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-final-summary-update")
        .arg("--output")
        .arg(&output_dir)
        .arg("--producer-performance")
        .arg(&performance_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("manifest-final-summary-update: invalid JSON"));
    assert!(stderr.contains("producer-performance.json"));
    Ok(())
}
