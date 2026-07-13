use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use std::process::Command;

use lumin_audit_core::pre_write_routing::{resolve_pre_write_route, PreWriteRoutingRequest};

fn request(
    requested_engine: &str,
    intent_flag: &str,
    intent_text: &str,
) -> Result<PreWriteRoutingRequest> {
    Ok(serde_json::from_value(json!({
        "schemaVersion": "lumin-pre-write-routing-request.v1",
        "requestedEngine": requested_engine,
        "intentFlag": intent_flag,
        "intentText": intent_text,
    }))?)
}

#[test]
fn auto_routes_rust_intent_to_rust_and_strips_route_only_language() -> Result<()> {
    let result = resolve_pre_write_route(request(
        "auto",
        "-",
        "{\n  \"language\": \"rust\",\n  \"names\": [\"Thing\"]\n}\n",
    )?)?;

    assert_eq!(result.engine, "rust");
    assert_eq!(result.child_intent_flag, "-");
    assert_eq!(
        result.child_intent_input.as_deref(),
        Some("{\n  \"names\": [\n    \"Thing\"\n  ]\n}\n")
    );
    assert_eq!(result.engine_selection.requested, "auto");
    assert_eq!(result.engine_selection.selected, "rust");
    assert_eq!(result.engine_selection.reason, "intent-language");
    assert_eq!(
        result.engine_selection.intent_language.as_deref(),
        Some("rust")
    );
    Ok(())
}

#[test]
fn auto_defaults_absent_language_to_js_without_rewriting_intent() -> Result<()> {
    let intent = "{\n  \"names\": [\"Thing\"]\n}\n";
    let result = resolve_pre_write_route(request("auto", "-", intent)?)?;

    assert_eq!(result.engine, "js");
    assert_eq!(result.child_intent_flag, "-");
    assert_eq!(result.child_intent_input.as_deref(), Some(intent));
    assert_eq!(
        result.engine_selection.reason,
        "intent-language-absent-default-js"
    );
    assert_eq!(result.engine_selection.intent_language, None);
    Ok(())
}

#[test]
fn explicit_js_file_intent_preserves_file_flag_without_stdin_payload() -> Result<()> {
    let result = resolve_pre_write_route(request(
        "js",
        "C:/repo/intent.json",
        "{\n  \"language\": \"js-ts\",\n  \"names\": [\"Thing\"]\n}\n",
    )?)?;

    assert_eq!(result.engine, "js");
    assert_eq!(result.child_intent_flag, "C:/repo/intent.json");
    assert_eq!(result.child_intent_input, None);
    assert_eq!(result.engine_selection.reason, "explicit-cli");
    assert_eq!(
        result.engine_selection.intent_language.as_deref(),
        Some("js-ts")
    );
    Ok(())
}

#[test]
fn explicit_engine_mismatch_hard_stops() -> Result<()> {
    let result = resolve_pre_write_route(request(
        "rust",
        "-",
        "{\n  \"language\": \"js-ts\",\n  \"names\": [\"Thing\"]\n}\n",
    )?);
    let error = match result {
        Ok(route) => anyhow::bail!("expected hard-stop, got {route:?}"),
        Err(error) => error,
    };

    assert!(error
        .to_string()
        .contains("intent.language \"js-ts\" is owned by native lumin-audit-core pre-write"));
    Ok(())
}

#[test]
fn cli_pre_write_route_emits_typed_json_and_rejects_bad_shape() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let input_path = temp.path().join("route.json");
    fs::write(
        &input_path,
        serde_json::to_string(&json!({
            "schemaVersion": "lumin-pre-write-routing-request.v1",
            "requestedEngine": "auto",
            "intentFlag": "-",
            "intentText": "{\n  \"language\": \"rust\",\n  \"names\": [\"Thing\"]\n}\n"
        }))?,
    )?;

    let output = Command::new(audit_core_bin())
        .arg("pre-write-route")
        .arg("--input")
        .arg(&input_path)
        .output()?;
    assert!(
        output.status.success(),
        "stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(result["engine"], "rust");
    assert_eq!(result["engineSelection"]["selected"], "rust");

    fs::write(&input_path, r#"{"schemaVersion":"wrong"}"#)?;
    let output = Command::new(audit_core_bin())
        .arg("pre-write-route")
        .arg("--input")
        .arg(&input_path)
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("pre-write-route"));
    Ok(())
}

fn audit_core_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lumin-audit-core")
}
