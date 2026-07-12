use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

use lumin_audit_core::lifecycle::summarize_lifecycle;

#[test]
fn lifecycle_summary_projects_raw_manifest_blocks_without_claiming_execution() -> Result<()> {
    let summary = serde_json::to_value(summarize_lifecycle(&json!({
        "preWrite": {
            "requested": true,
            "ran": true,
            "engine": "rust",
            "language": "rust",
            "producer": "lumin-rust-analyzer",
            "advisoryPath": "audit/pre-write-advisory.abc.json"
        },
        "postWrite": {
            "requested": true,
            "ran": true,
            "silentNew": 2,
            "requiredAcknowledgementCount": 1,
            "baselineStatus": "available",
            "scanRangeParity": "ok",
            "typeEscapeDeltaStatus": "computed",
            "afterComplete": true,
            "fileDeltaStatus": "computed",
            "unexpectedNewFileCount": 1,
            "plannedMissingFileCount": 0
        },
        "canonDraft": {
            "requested": true,
            "ran": true,
            "requestedSources": ["type-ownership", "helper-registry"],
            "draftPaths": ["canonical-draft/type-ownership.md"],
            "perSource": {
                "type-ownership": { "ran": true, "exitCode": 0 },
                "helper-registry": { "ran": false, "exitCode": 2 }
            }
        },
        "checkCanon": {
            "requested": true,
            "ran": true,
            "strict": true,
            "requestedSources": ["type-ownership", "helper-registry"],
            "executionMode": "per-source",
            "childInvocations": 2,
            "summary": {
                "driftCount": 3,
                "sourcesRequested": 2,
                "sourcesChecked": 1,
                "sourcesSkipped": 0,
                "sourcesFailed": 1
            },
            "perSource": {
                "type-ownership": { "ran": true, "status": "drift", "driftCount": 3 },
                "helper-registry": { "ran": false, "status": "parse-error" }
            }
        }
    })))?;

    assert_eq!(summary["summaryOwner"], "lumin-audit-core");
    assert_eq!(summary["executionOwner"], "audit-repo.mjs");
    assert_eq!(summary["sourceStatus"], "available");
    assert_eq!(summary["requestedCount"], 4);
    assert_eq!(summary["ranCount"], 4);
    assert_eq!(summary["notRunCount"], 0);
    assert_eq!(summary["preWrite"]["status"], "complete");
    assert_eq!(summary["preWrite"]["engine"], "rust");
    assert_eq!(summary["postWrite"]["silentNew"], 2);
    assert_eq!(summary["postWrite"]["afterComplete"], true);
    assert_eq!(summary["canonDraft"]["requestedSourceCount"], 2);
    assert_eq!(summary["canonDraft"]["ranSourceCount"], 1);
    assert_eq!(summary["canonDraft"]["failedSourceCount"], 1);
    assert_eq!(summary["canonDraft"]["draftCount"], 1);
    assert_eq!(summary["checkCanon"]["strict"], true);
    assert_eq!(summary["checkCanon"]["executionMode"], "per-source");
    assert_eq!(summary["checkCanon"]["childInvocations"], 2);
    assert_eq!(summary["checkCanon"]["driftCount"], 3);
    assert_eq!(summary["checkCanon"]["sourcesChecked"], 1);
    assert_eq!(summary["checkCanon"]["sourcesFailed"], 1);
    Ok(())
}

#[test]
fn lifecycle_summary_distinguishes_not_requested_not_run_and_unavailable() -> Result<()> {
    let summary = serde_json::to_value(summarize_lifecycle(&json!({
        "preWrite": {
            "requested": true,
            "ran": false,
            "engine": "auto",
            "reason": "--intent missing"
        },
        "postWrite": null,
        "canonDraft": "malformed"
    })))?;

    assert_eq!(summary["requestedCount"], 1);
    assert_eq!(summary["ranCount"], 0);
    assert_eq!(summary["notRunCount"], 1);
    assert_eq!(summary["preWrite"]["status"], "not-run");
    assert_eq!(summary["preWrite"]["reason"], "--intent missing");
    assert_eq!(summary["postWrite"]["status"], "not-requested");
    assert_eq!(summary["canonDraft"]["status"], "unavailable");
    assert_eq!(summary["checkCanon"]["status"], "not-requested");
    Ok(())
}

#[test]
fn lifecycle_summary_marks_non_object_source_invalid() -> Result<()> {
    let summary = serde_json::to_value(summarize_lifecycle(&json!("not-an-object")))?;

    assert_eq!(summary["sourceStatus"], "invalid-shape");
    assert_eq!(summary["preWrite"]["status"], "unavailable");
    assert_eq!(summary["requestedCount"], 0);
    Ok(())
}

#[test]
fn cli_lifecycle_summary_reads_json_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("lifecycle.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&json!({
            "checkCanon": {
                "requested": true,
                "ran": false,
                "strict": false,
                "reason": "unknown --sources values: bogus"
            }
        }))?,
    )?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("lifecycle-summary")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["checkCanon"]["status"], "not-run");
    assert_eq!(
        stdout["checkCanon"]["reason"],
        "unknown --sources values: bogus"
    );
    Ok(())
}

#[test]
fn cli_lifecycle_summary_reads_stdin() -> Result<()> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("lifecycle-summary")
        .arg("--input")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .context("stdin is piped")?
        .write_all(br#"{"preWrite":{"requested":true,"ran":true,"engine":"js"}}"#)?;

    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    assert_eq!(stdout["preWrite"]["status"], "complete");
    assert_eq!(stdout["preWrite"]["engine"], "js");
    Ok(())
}

#[test]
fn cli_lifecycle_summary_hard_stops_on_malformed_stdin() -> Result<()> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("lifecycle-summary")
        .arg("--input")
        .arg("-")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .as_mut()
        .context("stdin is piped")?
        .write_all(b"{not-json")?;

    let output = child.wait_with_output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("lifecycle-summary: invalid JSON in stdin"));
    Ok(())
}
