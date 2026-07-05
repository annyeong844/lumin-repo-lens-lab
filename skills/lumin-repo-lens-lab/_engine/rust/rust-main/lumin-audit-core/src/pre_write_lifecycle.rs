use anyhow::{bail, Context, Result};
use lumin_rust_common::{atomic_write_json_pretty, sha256_text};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::source_commit::git_head_commit_or_unknown;

pub const PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-rust-pre-write-lifecycle-request.v1";
pub const JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-js-pre-write-lifecycle-request.v1";
pub const PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION: &str = "lumin-pre-write-lifecycle-result.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteLifecycleRequest {
    pub schema_version: String,
    pub root: PathBuf,
    pub output: PathBuf,
    #[serde(default)]
    pub source_commit: Option<String>,
    #[serde(rename = "invocationId")]
    pub advisory_invocation_id: String,
    pub rust_native_artifact_path: PathBuf,
    pub rust_native_latest_path: PathBuf,
    #[serde(rename = "analyzer")]
    pub analyzer_invocation: AnalyzerInvocationRequest,
    pub intent_input: String,
    pub include_tests: bool,
    #[serde(default)]
    pub production: bool,
    #[serde(default)]
    pub excludes: Vec<String>,
    pub engine_selection: Value,
    pub file_inventory: Value,
    #[serde(default)]
    pub failures: Vec<Value>,
}

pub type RustPreWriteLifecycleRequest = PreWriteLifecycleRequest;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsPreWriteLifecycleRequest {
    pub schema_version: String,
    pub root: PathBuf,
    pub output: PathBuf,
    pub scripts_dir: PathBuf,
    pub node_executable: String,
    pub child_intent_flag: String,
    #[serde(default)]
    pub child_intent_input: Option<String>,
    pub engine_selection: Value,
    #[serde(default)]
    pub no_fresh_audit: bool,
    #[serde(default)]
    pub scan_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerInvocationRequest {
    pub command: String,
    #[serde(default)]
    pub prefix_args: Vec<String>,
    pub source: String,
    #[serde(default)]
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteLifecycleResult {
    pub schema_version: &'static str,
    pub block: PreWriteBlock,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteBlock {
    pub requested: bool,
    pub ran: bool,
    pub execution_owner: &'static str,
    pub engine: &'static str,
    pub language: &'static str,
    pub producer: &'static str,
    pub engine_selection: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_advisory_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advisory_invocation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_availability: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_native_artifact_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_native_latest_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<AnalyzerInvocationBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerInvocationBlock {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RustPreWriteArtifact {
    #[serde(default)]
    intent: Option<Value>,
    #[serde(default)]
    intent_warnings: Vec<Value>,
    #[serde(default)]
    lookups: Vec<Value>,
    #[serde(default)]
    shape_lookups: Vec<Value>,
    #[serde(default)]
    file_lookups: Vec<Value>,
    #[serde(default)]
    dependency_lookups: Vec<Value>,
    #[serde(default)]
    inline_pattern_lookups: Vec<Value>,
    #[serde(default)]
    cue_cards: Vec<Value>,
    #[serde(default)]
    suppressed_cues: Vec<Value>,
    #[serde(default)]
    unavailable_evidence: Vec<Value>,
    #[serde(default)]
    schema_version: Option<Value>,
    #[serde(default)]
    policy_version: Option<Value>,
    #[serde(default)]
    meta: Option<RustPreWriteMeta>,
    #[serde(default)]
    coverage: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RustPreWriteMeta {
    #[serde(default)]
    producer: Option<String>,
}

pub fn execute_pre_write_lifecycle(
    request: PreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_pre_write_lifecycle_with_stdio(request, ChildStdio::Capture)
}

pub fn execute_rust_pre_write_lifecycle(
    request: RustPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_pre_write_lifecycle(request)
}

pub fn execute_pre_write_lifecycle_streaming(
    request: PreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_pre_write_lifecycle_with_stdio(request, ChildStdio::Inherit)
}

pub fn execute_rust_pre_write_lifecycle_streaming(
    request: RustPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_pre_write_lifecycle_streaming(request)
}

pub fn execute_js_pre_write_lifecycle(
    request: JsPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_js_pre_write_lifecycle_with_stdio(request, ChildStdio::Capture)
}

pub fn execute_js_pre_write_lifecycle_streaming(
    request: JsPreWriteLifecycleRequest,
) -> Result<PreWriteLifecycleResult> {
    execute_js_pre_write_lifecycle_with_stdio(request, ChildStdio::Inherit)
}

fn execute_js_pre_write_lifecycle_with_stdio(
    request: JsPreWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> Result<PreWriteLifecycleResult> {
    validate_js_request(&request)?;
    let child = run_js_pre_write_child(&request, child_stdio);
    if !child.status_success {
        return Ok(PreWriteLifecycleResult {
            schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
            block: js_failure_block(
                &request,
                format!("pre-write.mjs exited non-zero: {}", child.reason),
            ),
            exit_code: child.exit_code.unwrap_or(1),
            stdout: nonempty(child.stdout),
            stderr: nonempty(child.stderr),
        });
    }

    let latest_advisory_path = advisory_latest_path(&request.output);
    let advisory = read_js_pre_write_advisory(&latest_advisory_path)?;
    let advisory_invocation_id = advisory
        .get("invocationId")
        .and_then(Value::as_str)
        .map(str::to_string);
    let advisory_path = js_advisory_path(&request.output, &advisory);
    let evidence_availability = advisory
        .get("evidenceAvailability")
        .cloned()
        .unwrap_or(Value::Null);

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
            advisory_invocation_id,
            evidence_availability: Some(evidence_availability),
            rust_native_artifact_path: None,
            rust_native_latest_path: None,
            source_commit: None,
            analyzer_invocation: None,
            reason: None,
        },
        exit_code: 0,
        stdout: nonempty(child.stdout),
        stderr: nonempty(child.stderr),
    })
}

fn execute_pre_write_lifecycle_with_stdio(
    request: PreWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> Result<PreWriteLifecycleResult> {
    validate_request(&request)?;
    let source_commit = effective_source_commit(&request);
    let child = run_rust_pre_write_child(&request, &source_commit, child_stdio);
    if !child.status_success {
        return Ok(PreWriteLifecycleResult {
            schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
            block: failure_block(
                &request,
                format!(
                    "lumin-rust-analyzer pre-write exited non-zero: {}",
                    child.reason
                ),
            ),
            exit_code: child.exit_code.unwrap_or(1),
            stdout: nonempty(child.stdout),
            stderr: nonempty(child.stderr),
        });
    }

    let rust_artifact = match read_rust_pre_write_artifact(&request.rust_native_artifact_path) {
        Ok(artifact) => artifact,
        Err(error) => {
            return Ok(PreWriteLifecycleResult {
                schema_version: PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION,
                block: failure_block(&request, error.to_string()),
                exit_code: 1,
                stdout: nonempty(child.stdout),
                stderr: nonempty(child.stderr),
            });
        }
    };

    copy_rust_native_latest(&request)?;
    let advisory = build_rust_pre_write_advisory(&request, &rust_artifact, &source_commit);
    let written = write_advisory(&request.output, &advisory)?;

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
            rust_native_artifact_path: Some(path_string(&request.rust_native_artifact_path)),
            rust_native_latest_path: Some(path_string(&request.rust_native_latest_path)),
            source_commit: Some(source_commit),
            analyzer_invocation: Some(analyzer_invocation_block(&request)),
            reason: None,
        },
        exit_code: 0,
        stdout: nonempty(child.stdout),
        stderr: nonempty(child.stderr),
    })
}

fn validate_js_request(request: &JsPreWriteLifecycleRequest) -> Result<()> {
    if request.schema_version != JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION {
        bail!(
            "execute-js-pre-write: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    validate_nonempty_path_for("execute-js-pre-write", "root", &request.root)?;
    validate_nonempty_path_for("execute-js-pre-write", "output", &request.output)?;
    validate_nonempty_path_for("execute-js-pre-write", "scriptsDir", &request.scripts_dir)?;
    if request.node_executable.trim().is_empty() {
        bail!("execute-js-pre-write: nodeExecutable must be a non-empty string");
    }
    if request.child_intent_flag.trim().is_empty() {
        bail!("execute-js-pre-write: childIntentFlag must be a non-empty string");
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildStdio {
    Capture,
    Inherit,
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
    validate_nonempty_path_for("execute-pre-write", field, path)
}

fn validate_nonempty_path_for(label: &str, field: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        bail!("{label}: {field} must be provided");
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

fn run_rust_pre_write_child(
    request: &PreWriteLifecycleRequest,
    source_commit: &str,
    child_stdio: ChildStdio,
) -> ChildOutput {
    let mut args = request.analyzer_invocation.prefix_args.clone();
    args.extend([
        "pre-write".to_string(),
        "--root".to_string(),
        path_string(&request.root),
        "--source-commit".to_string(),
        source_commit.to_string(),
        "--intent".to_string(),
        "-".to_string(),
        "--output".to_string(),
        path_string(&request.rust_native_artifact_path),
    ]);
    if !request.include_tests {
        args.push("--production".to_string());
    }
    for pattern in &request.excludes {
        args.extend(["--exclude".to_string(), pattern.clone()]);
    }

    match child_stdio {
        ChildStdio::Capture => run_child_capture(
            "lumin-rust-analyzer pre-write",
            &request.analyzer_invocation.command,
            &args,
            &request.intent_input,
        ),
        ChildStdio::Inherit => run_child_inherit(
            "lumin-rust-analyzer pre-write",
            &request.analyzer_invocation.command,
            &args,
            &request.intent_input,
        ),
    }
}

fn run_js_pre_write_child(
    request: &JsPreWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> ChildOutput {
    let mut args = vec![
        path_string(&request.scripts_dir.join("pre-write.mjs")),
        "--root".to_string(),
        path_string(&request.root),
        "--output".to_string(),
        path_string(&request.output),
        "--intent".to_string(),
        request.child_intent_flag.clone(),
    ];
    args.extend(request.scan_args.clone());
    if request.no_fresh_audit {
        args.push("--no-fresh-audit".to_string());
    }
    let input = request.child_intent_input.as_deref().unwrap_or("");
    match child_stdio {
        ChildStdio::Capture => {
            run_child_capture("pre-write.mjs", &request.node_executable, &args, input)
        }
        ChildStdio::Inherit => {
            run_child_inherit("pre-write.mjs", &request.node_executable, &args, input)
        }
    }
}

fn run_child_capture(label: &str, command: &str, args: &[String], input: &str) -> ChildOutput {
    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return child_start_error(error),
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(input.as_bytes()) {
            return ChildOutput {
                status_success: false,
                exit_code: Some(1),
                reason: format!("failed to write intent stdin: {error}"),
                stdout: String::new(),
                stderr: String::new(),
            };
        }
    }
    match child.wait_with_output() {
        Ok(output) => child_output(
            label,
            output.status.success(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ),
        Err(error) => child_start_error(error),
    }
}

fn run_child_inherit(label: &str, command: &str, args: &[String], input: &str) -> ChildOutput {
    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return child_start_error(error),
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(input.as_bytes()) {
            return ChildOutput {
                status_success: false,
                exit_code: Some(1),
                reason: format!("failed to write intent stdin: {error}"),
                stdout: String::new(),
                stderr: String::new(),
            };
        }
    }
    match child.wait() {
        Ok(status) => child_output(
            label,
            status.success(),
            status.code(),
            String::new(),
            String::new(),
        ),
        Err(error) => child_start_error(error),
    }
}

fn child_start_error(error: std::io::Error) -> ChildOutput {
    ChildOutput {
        status_success: false,
        exit_code: Some(1),
        reason: error.to_string(),
        stdout: String::new(),
        stderr: String::new(),
    }
}

fn child_output(
    label: &str,
    status_success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
) -> ChildOutput {
    let reason = exit_code
        .map(|code| format!("{label} exited {code}"))
        .unwrap_or_else(|| format!("{label} terminated by signal"));
    ChildOutput {
        status_success,
        exit_code,
        reason,
        stdout,
        stderr,
    }
}

fn read_js_pre_write_advisory(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "js pre-write advisory parse failed: failed to read {}",
            path.display()
        )
    })?;
    serde_json::from_str::<Value>(&text).with_context(|| {
        format!(
            "js pre-write advisory parse failed: invalid JSON in {}",
            path.display()
        )
    })
}

fn js_advisory_path(output: &Path, advisory: &Value) -> PathBuf {
    if let Some(path) = advisory
        .get("artifactPaths")
        .and_then(|paths| paths.get("invocationSpecific"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        return resolve_process_relative_path(path);
    }
    if let Some(invocation_id) = advisory
        .get("invocationId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        return advisory_specific_path(output, invocation_id);
    }
    advisory_latest_path(output)
}

fn resolve_process_relative_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn read_rust_pre_write_artifact(path: &Path) -> Result<RustPreWriteArtifact> {
    let text = fs::read_to_string(path).with_context(|| {
        format!(
            "rust pre-write artifact parse failed: failed to read {}",
            path.display()
        )
    })?;
    serde_json::from_str::<RustPreWriteArtifact>(&text).with_context(|| {
        format!(
            "rust pre-write artifact parse failed: invalid JSON in {}",
            path.display()
        )
    })
}

fn copy_rust_native_latest(request: &PreWriteLifecycleRequest) -> Result<()> {
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

fn build_rust_pre_write_advisory(
    request: &PreWriteLifecycleRequest,
    rust_artifact: &RustPreWriteArtifact,
    source_commit: &str,
) -> Value {
    let intent = rust_intent(rust_artifact.intent.as_ref());
    let producer = rust_artifact
        .meta
        .as_ref()
        .and_then(|meta| meta.producer.clone())
        .unwrap_or_else(|| "lumin-rust-analyzer".to_string());
    serde_json::json!({
        "invocationId": request.advisory_invocation_id,
        "intentHash": hash_intent(&intent),
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
            "producer": producer,
            "coverage": rust_artifact.coverage.clone(),
        },
        "capabilities": {
            "language": "rust",
            "producer": "lumin-rust-analyzer",
            "postWriteTypeEscapes": "not-applicable",
        },
        "failures": request.failures.clone(),
    })
}

fn rust_intent(intent: Option<&Value>) -> Value {
    let mut object = intent
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_else(Map::new);
    object.insert("language".to_string(), Value::String("rust".to_string()));
    Value::Object(object)
}

fn hash_intent(intent: &Value) -> String {
    let normalized = sorted_json_value(intent);
    let text = serde_json::to_string(&normalized).unwrap_or_else(|_| "{}".to_string());
    let digest = sha256_text(&text);
    digest
        .strip_prefix("sha256:")
        .unwrap_or(digest.as_str())
        .to_string()
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

struct AdvisoryWriteResult {
    latest_path: PathBuf,
    specific_path: PathBuf,
}

fn write_advisory(output: &Path, advisory: &Value) -> Result<AdvisoryWriteResult> {
    let invocation_id = advisory
        .get("invocationId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .context("writeAdvisory: advisory.invocationId is required")?;
    let latest_path = advisory_latest_path(output);
    let specific_path = advisory_specific_path(output, invocation_id);
    atomic_write_json_pretty(&specific_path, advisory)
        .with_context(|| format!("writeAdvisory: failed to write {}", specific_path.display()))?;
    atomic_write_json_pretty(&latest_path, advisory)
        .with_context(|| format!("writeAdvisory: failed to write {}", latest_path.display()))?;
    Ok(AdvisoryWriteResult {
        latest_path,
        specific_path,
    })
}

fn advisory_latest_path(output: &Path) -> PathBuf {
    output.join("pre-write-advisory.latest.json")
}

fn advisory_specific_path(output: &Path, invocation_id: &str) -> PathBuf {
    output.join(format!("pre-write-advisory.{invocation_id}.json"))
}

fn failure_block(request: &PreWriteLifecycleRequest, reason: String) -> PreWriteBlock {
    PreWriteBlock {
        requested: true,
        ran: false,
        execution_owner: "lumin-audit-core",
        engine: "rust",
        language: "rust",
        producer: "lumin-rust-analyzer",
        engine_selection: request.engine_selection.clone(),
        advisory_path: None,
        latest_advisory_path: None,
        advisory_invocation_id: None,
        evidence_availability: None,
        rust_native_artifact_path: None,
        rust_native_latest_path: None,
        source_commit: None,
        analyzer_invocation: None,
        reason: Some(reason),
    }
}

fn js_failure_block(request: &JsPreWriteLifecycleRequest, reason: String) -> PreWriteBlock {
    PreWriteBlock {
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
        evidence_availability: None,
        rust_native_artifact_path: None,
        rust_native_latest_path: None,
        source_commit: None,
        analyzer_invocation: None,
        reason: Some(reason),
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

fn nonempty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
