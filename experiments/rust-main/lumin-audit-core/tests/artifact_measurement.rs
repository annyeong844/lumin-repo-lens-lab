use anyhow::Result;
use lumin_audit_core::artifact_measurement::measure_artifact_sizes;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

#[test]
fn artifact_size_summary_matches_js_runner_measurement_contract() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("small.json"), b"abcd")?;
    fs::write(temp.path().join("large.json"), b"abcdefghij")?;
    fs::create_dir(temp.path().join("nested"))?;

    let artifacts = vec![
        "missing.json".to_string(),
        "large.json".to_string(),
        "nested".to_string(),
        "small.json".to_string(),
    ];
    let summary = serde_json::to_value(measure_artifact_sizes(temp.path(), &artifacts))?;

    assert_eq!(summary["producedCount"], 2);
    assert_eq!(summary["totalBytes"], 14);
    assert_eq!(
        summary["largest"],
        json!([
            { "name": "large.json", "bytes": 10 },
            { "name": "small.json", "bytes": 4 }
        ])
    );
    assert_eq!(summary["byName"]["small.json"]["bytes"], 4);
    assert_eq!(summary["byName"]["large.json"]["bytes"], 10);
    assert!(summary["byName"]["missing.json"].is_null());
    assert!(summary["byName"]["nested"].is_null());
    Ok(())
}

#[test]
fn artifact_size_summary_ties_sort_by_name() -> Result<()> {
    let temp = tempfile::tempdir()?;
    fs::write(temp.path().join("b.json"), b"xx")?;
    fs::write(temp.path().join("a.json"), b"xx")?;

    let summary = serde_json::to_value(measure_artifact_sizes(
        temp.path(),
        &["b.json".to_string(), "a.json".to_string()],
    ))?;

    assert_eq!(
        summary["largest"],
        json!([
            { "name": "a.json", "bytes": 2 },
            { "name": "b.json", "bytes": 2 }
        ])
    );
    Ok(())
}

#[test]
fn cli_artifact_size_summary_emits_json() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("artifacts.json");
    fs::write(temp.path().join("triage.json"), b"abc")?;
    fs::write(&input_path, serde_json::to_vec(&json!(["triage.json"]))?)?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("artifact-size-summary")
        .arg("--output")
        .arg(temp.path())
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = serde_json::from_slice::<Value>(&output.stdout)?;
    assert_eq!(stdout["producedCount"], 1);
    assert_eq!(stdout["totalBytes"], 3);
    Ok(())
}

#[test]
fn cli_artifact_size_summary_hard_stops_on_malformed_input() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("artifacts.json");
    fs::write(&input_path, r#"{"not":"an array"}"#)?;

    let output = Command::new(env!("CARGO_BIN_EXE_lumin-audit-core"))
        .arg("artifact-size-summary")
        .arg("--output")
        .arg(temp.path())
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("artifact-size-summary: invalid artifact list shape"));
    Ok(())
}
