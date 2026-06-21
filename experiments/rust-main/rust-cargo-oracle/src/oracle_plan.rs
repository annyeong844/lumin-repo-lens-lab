use crate::command::CommandOutput;
use crate::metadata::CargoPackage;
use crate::protocol::{
    CargoCheckMode, OraclePlan, OraclePlanReason, OraclePlanSelectedPackage, OraclePlanStatus,
};
use crate::target_selection::TargetPackageSelection;
use std::collections::BTreeSet;

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
                target_path_examples: target_paths.into_iter().take(5).collect(),
            }
        })
        .collect::<Vec<_>>();
    let omitted_package_examples = target_selection
        .candidate_package_names
        .iter()
        .skip(selected.len())
        .take(5)
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
        target_path_count: target_selection.target_paths.len(),
        target_path_examples: target_selection
            .target_paths
            .iter()
            .take(10)
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
