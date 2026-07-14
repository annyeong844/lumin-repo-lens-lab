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

fn js_request(root: &Path, out: &Path) -> Value {
    json!({
        "schemaVersion": "lumin-js-pre-write-lifecycle-request.v3",
        "root": path_string(root),
        "output": path_string(out),
        "invocationId": "JS-1",
        "intentInput": json!({
            "names": ["Thing"],
            "files": ["src/thing.ts"],
            "shapes": [{ "typeLiteral": "(value: string) => number" }],
            "refactorSources": [{ "file": "src/thing.ts" }],
            "plannedTypeEscapes": [],
        }).to_string(),
        "engineSelection": {
            "requested": "auto",
            "selected": "js",
            "reason": "intent-language-absent-default-js"
        },
        "generated": "2026-07-13T00:00:00.000Z",
        "includeTests": false,
        "production": true,
        "excludes": ["dist"],
        "incremental": { "enabled": false },
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
        .starts_with("lumin-rust-analyzer pre-write exited non-zero:"));
    assert!(!out.join("pre-write-advisory.latest.json").exists());
    Ok(())
}

#[test]
fn rust_pre_write_rejects_malformed_native_artifact_without_reusing_stale_outputs() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    fs::create_dir_all(&out)?;
    let fake = write_fake_analyzer(temp.path(), 0)?;
    fs::write(temp.path().join("rust-native-template.json"), "{}\n")?;
    for stale in [
        out.join("rust-pre-write-artifact.INV-1.json"),
        out.join("rust-pre-write-artifact.latest.json"),
        out.join("pre-write-advisory.INV-1.json"),
        out.join("pre-write-advisory.latest.json"),
    ] {
        fs::write(stale, "{\"stale\":true}\n")?;
    }

    let result =
        execute_rust_pre_write_lifecycle(parse_request(request(temp.path(), &out, &fake))?)?;

    assert_eq!(result.exit_code, 1, "result={result:#?}");
    assert!(!result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["failureKind"],
        "native-artifact-invalid"
    );
    assert!(result
        .block
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains("invalid shape or JSON")));
    assert!(!out.join("rust-pre-write-artifact.latest.json").exists());
    assert!(!out.join("pre-write-advisory.INV-1.json").exists());
    assert!(!out.join("pre-write-advisory.latest.json").exists());
    Ok(())
}

#[test]
fn js_pre_write_builds_current_native_evidence_and_advisory_without_node() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    write_js_native_fixture(temp.path())?;

    let result = execute_js_pre_write_lifecycle(parse_js_request(js_request(temp.path(), &out))?)?;

    assert_eq!(result.exit_code, 0, "result={result:#?}");
    assert!(result.block.ran);
    assert_eq!(result.block.execution_owner, "lumin-audit-core");
    assert_eq!(result.block.engine, "js");
    assert_eq!(result.block.language, "js-ts");
    assert_eq!(result.block.producer, "lumin-audit-core js-ts-pre-write");
    assert_eq!(result.block.advisory_invocation_id.as_deref(), Some("JS-1"));
    assert_eq!(
        result.block.rust_evidence_path.as_deref(),
        Some("pre-write-evidence.JS-1.json")
    );
    assert_eq!(
        result.block.any_inventory_path.as_deref(),
        Some("any-inventory.pre.JS-1.json")
    );
    assert!(result
        .stdout
        .as_deref()
        .unwrap_or_default()
        .contains("## pre-write advisory"));
    assert!(result.stderr.is_none());

    let advisory_specific = fs::read_to_string(out.join("pre-write-advisory.JS-1.json"))?;
    let advisory_latest = fs::read_to_string(out.join("pre-write-advisory.latest.json"))?;
    assert_eq!(advisory_specific, advisory_latest);
    let advisory: Value = serde_json::from_str(&advisory_latest)?;
    assert_eq!(advisory["invocationId"], "JS-1");
    assert!(advisory["lookups"]
        .as_array()
        .is_some_and(|lookups| lookups.iter().any(|lookup| {
            lookup.get("result").and_then(Value::as_str) == Some("SIGNATURE_MATCH")
        })));
    assert!(advisory["lookups"]
        .as_array()
        .is_some_and(|lookups| lookups.iter().any(|lookup| {
            lookup.get("result").and_then(Value::as_str) == Some("INLINE_PATTERN_MATCH")
        })));

    let evidence: Value = serde_json::from_str(&fs::read_to_string(
        out.join("pre-write-evidence.JS-1.json"),
    )?)?;
    assert_eq!(evidence["functionSignatures"]["meta"]["complete"], true);
    assert_eq!(evidence["inlinePatterns"]["meta"]["groupCount"], 1);
    assert_eq!(
        evidence["shapeIntentNormalizations"][0]["shapeKind"],
        "function-signature"
    );
    Ok(())
}

fn write_js_native_fixture(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("package.json"),
        serde_json::to_string_pretty(&json!({ "name": "native-pre-write-fixture" }))?,
    )?;
    fs::write(
        root.join("src").join("thing.ts"),
        concat!(
            "export function Thing(value: string): number {\n",
            "  try { return value.length; } catch { cleanup(); }\n",
            "}\n",
        ),
    )?;
    for file in ["second.ts", "third.ts"] {
        fs::write(
            root.join("src").join(file),
            "export function work(): void { try { perform(); } catch { cleanup(); } }\n",
        )?;
    }
    Ok(())
}

#[test]
fn js_pre_write_output_failure_clears_current_advisory_claims() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let output_file = temp.path().join("not-a-directory");
    write_js_native_fixture(temp.path())?;
    fs::write(&output_file, "occupied")?;

    let result =
        execute_js_pre_write_lifecycle(parse_js_request(js_request(temp.path(), &output_file))?)?;

    assert_eq!(result.exit_code, 1, "result={result:#?}");
    assert!(!result.block.ran);
    assert_eq!(
        serde_json::to_value(&result.block)?["failureKind"],
        "output-write-failed"
    );
    assert!(result.block.advisory_path.is_none());
    assert!(result.block.latest_advisory_path.is_none());
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
fn cli_execute_js_pre_write_runs_native_and_writes_clean_result_file() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let out = temp.path().join("out");
    write_js_native_fixture(temp.path())?;
    let input_path = temp.path().join("js-request.json");
    let result_path = temp.path().join("js-result.json");
    fs::write(
        &input_path,
        serde_json::to_string(&js_request(temp.path(), &out))?,
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
    assert!(String::from_utf8_lossy(&output.stdout).contains("## pre-write advisory"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("cueCards=`"));
    assert!(!String::from_utf8_lossy(&output.stdout).contains("### Agent review cues"));
    assert!(output.stderr.is_empty());
    let result: Value = serde_json::from_str(&fs::read_to_string(result_path)?)?;
    assert_eq!(result["block"]["ran"], true);
    assert_eq!(result["block"]["engine"], "js");
    assert_eq!(
        result["block"]["producer"],
        "lumin-audit-core js-ts-pre-write"
    );
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
            "policyVersion": "prewrite-token-policy-v1",
            "intent": {
                "names": ["Thing"],
                "shapes": [],
                "files": [],
                "dependencies": [],
                "plannedTypeEscapes": []
            },
            "intentWarnings": [],
            "meta": { "producer": "lumin-rust-analyzer" },
            "coverage": {
                "names": "ran",
                "shapes": "not-requested",
                "files": "not-requested",
                "dependencies": "not-requested",
                "inlinePatterns": "not-requested",
                "plannedTypeEscapes": "ran"
            },
            "lookups": [],
            "shapeLookups": [],
            "fileLookups": [],
            "dependencyLookups": [],
            "inlinePatternLookups": [],
            "cueCards": [],
            "suppressedCues": [],
            "unavailableEvidence": []
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
