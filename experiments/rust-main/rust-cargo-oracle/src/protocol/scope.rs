use serde::Serialize;

use super::{
    CargoTargetKind, OracleCfgSetSource, OracleScopeKind, OracleScopeProfile,
    OracleScopeTargetSource, OracleTargetTripleSource,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleScope {
    pub kind: OracleScopeKind,
    pub package: String,
    pub package_names: Vec<String>,
    pub target: String,
    pub targets: Vec<OracleScopeTarget>,
    pub feature_set: Vec<String>,
    pub feature_selection: OracleScopeFeatureSelection,
    pub target_triple: String,
    pub target_triples: Vec<String>,
    pub target_triple_source: OracleTargetTripleSource,
    pub cfg_set: Vec<String>,
    pub cfg_set_source: OracleCfgSetSource,
    pub cfg_set_complete: bool,
    pub profile: OracleScopeProfile,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleScopeFeatureSelection {
    pub default_features: bool,
    pub all_features: bool,
    pub explicit_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleScopeTarget {
    pub package_id: String,
    pub package_name: Option<String>,
    pub target_name: String,
    pub target_kinds: Vec<CargoTargetKind>,
    pub source: OracleScopeTargetSource,
}
