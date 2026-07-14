use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const AUDIT_SUMMARY_RENDER_REQUEST_SCHEMA_VERSION: &str =
    "lumin-audit-summary-render-request.v1";
pub const AUDIT_SUMMARY_RENDER_RESULT_SCHEMA_VERSION: &str = "lumin-audit-summary-render-result.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummaryRenderRequest {
    pub schema_version: String,
    #[serde(default)]
    pub manifest: Value,
    #[serde(default)]
    pub checklist_facts: Value,
    #[serde(default)]
    pub fix_plan: Value,
    #[serde(default)]
    pub topology: Value,
    #[serde(default)]
    pub discipline: Value,
    #[serde(default)]
    pub call_graph: Value,
    #[serde(default)]
    pub function_clones: Value,
    #[serde(default)]
    pub symbols: Value,
    #[serde(default)]
    pub module_reachability: Value,
    pub output_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummaryRenderResult {
    pub schema_version: &'static str,
    pub path: String,
    pub bytes: usize,
    pub preview: Option<String>,
}
