use lumin_rust_cargo_oracle::protocol::{
    CargoCheckMode, OraclePlan, OraclePlanReason, OraclePlanSelectedPackage, OraclePlanStatus,
};
use serde::Serialize;

use crate::policy::ORACLE_SCOPE_SAMPLE_LIMIT;

pub(in crate::product_artifact) fn oracle_plan_projection(
    plan: &OraclePlan,
) -> ProductOraclePlanProjection<'_> {
    ProductOraclePlanProjection::from_plan(plan)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::product_artifact) struct ProductOraclePlanProjection<'a> {
    schema_version: &'static str,
    mode: CargoCheckMode,
    status: OraclePlanStatus,
    reason: OraclePlanReason,
    target_path_count: usize,
    target_path_examples: &'a [String],
    selected_target_path_count: usize,
    omitted_target_path_count: usize,
    candidate_package_count: usize,
    selected_package_count: usize,
    selected_package_examples: Vec<ProductOraclePlanSelectedPackageProjection<'a>>,
    omitted_package_count: usize,
    omitted_package_examples: &'a [String],
    unmatched_target_path_count: usize,
    unmatched_target_path_examples: &'a [String],
}

impl<'a> ProductOraclePlanProjection<'a> {
    fn from_plan(plan: &'a OraclePlan) -> Self {
        Self {
            schema_version: plan.schema_version,
            mode: plan.mode,
            status: plan.status,
            reason: plan.reason,
            target_path_count: plan.target_path_count,
            target_path_examples: &plan.target_path_examples,
            selected_target_path_count: plan.selected_target_path_count,
            omitted_target_path_count: plan.omitted_target_path_count,
            candidate_package_count: plan.candidate_package_count,
            selected_package_count: plan.selected_package_count,
            selected_package_examples: plan
                .selected_packages
                .iter()
                .take(ORACLE_SCOPE_SAMPLE_LIMIT)
                .map(ProductOraclePlanSelectedPackageProjection::from_selected_package)
                .collect(),
            omitted_package_count: plan.omitted_package_count,
            omitted_package_examples: &plan.omitted_package_examples,
            unmatched_target_path_count: plan.unmatched_target_path_count,
            unmatched_target_path_examples: &plan.unmatched_target_paths[..plan
                .unmatched_target_paths
                .len()
                .min(ORACLE_SCOPE_SAMPLE_LIMIT)],
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductOraclePlanSelectedPackageProjection<'a> {
    package_name: &'a str,
    reason: OraclePlanReason,
    target_path_count: usize,
    target_path_examples: &'a [String],
}

impl<'a> ProductOraclePlanSelectedPackageProjection<'a> {
    fn from_selected_package(package: &'a OraclePlanSelectedPackage) -> Self {
        Self {
            package_name: &package.package_name,
            reason: package.reason,
            target_path_count: package.target_path_count,
            target_path_examples: &package.target_path_examples,
        }
    }
}
