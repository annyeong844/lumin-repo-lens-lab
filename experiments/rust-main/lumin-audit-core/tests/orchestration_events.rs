use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::orchestration_events::{
    build_producer_performance_artifact, OrchestrationLedger,
};

fn ledger(value: Value) -> Result<OrchestrationLedger> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn producer_performance_artifact_projects_ledger_without_js_summary_math() -> Result<()> {
    let ledger = ledger(json!({
        "schemaVersion": "lumin-audit-orchestration-ledger.v1",
        "generated": "2026-07-01T00:00:00.000Z",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "profile": "full",
        "scanRange": {
            "includeTests": true,
            "production": false,
            "excludes": ["dist"],
            "autoExcludes": [".audit"]
        },
        "cache": {
            "noIncremental": false,
            "cacheRoot": "C:/repo/.audit/.cache",
            "clearIncrementalCache": false
        },
        "generatedArtifacts": { "mode": "prepared" },
        "artifactReads": {
            "schemaVersion": "artifact-read-metrics.v1",
            "measurement": "audit-repo-orchestrator-json-reads",
            "totalReadCount": 3,
            "totalReadBytes": 120,
            "totalReadMs": 9,
            "totalJsonParseMs": 4,
            "parseFailureCount": 1,
            "largestReads": [{ "name": "symbols.json", "bytes": 100, "readCount": 1 }],
            "slowestJsonParses": [{ "name": "symbols.json", "jsonParseMs": 4, "readCount": 1 }],
            "byName": { "symbols.json": { "readCount": 1 } }
        },
        "artifacts": {
            "producedCount": 2,
            "totalBytes": 2048,
            "largest": [{ "name": "symbols.json", "bytes": 2000 }],
            "byName": { "symbols.json": { "bytes": 2000 } }
        },
        "events": [
            {
                "kind": "producer",
                "name": "triage-repo.mjs",
                "status": "ok",
                "wallMs": 12,
                "phases": [{ "name": "scan", "ms": 7 }],
                "counters": { "files": 10 },
                "memory": {
                    "before": { "rssBytes": 100, "heapTotalBytes": 20 },
                    "after": { "rssBytes": 140, "heapTotalBytes": 24 },
                    "delta": { "rssBytes": 40, "heapTotalBytes": 4 }
                }
            },
            {
                "kind": "producer",
                "name": "rank-fixes.mjs",
                "status": "failed-optional",
                "wallMs": 5,
                "stderrSnippet": "boom"
            },
            {
                "kind": "skipped",
                "name": "emit-sarif.mjs",
                "reason": "not in --sarif mode"
            }
        ]
    }))?;
    let artifact = serde_json::to_value(build_producer_performance_artifact(ledger)?)?;

    assert_eq!(artifact["schemaVersion"], "producer-performance.v1");
    assert_eq!(artifact["profile"], "full");
    assert_eq!(artifact["summary"]["producerCount"], 2);
    assert_eq!(artifact["summary"]["okCount"], 1);
    assert_eq!(artifact["summary"]["failedCount"], 1);
    assert_eq!(artifact["summary"]["skippedCount"], 1);
    assert_eq!(artifact["summary"]["totalWallMs"], 17);
    assert_eq!(artifact["summary"]["artifactCount"], 2);
    assert_eq!(artifact["summary"]["totalArtifactBytes"], 2048);
    assert_eq!(artifact["summary"]["artifactReadCount"], 3);
    assert_eq!(artifact["summary"]["totalArtifactReadBytes"], 120);
    assert_eq!(artifact["summary"]["totalJsonParseMs"], 4);
    assert_eq!(artifact["summary"]["maxObservedOrchestratorRssBytes"], 140);
    assert_eq!(artifact["summary"]["phaseSupportCount"], 1);
    assert_eq!(
        artifact["memory"]["measurement"],
        "orchestrator-process-snapshots"
    );
    assert_eq!(artifact["memory"]["childPeakRssAvailable"], false);
    assert_eq!(artifact["producers"][1]["stderrSnippet"], "boom");
    assert_eq!(artifact["skipped"][0]["status"], "skipped");
    Ok(())
}

#[test]
fn producer_performance_artifact_rejects_wrong_ledger_schema() -> Result<()> {
    let ledger = ledger(json!({
        "schemaVersion": "old-ledger",
        "generated": "2026-07-01T00:00:00.000Z",
        "root": "C:/repo",
        "output": "C:/repo/.audit",
        "profile": "quick",
        "scanRange": { "includeTests": true, "production": false },
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
        "events": []
    }))?;
    let error = match build_producer_performance_artifact(ledger) {
        Ok(_) => anyhow::bail!("wrong ledger schema should be rejected"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("unsupported ledger schemaVersion"));
    Ok(())
}

#[test]
fn cli_producer_performance_artifact_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("ledger.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-orchestration-ledger.v1",
            "generated": "2026-07-01T00:00:00.000Z",
            "root": "C:/repo",
            "output": "C:/repo/.audit",
            "profile": "quick",
            "scanRange": { "includeTests": true, "production": false },
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
            "events": [
                { "kind": "producer", "name": "triage-repo.mjs", "status": "ok", "wallMs": 3 }
            ]
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["schemaVersion"], "producer-performance.v1");
    assert_eq!(stdout["summary"]["okCount"], 1);
    Ok(())
}

#[test]
fn cli_producer_performance_artifact_hard_stops_on_malformed_ledger() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("ledger.json");
    fs::write(
        &input_path,
        r#"{"schemaVersion":"lumin-audit-orchestration-ledger.v1"}"#,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("producer-performance-artifact")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("producer-performance-artifact: invalid ledger shape"));
    Ok(())
}
