mod delta;
mod file_delta;
mod protocol;
mod render;

use anyhow::{bail, Context, Result};
use delta::{compute_delta, type_escape_delta_not_applicable};
use file_delta::{compute_file_delta, repo_relative_file_list};
use lumin_rust_common::atomic_write_json_pretty;
use protocol::{AnyInventory, PostWriteDeltaArtifact, PreWriteAdvisory};
pub use protocol::{
    PostWriteBlock, PostWriteFailureKind, PostWriteLifecycleRequest, PostWriteLifecycleResult,
    POST_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION, POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
};
use render::render_markdown;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::js_ts_pre_write::{
    collect_js_ts_pre_write_evidence, JsTsPreWriteEvidenceRequest,
    JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION,
};
use crate::scan_scope::{collect_source_files, ScanScopeOptions};

pub fn execute_post_write_lifecycle(
    request: PostWriteLifecycleRequest,
) -> Result<PostWriteLifecycleResult> {
    execute_native_post_write(request)
}

pub fn execute_post_write_lifecycle_streaming(
    request: PostWriteLifecycleRequest,
) -> Result<PostWriteLifecycleResult> {
    execute_native_post_write(request)
}

fn execute_native_post_write(
    request: PostWriteLifecycleRequest,
) -> Result<PostWriteLifecycleResult> {
    validate_request(&request)?;
    let delta_dir = delta_output_dir(&request);
    let latest_delta_path = delta_dir.join("post-write-delta.latest.json");
    if let Err(error) = remove_file_if_present(&latest_delta_path) {
        return Ok(failure_result(
            PostWriteFailureKind::OutputCleanupFailed,
            format!(
                "execute-post-write: failed to clear stale {}: {error}",
                latest_delta_path.display()
            ),
            1,
            None,
            None,
        ));
    }
    let Some(advisory_path) = request.advisory_path.as_deref() else {
        return Ok(failure_result(
            PostWriteFailureKind::MissingAdvisory,
            "--pre-write-advisory missing".to_string(),
            2,
            None,
            Some(
                "[audit-repo] --post-write requested but skipped: --pre-write-advisory <file> missing\n"
                    .to_string(),
            ),
        ));
    };
    let advisory = match read_advisory(advisory_path) {
        Ok(advisory) => advisory,
        Err(error) => {
            return Ok(failure_result(
                PostWriteFailureKind::InvalidAdvisory,
                error.to_string(),
                2,
                None,
                Some(format!(
                    "[audit-repo] post-write advisory invalid: {error}\n"
                )),
            ));
        }
    };

    let mut stderr = String::new();
    let skip_type_escapes = type_escape_delta_not_applicable(&advisory);
    let after_snapshot = match build_after_snapshot(&request, skip_type_escapes) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            let reason = cleanup_failure_reason(
                &[
                    &latest_delta_path,
                    &request.output.join("any-inventory.json"),
                ],
                format!("native post-write evidence failed: {error}"),
            );
            return Ok(failure_result(
                PostWriteFailureKind::EvidenceFailed,
                reason,
                1,
                Some(advisory.invocation_id),
                Some(format!(
                    "[post-write] Rust after-inventory failed: {error}\n"
                )),
            ));
        }
    };
    if !skip_type_escapes {
        stderr.push_str("[post-write] running lumin-audit-core for after-snapshot\n");
    }
    let (before_inventory, warnings) = load_before_inventory(&request, advisory_path, &advisory);
    for warning in warnings {
        stderr.push_str(&format!("[post-write] {warning}\n"));
    }
    let before_files = (advisory.pre_write.file_inventory.status.as_deref() == Some("available"))
        .then_some(advisory.pre_write.file_inventory.files.as_slice());
    let file_delta = compute_file_delta(
        &request.root,
        &advisory.intent.files,
        before_files,
        Some(&after_snapshot.files),
        None,
    );
    let delta = compute_delta(
        &advisory,
        before_inventory.as_ref(),
        after_snapshot.inventory.as_ref(),
        &request.delta_invocation_id,
        file_delta,
    );
    let specific_delta_path = delta_specific_path(&delta_dir, &delta);
    if let Err(error) = write_delta_artifacts(&specific_delta_path, &latest_delta_path, &delta) {
        let reason = cleanup_failure_reason(
            &[&specific_delta_path, &latest_delta_path],
            format!("post-write delta write failed: {error}"),
        );
        return Ok(failure_result(
            PostWriteFailureKind::DeltaArtifactInvalid,
            reason,
            1,
            Some(advisory.invocation_id),
            nonempty(stderr),
        ));
    }
    let markdown = render_markdown(&delta);
    let mut block = success_block(&latest_delta_path, &delta);
    project_delta_summary(&mut block, &delta);
    Ok(PostWriteLifecycleResult {
        schema_version: POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block,
        exit_code: 0,
        stdout: Some(markdown),
        stderr: nonempty(stderr),
    })
}

struct AfterSnapshot {
    inventory: Option<AnyInventory>,
    files: Vec<String>,
}

fn build_after_snapshot(
    request: &PostWriteLifecycleRequest,
    skip_type_escapes: bool,
) -> Result<AfterSnapshot> {
    if skip_type_escapes && request.host_evidence_transport.is_none() {
        let files = collect_source_files(
            &request.root,
            &ScanScopeOptions {
                include_tests: request.include_tests,
                exclude: request.excludes.clone(),
                ..ScanScopeOptions::default()
            },
        )?;
        return Ok(AfterSnapshot {
            inventory: None,
            files: repo_relative_file_list(&request.root, &files),
        });
    }

    let inventory_path = request.output.join("any-inventory.json");
    remove_file_if_present(&inventory_path)?;
    let mut evidence = collect_js_ts_pre_write_evidence(
        JsTsPreWriteEvidenceRequest {
            schema_version: JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            root: request.root.clone(),
            evidence_artifact: format!("post-write-evidence.{}.json", request.delta_invocation_id),
            any_inventory_artifact: "any-inventory.json".to_string(),
            generated: request.generated.clone(),
            include_tests: request.include_tests,
            excludes: request.excludes.clone(),
            dependency_roots: Vec::new(),
            shape_type_literals: Vec::new(),
            discover_files: true,
            files: Vec::new(),
            incremental: request.incremental.clone(),
        },
        request.host_evidence_transport.as_ref(),
        &request.output,
        &request.delta_invocation_id,
    )?;
    let object = evidence
        .as_object_mut()
        .context("native post-write evidence response must be an object")?;
    let inventory_value = object
        .remove("anyInventory")
        .context("native post-write evidence response missing anyInventory")?;
    let files_value = object
        .remove("files")
        .context("native post-write evidence response missing files")?;
    let inventory = (!skip_type_escapes)
        .then(|| {
            serde_json::from_value::<AnyInventory>(inventory_value)
                .context("native post-write anyInventory shape is invalid")
        })
        .transpose()?;
    let files = serde_json::from_value::<Vec<String>>(files_value)
        .context("native post-write files shape is invalid")?;
    if let Some(inventory) = inventory.as_ref() {
        if let Err(error) = atomic_write_json_pretty(&inventory_path, inventory) {
            let _ = remove_file_if_present(&inventory_path);
            return Err(error).context("failed to write native post-write any-inventory.json");
        }
    }
    Ok(AfterSnapshot { inventory, files })
}

fn load_before_inventory(
    request: &PostWriteLifecycleRequest,
    advisory_path: &Path,
    advisory: &PreWriteAdvisory,
) -> (Option<AnyInventory>, Vec<String>) {
    let Some(name) = advisory.pre_write.any_inventory_path.as_deref() else {
        return (None, Vec::new());
    };
    let mut directories = Vec::new();
    if let Some(parent) = advisory_path.parent() {
        directories.push(parent.to_path_buf());
    }
    if let Some(output) = advisory.scan_range.output.as_ref() {
        directories.push(output.clone());
    }
    directories.push(request.output.clone());
    directories.sort();
    directories.dedup();
    let mut warnings = Vec::new();
    let name_path = Path::new(name);
    let candidates = if name_path.is_absolute() {
        vec![name_path.to_path_buf()]
    } else {
        directories
            .into_iter()
            .map(|directory| directory.join(name_path))
            .collect()
    };
    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        match fs::read(&candidate)
            .map_err(anyhow::Error::from)
            .and_then(|bytes| serde_json::from_slice::<AnyInventory>(&bytes).map_err(Into::into))
        {
            Ok(inventory) => return (Some(inventory), warnings),
            Err(error) => {
                warnings.push(format!("failed to parse {}: {error}", candidate.display()))
            }
        }
    }
    (None, warnings)
}

fn validate_request(request: &PostWriteLifecycleRequest) -> Result<()> {
    if request.schema_version != POST_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-post-write: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty_path("root", &request.root)?;
    validate_nonempty_path("output", &request.output)?;
    validate_artifact_id(
        "post-write delta deltaInvocationId",
        &request.delta_invocation_id,
    )?;
    if request.generated.trim().is_empty() {
        bail!("execute-post-write: generated must be a non-empty string");
    }
    Ok(())
}

fn read_advisory(path: &Path) -> Result<PreWriteAdvisory> {
    let bytes = fs::read(path).with_context(|| {
        format!(
            "post-write advisory contract failed: failed to read {}",
            path.display()
        )
    })?;
    let advisory = serde_json::from_slice::<PreWriteAdvisory>(&bytes).with_context(|| {
        format!(
            "post-write advisory contract failed: invalid JSON or shape in {}",
            path.display()
        )
    })?;
    validate_artifact_id("post-write advisory invocationId", &advisory.invocation_id)?;
    Ok(advisory)
}

fn write_delta_artifacts(
    specific_path: &Path,
    latest_path: &Path,
    delta: &PostWriteDeltaArtifact,
) -> Result<()> {
    atomic_write_json_pretty(specific_path, delta)
        .with_context(|| format!("failed to write {}", specific_path.display()))?;
    atomic_write_json_pretty(latest_path, delta)
        .with_context(|| format!("failed to write {}", latest_path.display()))?;
    Ok(())
}

fn success_block(path: &Path, delta: &PostWriteDeltaArtifact) -> PostWriteBlock {
    PostWriteBlock {
        requested: true,
        ran: true,
        execution_owner: "lumin-audit-core",
        delta_path: Some(path_string(path)),
        silent_new: None,
        required_acknowledgement_count: None,
        baseline_status: None,
        scan_range_parity: None,
        type_escape_delta_status: None,
        after_complete: None,
        file_delta_status: None,
        unexpected_new_file_count: None,
        planned_missing_file_count: None,
        pre_write_invocation_id: Some(delta.pre_write_invocation_id.clone()),
        delta_invocation_id: Some(delta.delta_invocation_id.clone()),
        delta_schema_version: Some(delta.schema_version.clone()),
        failure_kind: None,
        child_exit_code: None,
        reason: None,
    }
}

fn project_delta_summary(block: &mut PostWriteBlock, delta: &PostWriteDeltaArtifact) {
    block.silent_new = Some(delta.summary.silent_new);
    block.required_acknowledgement_count = Some(
        delta
            .entries
            .iter()
            .filter(|entry| entry.label == "silent-new")
            .count(),
    );
    block.baseline_status = Some(delta.baseline.status.clone());
    block.scan_range_parity = Some(delta.scan_range_parity.status.clone());
    block.type_escape_delta_status = Some(delta.type_escape_delta.status.clone());
    block.after_complete = Some(
        delta
            .inventory_completeness
            .after_complete
            .map(Value::Bool)
            .unwrap_or(Value::Null),
    );
    block.file_delta_status = Some(delta.file_delta.status.clone());
    block.unexpected_new_file_count = delta
        .file_delta
        .summary
        .as_ref()
        .map(|summary| summary.unexpected_new);
    block.planned_missing_file_count = delta
        .file_delta
        .summary
        .as_ref()
        .map(|summary| summary.planned_missing)
        .or_else(|| {
            delta
                .file_delta
                .planned_missing
                .as_ref()
                .map(|files| files.len() as u64)
        });
}

fn failure_result(
    kind: PostWriteFailureKind,
    reason: String,
    exit_code: i32,
    pre_write_invocation_id: Option<String>,
    stderr: Option<String>,
) -> PostWriteLifecycleResult {
    PostWriteLifecycleResult {
        schema_version: POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block: PostWriteBlock {
            requested: true,
            ran: false,
            execution_owner: "lumin-audit-core",
            delta_path: None,
            silent_new: None,
            required_acknowledgement_count: None,
            baseline_status: None,
            scan_range_parity: None,
            type_escape_delta_status: None,
            after_complete: None,
            file_delta_status: None,
            unexpected_new_file_count: None,
            planned_missing_file_count: None,
            pre_write_invocation_id,
            delta_invocation_id: None,
            delta_schema_version: None,
            failure_kind: Some(kind),
            child_exit_code: None,
            reason: Some(reason),
        },
        exit_code,
        stdout: None,
        stderr,
    }
}

fn delta_output_dir(request: &PostWriteLifecycleRequest) -> PathBuf {
    request
        .delta_out
        .clone()
        .unwrap_or_else(|| request.output.clone())
}

fn delta_specific_path(output: &Path, delta: &PostWriteDeltaArtifact) -> PathBuf {
    output.join(format!(
        "post-write-delta.{}.{}.json",
        delta.pre_write_invocation_id, delta.delta_invocation_id
    ))
}

fn validate_nonempty_path(field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("execute-post-write: {field} must be provided");
    }
    Ok(())
}

fn validate_artifact_id(label: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        bail!("{label} must be a filename-safe non-empty identifier");
    }
    Ok(())
}

fn remove_file_if_present(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn cleanup_failure_reason(paths: &[&Path], mut reason: String) -> String {
    for path in paths {
        if let Err(error) = remove_file_if_present(path) {
            reason.push_str(&format!(
                "; failed to remove invalid artifact {}: {error}",
                path.display()
            ));
        }
    }
    reason
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}
