use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const POST_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-post-write-lifecycle-request.v1";
pub const POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION: &str = "lumin-post-write-lifecycle-result.v1";
const POST_WRITE_DELTA_SCHEMA_VERSION: &str = "lumin-post-write-delta.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostWriteLifecycleRequest {
    pub schema_version: String,
    pub root: PathBuf,
    pub output: PathBuf,
    #[serde(default)]
    pub advisory_path: Option<PathBuf>,
    #[serde(default)]
    pub delta_out: Option<PathBuf>,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    #[serde(default)]
    pub no_fresh_audit: bool,
    #[serde(default)]
    pub scan_args: Vec<String>,
    #[serde(default)]
    pub incremental_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostWriteLifecycleResult {
    pub schema_version: &'static str,
    pub block: PostWriteBlock,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostWriteBlock {
    pub requested: bool,
    pub ran: bool,
    pub execution_owner: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silent_new: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_acknowledgement_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan_range_parity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_escape_delta_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_complete: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_delta_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unexpected_new_file_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_missing_file_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_write_invocation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_invocation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_schema_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<PostWriteFailureKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PostWriteFailureKind {
    MissingAdvisory,
    InvalidAdvisory,
    OutputCleanupFailed,
    ChildFailed,
    DeltaArtifactInvalid,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostWriteDeltaArtifact {
    schema_version: String,
    pre_write_invocation_id: String,
    delta_invocation_id: String,
    summary: PostWriteDeltaSummary,
    entries: Vec<PostWriteDeltaEntry>,
    baseline: StatusBlock,
    scan_range_parity: StatusBlock,
    type_escape_delta: StatusBlock,
    inventory_completeness: InventoryCompletenessBlock,
    file_delta: FileDeltaBlock,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostWriteDeltaSummary {
    silent_new: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PostWriteDeltaEntry {
    label: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct StatusBlock {
    status: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InventoryCompletenessBlock {
    after_complete: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDeltaBlock {
    status: String,
    #[serde(default)]
    summary: Option<FileDeltaSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDeltaSummary {
    #[serde(default)]
    unexpected_new: Option<u64>,
    #[serde(default)]
    planned_missing: Option<u64>,
}

pub fn execute_post_write_lifecycle(
    request: PostWriteLifecycleRequest,
) -> Result<PostWriteLifecycleResult> {
    execute_post_write_lifecycle_with_stdio(request, ChildStdio::Capture)
}

pub fn execute_post_write_lifecycle_streaming(
    request: PostWriteLifecycleRequest,
) -> Result<PostWriteLifecycleResult> {
    execute_post_write_lifecycle_with_stdio(request, ChildStdio::Inherit)
}

fn execute_post_write_lifecycle_with_stdio(
    request: PostWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> Result<PostWriteLifecycleResult> {
    validate_request(&request)?;
    let Some(advisory_path) = request.advisory_path.as_ref() else {
        return Ok(failure_result(
            PostWriteFailureKind::MissingAdvisory,
            "--pre-write-advisory missing".to_string(),
            2,
            None,
            None,
            None,
            Some(
                "[audit-repo] --post-write requested but skipped: --pre-write-advisory <file> missing\n"
                    .to_string(),
            ),
        ));
    };

    let pre_write_invocation_id = match read_advisory_invocation_id(advisory_path) {
        Ok(invocation_id) => invocation_id,
        Err(error) => {
            return Ok(failure_result(
                PostWriteFailureKind::InvalidAdvisory,
                error.to_string(),
                2,
                None,
                None,
                None,
                Some(format!(
                    "[audit-repo] post-write advisory invalid: {error}\n"
                )),
            ));
        }
    };
    let delta_path = delta_output_dir(&request).join("post-write-delta.latest.json");
    if let Err(error) = remove_file_if_present(&delta_path) {
        return Ok(failure_result(
            PostWriteFailureKind::OutputCleanupFailed,
            format!(
                "execute-post-write: failed to clear stale {}: {error}",
                delta_path.display()
            ),
            1,
            Some(pre_write_invocation_id),
            None,
            None,
            None,
        ));
    }

    let child = run_post_write_child(&request, advisory_path, child_stdio);
    if !child.status_success {
        let reason = post_write_failure_reason(
            &delta_path,
            format!("post-write.mjs exited non-zero: {}", child.reason),
        );
        return Ok(failure_result(
            PostWriteFailureKind::ChildFailed,
            reason,
            child.exit_code.unwrap_or(1),
            Some(pre_write_invocation_id),
            child.exit_code,
            nonempty(child.stdout),
            nonempty(child.stderr),
        ));
    }

    let delta = match read_delta(&delta_path, &pre_write_invocation_id) {
        Ok(delta) => delta,
        Err(error) => {
            let reason = post_write_failure_reason(&delta_path, error.to_string());
            return Ok(failure_result(
                PostWriteFailureKind::DeltaArtifactInvalid,
                reason,
                1,
                Some(pre_write_invocation_id),
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };
    let specific_delta_path = delta_specific_path(delta_output_dir(&request), &delta);
    let specific_delta = match read_delta(&specific_delta_path, &pre_write_invocation_id) {
        Ok(delta) => delta,
        Err(error) => {
            let reason = post_write_failure_reason(&delta_path, error.to_string());
            return Ok(failure_result(
                PostWriteFailureKind::DeltaArtifactInvalid,
                reason,
                1,
                Some(pre_write_invocation_id),
                child.exit_code,
                nonempty(child.stdout),
                nonempty(child.stderr),
            ));
        }
    };
    if delta != specific_delta {
        let reason = post_write_failure_reason(
            &delta_path,
            format!(
                "post-write delta contract failed: latest and invocation-specific artifacts differ ({} != {})",
                delta_path.display(),
                specific_delta_path.display()
            ),
        );
        return Ok(failure_result(
            PostWriteFailureKind::DeltaArtifactInvalid,
            reason,
            1,
            Some(pre_write_invocation_id),
            child.exit_code,
            nonempty(child.stdout),
            nonempty(child.stderr),
        ));
    }

    let mut block = PostWriteBlock {
        requested: true,
        ran: true,
        execution_owner: "lumin-audit-core",
        delta_path: Some(path_string(&delta_path)),
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
    };
    project_delta_summary(&mut block, &delta);

    Ok(PostWriteLifecycleResult {
        schema_version: POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block,
        exit_code: 0,
        stdout: nonempty(child.stdout),
        stderr: nonempty(child.stderr),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildStdio {
    Capture,
    Inherit,
}

fn validate_request(request: &PostWriteLifecycleRequest) -> Result<()> {
    if request.schema_version != POST_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-post-write: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty("root", &request.root)?;
    validate_nonempty("output", &request.output)?;
    validate_nonempty("scriptsDir", &request.scripts_dir)?;
    if request.node_executable.trim().is_empty() {
        bail!("execute-post-write: nodeExecutable must be a non-empty string");
    }
    Ok(())
}

fn validate_nonempty(field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("execute-post-write: {field} must be provided");
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct ChildOutput {
    status_success: bool,
    exit_code: Option<i32>,
    reason: String,
    stdout: String,
    stderr: String,
}

fn run_post_write_child(
    request: &PostWriteLifecycleRequest,
    advisory_path: &Path,
    child_stdio: ChildStdio,
) -> ChildOutput {
    let post_write_path = request.scripts_dir.join("post-write.mjs");
    let mut args = vec![
        path_string(&post_write_path),
        "--root".to_string(),
        path_string(&request.root),
        "--output".to_string(),
        path_string(&request.output),
        "--pre-write-advisory".to_string(),
        path_string(advisory_path),
    ];
    if let Some(delta_out) = &request.delta_out {
        args.extend(["--delta-out".to_string(), path_string(delta_out)]);
    }
    if request.no_fresh_audit {
        args.push("--no-fresh-audit".to_string());
    }
    args.extend(request.scan_args.clone());
    args.extend(request.incremental_args.clone());

    match child_stdio {
        ChildStdio::Capture => match Command::new(&request.node_executable)
            .args(args)
            .stdin(Stdio::null())
            .output()
        {
            Ok(output) => {
                let status_success = output.status.success();
                let reason = output
                    .status
                    .code()
                    .map(|code| format!("post-write.mjs exited {code}"))
                    .unwrap_or_else(|| "post-write.mjs terminated by signal".to_string());
                ChildOutput {
                    status_success,
                    exit_code: output.status.code(),
                    reason,
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                }
            }
            Err(error) => ChildOutput {
                status_success: false,
                exit_code: None,
                reason: error.to_string(),
                stdout: String::new(),
                stderr: String::new(),
            },
        },
        ChildStdio::Inherit => match Command::new(&request.node_executable)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
        {
            Ok(status) => {
                let status_success = status.success();
                let reason = status
                    .code()
                    .map(|code| format!("post-write.mjs exited {code}"))
                    .unwrap_or_else(|| "post-write.mjs terminated by signal".to_string());
                ChildOutput {
                    status_success,
                    exit_code: status.code(),
                    reason,
                    stdout: String::new(),
                    stderr: String::new(),
                }
            }
            Err(error) => ChildOutput {
                status_success: false,
                exit_code: None,
                reason: error.to_string(),
                stdout: String::new(),
                stderr: String::new(),
            },
        },
    }
}

fn delta_output_dir(request: &PostWriteLifecycleRequest) -> &Path {
    request.delta_out.as_deref().unwrap_or(&request.output)
}

fn read_advisory_invocation_id(advisory_path: &Path) -> Result<String> {
    let bytes = fs::read(advisory_path).with_context(|| {
        format!(
            "post-write advisory contract failed: failed to read {}",
            advisory_path.display()
        )
    })?;
    let advisory: Value = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "post-write advisory contract failed: invalid JSON in {}",
            advisory_path.display()
        )
    })?;
    let invocation_id = advisory
        .as_object()
        .context("post-write advisory contract failed: advisory must be an object")?
        .get("invocationId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("post-write advisory contract failed: invocationId must be a non-empty string")?;
    validate_artifact_id("post-write advisory invocationId", invocation_id)?;
    Ok(invocation_id.to_string())
}

fn read_delta(delta_path: &Path, expected_pre_write_id: &str) -> Result<PostWriteDeltaArtifact> {
    let bytes = fs::read(delta_path).with_context(|| {
        format!(
            "post-write delta contract failed: failed to read {}",
            delta_path.display()
        )
    })?;
    let delta = serde_json::from_slice::<PostWriteDeltaArtifact>(&bytes).with_context(|| {
        format!(
            "post-write delta contract failed: invalid shape or JSON in {}",
            delta_path.display()
        )
    })?;
    delta.validate(expected_pre_write_id)?;
    Ok(delta)
}

impl PostWriteDeltaArtifact {
    fn validate(&self, expected_pre_write_id: &str) -> Result<()> {
        if self.schema_version != POST_WRITE_DELTA_SCHEMA_VERSION {
            bail!(
                "post-write delta contract failed: unsupported schemaVersion '{}'",
                self.schema_version
            );
        }
        validate_artifact_id(
            "post-write delta preWriteInvocationId",
            &self.pre_write_invocation_id,
        )?;
        validate_artifact_id(
            "post-write delta deltaInvocationId",
            &self.delta_invocation_id,
        )?;
        if self.pre_write_invocation_id != expected_pre_write_id {
            bail!(
                "post-write delta contract failed: preWriteInvocationId '{}' does not match advisory '{}'",
                self.pre_write_invocation_id,
                expected_pre_write_id
            );
        }
        if !matches!(
            self.inventory_completeness.after_complete,
            Value::Bool(_) | Value::Null
        ) {
            bail!(
                "post-write delta contract failed: inventoryCompleteness.afterComplete must be boolean or null"
            );
        }
        Ok(())
    }
}

fn validate_artifact_id(label: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        bail!("{label} must be a filename-safe non-empty identifier");
    }
    Ok(())
}

fn delta_specific_path(output: &Path, delta: &PostWriteDeltaArtifact) -> PathBuf {
    output.join(format!(
        "post-write-delta.{}.{}.json",
        delta.pre_write_invocation_id, delta.delta_invocation_id
    ))
}

fn remove_file_if_present(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn post_write_failure_reason(delta_path: &Path, reason: String) -> String {
    match remove_file_if_present(delta_path) {
        Ok(()) => reason,
        Err(cleanup_error) => format!(
            "{reason}; failed to remove invalid latest delta {}: {cleanup_error}",
            delta_path.display()
        ),
    }
}

fn project_delta_summary(block: &mut PostWriteBlock, delta: &PostWriteDeltaArtifact) {
    let type_escape_delta_status = delta.type_escape_delta.status.clone();

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
    block.type_escape_delta_status = Some(type_escape_delta_status);
    block.after_complete = Some(delta.inventory_completeness.after_complete.clone());
    block.file_delta_status = Some(delta.file_delta.status.clone());
    block.unexpected_new_file_count = Some(
        delta
            .file_delta
            .summary
            .as_ref()
            .and_then(|summary| summary.unexpected_new)
            .unwrap_or(0),
    );
    block.planned_missing_file_count = Some(
        delta
            .file_delta
            .summary
            .as_ref()
            .and_then(|summary| summary.planned_missing)
            .unwrap_or(0),
    );
}

fn failure_result(
    failure_kind: PostWriteFailureKind,
    reason: String,
    exit_code: i32,
    pre_write_invocation_id: Option<String>,
    child_exit_code: Option<i32>,
    stdout: Option<String>,
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
            failure_kind: Some(failure_kind),
            child_exit_code,
            reason: Some(reason),
        },
        exit_code,
        stdout,
        stderr,
    }
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
