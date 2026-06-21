use serde::Serialize;

use super::{CargoCheckMode, OraclePlanReason, OraclePlanStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OraclePlan {
    pub schema_version: &'static str,
    pub mode: CargoCheckMode,
    pub status: OraclePlanStatus,
    pub reason: OraclePlanReason,
    pub target_path_count: usize,
    pub target_path_examples: Vec<String>,
    pub selected_target_path_count: usize,
    pub omitted_target_path_count: usize,
    pub candidate_package_count: usize,
    pub selected_package_count: usize,
    pub selected_packages: Vec<OraclePlanSelectedPackage>,
    pub omitted_package_count: usize,
    pub omitted_package_examples: Vec<String>,
    pub unmatched_target_paths: Vec<String>,
    pub unmatched_target_path_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OraclePlanSelectedPackage {
    pub package_name: String,
    pub package_id: String,
    pub manifest_path: String,
    pub reason: OraclePlanReason,
    pub target_path_count: usize,
    pub target_path_examples: Vec<String>,
}
