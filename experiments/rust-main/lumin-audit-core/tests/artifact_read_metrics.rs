use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::artifact_read_metrics::{
    summarize_artifact_read_events, ArtifactReadMetricsRequest,
};

fn request(value: Value) -> anyhow::Result<ArtifactReadMetricsRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn summarizes_successful_and_failed_json_reads_like_js_contract() -> anyhow::Result<()> {
    let request = request(json!({
        "schemaVersion": "lumin-audit-artifact-read-events.v1",
        "rootDir": "C:/repo/.audit",
        "largestLimit": 2,
        "reads": [
            {
                "filePath": "C:/repo/.audit/symbols.json",
                "bytes": 10.4,
                "readMs": 1.4,
                "jsonParseMs": 2.5,
                "ok": true
            },
            {
                "filePath": "C:/repo/.audit/symbols.json",
                "bytes": "9.6",
                "readMs": -3,
                "jsonParseMs": "bad",
                "ok": false
            },
            {
                "filePath": "C:/repo/.audit/triage.json",
                "bytes": 3,
                "readMs": 4,
                "jsonParseMs": 5,
                "ok": true
            }
        ]
    }))?;

    let summary = serde_json::to_value(summarize_artifact_read_events(request)?)?;

    assert_eq!(summary["schemaVersion"], "artifact-read-metrics.v1");
    assert_eq!(summary["measurement"], "audit-repo-orchestrator-json-reads");
    assert_eq!(summary["totalReadCount"], 3);
    assert_eq!(summary["totalReadBytes"], 23);
    assert_eq!(summary["totalReadMs"], 5);
    assert_eq!(summary["totalJsonParseMs"], 8);
    assert_eq!(summary["parseFailureCount"], 1);
    assert_eq!(summary["byName"]["symbols.json"]["readCount"], 2);
    assert_eq!(summary["byName"]["symbols.json"]["totalBytes"], 20);
    assert_eq!(summary["byName"]["symbols.json"]["parseFailureCount"], 1);
    assert_eq!(summary["largestReads"][0]["name"], "symbols.json");
    assert_eq!(summary["slowestJsonParses"][0]["name"], "triage.json");
    Ok(())
}

#[test]
fn normalizes_paths_relative_to_root_and_uses_basename_outside_root() -> anyhow::Result<()> {
    let request = request(json!({
        "schemaVersion": "lumin-audit-artifact-read-events.v1",
        "rootDir": "C:/repo/.audit",
        "reads": [
            {
                "filePath": "C:/repo/.audit/.producer-phases/triage-repo.mjs.json",
                "bytes": 1,
                "readMs": 0,
                "jsonParseMs": 0,
                "ok": true
            },
            {
                "filePath": "D:/elsewhere/symbols.json",
                "bytes": 1,
                "readMs": 0,
                "jsonParseMs": 0,
                "ok": true
            }
        ]
    }))?;

    let summary = serde_json::to_value(summarize_artifact_read_events(request)?)?;

    assert_eq!(
        summary["byName"][".producer-phases/triage-repo.mjs.json"]["readCount"],
        1
    );
    assert_eq!(summary["byName"]["symbols.json"]["readCount"], 1);
    Ok(())
}

#[test]
fn cli_emits_artifact_read_summary_json() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("reads.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "schemaVersion": "lumin-audit-artifact-read-events.v1",
            "rootDir": temp.path().to_string_lossy(),
            "reads": [
                {
                    "filePath": temp.path().join("manifest.json").to_string_lossy(),
                    "bytes": 7,
                    "readMs": 1,
                    "jsonParseMs": 2,
                    "ok": true
                }
            ]
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("artifact-read-metrics-summary")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["schemaVersion"], "artifact-read-metrics.v1");
    assert_eq!(stdout["byName"]["manifest.json"]["readCount"], 1);
    Ok(())
}

#[test]
fn rejects_wrong_request_schema_version() -> anyhow::Result<()> {
    let request = request(json!({
        "schemaVersion": "old",
        "reads": []
    }))?;

    let error = match summarize_artifact_read_events(request) {
        Ok(summary) => anyhow::bail!("expected schema hard-stop, got {summary:?}"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported schemaVersion"));
    Ok(())
}
