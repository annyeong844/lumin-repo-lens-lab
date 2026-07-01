use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const POST_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-post-write-lifecycle-request.v1";
pub const POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION: &str = "lumin-post-write-lifecycle-result.v1";

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
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostWriteDeltaArtifact {
    #[serde(default)]
    summary: Option<PostWriteDeltaSummary>,
    #[serde(default)]
    entries: Vec<PostWriteDeltaEntry>,
    #[serde(default)]
    baseline: Option<StatusBlock>,
    #[serde(default)]
    scan_range_parity: Option<StatusBlock>,
    #[serde(default)]
    type_escape_delta: Option<StatusBlock>,
    #[serde(default)]
    inventory_completeness: Option<InventoryCompletenessBlock>,
    #[serde(default)]
    file_delta: Option<FileDeltaBlock>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostWriteDeltaSummary {
    #[serde(default)]
    silent_new: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PostWriteDeltaEntry {
    #[serde(default)]
    label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct StatusBlock {
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InventoryCompletenessBlock {
    #[serde(default)]
    after_complete: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileDeltaBlock {
    #[serde(default)]
    status: Option<String>,
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
    validate_request(&request)?;
    let Some(advisory_path) = request.advisory_path.as_ref() else {
        return Ok(PostWriteLifecycleResult {
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
                reason: Some("--pre-write-advisory missing".to_string()),
            },
            exit_code: 2,
            stdout: None,
            stderr: Some(
                "[audit-repo] --post-write requested but skipped: --pre-write-advisory <file> missing\n"
                    .to_string(),
            ),
        });
    };

    let child = run_post_write_child(&request, advisory_path);
    if !child.status_success {
        return Ok(PostWriteLifecycleResult {
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
                reason: Some(format!("post-write.mjs exited non-zero: {}", child.reason)),
            },
            exit_code: 0,
            stdout: nonempty(child.stdout),
            stderr: nonempty(child.stderr),
        });
    }

    let delta_path = delta_output_dir(&request).join("post-write-delta.latest.json");
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
        reason: None,
    };
    if let Some(delta) = read_delta(&delta_path) {
        project_delta_summary(&mut block, &delta);
    }

    Ok(PostWriteLifecycleResult {
        schema_version: POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
        block,
        exit_code: 0,
        stdout: nonempty(child.stdout),
        stderr: nonempty(child.stderr),
    })
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
    reason: String,
    stdout: String,
    stderr: String,
}

fn run_post_write_child(request: &PostWriteLifecycleRequest, advisory_path: &Path) -> ChildOutput {
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

    match Command::new(&request.node_executable)
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
                reason,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }
        }
        Err(error) => ChildOutput {
            status_success: false,
            reason: error.to_string(),
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

fn delta_output_dir(request: &PostWriteLifecycleRequest) -> &Path {
    request.delta_out.as_deref().unwrap_or(&request.output)
}

fn read_delta(delta_path: &Path) -> Option<PostWriteDeltaArtifact> {
    let bytes = std::fs::read(delta_path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn project_delta_summary(block: &mut PostWriteBlock, delta: &PostWriteDeltaArtifact) {
    let type_escape_delta_status = delta
        .type_escape_delta
        .as_ref()
        .and_then(|status| status.status.clone())
        .unwrap_or_else(|| "computed".to_string());

    block.silent_new = Some(
        delta
            .summary
            .as_ref()
            .and_then(|summary| summary.silent_new)
            .unwrap_or(0),
    );
    block.required_acknowledgement_count = Some(
        delta
            .entries
            .iter()
            .filter(|entry| entry.label.as_deref() == Some("silent-new"))
            .count(),
    );
    block.baseline_status = Some(
        delta
            .baseline
            .as_ref()
            .and_then(|status| status.status.clone())
            .unwrap_or_else(|| "missing".to_string()),
    );
    block.scan_range_parity = Some(
        delta
            .scan_range_parity
            .as_ref()
            .and_then(|status| status.status.clone())
            .unwrap_or_else(|| "baseline-missing".to_string()),
    );
    block.type_escape_delta_status = Some(type_escape_delta_status.clone());
    block.after_complete = Some(
        delta
            .inventory_completeness
            .as_ref()
            .and_then(|completeness| completeness.after_complete)
            .map(Value::Bool)
            .unwrap_or_else(|| {
                if type_escape_delta_status == "not-applicable" {
                    Value::Null
                } else {
                    Value::Bool(false)
                }
            }),
    );
    block.file_delta_status = Some(
        delta
            .file_delta
            .as_ref()
            .and_then(|file_delta| file_delta.status.clone())
            .unwrap_or_else(|| "missing".to_string()),
    );
    block.unexpected_new_file_count = Some(
        delta
            .file_delta
            .as_ref()
            .and_then(|file_delta| file_delta.summary.as_ref())
            .and_then(|summary| summary.unexpected_new)
            .unwrap_or(0),
    );
    block.planned_missing_file_count = Some(
        delta
            .file_delta
            .as_ref()
            .and_then(|file_delta| file_delta.summary.as_ref())
            .and_then(|summary| summary.planned_missing)
            .unwrap_or(0),
    );
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
