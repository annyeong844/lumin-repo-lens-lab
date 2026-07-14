use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

use lumin_audit_core::post_write_lifecycle::{
    execute_post_write_lifecycle, PostWriteLifecycleRequest,
};

fn request(root: &Path, out: &Path, advisory: Option<&Path>) -> Value {
    json!({
        "schemaVersion": "lumin-post-write-lifecycle-request.v3",
        "root": path_string(root),
        "output": path_string(out),
        "advisoryPath": advisory.map(path_string),
        "deltaOut": null,
        "deltaInvocationId": "DELTA-1",
        "generated": "2026-07-13T00:00:00.000Z",
        "includeTests": false,
        "excludes": [],
        "incremental": { "enabled": false, "clear": false }
    })
}

fn parse_request(value: Value) -> Result<PostWriteLifecycleRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn native_js_post_write_scans_classifies_and_dual_writes_delta() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let out = temp.path().join("out");
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(&out)?;
    fs::write(root.join("src/a.ts"), "export const value = 1 as any;\n")?;
    let advisory = out.join("pre-write-advisory.PRE-1.json");
    let before = out.join("any-inventory.pre.PRE-1.json");
    write_js_advisory(&advisory, &[], &[])?;
    write_inventory(&before, Vec::new())?;

    let result =
        execute_post_write_lifecycle(parse_request(request(&root, &out, Some(&advisory)))?)?;

    assert_eq!(result.exit_code, 0);
    assert!(result.block.ran);
    assert_eq!(result.block.silent_new, Some(1));
    assert_eq!(result.block.required_acknowledgement_count, Some(1));
    assert_eq!(result.block.baseline_status.as_deref(), Some("available"));
    assert_eq!(result.block.scan_range_parity.as_deref(), Some("ok"));
    assert_eq!(
        result.block.type_escape_delta_status.as_deref(),
        Some("computed")
    );
    assert_eq!(result.block.after_complete, Some(Value::Bool(true)));
    assert!(result
        .stdout
        .as_deref()
        .unwrap_or_default()
        .contains("silent-new — REQUIRE acknowledgment: 1"));
    let latest: Value = serde_json::from_str(&fs::read_to_string(
        out.join("post-write-delta.latest.json"),
    )?)?;
    let specific: Value = serde_json::from_str(&fs::read_to_string(
        out.join("post-write-delta.PRE-1.DELTA-1.json"),
    )?)?;
    assert_eq!(latest, specific);
    assert_eq!(latest["summary"]["silentNew"], 1);
    assert!(out.join("any-inventory.json").is_file());
    Ok(())
}

#[test]
fn missing_advisory_is_a_force_exit_contract_failure() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let result = execute_post_write_lifecycle(parse_request(request(temp.path(), &out, None))?)?;

    assert_eq!(result.exit_code, 2);
    assert!(!result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["failureKind"],
        "missing-advisory"
    );
    assert!(result
        .stderr
        .as_deref()
        .unwrap_or_default()
        .contains("--pre-write-advisory <file> missing"));
    Ok(())
}

#[test]
fn invalid_advisory_removes_stale_latest_before_validation() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    fs::write(&advisory, "{")?;
    let latest = out.join("post-write-delta.latest.json");
    fs::write(&latest, "{}")?;

    let result =
        execute_post_write_lifecycle(parse_request(request(temp.path(), &out, Some(&advisory)))?)?;

    assert_eq!(result.exit_code, 2);
    assert_eq!(
        serde_json::to_value(&result.block)?["failureKind"],
        "invalid-advisory"
    );
    assert!(!latest.exists());
    Ok(())
}

#[test]
fn rust_post_write_skips_type_escapes_but_computes_file_delta() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let out = temp.path().join("out");
    let delta_out = temp.path().join("delta");
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(&out)?;
    fs::write(root.join("src/new.ts"), "export const value = 1;\n")?;
    let advisory = out.join("pre-write-advisory.PRE-1.json");
    write_rust_advisory(&advisory)?;
    let mut request_value = request(&root, &out, Some(&advisory));
    request_value["deltaOut"] = json!(path_string(&delta_out));

    let result = execute_post_write_lifecycle(parse_request(request_value)?)?;

    assert_eq!(result.exit_code, 0);
    assert_eq!(
        result.block.type_escape_delta_status.as_deref(),
        Some("not-applicable")
    );
    assert_eq!(result.block.after_complete, Some(Value::Null));
    assert_eq!(result.block.file_delta_status.as_deref(), Some("computed"));
    let delta: Value = serde_json::from_str(&fs::read_to_string(
        delta_out.join("post-write-delta.latest.json"),
    )?)?;
    assert_eq!(
        result.block.unexpected_new_file_count,
        Some(1),
        "delta={delta:#}"
    );
    assert!(delta_out.join("post-write-delta.latest.json").is_file());
    Ok(())
}

#[test]
fn cli_execute_post_write_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-post-write")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-post-write"));
    Ok(())
}

#[test]
fn cli_result_output_replays_markdown_and_writes_small_result() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("repo");
    let out = temp.path().join("out");
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(&out)?;
    fs::write(root.join("src/a.ts"), "export const value = 1;\n")?;
    let advisory = out.join("pre-write-advisory.PRE-1.json");
    let before = out.join("any-inventory.pre.PRE-1.json");
    write_js_advisory(&advisory, &["src/a.ts"], &[])?;
    write_inventory(&before, Vec::new())?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_vec(&request(&root, &out, Some(&advisory)))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-post-write")
        .arg("--input")
        .arg(&input_path)
        .arg("--result-output")
        .arg(&result_path)
        .output()?;

    assert!(
        output.status.success(),
        "status={:?}\nstdout={}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("## post-write delta"));
    let result: Value = serde_json::from_str(&fs::read_to_string(result_path)?)?;
    assert_eq!(result["block"]["ran"], true);
    assert!(result.get("stdout").is_none());
    assert!(result.get("stderr").is_none());
    Ok(())
}

fn write_js_advisory(path: &Path, before_files: &[&str], planned_files: &[&str]) -> Result<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "invocationId": "PRE-1",
            "intentHash": "intent-hash",
            "intent": {
                "names": [],
                "shapes": [],
                "files": planned_files,
                "dependencies": [],
                "plannedTypeEscapes": []
            },
            "scanRange": {
                "output": path.parent().map(path_string),
            },
            "preWrite": {
                "anyInventoryPath": "any-inventory.pre.PRE-1.json",
                "fileInventory": {
                    "status": "available",
                    "files": before_files,
                }
            },
            "capabilities": {
                "language": "js-ts",
                "postWriteTypeEscapes": "available"
            }
        }))?,
    )?;
    Ok(())
}

fn write_rust_advisory(path: &Path) -> Result<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "invocationId": "PRE-1",
            "intentHash": "intent-hash",
            "intent": {
                "language": "rust",
                "files": [],
                "plannedTypeEscapes": []
            },
            "preWrite": {
                "fileInventory": { "status": "available", "files": [] }
            },
            "capabilities": {
                "language": "rust",
                "postWriteTypeEscapes": "not-applicable"
            },
            "rustPreWrite": { "schemaVersion": "probe" }
        }))?,
    )?;
    Ok(())
}

fn write_inventory(path: &Path, type_escapes: Vec<Value>) -> Result<()> {
    fs::write(
        path,
        serde_json::to_vec_pretty(&json!({
            "meta": {
                "complete": true,
                "scope": "TS/JS production files",
                "includeTests": false,
                "exclude": [],
                "filesWithParseErrors": [],
                "supports": {
                    "typeEscapes": true,
                    "escapeKinds": [
                        "explicit-any", "as-any", "angle-any", "as-unknown-as-T",
                        "rest-any-args", "index-sig-any", "generic-default-any",
                        "ts-ignore", "ts-expect-error", "no-explicit-any-disable",
                        "jsdoc-any"
                    ]
                }
            },
            "typeEscapes": type_escapes
        }))?,
    )?;
    Ok(())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
