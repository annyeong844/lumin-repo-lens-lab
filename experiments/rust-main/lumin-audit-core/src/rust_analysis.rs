use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustAnalysisSummary {
    pub artifact: &'static str,
    pub status: String,
    pub available: bool,
}

pub fn summarize_rust_analysis_artifact(
    _root: &Path,
    _artifact: &Value,
) -> Option<RustAnalysisSummary> {
    None
}
