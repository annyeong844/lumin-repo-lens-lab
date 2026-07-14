use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::cache::JsTsPreWriteIncrementalRequest;

pub const JS_TS_PRE_WRITE_EVIDENCE_REQUEST_SCHEMA_VERSION: &str =
    "lumin-js-ts-pre-write-evidence-request.v1";
pub const JS_TS_PRE_WRITE_EVIDENCE_RESPONSE_SCHEMA_VERSION: &str =
    "lumin-js-ts-pre-write-evidence-response.v1";
pub const JS_TS_PRE_WRITE_HOST_TRANSPORT_SCHEMA_VERSION: &str =
    "lumin-js-ts-pre-write-host-transport.v1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsTsPreWriteEvidenceRequest {
    pub schema_version: String,
    pub root: PathBuf,
    pub evidence_artifact: String,
    pub any_inventory_artifact: String,
    pub generated: String,
    pub include_tests: bool,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub dependency_roots: Vec<String>,
    #[serde(default)]
    pub shape_type_literals: Vec<String>,
    #[serde(default)]
    pub discover_files: bool,
    #[serde(default)]
    pub files: Vec<JsTsPreWriteSourceFile>,
    #[serde(default)]
    pub incremental: JsTsPreWriteIncrementalRequest,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsTsPreWriteSourceFile {
    pub file_path: PathBuf,
    pub artifact_file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct JsTsPreWriteHostTransport {
    pub schema_version: String,
    pub command: PathBuf,
    pub root: PathBuf,
    pub output: PathBuf,
    #[serde(default)]
    pub cache_root: Option<PathBuf>,
}
