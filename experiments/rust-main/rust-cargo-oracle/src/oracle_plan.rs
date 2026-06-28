use crate::command::CommandOutput;
use crate::metadata::CargoPackage;
use crate::protocol::{
    CargoCheckMode, OraclePlan, OraclePlanReason, OraclePlanSampleLimits,
    OraclePlanSelectedPackage, OraclePlanStatus,
};
use crate::target_selection::TargetPackageSelection;
use std::collections::BTreeSet;

pub(crate) const ORACLE_PLAN_TARGET_PATH_EXAMPLE_LIMIT: usize = 10;
pub(crate) const ORACLE_PLAN_OMITTED_PACKAGE_EXAMPLE_LIMIT: usize = 5;
pub(crate) const ORACLE_PLAN_SELECTED_PACKAGE_TARGET_PATH_EXAMPLE_LIMIT: usize = 5;

pub(crate) fn oracle_plan(
    mode: CargoCheckMode,
    target_selection: &TargetPackageSelection,
    selected: &[CargoPackage],
    check_output: &CommandOutput,
) -> OraclePlan {
    let status = if mode == CargoCheckMode::MetadataOnly || check_output.status.is_none() {
        OraclePlanStatus::NotRun
    } else {
        OraclePlanStatus::Ran
    };
    let reason = match mode {
        CargoCheckMode::MetadataOnly => OraclePlanReason::MetadataOnlyFastPath,
        CargoCheckMode::CargoCheck => OraclePlanReason::ExplicitCargoCheckMode,
        CargoCheckMode::TargetedCargoCheck if selected.is_empty() => {
            OraclePlanReason::TargetedCargoCheckSelectedNoPackages
        }
        CargoCheckMode::TargetedCargoCheck => OraclePlanReason::ReviewSyntaxEvidencePackageScope,
    };
    let selected_packages = selected
        .iter()
        .map(|pkg| {
            let target_paths = target_selection
                .paths_by_package
                .get(&pkg.name)
                .cloned()
                .unwrap_or_default();
            OraclePlanSelectedPackage {
                package_name: pkg.name.clone(),
                package_id: pkg.id.clone(),
                manifest_path: pkg.manifest_path.clone(),
                reason,
                target_path_count: target_paths.len(),
                target_path_examples: target_paths
                    .into_iter()
                    .take(ORACLE_PLAN_SELECTED_PACKAGE_TARGET_PATH_EXAMPLE_LIMIT)
                    .collect(),
            }
        })
        .collect::<Vec<_>>();
    let omitted_package_examples = target_selection
        .candidate_package_names
        .iter()
        .skip(selected.len())
        .take(ORACLE_PLAN_OMITTED_PACKAGE_EXAMPLE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();
    let selected_package_names = selected
        .iter()
        .map(|pkg| pkg.name.as_str())
        .collect::<BTreeSet<_>>();
    let selected_target_path_count = target_selection
        .paths_by_package
        .iter()
        .filter(|(name, _)| selected_package_names.contains(name.as_str()))
        .map(|(_, paths)| paths.len())
        .sum();
    let omitted_target_path_count = target_selection
        .paths_by_package
        .iter()
        .filter(|(name, _)| !selected_package_names.contains(name.as_str()))
        .map(|(_, paths)| paths.len())
        .sum();

    OraclePlan {
        schema_version: "rust-oracle-plan.v1",
        mode,
        status,
        reason,
        sample_limits: OraclePlanSampleLimits {
            target_path_examples: ORACLE_PLAN_TARGET_PATH_EXAMPLE_LIMIT,
            omitted_package_examples: ORACLE_PLAN_OMITTED_PACKAGE_EXAMPLE_LIMIT,
            selected_package_target_path_examples:
                ORACLE_PLAN_SELECTED_PACKAGE_TARGET_PATH_EXAMPLE_LIMIT,
        },
        target_path_count: target_selection.target_paths.len(),
        target_path_examples: target_selection
            .target_paths
            .iter()
            .take(ORACLE_PLAN_TARGET_PATH_EXAMPLE_LIMIT)
            .cloned()
            .collect(),
        selected_target_path_count,
        omitted_target_path_count,
        candidate_package_count: target_selection.candidate_package_names.len(),
        selected_package_count: selected.len(),
        selected_packages,
        omitted_package_count: target_selection
            .candidate_package_names
            .len()
            .saturating_sub(selected.len()),
        omitted_package_examples,
        unmatched_target_paths: target_selection.unmatched_paths.clone(),
        unmatched_target_path_count: target_selection.unmatched_paths.len(),
    }
}
