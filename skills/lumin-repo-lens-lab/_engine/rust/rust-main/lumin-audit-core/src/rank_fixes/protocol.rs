use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const RANK_FIXES_REQUEST_SCHEMA_VERSION: &str = "lumin-rank-fixes-producer-request.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesRequest {
    pub schema_version: String,
    pub root: String,
    pub generated: String,
    pub artifacts: RankFixesArtifacts,
    #[serde(default)]
    pub public_deep_import_risk_by_file: BTreeMap<String, PublicDeepImportRisk>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesArtifacts {
    pub dead_classify: Value,
    #[serde(default)]
    pub runtime_evidence: Option<Value>,
    #[serde(default)]
    pub staleness: Option<Value>,
    #[serde(default)]
    pub symbols: Option<Value>,
    #[serde(default)]
    pub export_action_safety: Option<Value>,
    #[serde(default)]
    pub call_graph: Option<Value>,
    #[serde(default)]
    pub entry_surface: Option<Value>,
    #[serde(default)]
    pub module_reachability: Option<Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDeepImportRisk {
    #[serde(default)]
    pub risk: Option<bool>,
    #[serde(flatten)]
    pub detail: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankFixesArtifact {
    pub meta: Value,
    pub summary: Value,
    pub safe_fixes: Vec<Value>,
    pub safe_fix_groups: Vec<Value>,
    pub review_fixes: Vec<Value>,
    pub degraded: Vec<Value>,
    pub muted: Vec<Value>,
}
