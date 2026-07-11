use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lumin_audit_core::canon_draft_lifecycle::{
    execute_canon_draft_lifecycle, CanonDraftLifecycleRequest,
};

fn request(root: &Path, fake_node: &Path) -> Value {
    json!({
        "schemaVersion": "lumin-canon-draft-lifecycle-request.v1",
        "sourcesValue": null,
        "root": path_string(root),
        "output": path_string(&root.join("out")),
        "canonOutput": null,
        "scriptsDir": path_string(root),
        "nodeExecutable": path_string(fake_node),
        "scanArgs": []
    })
}

fn parse_request(value: Value) -> Result<CanonDraftLifecycleRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn canon_draft_runs_requested_sources_and_uses_fallback_draft_paths() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_child(temp.path(), 0)?;
    let canon_output = temp.path().join("drafts");
    let mut value = request(temp.path(), &fake_node);
    value["sourcesValue"] = json!("naming,type-ownership,naming");
    value["canonOutput"] = json!(path_string(&canon_output));
    value["scanArgs"] = json!(["--production", "--exclude", "dist"]);

    let result = execute_canon_draft_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 0);
    assert!(!result.force_exit_code);
    assert!(result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(
        result.block.requested_sources,
        Some(vec!["naming".to_string(), "type-ownership".to_string()])
    );
    let per_source = result
        .block
        .per_source
        .ok_or_else(|| anyhow!("perSource should be present"))?;
    assert_eq!(per_source["naming"].exit_code, 0);
    assert_eq!(
        per_source["naming"].draft_path.as_deref(),
        Some(path_string(&canon_output.join("naming.md")).as_str())
    );
    assert_eq!(
        per_source["type-ownership"].draft_path.as_deref(),
        Some(path_string(&canon_output.join("type-ownership.md")).as_str())
    );
    Ok(())
}

#[test]
fn canon_draft_unknown_sources_force_exit_without_spawning() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_child(temp.path(), 0)?;
    let mut value = request(temp.path(), &fake_node);
    value["sourcesValue"] = json!("naming,unknown");

    let result = execute_canon_draft_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 1);
    assert!(result.force_exit_code);
    assert!(!result.block.ran);
    assert_eq!(
        result.block.reason.as_deref(),
        Some("unknown --sources values: unknown")
    );
    assert!(result.block.per_source.is_none());
    Ok(())
}

#[test]
fn canon_draft_all_failed_is_advisory_not_forced_exit() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let fake_node = write_fake_child(temp.path(), 2)?;
    let value = request(temp.path(), &fake_node);

    let result = execute_canon_draft_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 1);
    assert!(!result.force_exit_code);
    assert!(!result.block.ran);
    assert_eq!(
        result.block.reason.as_deref(),
        Some("all requested sources failed")
    );
    let per_source = result
        .block
        .per_source
        .ok_or_else(|| anyhow!("perSource should be present"))?;
    assert_eq!(per_source.len(), 4);
    assert!(per_source.values().all(|entry| {
        entry.exit_code == 2
            && entry.reason.as_deref()
                == Some("required producer artifact absent (see stderr of child process)")
    }));
    Ok(())
}

#[test]
fn cli_execute_canon_draft_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-canon-draft")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-canon-draft"));
    Ok(())
}

#[cfg(windows)]
fn write_fake_child(dir: &Path, exit_code: i32) -> Result<PathBuf> {
    let path = dir.join(format!("fake-child-{exit_code}.cmd"));
    fs::write(&path, format!("@echo off\r\nexit /b {exit_code}\r\n"))?;
    Ok(path)
}

#[cfg(not(windows))]
fn write_fake_child(dir: &Path, exit_code: i32) -> Result<PathBuf> {
    let path = dir.join("generate-canon-draft.mjs");
    fs::write(&path, format!("#!/bin/sh\nexit {exit_code}\n"))?;
    Ok(PathBuf::from("/bin/sh"))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
