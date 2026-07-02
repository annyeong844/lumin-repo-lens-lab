use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::manifest_final::{
    build_manifest_artifacts_produced_update, build_manifest_closeout_update,
    build_manifest_final_summary_update, build_manifest_final_summary_update_for_rust_analysis,
    ManifestCloseoutCompanionInput,
};

#[test]
fn manifest_artifacts_produced_update_projects_patch_shape() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("topology.mermaid.md"), "flowchart TD\n")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let update = serde_json::to_value(build_manifest_artifacts_produced_update(
        &output_dir,
        Some(&json!({ "status": "complete", "available": true })),
    )?)?;

    assert_eq!(
        update,
        json!({
            "artifactsProduced": [
                "rust-analyzer-health.latest.json",
                "topology.mermaid.md"
            ]
        })
    );
    Ok(())
}

#[test]
fn cli_manifest_artifacts_produced_update_emits_patch_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("triage.json"), "{}")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-artifacts-produced-update")
        .arg("--output")
        .arg(&output_dir)
        .arg("--rust-analysis-block")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let mut child = output;
    use std::io::Write;
    let mut stdin = child.stdin.take().context("stdin is piped")?;
    stdin.write_all(br#"{"status":"complete","available":true}"#)?;
    drop(stdin);
    let output = child.wait_with_output()?;

    assert!(output.status.success());
    let update = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(
        update["artifactsProduced"],
        json!(["rust-analyzer-health.latest.json", "triage.json"])
    );
    Ok(())
}

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
fn manifest_closeout_update_projects_final_and_companion_patch() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("producer-performance.json"), "{}")?;
    fs::write(output_dir.join("audit-summary.latest.md"), "# Summary\n")?;
    fs::write(output_dir.join("audit-review-pack.latest.md"), "# Review\n")?;
    fs::write(output_dir.join("rust-analyzer-health.latest.json"), "{}")?;

    let producer_performance = json!({
        "schemaVersion": "producer-performance.v1",
        "summary": {
            "producerCount": 1,
            "okCount": 1,
            "failedCount": 0,
            "skippedCount": 0,
            "totalWallMs": 12
        },
        "producers": [
            { "name": "triage-repo.mjs", "status": "ok" }
        ],
        "skipped": []
    });

    let update = serde_json::to_value(build_manifest_closeout_update(
        &output_dir,
        &producer_performance,
        Some(&json!({ "status": "complete", "available": true })),
        ManifestCloseoutCompanionInput {
            topology_mermaid_path: None,
            audit_summary_path: Some("C:/repo/.audit/audit-summary.latest.md".to_string()),
            review_pack_path: Some("C:/repo/.audit/audit-review-pack.latest.md".to_string()),
        },
    )?)?;

    assert_eq!(update["performance"]["producerCount"], 1);
    assert_eq!(update["orchestration"]["status"], "complete");
    assert_eq!(
        update["artifactsProduced"],
        json!([
            "audit-review-pack.latest.md",
            "audit-summary.latest.md",
            "producer-performance.json",
            "rust-analyzer-health.latest.json"
        ])
    );
    assert_eq!(
        update["auditSummary"],
        json!({
            "path": "C:/repo/.audit/audit-summary.latest.md",
            "format": "markdown"
        })
    );
    assert_eq!(update["reviewPack"]["format"], "markdown");
    assert!(update.get("topologyMermaid").is_none());
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
fn manifest_final_summary_update_uses_rust_analysis_block_for_current_artifact() -> Result<()> {
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

    let unavailable = json!({ "status": "invalid-shape", "available": false });
    let without_rust = build_manifest_final_summary_update_for_rust_analysis(
        &output_dir,
        &producer_performance,
        Some(&unavailable),
    )?;
    assert_eq!(
        without_rust.artifacts_produced,
        vec!["producer-performance.json".to_string()]
    );

    let complete = json!({ "status": "complete", "available": true });
    let with_rust = build_manifest_final_summary_update_for_rust_analysis(
        &output_dir,
        &producer_performance,
        Some(&complete),
    )?;
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
fn cli_manifest_closeout_update_reads_performance_and_projects_patch() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    let performance_path = output_dir.join("producer-performance.json");
    fs::write(
        &performance_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "producer-performance.v1",
            "summary": { "producerCount": 1, "okCount": 1, "failedCount": 0, "skippedCount": 0 },
            "producers": [{ "name": "triage-repo.mjs", "status": "ok" }],
            "skipped": []
        }))?,
    )?;
    fs::write(
        output_dir.join("producer-performance.json"),
        fs::read(&performance_path)?,
    )?;
    fs::write(output_dir.join("audit-summary.latest.md"), "# Summary\n")?;

    let input = json!({
        "output": output_dir,
        "producerPerformancePath": performance_path,
        "rustAnalysis": { "status": "unavailable", "available": false },
        "companion": {
            "auditSummaryPath": "C:/repo/.audit/audit-summary.latest.md"
        }
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-closeout-update")
        .arg("--input")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    use std::io::Write;
    let mut stdin = child.stdin.take().context("stdin is piped")?;
    stdin.write_all(input.to_string().as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;

    assert!(output.status.success());
    let update = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(update["performance"]["producerCount"], 1);
    assert_eq!(update["auditSummary"]["format"], "markdown");
    assert_eq!(
        update["artifactsProduced"],
        json!(["audit-summary.latest.md", "producer-performance.json"])
    );
    Ok(())
}

#[test]
fn cli_manifest_closeout_write_applies_patch_and_writes_manifest() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    let performance_path = output_dir.join("producer-performance.json");
    fs::write(
        &performance_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "producer-performance.v1",
            "summary": { "producerCount": 1, "okCount": 1, "failedCount": 0, "skippedCount": 0 },
            "producers": [{ "name": "triage-repo.mjs", "status": "ok" }],
            "skipped": []
        }))?,
    )?;
    fs::write(output_dir.join("audit-summary.latest.md"), "# Summary\n")?;

    let input = json!({
        "manifest": {
            "meta": { "generated": "2026-07-02T00:00:00.000Z" },
            "profile": "quick",
            "artifactsProduced": []
        },
        "output": output_dir,
        "producerPerformancePath": performance_path,
        "rustAnalysis": { "status": "unavailable", "available": false },
        "companion": {
            "auditSummaryPath": "C:/repo/.audit/audit-summary.latest.md"
        }
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("manifest-closeout-write")
        .arg("--input")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    use std::io::Write;
    let mut stdin = child.stdin.take().context("stdin is piped")?;
    stdin.write_all(input.to_string().as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;

    assert!(output.status.success());
    let result = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert!(result["manifestPath"]
        .as_str()
        .is_some_and(|path| path.ends_with("manifest.json")));
    assert!(result.get("manifest").is_none());
    assert_eq!(result["closeoutUpdate"]["performance"]["producerCount"], 1);
    assert_eq!(
        result["closeoutUpdate"]["orchestration"]["status"],
        "complete"
    );
    assert_eq!(
        result["closeoutUpdate"]["auditSummary"]["format"],
        "markdown"
    );
    let written = fs::read_to_string(output_dir.join("manifest.json"))?;
    let written = serde_json::from_str::<serde_json::Value>(&written)?;
    assert_eq!(written["performance"]["producerCount"], 1);
    assert_eq!(written["orchestration"]["status"], "complete");
    assert_eq!(written["auditSummary"]["format"], "markdown");
    Ok(())
}

#[test]
fn cli_finalize_audit_run_writes_performance_and_manifest() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_dir = temp.path().join(".audit");
    fs::create_dir_all(&output_dir)?;
    fs::write(output_dir.join("triage.json"), "{}")?;
    fs::write(output_dir.join("audit-summary.latest.md"), "# Summary\n")?;

    let input = json!({
        "manifest": {
            "meta": { "generated": "2026-07-02T00:00:00.000Z" },
            "profile": "quick",
            "artifactsProduced": []
        },
        "context": {
            "generated": "2026-07-02T00:00:00.000Z",
            "root": temp.path(),
            "output": output_dir,
            "profile": "quick",
            "includeTests": true,
            "production": false,
            "excludes": ["dist"],
            "autoExcludes": [".audit"],
            "noIncremental": true,
            "cacheRoot": output_dir.join(".cache"),
            "clearIncrementalCache": false,
            "generatedArtifactsMode": "default"
        },
        "observations": {
            "artifactReads": {
                "schemaVersion": "artifact-read-metrics.v1",
                "measurement": "audit-repo-orchestrator-json-reads",
                "totalReadCount": 0,
                "totalReadBytes": 0,
                "totalReadMs": 0,
                "totalJsonParseMs": 0,
                "parseFailureCount": 0,
                "byName": {}
            },
            "rustAnalysis": {
                "status": "unavailable",
                "available": false
            },
            "commandsRun": [
                { "step": "triage-repo.mjs", "status": "ok", "ms": 3 }
            ],
            "skipped": [
                { "step": "emit-sarif.mjs", "reason": "not in --sarif mode" }
            ]
        },
        "rustAnalysis": {
            "status": "unavailable",
            "available": false
        },
        "companion": {
            "auditSummaryPath": "C:/repo/.audit/audit-summary.latest.md"
        }
    });

    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("finalize-audit-run")
        .arg("--input")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    use std::io::Write;
    let mut stdin = child.stdin.take().context("stdin is piped")?;
    stdin.write_all(input.to_string().as_bytes())?;
    drop(stdin);
    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result = serde_json::from_slice::<Value>(&output.stdout)?;
    assert!(result["producerPerformancePath"]
        .as_str()
        .is_some_and(|path| path.ends_with("producer-performance.json")));
    assert!(result["manifestPath"]
        .as_str()
        .is_some_and(|path| path.ends_with("manifest.json")));
    assert!(result.get("manifest").is_none());
    assert_eq!(result["closeoutUpdate"]["performance"]["producerCount"], 1);
    assert_eq!(
        result["closeoutUpdate"]["orchestration"]["status"],
        "complete"
    );
    assert_eq!(
        result["closeoutUpdate"]["auditSummary"]["format"],
        "markdown"
    );
    let produced = result["closeoutUpdate"]["artifactsProduced"]
        .as_array()
        .context("manifest artifactsProduced should be an array")?;
    assert!(produced
        .iter()
        .any(|artifact| artifact.as_str() == Some("producer-performance.json")));
    assert!(produced
        .iter()
        .any(|artifact| artifact.as_str() == Some("audit-summary.latest.md")));

    let performance = fs::read_to_string(output_dir.join("producer-performance.json"))?;
    let performance = serde_json::from_str::<Value>(&performance)?;
    assert_eq!(performance["schemaVersion"], "producer-performance.v1");
    assert_eq!(performance["scanRange"]["excludes"], json!(["dist"]));
    let written = fs::read_to_string(output_dir.join("manifest.json"))?;
    let written = serde_json::from_str::<Value>(&written)?;
    assert_eq!(written["performance"]["producerCount"], 1);
    assert_eq!(written["orchestration"]["status"], "complete");
    assert_eq!(written["auditSummary"]["format"], "markdown");
    assert!(written["artifactsProduced"]
        .as_array()
        .context("written artifactsProduced should be an array")?
        .iter()
        .any(|artifact| artifact.as_str() == Some("producer-performance.json")));
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
