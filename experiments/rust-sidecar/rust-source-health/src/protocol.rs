use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;
pub const POLICY_VERSION: &str = "m6-rust-source-health-syntax-v1";
pub const PARSER_KIND: &str = "ra_ap_syntax";
pub const PARSER_VERSION: &str = "0.0.337";
pub const PARSER_EDITION: &str = "2021";
pub const PARSER_EDITION_POLICY: &str = "fixed";
pub const PARSER_EDITION_SOURCE: &str = "m6-policy-default";
pub const DEFAULT_WORKER_STACK_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthRequest {
    pub schema_version: u32,
    pub root: String,
    pub files: Vec<RequestFile>,
    pub path_policy: PathPolicy,
    pub parser: ParserRequest,
    #[serde(default)]
    pub runtime: RuntimeRequest,
}

#[derive(Debug, Deserialize)]
pub struct RequestFile {
    pub path: String,
    pub sha256: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PathPolicy {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserRequest {
    pub edition_policy: String,
    pub edition: String,
    pub edition_source: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRequest {
    #[serde(default)]
    pub thread_count: Option<usize>,
    #[serde(default = "default_worker_stack_bytes")]
    pub worker_stack_bytes: usize,
}

impl Default for RuntimeRequest {
    fn default() -> Self {
        Self {
            thread_count: None,
            worker_stack_bytes: DEFAULT_WORKER_STACK_BYTES,
        }
    }
}

fn default_worker_stack_bytes() -> usize {
    DEFAULT_WORKER_STACK_BYTES
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub schema_version: u32,
    pub meta: ResponseMeta,
    pub summary: Summary,
    pub skipped_files: Vec<SkippedFile>,
    pub files: BTreeMap<String, FileHealth>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    pub producer: String,
    pub mode: String,
    pub parser: ParserMeta,
    pub policy: PolicyMeta,
    pub runtime: RuntimeMeta,
    pub limits: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserMeta {
    pub kind: String,
    pub version: String,
    pub edition_policy: String,
    pub edition: String,
    pub edition_source: String,
}

#[derive(Debug, Serialize)]
pub struct PolicyMeta {
    pub version: String,
    pub thresholds: Thresholds,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Thresholds {
    pub max_function_lines: usize,
    pub max_impl_lines: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMeta {
    pub thread_count: usize,
    pub worker_stack_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHealth {
    pub sha256: String,
    pub facts: Facts,
    pub signals: Vec<Signal>,
    pub parse: ParseStatus,
    pub path: PathMeta,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Facts {
    pub items: usize,
    pub functions: usize,
    pub max_function_lines: usize,
    pub unsafe_blocks: usize,
    pub unsafe_functions: usize,
}

#[derive(Debug, Serialize)]
pub struct Signal {
    pub kind: String,
    pub severity: String,
    pub claim: String,
    pub location: Location,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParseStatus {
    pub ok: bool,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Serialize)]
pub struct ParseError {
    pub message: String,
    pub claim: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Serialize)]
pub struct PathMeta {
    pub classifications: Vec<String>,
    pub suppressed: bool,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub files: usize,
    pub skipped_files: usize,
    pub parse_error_files: usize,
    pub parse_errors: usize,
    pub functions: usize,
    pub unsafe_blocks: usize,
    pub unsafe_functions: usize,
    pub signals: usize,
    pub signals_by_kind: BTreeMap<String, usize>,
}
