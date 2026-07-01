use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lumin_audit_core::check_canon_lifecycle::{
    execute_check_canon_lifecycle, CheckCanonLifecycleRequest,
};

fn request(root: &Path, out: &Path, fake_node: &Path) -> Value {
    json!({
        "schemaVersion": "lumin-check-canon-lifecycle-request.v1",
        "sourcesValue": null,
        "strict": false,
        "root": path_string(root),
        "output": path_string(out),
        "scriptsDir": path_string(root),
        "nodeExecutable": path_string(fake_node),
        "scanArgs": []
    })
}

fn parse_request(value: Value) -> Result<CheckCanonLifecycleRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn check_canon_all_sources_uses_single_child_and_preserves_canon_drift_statuses() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    fs::write(out.join("symbols.json"), "{}")?;
    fs::write(out.join("topology.json"), "{}")?;
    write_canon_drift(
        &out,
        json!({
            "perSource": {
                "type-ownership": {"status": "clean", "driftCount": 0, "reportPath": "type.md"},
                "helper-registry": {
                    "status": "drift",
                    "driftCount": 3,
                    "reportPath": "helper.md",
                    "diagnostics": [{"message": "helper drift"}]
                },
                "topology": {"status": "skipped-missing-canon", "driftCount": 0},
                "naming": {"status": "parse-error", "driftCount": 0}
            }
        }),
    )?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 1, &log)?;
    let mut value = request(temp.path(), &out, &fake_node);
    value["strict"] = json!(true);

    let result = execute_check_canon_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 1);
    assert!(result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(result.block.execution_mode, Some("single-invocation-all"));
    assert_eq!(result.block.child_invocations, Some(1));
    assert_eq!(
        result.block.requested_sources.as_deref(),
        Some(
            [
                "type-ownership".to_string(),
                "helper-registry".to_string(),
                "topology".to_string(),
                "naming".to_string(),
            ]
            .as_slice()
        )
    );
    let summary = result
        .block
        .summary
        .ok_or_else(|| anyhow!("summary should be present"))?;
    assert_eq!(summary.drift_count, 3);
    assert_eq!(summary.sources_requested, 4);
    assert_eq!(summary.sources_checked, 2);
    assert_eq!(summary.sources_skipped, 1);
    assert_eq!(summary.sources_failed, 1);
    let per_source = result
        .block
        .per_source
        .ok_or_else(|| anyhow!("perSource should be present"))?;
    assert_eq!(per_source["type-ownership"].exit_code, 0);
    assert_eq!(per_source["helper-registry"].exit_code, 1);
    assert_eq!(per_source["helper-registry"].drift_count, Some(3));
    assert_eq!(per_source["topology"].exit_code, 2);
    assert_eq!(per_source["naming"].exit_code, 2);
    assert_eq!(log_lines(&log)?.len(), 1);
    assert!(log_lines(&log)?[0].contains("--source all"));
    Ok(())
}

#[test]
fn check_canon_all_sources_without_primary_artifacts_falls_back_to_per_source() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 0, &log)?;
    let value = request(temp.path(), &out, &fake_node);

    let result = execute_check_canon_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 0);
    assert_eq!(
        result.block.execution_mode,
        Some("per-source-artifact-fallback")
    );
    assert_eq!(result.block.child_invocations, Some(4));
    let per_source = result
        .block
        .per_source
        .ok_or_else(|| anyhow!("perSource should be present"))?;
    assert_eq!(per_source.len(), 4);
    assert!(per_source
        .values()
        .all(|entry| entry.ran && entry.status.as_deref() == Some("unknown")));
    assert_eq!(log_lines(&log)?.len(), 4);
    Ok(())
}

#[test]
fn check_canon_unknown_source_is_a_hard_contract_failure_without_spawning() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 0, &log)?;
    let mut value = request(temp.path(), &out, &fake_node);
    value["sourcesValue"] = json!("naming,unknown");
    value["strict"] = json!(true);

    let result = execute_check_canon_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 1);
    assert!(!result.block.ran);
    assert_eq!(
        result.block.reason.as_deref(),
        Some("unknown --sources values: unknown")
    );
    assert!(result.block.per_source.is_none());
    assert!(!log.exists());
    Ok(())
}

#[test]
fn check_canon_strict_mode_returns_two_when_every_source_is_unchecked() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    fs::write(out.join("symbols.json"), "{}")?;
    fs::write(out.join("topology.json"), "{}")?;
    write_canon_drift(
        &out,
        json!({
            "perSource": {
                "type-ownership": {"status": "skipped-missing-canon", "driftCount": 0},
                "helper-registry": {"status": "skipped-missing-canon", "driftCount": 0},
                "topology": {"status": "skipped-missing-canon", "driftCount": 0},
                "naming": {"status": "skipped-missing-canon", "driftCount": 0}
            }
        }),
    )?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 2, &log)?;
    let mut value = request(temp.path(), &out, &fake_node);
    value["strict"] = json!(true);

    let result = execute_check_canon_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 2);
    let summary = result
        .block
        .summary
        .ok_or_else(|| anyhow!("summary should be present"))?;
    assert_eq!(summary.sources_checked, 0);
    assert_eq!(summary.sources_skipped, 4);
    assert_eq!(summary.sources_failed, 0);
    assert_eq!(log_lines(&log)?.len(), 1);
    Ok(())
}

#[test]
fn cli_execute_check_canon_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-check-canon")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-check-canon"));
    Ok(())
}

fn write_canon_drift(out: &Path, value: Value) -> Result<()> {
    fs::write(out.join("canon-drift.json"), serde_json::to_string(&value)?)?;
    Ok(())
}

fn log_lines(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    Ok(text.lines().map(str::to_string).collect())
}

#[cfg(windows)]
fn write_fake_child(dir: &Path, exit_code: i32, log: &Path) -> Result<PathBuf> {
    let path = dir.join(format!("fake-check-canon-{exit_code}.cmd"));
    fs::write(
        &path,
        format!(
            "@echo off\r\necho %*>>\"{}\"\r\nexit /b {exit_code}\r\n",
            path_string(log)
        ),
    )?;
    Ok(path)
}

#[cfg(not(windows))]
fn write_fake_child(dir: &Path, exit_code: i32, log: &Path) -> Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(format!("fake-check-canon-{exit_code}"));
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexit {exit_code}\n",
            path_string(log)
        ),
    )?;
    let mut permissions = fs::metadata(&path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions)?;
    Ok(path)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
