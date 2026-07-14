use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

use crate::js_ts_pre_write::{JsTsPreWriteHostTransport, JsTsPreWriteIncrementalRequest};

pub const PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-rust-pre-write-lifecycle-request.v1";
pub const JS_PRE_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-js-pre-write-lifecycle-request.v3";
pub const PRE_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION: &str = "lumin-pre-write-lifecycle-result.v1";

pub(super) const RUST_PRE_WRITE_ARTIFACT_SCHEMA_VERSION: &str = "rust-pre-write.v1";
pub(super) const RUST_PRE_WRITE_POLICY_VERSION: &str = "prewrite-token-policy-v1";
pub(super) const RUST_PRE_WRITE_PRODUCER: &str = "lumin-rust-analyzer";

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
    #[serde(default, rename = "invocationId")]
    pub advisory_invocation_id: Option<String>,
    #[serde(default)]
    pub intent_input: Option<String>,
    pub engine_selection: Value,
    #[serde(default)]
    pub generated: Option<String>,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub production: bool,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub incremental: JsTsPreWriteIncrementalRequest,
    #[serde(default)]
    pub host_evidence_transport: Option<JsTsPreWriteHostTransport>,
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
    pub rust_evidence_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_inventory_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_native_artifact_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_native_latest_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_invocation: Option<AnalyzerInvocationBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<PreWriteFailureKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreWriteFailureKind {
    OutputCleanupFailed,
    OutputWriteFailed,
    EvidenceCollectionFailed,
    ChildFailed,
    NativeArtifactInvalid,
    AdvisoryArtifactInvalid,
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
pub(super) struct RustPreWriteArtifact {
    pub(super) schema_version: String,
    pub(super) policy_version: String,
    pub(super) meta: RustPreWriteMeta,
    pub(super) intent: Value,
    pub(super) intent_warnings: Vec<Value>,
    pub(super) lookups: Vec<Value>,
    pub(super) shape_lookups: Vec<Value>,
    pub(super) file_lookups: Vec<Value>,
    pub(super) dependency_lookups: Vec<Value>,
    pub(super) inline_pattern_lookups: Vec<Value>,
    pub(super) cue_cards: Vec<Value>,
    pub(super) suppressed_cues: Vec<Value>,
    pub(super) unavailable_evidence: Vec<Value>,
    pub(super) coverage: RustPreWriteCoverage,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RustPreWriteMeta {
    pub(super) producer: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RustPreWriteCoverage {
    pub(super) names: String,
    pub(super) shapes: String,
    pub(super) files: String,
    pub(super) dependencies: String,
    pub(super) inline_patterns: String,
    pub(super) planned_type_escapes: String,
}
