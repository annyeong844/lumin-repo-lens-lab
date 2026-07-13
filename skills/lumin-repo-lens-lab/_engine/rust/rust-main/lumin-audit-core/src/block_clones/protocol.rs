use serde::Deserialize;
use serde_json::Value;

pub const BLOCK_CLONES_REQUEST_SCHEMA_VERSION: &str = "lumin-block-clones-producer-request.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockClonesRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<Value>,
    #[serde(default)]
    pub files: Vec<TokenizedFile>,
    #[serde(default)]
    pub thresholds: Option<Value>,
    #[serde(default)]
    pub incremental: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizedFile {
    pub rel_file: String,
    #[serde(default)]
    pub tokens: Vec<BlockCloneToken>,
    #[serde(default)]
    pub skipped: Option<Value>,
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    #[serde(default)]
    pub token_limit_exceeded: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockCloneToken {
    pub value: String,
    pub file: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub end_line: usize,
    #[serde(default)]
    pub container: Option<Value>,
}
