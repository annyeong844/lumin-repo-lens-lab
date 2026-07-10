use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

use lumin_audit_core::pre_write_lifecycle::{
    execute_js_pre_write_lifecycle, execute_rust_pre_write_lifecycle, JsPreWriteLifecycleRequest,
    RustPreWriteLifecycleRequest,
};

fn request(root: &Path, out: &Path, fake: &FakeAnalyzer) -> Value {
    json!({
        "schemaVersion": "lumin-rust-pre-write-lifecycle-request.v1",
        "root": path_string(root),
        "output": path_string(out),
        "sourceCommit": "abc123",
        "invocationId": "INV-1",
        "rustNativeArtifactPath": path_string(&out.join("rust-pre-write-artifact.INV-1.json")),
        "rustNativeLatestPath": path_string(&out.join("rust-pre-write-artifact.latest.json")),
        "analyzer": {
            "command": fake.command,
            "prefixArgs": fake.prefix_args,
            "source": "fixture",
            "manifestPath": path_string(root.join("experiments/Cargo.toml").as_path()),
        },
        "intentInput": "{\n  \"names\": [\"Thing\"]\n}\n",
        "engineSelection": {
            "requested": "rust",
            "selected": "rust",
            "reason": "explicit-cli",
            "intentLanguage": "rust"
        },
        "includeTests": false,
        "production": true,
        "excludes": ["target"],
        "fileInventory": {
            "status": "available",
            "pathMode": "repo-relative",
            "fileCount": 1,
            "files": ["js-owned.ts"]
        },
        "failures": [{
            "kind": "js-owned-inventory-note",
            "reason": "preserved by audit-core"
        }],
    })
}

fn parse_request(value: Value) -> Result<RustPreWriteLifecycleRequest> {
    Ok(serde_json::from_value(value)?)
}

fn js_request(root: &Path, out: &Path, fake: &FakeJsPreWrite) -> Value {
    json!({
        "schemaVersion": "lumin-js-pre-write-lifecycle-request.v1",
        "root": path_string(root),
        "output": path_string(out),
        "scriptsDir": path_string(&fake.scripts_dir),
        "nodeExecutable": fake.command,
        "childIntentFlag": "-",
        "childIntentInput": "{\n  \"names\": [\"Thing\"]\n}\n",
        "engineSelection": {
            "requested": "auto",
            "selected": "js",
            "reason": "intent-language-absent-default-js"
        },
        "noFreshAudit": true,
        "scanArgs": ["--production", "--exclude", "dist"],
    })
}

fn parse_js_request(value: Value) -> Result<JsPreWriteLifecycleRequest> {
    Ok(serde_json::from_value(value)?)
}

#[test]
fn rust_pre_write_runs_analyzer_and_wraps_native_artifact_as_standard_advisory() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    write_inventory_fixture(temp.path())?;
    let fake = write_fake_analyzer(temp.path(), 0)?;

    let result =
        execute_rust_pre_write_lifecycle(parse_request(request(temp.path(), &out, &fake))?)?;

    assert_eq!(result.exit_code, 0, "result={result:#?}");
    assert!(result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(result.block.engine, "rust");
    assert_eq!(result.block.language, "rust");
    assert_eq!(
        result.block.advisory_path.as_deref(),
        Some(path_string(&out.join("pre-write-advisory.INV-1.json")).as_str())
    );
    assert_eq!(
        result.block.latest_advisory_path.as_deref(),
        Some(path_string(&out.join("pre-write-advisory.latest.json")).as_str())
    );
    assert_eq!(
        result.block.rust_native_latest_path.as_deref(),
        Some(path_string(&out.join("rust-pre-write-artifact.latest.json")).as_str())
    );
    assert!(result
        .stdout
        .as_deref()
        .unwrap_or_default()
        .contains("## rust pre-write"));
    assert!(result
        .stderr
        .as_deref()
        .unwrap_or_default()
        .contains("[rust-pre-write] diagnostic"));

    let native = fs::read_to_string(out.join("rust-pre-write-artifact.INV-1.json"))?;
    let native_latest = fs::read_to_string(out.join("rust-pre-write-artifact.latest.json"))?;
    assert_eq!(native_latest, native);

    let advisory_specific = fs::read_to_string(out.join("pre-write-advisory.INV-1.json"))?;
    let advisory_latest = fs::read_to_string(out.join("pre-write-advisory.latest.json"))?;
    assert_eq!(advisory_latest, advisory_specific);
    let advisory: Value = serde_json::from_str(&advisory_latest)?;
    assert_eq!(advisory["invocationId"], "INV-1");
    assert_eq!(advisory["intent"]["language"], "rust");
    assert_eq!(advisory["intent"]["names"][0], "Thing");
    assert_eq!(advisory["scanRange"]["production"], true);
    assert_eq!(advisory["preWrite"]["fileInventory"]["fileCount"], 1);
    assert_eq!(
        advisory["preWrite"]["fileInventory"]["files"][0],
        "js-owned.ts"
    );
    assert_eq!(advisory["failures"][0]["kind"], "js-owned-inventory-note");
    assert_eq!(
        advisory["preWrite"]["rustNativeArtifactPath"],
        path_string(&out.join("rust-pre-write-artifact.INV-1.json"))
    );
    assert_eq!(advisory["rustPreWrite"]["coverage"]["names"], "ran");
    assert_eq!(
        advisory["capabilities"]["postWriteTypeEscapes"],
        "not-applicable"
    );
    assert_eq!(
        advisory["artifactPaths"]["invocationSpecific"],
        path_string(&out.join("pre-write-advisory.INV-1.json"))
    );
    assert!(advisory["intentHash"]
        .as_str()
        .is_some_and(|hash| hash.len() == 64 && hash.chars().all(|ch| ch.is_ascii_hexdigit())));

    let logged_args = fs::read_to_string(temp.path().join("args.log"))?.replace("\r\n", "\n");
    assert!(logged_args.contains("pre-write"));
    assert!(logged_args.contains("--source-commit\nabc123"));
    assert!(logged_args.contains("--intent\n-"));
    assert!(logged_args.contains("--production"));
    assert!(logged_args.contains("--exclude\ntarget"));
    let captured_intent = fs::read_to_string(temp.path().join("intent.stdin"))?
        .replace("\r\n", "\n")
        .trim()
        .to_string();
    assert!(captured_intent.contains("\"names\": [\"Thing\"]"));
    assert!(!captured_intent.contains("language"));
    Ok(())
}

#[test]
fn rust_pre_write_fills_missing_source_commit_inside_audit_core() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    write_inventory_fixture(temp.path())?;
    let fake = write_fake_analyzer(temp.path(), 0)?;
    let mut value = request(temp.path(), &out, &fake);
    value
        .as_object_mut()
        .ok_or_else(|| anyhow!("request should be an object"))?
        .remove("sourceCommit");

    let result = execute_rust_pre_write_lifecycle(parse_request(value)?)?;

    assert_eq!(result.exit_code, 0, "result={result:#?}");
    assert_eq!(result.block.source_commit.as_deref(), Some("unknown"));
    let logged_args = fs::read_to_string(temp.path().join("args.log"))?.replace("\r\n", "\n");
    assert!(logged_args.contains("--source-commit\nunknown"));
    let advisory: Value = serde_json::from_str(&fs::read_to_string(
        out.join("pre-write-advisory.latest.json"),
    )?)?;
    assert_eq!(advisory["preWrite"]["sourceCommit"], "unknown");
    Ok(())
}

fn write_inventory_fixture(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn ignored_by_js_inventory() {}\n",
    )?;
    fs::write(root.join("src").join("app.ts"), "export const app = 1;\n")?;
    fs::write(
        root.join("src").join("app.test.ts"),
        "test('app', () => {});\n",
    )?;
    Ok(())
}

#[test]
fn rust_pre_write_child_failure_records_raw_block_without_advisory() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let fake = write_fake_analyzer(temp.path(), 7)?;

    let result =
        execute_rust_pre_write_lifecycle(parse_request(request(temp.path(), &out, &fake))?)?;

    assert_eq!(result.exit_code, 7, "result={result:#?}");
    assert!(!result.block.ran);
    assert!(result
        .block
        .reason
        .as_deref()
        .ok_or_else(|| anyhow!("reason should be present"))?
        .starts_with("lumin-rust-analyzer pre-write exited non-zero:"));
    assert!(!out.join("pre-write-advisory.latest.json").exists());
    Ok(())
}

#[test]
fn js_pre_write_runs_existing_producer_and_projects_advisory_block() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let fake = write_fake_js_pre_write(temp.path(), &out, 0)?;

    let result =
        execute_js_pre_write_lifecycle(parse_js_request(js_request(temp.path(), &out, &fake))?)?;

    assert_eq!(result.exit_code, 0, "result={result:#?}");
    assert!(result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["executionOwner"],
        "lumin-audit-core"
    );
    assert_eq!(result.block.engine, "js");
    assert_eq!(result.block.language, "js-ts");
    assert_eq!(result.block.producer, "pre-write.mjs");
    assert_eq!(result.block.advisory_invocation_id.as_deref(), Some("JS-1"));
    assert_eq!(
        result.block.advisory_path.as_deref(),
        Some(path_string(&out.join("pre-write-advisory.JS-1.json")).as_str())
    );
    assert_eq!(
        result.block.latest_advisory_path.as_deref(),
        Some(path_string(&out.join("pre-write-advisory.latest.json")).as_str())
    );
    assert_eq!(
        result
            .block
            .evidence_availability
            .as_ref()
            .and_then(|value| value.get("status"))
            .and_then(Value::as_str),
        Some("available")
    );
    assert!(result
        .stdout
        .as_deref()
        .unwrap_or_default()
        .contains("## js pre-write"));
    assert!(result
        .stderr
        .as_deref()
        .unwrap_or_default()
        .contains("[js-pre-write] diagnostic"));

    let logged_args = fs::read_to_string(temp.path().join("js-args.log"))?.replace("\r\n", "\n");
    assert!(logged_args.contains("pre-write.mjs"));
    assert!(logged_args.contains("--root"));
    assert!(logged_args.contains("--output"));
    assert!(logged_args.contains("--intent\n-"));
    assert!(logged_args.contains("--no-fresh-audit"));
    assert!(logged_args.contains("--exclude\ndist"));
    let captured_intent = fs::read_to_string(temp.path().join("js-intent.stdin"))?
        .replace("\r\n", "\n")
        .trim()
        .to_string();
    assert!(captured_intent.contains("\"names\": [\"Thing\"]"));
    Ok(())
}

#[test]
fn js_pre_write_child_failure_records_block_without_advisory() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let fake = write_fake_js_pre_write(temp.path(), &out, 9)?;

    let result =
        execute_js_pre_write_lifecycle(parse_js_request(js_request(temp.path(), &out, &fake))?)?;

    assert_eq!(result.exit_code, 9, "result={result:#?}");
    assert!(!result.block.ran);
    assert!(result
        .block
        .reason
        .as_deref()
        .ok_or_else(|| anyhow!("reason should be present"))?
        .starts_with("pre-write.mjs exited non-zero:"));
    assert!(!out.join("pre-write-advisory.latest.json").exists());
    Ok(())
}

#[test]
fn cli_execute_rust_pre_write_result_output_streams_child_and_writes_clean_result_file(
) -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    write_inventory_fixture(temp.path())?;
    let fake = write_fake_analyzer(temp.path(), 0)?;
    let input_path = temp.path().join("request.json");
    let result_path = temp.path().join("result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&request(temp.path(), &out, &fake))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-rust-pre-write")
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
    assert!(String::from_utf8_lossy(&output.stdout).contains("## rust pre-write"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("[rust-pre-write] diagnostic"));
    let result: Value = serde_json::from_str(&fs::read_to_string(result_path)?)?;
    assert_eq!(result["block"]["ran"], true);
    assert!(result.get("stdout").is_none());
    assert!(result.get("stderr").is_none());
    Ok(())
}

#[test]
fn cli_execute_js_pre_write_result_output_streams_child_and_writes_clean_result_file() -> Result<()>
{
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let fake = write_fake_js_pre_write(temp.path(), &out, 0)?;
    let input_path = temp.path().join("js-request.json");
    let result_path = temp.path().join("js-result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&js_request(temp.path(), &out, &fake))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("execute-js-pre-write")
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
    assert!(String::from_utf8_lossy(&output.stdout).contains("## js pre-write"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("[js-pre-write] diagnostic"));
    let result: Value = serde_json::from_str(&fs::read_to_string(result_path)?)?;
    assert_eq!(result["block"]["ran"], true);
    assert_eq!(result["block"]["engine"], "js");
    assert!(result.get("stdout").is_none());
    assert!(result.get("stderr").is_none());
    Ok(())
}

#[test]
fn cli_execute_js_pre_write_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("js-request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-js-pre-write")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-js-pre-write"));
    Ok(())
}

#[test]
fn cli_execute_rust_pre_write_hard_stops_on_malformed_request() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("request.json");
    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("execute-rust-pre-write")
        .arg("--input")
        .arg(&input_path)
        .output()?;

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("execute-rust-pre-write"));
    Ok(())
}

struct FakeAnalyzer {
    command: String,
    prefix_args: Vec<String>,
}

struct FakeJsPreWrite {
    command: String,
    scripts_dir: std::path::PathBuf,
}

#[cfg(windows)]
fn write_fake_analyzer(dir: &Path, exit_code: i32) -> Result<FakeAnalyzer> {
    let script = dir.join(format!("fake-rust-pre-write-{exit_code}.cmd"));
    let template = dir.join("rust-native-template.json");
    write_native_template(&template)?;
    fs::write(
        &script,
        format!(
            concat!(
                "@echo off\r\n",
                "setlocal EnableExtensions\r\n",
                "set \"OUT=\"\r\n",
                ":loop\r\n",
                "if \"%~1\"==\"\" goto done\r\n",
                "echo %~1>>\"{args_log}\"\r\n",
                "if \"%~1\"==\"--output\" (\r\n",
                "  set \"OUT=%~2\"\r\n",
                "  echo %~2>>\"{args_log}\"\r\n",
                ")\r\n",
                "shift\r\n",
                "goto loop\r\n",
                ":done\r\n",
                "more > \"{stdin_log}\"\r\n",
                "if {exit_code} EQU 0 (\r\n",
                "  copy /Y \"{template}\" \"%OUT%\" >NUL\r\n",
                "  echo ## rust pre-write\r\n",
                "  echo [rust-pre-write] diagnostic 1>&2\r\n",
                ")\r\n",
                "exit /b {exit_code}\r\n"
            ),
            args_log = path_string(&dir.join("args.log")),
            stdin_log = path_string(&dir.join("intent.stdin")),
            template = path_string(&template),
            exit_code = exit_code,
        ),
    )?;
    Ok(FakeAnalyzer {
        command: path_string(&script),
        prefix_args: Vec::new(),
    })
}

#[cfg(not(windows))]
fn write_fake_analyzer(dir: &Path, exit_code: i32) -> Result<FakeAnalyzer> {
    use std::os::unix::fs::PermissionsExt;

    let script = dir.join(format!("fake-rust-pre-write-{exit_code}"));
    let template = dir.join("rust-native-template.json");
    write_native_template(&template)?;
    fs::write(
        &script,
        format!(
            r#"#!/bin/sh
printf '%s\n' "$@" > '{args_log}'
cat > '{stdin_log}'
out=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = '--output' ]; then
    shift
    out="$1"
  fi
  shift || true
done
if [ {exit_code} -eq 0 ]; then
  cp '{template}' "$out"
  printf '%s\n' '## rust pre-write'
  printf '%s\n' '[rust-pre-write] diagnostic' >&2
fi
exit {exit_code}
"#,
            args_log = path_string(&dir.join("args.log")),
            stdin_log = path_string(&dir.join("intent.stdin")),
            template = path_string(&template),
        ),
    )?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    Ok(FakeAnalyzer {
        command: path_string(&script),
        prefix_args: Vec::new(),
    })
}

fn write_native_template(path: &Path) -> Result<()> {
    fs::write(
        path,
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "rust-pre-write.v1",
            "policyVersion": "rust-pre-write-policy.v1",
            "intent": { "names": ["Thing"] },
            "meta": { "producer": "lumin-rust-analyzer" },
            "coverage": { "names": "ran" },
            "lookups": [],
            "cueCards": []
        }))?,
    )?;
    Ok(())
}

#[cfg(windows)]
fn write_fake_js_pre_write(dir: &Path, out: &Path, exit_code: i32) -> Result<FakeJsPreWrite> {
    let scripts_dir = dir.join(format!("js-scripts-{exit_code}"));
    fs::create_dir_all(&scripts_dir)?;
    fs::write(scripts_dir.join("pre-write.mjs"), "// fake path only\n")?;
    let script = dir.join(format!("fake-js-pre-write-{exit_code}.cmd"));
    let template = dir.join("js-advisory-template.json");
    write_js_advisory_template(&template, out)?;
    fs::write(
        &script,
        format!(
            concat!(
                "@echo off\r\n",
                "setlocal EnableExtensions\r\n",
                "set \"OUT=\"\r\n",
                ":loop\r\n",
                "if \"%~1\"==\"\" goto done\r\n",
                "echo %~1>>\"{args_log}\"\r\n",
                "if \"%~1\"==\"--output\" (\r\n",
                "  set \"OUT=%~2\"\r\n",
                "  echo %~2>>\"{args_log}\"\r\n",
                ")\r\n",
                "shift\r\n",
                "goto loop\r\n",
                ":done\r\n",
                "more > \"{stdin_log}\"\r\n",
                "if {exit_code} EQU 0 (\r\n",
                "  copy /Y \"{template}\" \"%OUT%\\pre-write-advisory.latest.json\" >NUL\r\n",
                "  copy /Y \"{template}\" \"%OUT%\\pre-write-advisory.JS-1.json\" >NUL\r\n",
                "  echo ## js pre-write\r\n",
                "  echo [js-pre-write] diagnostic 1>&2\r\n",
                ")\r\n",
                "exit /b {exit_code}\r\n"
            ),
            args_log = path_string(&dir.join("js-args.log")),
            stdin_log = path_string(&dir.join("js-intent.stdin")),
            template = path_string(&template),
            exit_code = exit_code,
        ),
    )?;
    Ok(FakeJsPreWrite {
        command: path_string(&script),
        scripts_dir,
    })
}

#[cfg(not(windows))]
fn write_fake_js_pre_write(dir: &Path, out: &Path, exit_code: i32) -> Result<FakeJsPreWrite> {
    use std::os::unix::fs::PermissionsExt;

    let scripts_dir = dir.join(format!("js-scripts-{exit_code}"));
    fs::create_dir_all(&scripts_dir)?;
    fs::write(scripts_dir.join("pre-write.mjs"), "// fake path only\n")?;
    let script = dir.join(format!("fake-js-pre-write-{exit_code}"));
    let template = dir.join("js-advisory-template.json");
    write_js_advisory_template(&template, out)?;
    fs::write(
        &script,
        format!(
            r#"#!/bin/sh
printf '%s\n' "$@" > '{args_log}'
cat > '{stdin_log}'
out=''
while [ "$#" -gt 0 ]; do
  if [ "$1" = '--output' ]; then
    shift
    out="$1"
  fi
  shift || true
done
if [ {exit_code} -eq 0 ]; then
  cp '{template}' "$out/pre-write-advisory.latest.json"
  cp '{template}' "$out/pre-write-advisory.JS-1.json"
  printf '%s\n' '## js pre-write'
  printf '%s\n' '[js-pre-write] diagnostic' >&2
fi
exit {exit_code}
"#,
            args_log = path_string(&dir.join("js-args.log")),
            stdin_log = path_string(&dir.join("js-intent.stdin")),
            template = path_string(&template),
        ),
    )?;
    let mut permissions = fs::metadata(&script)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script, permissions)?;
    Ok(FakeJsPreWrite {
        command: path_string(&script),
        scripts_dir,
    })
}

fn write_js_advisory_template(path: &Path, out: &Path) -> Result<()> {
    fs::write(
        path,
        serde_json::to_string_pretty(&json!({
            "invocationId": "JS-1",
            "artifactPaths": {
                "invocationSpecific": path_string(&out.join("pre-write-advisory.JS-1.json")),
                "latest": path_string(&out.join("pre-write-advisory.latest.json"))
            },
            "evidenceAvailability": {
                "status": "available",
                "producer": "pre-write.mjs"
            }
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
