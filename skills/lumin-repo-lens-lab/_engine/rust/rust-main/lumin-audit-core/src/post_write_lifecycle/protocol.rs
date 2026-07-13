use crate::js_ts_pre_write::JsTsPreWriteIncrementalRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

pub const POST_WRITE_LIFECYCLE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-post-write-lifecycle-request.v2";
pub const POST_WRITE_LIFECYCLE_RESULT_SCHEMA_VERSION: &str = "lumin-post-write-lifecycle-result.v2";
pub const POST_WRITE_DELTA_SCHEMA_VERSION: &str = "lumin-post-write-delta.v1";

pub const CANONICAL_ESCAPE_KINDS: &[&str] = &[
    "explicit-any",
    "as-any",
    "angle-any",
    "as-unknown-as-T",
    "rest-any-args",
    "index-sig-any",
    "generic-default-any",
    "ts-ignore",
    "ts-expect-error",
    "no-explicit-any-disable",
    "jsdoc-any",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PostWriteLifecycleRequest {
    pub schema_version: String,
    pub root: PathBuf,
    pub output: PathBuf,
    #[serde(default)]
    pub advisory_path: Option<PathBuf>,
    #[serde(default)]
    pub delta_out: Option<PathBuf>,
    pub delta_invocation_id: String,
    pub generated: String,
    pub include_tests: bool,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub incremental: JsTsPreWriteIncrementalRequest,
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
    EvidenceFailed,
    DeltaArtifactInvalid,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreWriteAdvisory {
    pub invocation_id: String,
    #[serde(default)]
    pub intent_hash: String,
    #[serde(default)]
    pub intent: AdvisoryIntent,
    #[serde(default)]
    pub pre_write: AdvisoryPreWrite,
    #[serde(default)]
    pub scan_range: AdvisoryScanRange,
    #[serde(default)]
    pub capabilities: AdvisoryCapabilities,
    #[serde(default)]
    pub rust_pre_write: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisoryIntent {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub planned_type_escapes: Vec<PlannedTypeEscape>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedTypeEscape {
    pub escape_kind: String,
    pub location_hint: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_shape: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alternative_considered: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisoryPreWrite {
    #[serde(default)]
    pub any_inventory_path: Option<String>,
    #[serde(default)]
    pub file_inventory: AdvisoryFileInventory,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdvisoryFileInventory {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AdvisoryScanRange {
    #[serde(default)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisoryCapabilities {
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub post_write_type_escapes: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyInventory {
    #[serde(default)]
    pub meta: AnyInventoryMeta,
    #[serde(default)]
    pub type_escapes: Vec<TypeEscapeOccurrence>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyInventoryMeta {
    #[serde(default)]
    pub complete: bool,
    #[serde(default)]
    pub scope: Option<Value>,
    #[serde(default)]
    pub include_tests: Option<bool>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub files_with_parse_errors: Vec<InventoryParseError>,
    #[serde(default)]
    pub supports: AnyInventorySupports,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyInventorySupports {
    #[serde(default)]
    pub type_escapes: bool,
    #[serde(default)]
    pub escape_kinds: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InventoryParseError {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub line: Value,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeEscapeOccurrence {
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
    #[serde(default)]
    pub escape_kind: Option<String>,
    #[serde(default)]
    pub code_shape: Option<String>,
    #[serde(default)]
    pub normalized_code_shape: Option<String>,
    #[serde(default)]
    pub inside_exported_identity: Option<String>,
    #[serde(default)]
    pub occurrence_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusBlock {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mismatch_detail: Option<String>,
}

impl StatusBlock {
    pub fn status(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            source: None,
            reason: None,
            mismatch_detail: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CapabilityFailure {
    pub kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryCompleteness {
    pub after_complete: Option<bool>,
    pub before_complete: Option<bool>,
    pub files_with_parse_errors: Vec<SidedParseError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SidedParseError {
    pub side: String,
    pub file: String,
    pub message: String,
    pub line: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaEntry {
    pub label: String,
    pub escape_kind: Option<String>,
    pub file: Option<String>,
    pub line: Option<u64>,
    pub code_shape: Option<String>,
    pub normalized_code_shape: Option<String>,
    pub inside_exported_identity: Option<String>,
    pub occurrence_key: Option<String>,
    pub planned_entry: Option<PlannedTypeEscape>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaSummary {
    pub planned: u64,
    pub planned_not_observed: u64,
    pub silent_new: u64,
    pub pre_existing: u64,
    pub removed: u64,
    pub observed_unbaselined: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDelta {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub planned_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_new: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unexpected_new: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_observed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_missing: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<FileDeltaSummary>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDeltaSummary {
    pub new_files: u64,
    pub removed: u64,
    pub planned_new: u64,
    pub unexpected_new: u64,
    pub planned_observed: u64,
    pub planned_missing: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostWriteDeltaArtifact {
    pub schema_version: String,
    pub pre_write_invocation_id: String,
    pub delta_invocation_id: String,
    pub intent_hash: String,
    pub baseline: StatusBlock,
    pub capability_parity: StatusBlock,
    pub scan_range_parity: StatusBlock,
    pub inventory_completeness: InventoryCompleteness,
    pub type_escape_delta: StatusBlock,
    pub entries: Vec<DeltaEntry>,
    pub summary: DeltaSummary,
    pub capability_failures: Vec<CapabilityFailure>,
    pub file_delta: FileDelta,
}
