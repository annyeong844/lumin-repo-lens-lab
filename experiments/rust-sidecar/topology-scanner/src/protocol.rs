use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub root: String,
    pub files: Vec<String>,
    #[serde(rename = "policyVersion")]
    pub policy_version: String,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "policyVersion")]
    pub policy_version: String,
    pub files: Vec<FileScanResult>,
    pub timing: Timing,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
pub struct ModuleEdge {
    pub source: String,
    pub line: usize,
    #[serde(rename = "typeOnly")]
    pub type_only: bool,
    #[serde(rename = "reExport")]
    pub re_export: bool,
    pub dynamic: bool,
}

#[derive(Debug, Serialize)]
pub struct FileScanResult {
    pub file: String,
    pub ok: bool,
    pub loc: usize,
    pub edges: Vec<ModuleEdge>,
    pub risk: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Timing {
    pub files: usize,
    #[serde(rename = "elapsedMs")]
    pub elapsed_ms: u128,
}
