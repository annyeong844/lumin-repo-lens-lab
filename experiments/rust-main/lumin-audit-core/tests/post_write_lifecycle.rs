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
    write_advisory(&advisory)?;
    let delta_path = out.join("post-write-delta.latest.json");
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
    assert_eq!(
        result.block.pre_write_invocation_id.as_deref(),
        Some("PRE-1")
    );
    assert_eq!(result.block.delta_invocation_id.as_deref(), Some("DELTA-1"));
    assert_eq!(
        result.block.delta_schema_version.as_deref(),
        Some("lumin-post-write-delta.v1")
    );
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
    write_advisory(&advisory)?;
    let delta_path = delta_out.join("post-write-delta.latest.json");
    let log = temp.path().join("child.log");
    let fake_node =
        write_fake_child_with_delta(temp.path(), 0, &log, Some(not_applicable_delta()))?;
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
fn post_write_child_failure_is_a_nonzero_lifecycle_failure() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    write_advisory(&advisory)?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child(temp.path(), 7, &log)?;

    let result = execute_post_write_lifecycle(parse_request(request(
        temp.path(),
        &out,
        &fake_node,
        Some(&advisory),
    ))?)?;

    assert_eq!(result.exit_code, 7);
    assert!(!result.block.ran);
    assert_eq!(result.block.child_exit_code, Some(7));
    assert_eq!(
        serde_json::to_value(&result.block)?["failureKind"],
        "child-failed"
    );
    assert!(result
        .block
        .reason
        .as_deref()
        .ok_or_else(|| anyhow!("reason should be present"))?
        .starts_with("post-write.mjs exited non-zero:"));
    Ok(())
}

#[test]
fn post_write_success_without_current_delta_rejects_and_removes_stale_latest() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let advisory = temp.path().join("pre-write-advisory.latest.json");
    write_advisory(&advisory)?;
    let delta_path = out.join("post-write-delta.latest.json");
    fs::write(&delta_path, serde_json::to_string(&default_delta())?)?;
    let log = temp.path().join("child.log");
    let fake_node = write_fake_child_with_delta(temp.path(), 0, &log, None)?;

    let result = execute_post_write_lifecycle(parse_request(request(
        temp.path(),
        &out,
        &fake_node,
        Some(&advisory),
    ))?)?;

    assert_eq!(result.exit_code, 1, "result={result:#?}");
    assert!(!result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["failureKind"],
        "delta-artifact-invalid"
    );
    assert!(result
        .block
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains("failed to read")));
    assert!(
        !delta_path.exists(),
        "stale latest must not survive the run"
    );
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
    write_advisory(&advisory)?;
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

fn write_advisory(path: &Path) -> Result<()> {
    fs::write(
        path,
        serde_json::to_string(&json!({ "invocationId": "PRE-1" }))?,
    )?;
    Ok(())
}

fn default_delta() -> Value {
    json!({
        "schemaVersion": "lumin-post-write-delta.v1",
        "preWriteInvocationId": "PRE-1",
        "deltaInvocationId": "DELTA-1",
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
    })
}

fn not_applicable_delta() -> Value {
    json!({
        "schemaVersion": "lumin-post-write-delta.v1",
        "preWriteInvocationId": "PRE-1",
        "deltaInvocationId": "DELTA-1",
        "summary": { "silentNew": 0 },
        "entries": [],
        "baseline": { "status": "missing" },
        "scanRangeParity": { "status": "baseline-missing" },
        "typeEscapeDelta": { "status": "not-applicable" },
        "inventoryCompleteness": { "afterComplete": null },
        "fileDelta": { "status": "missing" }
    })
}

#[cfg(windows)]
fn write_fake_child(dir: &Path, exit_code: i32, log: &Path) -> Result<PathBuf> {
    write_fake_child_with_delta(dir, exit_code, log, Some(default_delta()))
}

#[cfg(windows)]
fn write_fake_child_with_delta(
    dir: &Path,
    exit_code: i32,
    log: &Path,
    delta: Option<Value>,
) -> Result<PathBuf> {
    let path = dir.join(format!("fake-post-write-{exit_code}.cmd"));
    let template = dir.join("post-write-delta-template.json");
    if let Some(delta) = delta {
        fs::write(&template, serde_json::to_string_pretty(&delta)?)?;
    }
    let emit_delta = template.is_file() && exit_code == 0;
    fs::write(
        &path,
        format!(
            concat!(
                "@echo off\r\n",
                "setlocal EnableExtensions\r\n",
                "echo %*>>\"{log}\"\r\n",
                "set \"OUT=\"\r\n",
                "set \"DELTA_OUT=\"\r\n",
                ":loop\r\n",
                "if \"%~1\"==\"\" goto done\r\n",
                "if \"%~1\"==\"--output\" set \"OUT=%~2\"\r\n",
                "if \"%~1\"==\"--delta-out\" set \"DELTA_OUT=%~2\"\r\n",
                "shift\r\n",
                "goto loop\r\n",
                ":done\r\n",
                "if not defined DELTA_OUT set \"DELTA_OUT=%OUT%\"\r\n",
                "if {emit_delta} EQU 1 (\r\n",
                "  copy /Y \"{template}\" \"%DELTA_OUT%\\post-write-delta.latest.json\" >NUL\r\n",
                "  copy /Y \"{template}\" \"%DELTA_OUT%\\post-write-delta.PRE-1.DELTA-1.json\" >NUL\r\n",
                ")\r\n",
                "echo ## post-write delta\r\n",
                "echo [post-write] diagnostic 1>&2\r\n",
                "exit /b {exit_code}\r\n"
            ),
            log = path_string(log),
            template = path_string(&template),
            emit_delta = i32::from(emit_delta),
            exit_code = exit_code,
        ),
    )?;
    Ok(path)
}

#[cfg(not(windows))]
fn write_fake_child(dir: &Path, exit_code: i32, log: &Path) -> Result<PathBuf> {
    write_fake_child_with_delta(dir, exit_code, log, Some(default_delta()))
}

#[cfg(not(windows))]
fn write_fake_child_with_delta(
    dir: &Path,
    exit_code: i32,
    log: &Path,
    delta: Option<Value>,
) -> Result<PathBuf> {
    let path = dir.join("post-write.mjs");
    let template = dir.join("post-write-delta-template.json");
    if let Some(delta) = delta {
        fs::write(&template, serde_json::to_string_pretty(&delta)?)?;
    }
    let emit_delta = template.is_file() && exit_code == 0;
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
printf '%s %s\n' "$0" "$*" >> '{log}'
out=''
delta_out=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = '--output' ]; then
    shift
    out="$1"
  elif [ "$1" = '--delta-out' ]; then
    shift
    delta_out="$1"
  fi
  shift || true
done
if [ -z "$delta_out" ]; then delta_out="$out"; fi
if [ {emit_delta} -eq 1 ]; then
  cp '{template}' "$delta_out/post-write-delta.latest.json"
  cp '{template}' "$delta_out/post-write-delta.PRE-1.DELTA-1.json"
fi
printf '%s\n' '## post-write delta'
printf '%s\n' '[post-write] diagnostic' >&2
exit {exit_code}
"#,
            log = path_string(log),
            template = path_string(&template),
            emit_delta = i32::from(emit_delta),
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
