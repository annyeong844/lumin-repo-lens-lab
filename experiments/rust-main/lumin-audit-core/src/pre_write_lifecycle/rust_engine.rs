use anyhow::{bail, Context, Result};
use lumin_rust_common::sha256_text;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

use crate::source_commit::git_head_commit_or_unknown;

use super::advisory::{
    advisory_latest_path, advisory_specific_path, path_string, remove_file_if_present,
    write_advisory,
};
use super::child::{nonempty, run_rust_pre_write_child, ChildStdio};
use super::protocol::{
    AnalyzerInvocationBlock, PreWriteBlock, PreWriteFailureKind, PreWriteLifecycleRequest,
    PreWriteLifecycleResult, RustPreWriteArtifact, RustPreWriteCoverage,
    PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION, PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
    RUST_PRE_WRITE_ARTIFACT_SCHEMA_VERSION, RUST_PRE_WRITE_POLICY_VERSION, RUST_PRE_WRITE_PRODUCER,
};

pub(super) fn execute(
    request: PreWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> Result<PreWriteLifecycleResult> {
    validate_request(&request)?;
    let source_commit = effective_source_commit(&request);
    if let Err(error) = clear_outputs(&request) {
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::OutputCleanupFailed,
            error.to_string(),
            1,
            None,
            None,
            None,
        ));
    }
    let child = run_rust_pre_write_child(&request, &source_commit, child_stdio);
    if !child.status_success {
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::ChildFailed,
            format!(
                "lumin-rust-analyzer pre-write exited non-zero: {}",
                child.reason
            ),
            child.exit_code.unwrap_or(1),
            child.exit_code,
            nonempty(child.stdout),
            nonempty(child.stderr),
        ));
    }

    let rust_artifact = match read_native_artifact(&request.rust_native_artifact_path) {
        Ok(artifact) => artifact,
        Err(error) => {
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::NativeArtifactInvalid,
                error.to_string(),
                1,
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };

    let advisory = match build_advisory(&request, &rust_artifact, &source_commit) {
        Ok(advisory) => advisory,
        Err(error) => {
            let reason = output_failure_reason(&request, error.to_string());
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::OutputWriteFailed,
                reason,
                1,
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };
    if let Err(error) = copy_native_latest(&request) {
        let reason = output_failure_reason(&request, error.to_string());
        return Ok(failure_result(
            &request,
            PreWriteFailureKind::OutputWriteFailed,
            reason,
            1,
            child.exit_code,
            nonempty(child.stdout),
            nonempty(child.stderr),
        ));
    }
    let written = match write_advisory(&request.output, &advisory) {
        Ok(written) => written,
        Err(error) => {
            let reason = output_failure_reason(&request, error.to_string());
            return Ok(failure_result(
                &request,
                PreWriteFailureKind::OutputWriteFailed,
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
            engine: "rust",
            language: "rust",
            producer: "lumin-rust-analyzer",
            engine_selection: request.engine_selection.clone(),
            advisory_path: Some(path_string(&written.specific_path)),
            latest_advisory_path: Some(path_string(&written.latest_path)),
            advisory_invocation_id: Some(request.advisory_invocation_id.clone()),
            evidence_availability: None,
            rust_evidence_path: None,
            any_inventory_path: None,
            rust_native_artifact_path: Some(path_string(&request.rust_native_artifact_path)),
            rust_native_latest_path: Some(path_string(&request.rust_native_latest_path)),
            source_commit: Some(source_commit),
            analyzer_invocation: Some(analyzer_invocation_block(&request)),
            failure_kind: None,
            child_exit_code: None,
            reason: None,
        },
        exit_code: 0,
        stdout: nonempty(child.stdout),
        stderr: nonempty(child.stderr),
    })
}

fn validate_request(request: &PreWriteLifecycleRequest) -> Result<()> {
    if request.schema_version != PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-pre-write: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty_path("root", &request.root)?;
    validate_nonempty_path("output", &request.output)?;
    validate_nonempty_path("rustNativeArtifactPath", &request.rust_native_artifact_path)?;
    validate_nonempty_path("rustNativeLatestPath", &request.rust_native_latest_path)?;
    if request.advisory_invocation_id.trim().is_empty() {
        bail!("execute-pre-write: advisoryInvocationId must be a non-empty string");
    }
    if request.analyzer_invocation.command.trim().is_empty() {
        bail!("execute-pre-write: analyzerInvocation.command must be a non-empty string");
    }
    if request.analyzer_invocation.source.trim().is_empty() {
        bail!("execute-pre-write: analyzerInvocation.source must be a non-empty string");
    }
    Ok(())
}

fn validate_nonempty_path(field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("execute-pre-write: {field} must be provided");
    }
    Ok(())
}

fn effective_source_commit(request: &PreWriteLifecycleRequest) -> String {
    request
        .source_commit
        .as_deref()
        .map(str::trim)
        .filter(|commit| !commit.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| git_head_commit_or_unknown(&request.root))
}

fn read_native_artifact(path: &Path) -> Result<RustPreWriteArtifact> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "rust pre-write artifact parse failed: failed to read {}",
            path.display()
        )
    })?;
    let artifact = serde_json::from_str::<RustPreWriteArtifact>(&text).with_context(|| {
        format!(
            "rust pre-write artifact parse failed: invalid shape or JSON in {}",
            path.display()
        )
    })?;
    validate_native_contract(&artifact)?;
    Ok(artifact)
}

fn validate_native_contract(artifact: &RustPreWriteArtifact) -> Result<()> {
    if artifact.schema_version != RUST_PRE_WRITE_ARTIFACT_SCHEMA_VERSION {
        bail!(
            "rust pre-write artifact contract failed: unsupported schemaVersion '{}'",
            artifact.schema_version
        );
    }
    if artifact.policy_version != RUST_PRE_WRITE_POLICY_VERSION {
        bail!(
            "rust pre-write artifact contract failed: unsupported policyVersion '{}'",
            artifact.policy_version
        );
    }
    if artifact.meta.producer != RUST_PRE_WRITE_PRODUCER {
        bail!(
            "rust pre-write artifact contract failed: unexpected producer '{}'",
            artifact.meta.producer
        );
    }

    let intent = artifact
        .intent
        .as_object()
        .context("rust pre-write artifact contract failed: intent must be an object")?;
    let shapes_requested = required_intent_array(intent, "shapes")? > 0;
    let files_requested = required_intent_array(intent, "files")? > 0;
    let dependencies_requested = required_intent_array(intent, "dependencies")? > 0;
    let inline_patterns_requested = intent
        .get("refactorSources")
        .map(|value| {
            value.as_array().map(|values| !values.is_empty()).context(
                "rust pre-write artifact contract failed: intent.refactorSources must be an array",
            )
        })
        .transpose()?
        .unwrap_or(false);
    required_intent_array(intent, "names")?;
    required_intent_array(intent, "plannedTypeEscapes")?;

    validate_coverage(
        &artifact.coverage,
        shapes_requested,
        files_requested,
        dependencies_requested,
        inline_patterns_requested,
    )
}

fn validate_coverage(
    coverage: &RustPreWriteCoverage,
    shapes_requested: bool,
    files_requested: bool,
    dependencies_requested: bool,
    inline_patterns_requested: bool,
) -> Result<()> {
    validate_coverage_lane("names", &coverage.names, true)?;
    validate_coverage_lane("shapes", &coverage.shapes, shapes_requested)?;
    validate_coverage_lane("files", &coverage.files, files_requested)?;
    validate_coverage_lane(
        "dependencies",
        &coverage.dependencies,
        dependencies_requested,
    )?;
    validate_coverage_lane(
        "inlinePatterns",
        &coverage.inline_patterns,
        inline_patterns_requested,
    )?;
    validate_coverage_lane("plannedTypeEscapes", &coverage.planned_type_escapes, true)
}

fn required_intent_array(intent: &Map<String, Value>, field: &str) -> Result<usize> {
    intent
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::len)
        .with_context(|| {
            format!("rust pre-write artifact contract failed: intent.{field} must be an array")
        })
}

fn validate_coverage_lane(field: &str, actual: &str, requested: bool) -> Result<()> {
    let expected = if requested { "ran" } else { "not-requested" };
    if actual != expected {
        bail!(
            "rust pre-write artifact contract failed: coverage.{field} must be '{expected}', got '{actual}'"
        );
    }
    Ok(())
}

fn copy_native_latest(request: &PreWriteLifecycleRequest) -> Result<()> {
    let bytes = fs::read(&request.rust_native_artifact_path).with_context(|| {
        format!(
            "execute-pre-write: failed to read {}",
            request.rust_native_artifact_path.display()
        )
    })?;
    atomic_write_bytes(&request.rust_native_latest_path, &bytes).with_context(|| {
        format!(
            "execute-pre-write: failed to write {}",
            request.rust_native_latest_path.display()
        )
    })
}

fn clear_outputs(request: &PreWriteLifecycleRequest) -> Result<()> {
    let latest_advisory = advisory_latest_path(&request.output);
    let specific_advisory =
        advisory_specific_path(&request.output, &request.advisory_invocation_id);
    for path in [
        request.rust_native_artifact_path.as_path(),
        request.rust_native_latest_path.as_path(),
        latest_advisory.as_path(),
        specific_advisory.as_path(),
    ] {
        remove_file_if_present(path).with_context(|| {
            format!(
                "execute-pre-write: failed to clear stale output {}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn clear_projected_outputs(request: &PreWriteLifecycleRequest) -> Result<()> {
    let latest_advisory = advisory_latest_path(&request.output);
    let specific_advisory =
        advisory_specific_path(&request.output, &request.advisory_invocation_id);
    for path in [
        request.rust_native_latest_path.as_path(),
        latest_advisory.as_path(),
        specific_advisory.as_path(),
    ] {
        remove_file_if_present(path).with_context(|| {
            format!(
                "execute-pre-write: failed to remove invalid projected output {}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn output_failure_reason(request: &PreWriteLifecycleRequest, reason: String) -> String {
    match clear_projected_outputs(request) {
        Ok(()) => reason,
        Err(cleanup_error) => format!("{reason}; projected output cleanup failed: {cleanup_error}"),
    }
}

fn build_advisory(
    request: &PreWriteLifecycleRequest,
    rust_artifact: &RustPreWriteArtifact,
    source_commit: &str,
) -> Result<Value> {
    let intent = rust_intent(&rust_artifact.intent)?;
    let intent_hash = hash_intent(&intent)?;
    Ok(serde_json::json!({
        "invocationId": request.advisory_invocation_id,
        "intentHash": intent_hash,
        "artifactPaths": {
            "invocationSpecific": path_string(&advisory_specific_path(&request.output, &request.advisory_invocation_id)),
            "latest": path_string(&advisory_latest_path(&request.output)),
            "rustNative": path_string(&request.rust_native_artifact_path),
        },
        "scanRange": {
            "root": path_string(&request.root),
            "output": path_string(&request.output),
            "includeTests": request.include_tests,
            "production": request.production,
            "excludes": request.excludes.clone(),
        },
        "intent": intent,
        "intentWarnings": rust_artifact.intent_warnings.clone(),
        "evidenceAvailability": {
            "status": "available",
            "producer": "lumin-rust-analyzer",
            "rustNativeArtifactPath": path_string(&request.rust_native_artifact_path),
        },
        "lookups": rust_artifact.lookups.clone(),
        "shapeLookups": rust_artifact.shape_lookups.clone(),
        "fileLookups": rust_artifact.file_lookups.clone(),
        "dependencyLookups": rust_artifact.dependency_lookups.clone(),
        "inlinePatternLookups": rust_artifact.inline_pattern_lookups.clone(),
        "cueCards": rust_artifact.cue_cards.clone(),
        "suppressedCues": rust_artifact.suppressed_cues.clone(),
        "unavailableEvidence": rust_artifact.unavailable_evidence.clone(),
        "cuePolicy": Value::Null,
        "boundaryChecks": [],
        "drift": Value::Null,
        "preWrite": {
            "fileInventory": request.file_inventory.clone(),
            "rustNativeArtifactPath": path_string(&request.rust_native_artifact_path),
            "sourceCommit": source_commit,
        },
        "rustPreWrite": {
            "schemaVersion": rust_artifact.schema_version.clone(),
            "policyVersion": rust_artifact.policy_version.clone(),
            "producer": rust_artifact.meta.producer.clone(),
            "coverage": rust_artifact.coverage.clone(),
        },
        "capabilities": {
            "language": "rust",
            "producer": "lumin-rust-analyzer",
            "postWriteTypeEscapes": "not-applicable",
        },
        "failures": request.failures.clone(),
    }))
}

fn rust_intent(intent: &Value) -> Result<Value> {
    let mut object = intent
        .as_object()
        .cloned()
        .context("execute-pre-write: validated native intent must be an object")?;
    object.insert("language".to_string(), Value::String("rust".to_string()));
    Ok(Value::Object(object))
}

fn hash_intent(intent: &Value) -> Result<String> {
    let normalized = sorted_json_value(intent);
    let text = serde_json::to_string(&normalized)
        .context("execute-pre-write: failed to serialize normalized intent")?;
    let digest = sha256_text(&text);
    Ok(digest
        .strip_prefix("sha256:")
        .unwrap_or(digest.as_str())
        .to_string())
}

fn sorted_json_value(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(sorted_json_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                sorted.insert(key.clone(), sorted_json_value(&map[key]));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

fn failure_result(
    request: &PreWriteLifecycleRequest,
    failure_kind: PreWriteFailureKind,
    reason: String,
    exit_code: i32,
    child_exit_code: Option<i32>,
    stdout: Option<String>,
    stderr: Option<String>,
) -> PreWriteLifecycleResult {
    let evidence_availability = serde_json::json!({
        "status": "unavailable",
        "producer": RUST_PRE_WRITE_PRODUCER,
        "failureKind": failure_kind,
        "reason": reason,
        "rustNativeArtifactPath": path_string(&request.rust_native_artifact_path),
    });
    PreWriteLifecycleResult {
        schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block: PreWriteBlock {
            requested: true,
            ran: false,
            execution_owner: "lumin-audit-core",
            engine: "rust",
            language: "rust",
            producer: RUST_PRE_WRITE_PRODUCER,
            engine_selection: request.engine_selection.clone(),
            advisory_path: None,
            latest_advisory_path: None,
            advisory_invocation_id: Some(request.advisory_invocation_id.clone()),
            evidence_availability: Some(evidence_availability),
            rust_evidence_path: None,
            any_inventory_path: None,
            rust_native_artifact_path: Some(path_string(&request.rust_native_artifact_path)),
            rust_native_latest_path: None,
            source_commit: request.source_commit.clone(),
            analyzer_invocation: Some(analyzer_invocation_block(request)),
            failure_kind: Some(failure_kind),
            child_exit_code,
            reason: Some(reason),
        },
        exit_code,
        stdout,
        stderr,
    }
}

fn analyzer_invocation_block(request: &PreWriteLifecycleRequest) -> AnalyzerInvocationBlock {
    AnalyzerInvocationBlock {
        source: request.analyzer_invocation.source.clone(),
        manifest_path: request
            .analyzer_invocation
            .manifest_path
            .as_ref()
            .map(|path| path_string(path)),
    }
}

fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes)?;
    fs::rename(temp, path)
}
