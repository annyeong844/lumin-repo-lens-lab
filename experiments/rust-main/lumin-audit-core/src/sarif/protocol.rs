use serde::Deserialize;
use serde_json::Value;

pub const SARIF_REQUEST_SCHEMA_VERSION: &str = "lumin-sarif-producer-request.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SarifRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default)]
    pub generated: Option<String>,
    #[serde(default)]
    pub fix_plan: Option<Value>,
    #[serde(default)]
    pub runtime_evidence: Option<Value>,
    #[serde(default)]
    pub staleness: Option<Value>,
    #[serde(default)]
    pub dead_classify: Option<Value>,
    #[serde(default)]
    pub symbols: Option<Value>,
    #[serde(default)]
    pub topology: Option<Value>,
    #[serde(default)]
    pub discipline: Option<Value>,
    #[serde(default)]
    pub barrels: Option<Value>,
}
