use serde::Deserialize;
use serde_json::Value;

pub const FUNCTION_CLONES_REQUEST_SCHEMA_VERSION: &str =
    "lumin-function-clones-producer-request.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionClonesRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<Value>,
    pub scope: String,
    #[serde(default)]
    pub observed_at: Option<String>,
    pub file_count: usize,
    #[serde(default)]
    pub facts: Vec<Value>,
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    #[serde(default)]
    pub files_with_parse_errors: Vec<Value>,
    #[serde(default)]
    pub files_with_read_errors: Vec<Value>,
    #[serde(default)]
    pub incremental: Option<Value>,
}
