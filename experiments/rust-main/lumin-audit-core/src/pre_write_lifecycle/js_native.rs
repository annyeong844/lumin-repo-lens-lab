mod cues;
mod lookup;
mod render;

use anyhow::{bail, Context, Result};
use lumin_rust_common::{atomic_write_json_pretty, sha256_text};
use serde_json::{json, Map, Value};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::js_ts_pre_write::{
    collect_js_ts_pre_write_evidence, JsTsPreWriteEvidenceRequest,
    JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION,
};
use crate::pre_write_intent::normalize_js_ts_intent_text;

use super::advisory::{
    advisory_latest_path, advisory_specific_path, path_string, remove_file_if_present,
    write_advisory,
};
use super::child::ChildStdio;
use super::protocol::{
    JsPreWriteLifecycleRequest, PreWriteBlock, PreWriteFailureKind, PreWriteLifecycleResult,
    JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION, PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
};

const PRODUCER: &str = "lumin-audit-core js-ts-pre-write";

pub(super) fn execute(
    request: JsPreWriteLifecycleRequest,
    stdio: ChildStdio,
) -> Result<PreWriteLifecycleResult> {
    validate_request(&request)?;
    let invocation_id = request
        .advisory_invocation_id
        .as_deref()
        .context("execute-js-pre-write: invocationId must be provided")?;
    clear_current_outputs(&request, invocation_id).with_context(|| {
        format!("execute-js-pre-write: failed to clear current-run outputs for {invocation_id}")
    })?;

    let normalized = normalize_js_ts_intent_text(
        request
            .intent_input
            .as_deref()
            .context("execute-js-pre-write: intentInput must be provided")?,
    )?;
    let intent = normalized.value();
    let evidence_artifact = format!("pre-write-evidence.{invocation_id}.json");
    let inventory_artifact = format!("any-inventory.pre.{invocation_id}.json");
    let generated = request
        .generated
        .as_deref()
        .context("execute-js-pre-write: generated must be provided")?;
    let evidence = match collect_js_ts_pre_write_evidence(
        JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: request.root.clone(),
            evidence_artifact: evidence_artifact.clone(),
            any_inventory_artifact: inventory_artifact.clone(),
            generated: generated.to_string(),
            include_tests: request.include_tests,
            excludes: request.excludes.clone(),
            dependency_roots: intent
                .get("dependencies")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter_map(package_root)
                .map(str::to_string)
                .collect(),
            shape_type_literals: intent
                .get("shapes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|shape| shape.get("typeLiteral").and_then(Value::as_str))
                .map(str::to_string)
                .collect(),
            discover_files: true,
            files: Vec::new(),
            incremental: request.incremental.clone(),
        },
        request.host_evidence_transport.as_ref(),
        &request.output,
        invocation_id,
    ) {
        Ok(evidence) => evidence,
        Err(error) => {
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::EvidenceCollectionFailed,
                format!("native JS/TS pre-write evidence failed: {error:#}"),
            ));
        }
    };

    if let Err(error) = write_evidence(
        &request.output,
        &evidence_artifact,
        &inventory_artifact,
        &evidence,
    ) {
        cleanup_evidence(&request.output, &evidence_artifact, &inventory_artifact);
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::OutputWriteFailed,
            format!("native JS/TS pre-write evidence write failed: {error:#}"),
        ));
    }

    let mut failures = Vec::new();
    let lookup_projection = lookup::project(&request.root, intent, &evidence, &mut failures)?;
    let cue_projection = cues::project(&lookup_projection.lookups);
    let drift = lookup::compute_drift(&lookup_projection.lookups);
    let evidence_availability = evidence_availability(intent, &request.output, &evidence_artifact);
    let intent_hash = hash_intent(intent)?;
    let file_inventory = json!({
        "status": "available",
        "pathMode": "repo-relative",
        "fileCount": evidence.get("files").and_then(Value::as_array).map_or(0, Vec::len),
        "files": evidence.get("files").cloned().unwrap_or(json!([])),
    });
    let advisory = json!({
        "invocationId": invocation_id,
        "intentHash": intent_hash,
        "artifactPaths": {
            "invocationSpecific": path_string(&advisory_specific_path(&request.output, invocation_id)),
            "latest": path_string(&advisory_latest_path(&request.output)),
        },
        "scanRange": {
            "root": path_string(&request.root),
            "output": path_string(&request.output),
            "includeTests": request.include_tests,
            "production": request.production,
            "excludes": request.excludes,
        },
        "intent": intent,
        "intentWarnings": normalized.warnings(),
        "evidenceAvailability": evidence_availability,
        "lookups": lookup_projection.lookups,
        "cueCards": cue_projection.cue_cards,
        "suppressedCues": cue_projection.suppressed_cues,
        "unavailableEvidence": cue_projection.unavailable_evidence,
        "cuePolicy": cue_projection.cue_policy,
        "boundaryChecks": [],
        "drift": drift,
        "capabilities": evidence.pointer("/symbols/meta/supports").cloned().unwrap_or(Value::Null),
        "failures": failures,
        "preWrite": {
            "rustEvidencePath": evidence_artifact,
            "anyInventoryPath": inventory_artifact,
            "rustEvidence": {
                "schemaVersion": evidence.get("schemaVersion").cloned().unwrap_or(Value::Null),
                "summary": evidence.get("summary").cloned().unwrap_or(Value::Null),
            },
            "fileInventory": file_inventory,
        },
    });

    let written = match write_advisory(&request.output, &advisory) {
        Ok(written) => written,
        Err(error) => {
            cleanup_evidence(&request.output, &evidence_artifact, &inventory_artifact);
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::OutputWriteFailed,
                format!("native JS/TS pre-write advisory write failed: {error:#}"),
            ));
        }
    };
    let stdout = match stdio {
        ChildStdio::Capture => Some(format!("{}\n", render::markdown(&advisory))),
        ChildStdio::Inherit => {
            let handoff = format!("{}\n", render::handoff_markdown(&advisory));
            io::stdout()
                .write_all(handoff.as_bytes())
                .context("execute-js-pre-write: failed to write advisory handoff")?;
            None
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
            producer: PRODUCER,
            engine_selection: request.engine_selection,
            advisory_path: Some(path_string(&written.specific_path)),
            latest_advisory_path: Some(path_string(&written.latest_path)),
            advisory_invocation_id: Some(invocation_id.to_string()),
            evidence_availability: Some(evidence_availability),
            rust_evidence_path: Some(evidence_artifact),
            any_inventory_path: Some(inventory_artifact),
            rust_native_artifact_path: None,
            rust_native_latest_path: None,
            source_commit: None,
            analyzer_invocation: None,
            failure_kind: None,
            child_exit_code: None,
            reason: None,
        },
        exit_code: 0,
        stdout,
        stderr: None,
    })
}

fn validate_request(request: &JsPreWriteLifecycleRequest) -> Result<()> {
    if request.schema_version != JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-js-pre-write: unsupported native schemaVersion '{}'",
            request.schema_version
        );
    }
    if request.root.as_os_str().is_empty() || !request.root.is_dir() {
        bail!("execute-js-pre-write: root must be an existing directory");
    }
    if request.output.as_os_str().is_empty() {
        bail!("execute-js-pre-write: output must be provided");
    }
    let invocation_id = request
        .advisory_invocation_id
        .as_deref()
        .filter(|value| !value.is_empty())
        .context("execute-js-pre-write: invocationId must be a non-empty string")?;
    if invocation_id.contains(['/', '\\']) || invocation_id.contains("..") {
        bail!("execute-js-pre-write: invocationId must be filename-safe");
    }
    if request.intent_input.as_deref().is_none_or(str::is_empty) {
        bail!("execute-js-pre-write: intentInput must be a non-empty string");
    }
    if request.generated.as_deref().is_none_or(str::is_empty) {
        bail!("execute-js-pre-write: generated must be a non-empty string");
    }
    if request.production == request.include_tests {
        bail!("execute-js-pre-write: production must be the inverse of includeTests");
    }
    Ok(())
}

fn write_evidence(
    output: &Path,
    evidence_name: &str,
    inventory_name: &str,
    evidence: &Value,
) -> Result<()> {
    fs::create_dir_all(output).with_context(|| format!("failed to create {}", output.display()))?;
    let inventory = evidence
        .get("anyInventory")
        .context("native JS/TS evidence.anyInventory must be present")?;
    atomic_write_json_pretty(&output.join(inventory_name), inventory)?;
    atomic_write_json_pretty(&output.join(evidence_name), evidence)?;
    atomic_write_json_pretty(&output.join("pre-write-evidence.latest.json"), evidence)?;
    Ok(())
}

fn clear_current_outputs(request: &JsPreWriteLifecycleRequest, invocation_id: &str) -> Result<()> {
    for path in [
        advisory_latest_path(&request.output),
        advisory_specific_path(&request.output, invocation_id),
        request.output.join("pre-write-evidence.latest.json"),
        request
            .output
            .join(format!("pre-write-evidence.{invocation_id}.json")),
        request
            .output
            .join(format!("any-inventory.pre.{invocation_id}.json")),
    ] {
        remove_file_if_present(&path)
            .with_context(|| format!("failed to remove {}", path.display()))?;
    }
    Ok(())
}

fn cleanup_evidence(output: &Path, evidence_name: &str, inventory_name: &str) {
    for path in [
        output.join(evidence_name),
        output.join(inventory_name),
        output.join("pre-write-evidence.latest.json"),
    ] {
        let _ = remove_file_if_present(&path);
    }
}

fn evidence_availability(intent: &Value, output: &Path, evidence_artifact: &str) -> Value {
    let mut artifacts = Vec::new();
    let names = array_nonempty(intent, "names");
    let files = array_nonempty(intent, "files");
    let dependencies = array_nonempty(intent, "dependencies");
    if names || files || dependencies {
        let mut required = Vec::new();
        if names {
            required.push("names");
        }
        if files {
            required.push("files");
        }
        if dependencies {
            required.push("dependencies");
        }
        artifacts.push(json!({
            "artifact": evidence_artifact,
            "status": "available",
            "requiredFor": required,
            "canGroundEvidence": true,
        }));
    }
    let shapes = intent
        .get("shapes")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    if shapes.iter().any(|shape| {
        shape
            .get("typeLiteral")
            .and_then(Value::as_str)
            .is_some_and(|literal| literal.trim_start().starts_with(['(', '<']))
    }) {
        artifacts.push(json!({
            "artifact": format!("{evidence_artifact}#functionSignatures"),
            "status": "available",
            "requiredFor": ["function-signature"],
            "canGroundEvidence": true,
        }));
    }
    if !shapes.is_empty() {
        artifacts.push(json!({
            "artifact": format!("{evidence_artifact}#shapeIndex"),
            "status": "available",
            "requiredFor": ["shapes"],
            "canGroundEvidence": true,
        }));
    }
    if array_nonempty(intent, "refactorSources") {
        artifacts.push(json!({
            "artifact": format!("{evidence_artifact}#inlinePatterns"),
            "status": "available",
            "requiredFor": ["refactorSources"],
            "canGroundEvidence": true,
        }));
    }
    let available = artifacts
        .iter()
        .filter(|entry| entry.get("status").and_then(Value::as_str) == Some("available"))
        .count();
    let status = if artifacts.is_empty() {
        "not-needed"
    } else if available == artifacts.len() {
        "available"
    } else if available == 0 {
        "missing"
    } else {
        "partial"
    };
    json!({
        "status": status,
        "freshAudit": true,
        "output": path_string(output),
        "artifacts": artifacts,
        "guidance": "Pre-write grounds cues only from current-run Rust evidence produced by this invocation.",
    })
}

fn hash_intent(intent: &Value) -> Result<String> {
    let text = serde_json::to_string(&sorted_json_value(intent))?;
    let digest = sha256_text(&text);
    Ok(digest
        .strip_prefix("sha256:")
        .unwrap_or(&digest)
        .to_string())
}

fn sorted_json_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(sorted_json_value).collect()),
        Value::Object(values) => {
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort();
            let mut result = Map::new();
            for key in keys {
                result.insert(key.clone(), sorted_json_value(&values[key]));
            }
            Value::Object(result)
        }
        other => other.clone(),
    }
}

fn package_root(specifier: &str) -> Option<&str> {
    if specifier.is_empty() || specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }
    if specifier.starts_with('@') {
        let parts = specifier.split('/').collect::<Vec<_>>();
        if parts.len() < 2 || parts[1].is_empty() {
            return None;
        }
        return Some(&specifier[..parts[0].len() + parts[1].len() + 1]);
    }
    specifier.split('/').next()
}

fn array_nonempty(value: &Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(Value::as_array)
        .is_some_and(|values| !values.is_empty())
}

fn failure_result(
    request: &JsPreWriteLifecycleRequest,
    failure_kind: PreWriteFailureKind,
    reason: String,
) -> PreWriteLifecycleResult {
    PreWriteLifecycleResult {
        schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block: PreWriteBlock {
            requested: true,
            ran: false,
            execution_owner: "lumin-audit-core",
            engine: "js",
            language: "js-ts",
            producer: PRODUCER,
            engine_selection: request.engine_selection.clone(),
            advisory_path: None,
            latest_advisory_path: None,
            advisory_invocation_id: request.advisory_invocation_id.clone(),
            evidence_availability: Some(json!({
                "status": "unavailable",
                "producer": PRODUCER,
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
            child_exit_code: None,
            reason: Some(reason),
        },
        exit_code: 1,
        stdout: None,
        stderr: None,
    }
}
