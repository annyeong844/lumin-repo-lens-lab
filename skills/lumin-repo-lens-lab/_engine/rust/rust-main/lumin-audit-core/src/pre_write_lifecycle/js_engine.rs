use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

use super::advisory::{
    advisory_latest_path, advisory_specific_path, path_string, read_required_json,
    remove_file_if_present, required_invocation_id, validate_matching_json_artifacts,
};
use super::child::{nonempty, run_js_pre_write_child, ChildStdio};
use super::protocol::{
    JsPreWriteLifecycleRequest, PreWriteBlock, PreWriteFailureKind, PreWriteLifecycleResult,
    JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION, PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
};

pub(super) fn execute(
    request: JsPreWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> Result<PreWriteLifecycleResult> {
    validate_request(&request)?;
    let latest_advisory_path = advisory_latest_path(&request.output);
    if let Err(error) = remove_file_if_present(&latest_advisory_path) {
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::OutputCleanupFailed,
            format!(
                "execute-js-pre-write: failed to clear stale {}: {error}",
                latest_advisory_path.display()
            ),
            1,
            None,
            None,
            None,
        ));
    }
    let child = run_js_pre_write_child(&request, child_stdio);
    if !child.status_success {
        let reason = advisory_failure_reason(
            &latest_advisory_path,
            None,
            format!("pre-write.mjs exited non-zero: {}", child.reason),
        );
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::ChildFailed,
            reason,
            child.exit_code.unwrap_or(1),
            child.exit_code,
            nonempty(child.stdout),
            nonempty(child.stderr),
        ));
    }

    let advisory = match read_advisory(&latest_advisory_path) {
        Ok(advisory) => advisory,
        Err(error) => {
            let reason = advisory_failure_reason(&latest_advisory_path, None, error.to_string());
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::AdvisoryArtifactInvalid,
                reason,
                1,
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };
    let advisory_invocation_id = match required_invocation_id(&advisory, "js pre-write advisory") {
        Ok(invocation_id) => invocation_id.to_string(),
        Err(error) => {
            let reason = advisory_failure_reason(&latest_advisory_path, None, error.to_string());
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::AdvisoryArtifactInvalid,
                reason,
                1,
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };
    let advisory_path = advisory_specific_path(&request.output, &advisory_invocation_id);
    if let Err(error) = validate_matching_json_artifacts(
        &latest_advisory_path,
        &advisory_path,
        "js pre-write advisory",
    ) {
        let reason = advisory_failure_reason(
            &latest_advisory_path,
            Some(&advisory_path),
            error.to_string(),
        );
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::AdvisoryArtifactInvalid,
            reason,
            1,
            child.exit_code,
            nonempty(child.stdout),
            nonempty(child.stderr),
        ));
    }
    let evidence_availability = match advisory
        .get("evidenceAvailability")
        .filter(|value| value.is_object())
        .cloned()
    {
        Some(evidence_availability) => evidence_availability,
        None => {
            let reason = advisory_failure_reason(
                &latest_advisory_path,
                Some(&advisory_path),
                "js pre-write advisory.evidenceAvailability must be an object".to_string(),
            );
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::AdvisoryArtifactInvalid,
                reason,
                1,
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };
    let (rust_evidence_path, any_inventory_path) = match evidence_paths(
        &advisory,
        &request.output,
        &advisory_invocation_id,
        request.no_fresh_audit,
    ) {
        Ok(paths) => paths,
        Err(error) => {
            let reason = advisory_failure_reason(
                &latest_advisory_path,
                Some(&advisory_path),
                error.to_string(),
            );
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::AdvisoryArtifactInvalid,
                reason,
                1,
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };

    Ok(PreWriteLifecycleResult {
        schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block: PreWriteBlock {
            requested: true,
            ran: true,
            execution_owner: "lumin-audit-core",
            engine: "js",
            language: "js-ts",
            producer: "pre-write.mjs",
            engine_selection: request.engine_selection,
            advisory_path: Some(path_string(&advisory_path)),
            latest_advisory_path: Some(path_string(&latest_advisory_path)),
            advisory_invocation_id: Some(advisory_invocation_id),
            evidence_availability: Some(evidence_availability),
            rust_evidence_path,
            any_inventory_path,
            rust_native_artifact_path: None,
            rust_native_latest_path: None,
            source_commit: None,
            analyzer_invocation: None,
            failure_kind: None,
            child_exit_code: None,
            reason: None,
        },
        exit_code: 0,
        stdout: nonempty(child.stdout),
        stderr: nonempty(child.stderr),
    })
}

fn validate_request(request: &JsPreWriteLifecycleRequest) -> Result<()> {
    if request.schema_version != JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-js-pre-write: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty_path("root", &request.root)?;
    validate_nonempty_path("output", &request.output)?;
    validate_nonempty_path("scriptsDir", &request.scripts_dir)?;
    if request.node_executable.trim().is_empty() {
        bail!("execute-js-pre-write: nodeExecutable must be a non-empty string");
    }
    if request.child_intent_flag.trim().is_empty() {
        bail!("execute-js-pre-write: childIntentFlag must be a non-empty string");
    }
    Ok(())
}

fn validate_nonempty_path(field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("execute-js-pre-write: {field} must be provided");
    }
    Ok(())
}

fn read_advisory(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "js pre-write advisory parse failed: failed to read {}",
            path.display()
        )
    })?;
    let advisory = serde_json::from_str::<Value>(&text).with_context(|| {
        format!(
            "js pre-write advisory parse failed: invalid JSON in {}",
            path.display()
        )
    })?;
    required_invocation_id(&advisory, "js pre-write advisory")?;
    Ok(advisory)
}

fn evidence_paths(
    advisory: &Value,
    output: &Path,
    invocation_id: &str,
    no_fresh_audit: bool,
) -> Result<(Option<String>, Option<String>)> {
    if no_fresh_audit {
        return Ok((None, None));
    }
    let expected_evidence = format!("pre-write-evidence.{invocation_id}.json");
    let expected_inventory = format!("any-inventory.pre.{invocation_id}.json");
    let rust_evidence_path = advisory
        .pointer("/preWrite/rustEvidencePath")
        .and_then(Value::as_str)
        .context("js pre-write advisory.preWrite.rustEvidencePath must be a string")?;
    let any_inventory_path = advisory
        .pointer("/preWrite/anyInventoryPath")
        .and_then(Value::as_str)
        .context("js pre-write advisory.preWrite.anyInventoryPath must be a string")?;
    if rust_evidence_path != expected_evidence || any_inventory_path != expected_inventory {
        bail!("js pre-write advisory evidence paths do not match invocationId {invocation_id}");
    }

    let evidence = read_required_json(&output.join(&expected_evidence), "js pre-write evidence")?;
    if evidence.get("schemaVersion").and_then(Value::as_str)
        != Some("lumin-js-ts-pre-write-evidence-response.v1")
        || evidence
            .pointer("/anyInventory/meta/artifact")
            .and_then(Value::as_str)
            != Some(expected_inventory.as_str())
    {
        bail!("js pre-write evidence contract is invalid");
    }
    let inventory = read_required_json(
        &output.join(&expected_inventory),
        "js pre-write any inventory",
    )?;
    if inventory.pointer("/meta/artifact").and_then(Value::as_str)
        != Some(expected_inventory.as_str())
        || inventory
            .pointer("/meta/supports/typeEscapes")
            .and_then(Value::as_bool)
            != Some(true)
        || !inventory.get("typeEscapes").is_some_and(Value::is_array)
    {
        bail!("js pre-write any inventory contract is invalid");
    }

    Ok((Some(expected_evidence), Some(expected_inventory)))
}

fn advisory_failure_reason(latest: &Path, specific: Option<&Path>, reason: String) -> String {
    let mut cleanup_errors = Vec::new();
    for path in std::iter::once(latest).chain(specific) {
        if let Err(error) = remove_file_if_present(path) {
            cleanup_errors.push(format!("{}: {error}", path.display()));
        }
    }
    if cleanup_errors.is_empty() {
        reason
    } else {
        format!(
            "{reason}; invalid advisory cleanup failed: {}",
            cleanup_errors.join(", ")
        )
    }
}

fn failure_result(
    request: &JsPreWriteLifecycleRequest,
    failure_kind: PreWriteFailureKind,
    reason: String,
    exit_code: i32,
    child_exit_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
) -> PreWriteLifecycleResult {
    PreWriteLifecycleResult {
        schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block: PreWriteBlock {
            requested: true,
            ran: false,
            execution_owner: "lumin-audit-core",
            engine: "js",
            language: "js-ts",
            producer: "pre-write.mjs",
            engine_selection: request.engine_selection.clone(),
            advisory_path: None,
            latest_advisory_path: None,
            advisory_invocation_id: None,
            evidence_availability: Some(serde_json::json!({
                "status": "unavailable",
                "producer": "pre-write.mjs",
                "failureKind": failure_kind,
                "reason": reason,
            })),
            rust_evidence_path: None,
            any_inventory_path: None,
            rust_native_artifact_path: None,
            rust_native_latest_path: None,
            source_commit: None,
            analyzer_invocation: None,
            failure_kind: Some(failure_kind),
            child_exit_code,
            reason: Some(reason),
        },
        exit_code,
        stdout,
        stderr,
    }
}
