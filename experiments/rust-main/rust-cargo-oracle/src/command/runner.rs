use anyhow::{bail, Context, Result};
use std::path::Path;

use super::args::{cargo_check_args_for_packages, cargo_metadata_args, MetadataDependencyMode};
use super::{run_command, CommandOutput};
use crate::metadata::CargoMetadata;
use crate::protocol::CargoCheckMode;
use crate::OracleOptions;

pub(crate) fn run_cargo_check(
    root: &Path,
    options: &OracleOptions,
    cargo_target_dir: &Path,
) -> Result<CommandOutput> {
    let package_names = options
        .package_name
        .as_ref()
        .map(|package_name| vec![package_name.clone()])
        .unwrap_or_default();
    run_cargo_check_for_packages(root, options, &package_names, cargo_target_dir)
}

pub(crate) fn run_cargo_check_for_packages(
    root: &Path,
    options: &OracleOptions,
    package_names: &[String],
    cargo_target_dir: &Path,
) -> Result<CommandOutput> {
    run_command(
        &options.cargo_bin,
        &cargo_check_args_for_packages(options.features.as_deref(), package_names),
        root,
        Some(cargo_target_dir),
    )
}

pub(crate) fn run_cargo_metadata(
    root: &Path,
    cargo_bin: &str,
    features: Option<&str>,
    cargo_check_mode: CargoCheckMode,
    cargo_target_dir: &Path,
) -> Result<CargoMetadata> {
    let dependency_mode = match cargo_check_mode {
        CargoCheckMode::MetadataOnly => MetadataDependencyMode::WorkspaceOnly,
        CargoCheckMode::CargoCheck | CargoCheckMode::TargetedCargoCheck => {
            MetadataDependencyMode::IncludeDependencies
        }
    };
    let output = run_command(
        cargo_bin,
        &cargo_metadata_args(features, dependency_mode),
        root,
        Some(cargo_target_dir),
    )?;
    if output.status != Some(0) {
        bail!("cargo metadata failed: {}", output.stderr.trim());
    }
    serde_json::from_str(&output.stdout).context("invalid cargo metadata JSON")
}
