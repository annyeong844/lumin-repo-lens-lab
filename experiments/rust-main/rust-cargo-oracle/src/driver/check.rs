use anyhow::Result;
use std::path::Path;

use crate::classify::{parse_cargo_jsonl, skipped_cargo_jsonl, ParsedJsonl};
use crate::command::{
    cargo_check_args, cargo_check_args_for_packages, run_cargo_check, run_cargo_check_for_packages,
    skipped_command_output, skipped_command_output_with_reason, CommandOutput, CommandSkipReason,
};
use crate::metadata::{selected_packages, CargoMetadata, CargoPackage};
use crate::options::OracleOptions;
use crate::protocol::CargoCheckMode;
use crate::target_selection::TargetPackageSelection;

pub(super) struct CargoCheckPhase {
    pub(super) cargo_args: Vec<String>,
    pub(super) selected_packages: Vec<CargoPackage>,
    pub(super) output: CommandOutput,
    pub(super) parsed: ParsedJsonl,
}

pub(super) fn run_cargo_check_phase(
    root: &Path,
    options: &OracleOptions,
    metadata: Option<&CargoMetadata>,
    target_selection: &TargetPackageSelection,
    cargo_target_dir: &Path,
) -> Result<CargoCheckPhase> {
    let run = cargo_check_output(root, options, metadata, target_selection, cargo_target_dir)?;
    let parsed = if run.output.status.is_none() {
        skipped_cargo_jsonl()
    } else {
        parse_cargo_jsonl(&run.output.stdout)
    };

    Ok(CargoCheckPhase {
        cargo_args: run.cargo_args,
        selected_packages: run.selected_packages,
        output: run.output,
        parsed,
    })
}

struct CargoCheckRun {
    cargo_args: Vec<String>,
    selected_packages: Vec<CargoPackage>,
    output: CommandOutput,
}

fn cargo_check_output(
    root: &Path,
    options: &OracleOptions,
    metadata: Option<&CargoMetadata>,
    target_selection: &TargetPackageSelection,
    cargo_target_dir: &Path,
) -> Result<CargoCheckRun> {
    match options.cargo_check_mode {
        CargoCheckMode::CargoCheck => {
            let cargo_args =
                cargo_check_args(options.features.as_deref(), options.package_name.as_deref());
            let selected_packages =
                selected_packages(metadata, options.package_name.as_deref(), root);
            let output = run_cargo_check(root, options, cargo_target_dir)?;
            Ok(CargoCheckRun {
                cargo_args,
                selected_packages,
                output,
            })
        }
        CargoCheckMode::TargetedCargoCheck if target_selection.package_names.is_empty() => {
            Ok(CargoCheckRun {
                cargo_args: cargo_check_args_for_packages(options.features.as_deref(), &[]),
                selected_packages: Vec::new(),
                output: skipped_command_output_with_reason(
                    CommandSkipReason::TargetedCargoCheckSelectedNoPackages,
                ),
            })
        }
        CargoCheckMode::TargetedCargoCheck => {
            run_targeted_cargo_checks(root, options, target_selection, cargo_target_dir)
        }
        CargoCheckMode::MetadataOnly => Ok(CargoCheckRun {
            cargo_args: cargo_check_args(
                options.features.as_deref(),
                options.package_name.as_deref(),
            ),
            selected_packages: selected_packages(metadata, options.package_name.as_deref(), root),
            output: skipped_command_output(),
        }),
    }
}

fn run_targeted_cargo_checks(
    root: &Path,
    options: &OracleOptions,
    target_selection: &TargetPackageSelection,
    cargo_target_dir: &Path,
) -> Result<CargoCheckRun> {
    let selected_packages = target_selection.packages.clone();
    Ok(CargoCheckRun {
        cargo_args: cargo_check_args_for_packages(
            options.features.as_deref(),
            &target_selection.package_names,
        ),
        selected_packages,
        output: run_cargo_check_for_packages(
            root,
            options,
            &target_selection.package_names,
            cargo_target_dir,
        )?,
    })
}
