use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use lumin_audit_core::post_write_lifecycle::{
    execute_post_write_lifecycle, PostWriteLifecycleRequest,
};

fn request(root: &Path, out: &Path, fake_node: &Path, advisory: Option<&Path>) -> Value {
    json!({
        "schemaVersion": "lumin-post-write-lifecycle-request.v1",
        "root": path_string(root),
        "output": path_string(out),
        "scriptsDir": path_string(root),
        "nodeExecutable": path_string(fake_node),
        "advisoryPath": advisory.map(path_string),
        "deltaOut": null,
        "noFreshAudit": false,
        "scanArgs": ["--production"],
        "incrementalArgs": ["--no-incremental"]
    })
}

fn parse_request(value: Value) -> Result<PostWriteLifecycleRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn post_write_runs_child_and_projects_delta_summary_fields() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    fs::write(&advisory, "{}")?;
    let delta_path = out.join("post-write-delta.latest.json");
    write_delta(&delta_path)?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 0, &log)?;

    let result = execute_post_write_lifecycle(parse_request(request(
        temp.path(),
        &out,
        &fake_node,
        Some(&advisory),
    ))?)?;

    assert_eq!(result.exit_code, 0);
    assert!(result.block.ran);
    assert!(result
        .stdout
        .as_deref()
        .unwrap_or_default()
        .contains("## post-write delta"));
    assert!(result
        .stderr
        .as_deref()
        .unwrap_or_default()
        .contains("[post-write] diagnostic"));
    assert_eq!(
        serde_json::to_value(&result.block)?["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(
        result.block.delta_path.as_deref(),
        Some(path_string(&delta_path).as_str())
    );
    assert_eq!(result.block.silent_new, Some(2));
    assert_eq!(result.block.required_acknowledgement_count, Some(2));
    assert_eq!(result.block.baseline_status.as_deref(), Some("available"));
    assert_eq!(result.block.scan_range_parity.as_deref(), Some("ok"));
    assert_eq!(
        result.block.type_escape_delta_status.as_deref(),
        Some("computed")
    );
    assert_eq!(result.block.after_complete, Some(Value::Bool(true)));
    assert_eq!(result.block.file_delta_status.as_deref(), Some("computed"));
    assert_eq!(result.block.unexpected_new_file_count, Some(1));
    assert_eq!(result.block.planned_missing_file_count, Some(0));
    let log_line = fs::read_to_string(log)?;
    assert!(log_line.contains("post-write.mjs"));
    assert!(log_line.contains("--pre-write-advisory"));
    assert!(log_line.contains("--production"));
    assert!(log_line.contains("--no-incremental"));
    Ok(())
}

#[test]
fn post_write_missing_advisory_is_a_force_exit_contract_failure() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 0, &log)?;

    let result =
        execute_post_write_lifecycle(parse_request(request(temp.path(), &out, &fake_node, None))?)?;

    assert_eq!(result.exit_code, 2);
    assert!(!result.block.ran);
    assert!(result
        .stderr
        .as_deref()
        .unwrap_or_default()
        .contains("--pre-write-advisory <file> missing"));
    assert_eq!(
        result.block.reason.as_deref(),
        Some("--pre-write-advisory missing")
    );
    assert!(!log.exists());
    Ok(())
}

#[test]
fn post_write_delta_out_relocates_delta_path_and_allows_null_after_complete() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    let delta_out = temp.path().join("delta");
    fs::create_dir_all(&out)?;
    fs::create_dir_all(&delta_out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    fs::write(&advisory, "{}")?;
    let delta_path = delta_out.join("post-write-delta.latest.json");
    fs::write(
        &delta_path,
        serde_json::to_string(&json!({
            "summary": {},
            "entries": [],
            "typeEscapeDelta": { "status": "not-applicable" },
            "fileDelta": { "summary": {} }
        }))?,
    )?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 0, &log)?;
    let mut value = request(temp.path(), &out, &fake_node, Some(&advisory));
    value["deltaOut"] = json!(path_string(&delta_out));

    let result = execute_post_write_lifecycle(parse_request(value)?)?;

    assert!(
        result.block.ran,
        "post-write child should run successfully: {result:?}"
    );
    assert_eq!(
        result.block.delta_path.as_deref(),
        Some(path_string(&delta_path).as_str())
    );
    assert_eq!(result.block.baseline_status.as_deref(), Some("missing"));
    assert_eq!(
        result.block.scan_range_parity.as_deref(),
        Some("baseline-missing")
    );
    assert_eq!(
        result.block.type_escape_delta_status.as_deref(),
        Some("not-applicable")
    );
    assert_eq!(result.block.after_complete, Some(Value::Null));
    assert_eq!(result.block.file_delta_status.as_deref(), Some("missing"));
    assert!(fs::read_to_string(log)?.contains("--delta-out"));
    Ok(())
}

#[test]
fn post_write_child_failure_is_advisory_by_default() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    fs::write(&advisory, "{}")?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 7, &log)?;

    let result = execute_post_write_lifecycle(parse_request(request(
        temp.path(),
        &out,
        &fake_node,
        Some(&advisory),
    ))?)?;

    assert_eq!(result.exit_code, 0);
    assert!(!result.block.ran);
    assert!(result
        .block
        .reason
        .as_deref()
        .ok_or_else(|| anyhow!("reason should be present"))?
        .starts_with("post-write.mjs exited non-zero:"));
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
fn cli_execute_post_write_result_output_streams_child_and_writes_clean_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    fs::write(&advisory, "{}")?;
    write_delta(&out.join("post-write-delta.latest.json"))?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 0, &log)?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("post-write-result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&request(temp.path(), &out, &fake_node, Some(&advisory)))?,
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
    assert!(String::from_utf8_lossy(&output.stderr).contains("[post-write] diagnostic"));
    let result: Value = serde_json::from_str(&fs::read_to_string(result_path)?)?;
    assert_eq!(result["block"]["ran"], true);
    assert!(result.get("stdout").is_none());
    assert!(result.get("stderr").is_none());
    Ok(())
}

fn write_delta(path: &Path) -> Result<()> {
    fs::write(
        path,
        serde_json::to_string(&json!({
            "summary": { "silentNew": 2 },
            "entries": [
                { "label": "silent-new" },
                { "label": "planned" },
                { "label": "silent-new" }
            ],
            "baseline": { "status": "available" },
            "scanRangeParity": { "status": "ok" },
            "typeEscapeDelta": { "status": "computed" },
            "inventoryCompleteness": { "afterComplete": true },
            "fileDelta": {
                "status": "computed",
                "summary": {
                    "unexpectedNew": 1,
                    "plannedMissing": 0
                }
            }
        }))?,
    )?;
    Ok(())
}

#[cfg(windows)]
fn write_fake_child(dir: &Path, exit_code: i32, log: &Path) -> Result<PathBuf> {
    let path = dir.join(format!("fake-post-write-{exit_code}.cmd"));
    fs::write(
        &path,
        format!(
            "@echo off\r\necho %*>>\"{}\"\r\necho ## post-write delta\r\necho [post-write] diagnostic 1>&2\r\nexit /b {exit_code}\r\n",
            path_string(log)
        ),
    )?;
    Ok(path)
}

#[cfg(not(windows))]
fn write_fake_child(dir: &Path, exit_code: i32, log: &Path) -> Result<PathBuf> {
    let path = dir.join("post-write.mjs");
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nprintf '%s %s\\n' \"$0\" \"$*\" >> '{}'\nprintf '%s\\n' '## post-write delta'\nprintf '%s\\n' '[post-write] diagnostic' >&2\nexit {exit_code}\n",
            path_string(log)
        ),
    )?;
    Ok(PathBuf::from("/bin/sh"))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
