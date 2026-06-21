use anyhow::Result;
use std::path::Path;
use std::time::Instant;

use crate::classify::{parse_cargo_jsonl, skipped_cargo_jsonl, ParsedJsonl};
use crate::command::{
    cargo_check_args, cargo_check_args_for_packages, combined_command_output, run_cargo_check,
    run_cargo_check_for_packages_with_timeout, skipped_command_output,
    skipped_command_output_with_reason, CommandOutput, CommandSkipReason,
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
        parse_cargo_jsonl(&run.output.stdout, run.output.timed_out)
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
    let mut outputs = Vec::with_capacity(target_selection.package_names.len());
    let mut selected_packages = Vec::with_capacity(target_selection.package_names.len());
    let started = Instant::now();
    for package in &target_selection.packages {
        let Some(timeout_ms) = remaining_timeout_ms(started, options.timeout_ms) else {
            outputs.push(targeted_budget_timeout_output(package.name.as_str()));
            break;
        };
        let package_names = std::slice::from_ref(&package.name);
        let output = run_cargo_check_for_packages_with_timeout(
            root,
            options,
            package_names,
            cargo_target_dir,
            timeout_ms,
        )?;
        let timed_out = output.timed_out;
        outputs.push(output);
        selected_packages.push(package.clone());
        if timed_out {
            break;
        }
    }
    let package_names = selected_packages
        .iter()
        .map(|package| package.name.clone())
        .collect::<Vec<_>>();
    Ok(CargoCheckRun {
        cargo_args: cargo_check_args_for_packages(options.features.as_deref(), &package_names),
        selected_packages,
        output: combined_command_output(outputs),
    })
}

fn remaining_timeout_ms(started: Instant, timeout_ms: u64) -> Option<u64> {
    let timeout = u128::from(timeout_ms);
    let elapsed = started.elapsed().as_millis();
    let remaining = timeout.saturating_sub(elapsed);
    if remaining == 0 {
        None
    } else {
        Some(remaining.min(u128::from(u64::MAX)) as u64)
    }
}

fn targeted_budget_timeout_output(package_name: &str) -> CommandOutput {
    CommandOutput {
        status: Some(1),
        stdout: String::new(),
        stderr: format!("targeted cargo check timed out before package {package_name}"),
        timed_out: true,
        elapsed_ms: 0,
        skip_reason: None,
    }
}
