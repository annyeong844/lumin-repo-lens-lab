use anyhow::{Context, Result};
use lumin_rust_common::{atomic_write_json_pretty, canonical_existing_dir_usage, sha256_file};

mod check;

use crate::artifact::{build_artifact, BuildArtifactInput};
use crate::command::{run_cargo_metadata, CargoTargetDir};
use crate::environment::CompilationEnvironment;
use crate::input_hash::{analysis_input_set_hash, AnalysisInputSet};
use crate::options::OracleOptions;
use crate::oracle_plan::oracle_plan;
use crate::protocol::SemanticHealthArtifact;
use crate::target_selection::target_package_selection;
use crate::toolchain::collect_toolchain;
use crate::usage_error;

use self::check::run_cargo_check_phase;

pub fn run_oracle(options: OracleOptions) -> Result<SemanticHealthArtifact> {
    let root = canonical_existing_dir_usage(&options.root, "--root")?;
    validate_package_name(options.package_name.as_deref())?;
    validate_targeted_package_cap(options.targeted_package_cap)?;

    let registry_path = options
        .repo_root
        .join("canonical")
        .join("oracle-registry.json");
    if !registry_path.is_file() {
        return Err(usage_error(format!(
            "invalid --repo-root {}: missing canonical/oracle-registry.json",
            options.repo_root.display()
        )));
    }
    let registry_hash = sha256_file(&registry_path)
        .with_context(|| format!("failed to hash {}", registry_path.display()))?;

    let toolchain = collect_toolchain(&root, &options.cargo_bin, options.timeout_ms);
    let compilation_environment = CompilationEnvironment::from_process();
    let cargo_target_dir = CargoTargetDir::create(
        options.cargo_target_dir_mode,
        &root,
        &options.cargo_bin,
        &toolchain.rustc_bin,
    )?;

    let metadata_result = run_cargo_metadata(
        &root,
        &options.cargo_bin,
        options.timeout_ms,
        options.features.as_deref(),
        options.cargo_check_mode,
        cargo_target_dir.path(),
    );
    let (metadata, metadata_unavailable_reason) = match metadata_result {
        Ok(metadata) => (Some(metadata), None),
        Err(error) => (None, Some(error.to_string())),
    };
    validate_package_exists(metadata.as_ref(), options.package_name.as_deref())?;

    let target_selection = target_package_selection(
        &root,
        metadata.as_ref(),
        &options.target_paths,
        options.package_name.as_deref(),
        options.targeted_package_cap,
    );
    let check_phase = run_cargo_check_phase(
        &root,
        &options,
        metadata.as_ref(),
        &target_selection,
        cargo_target_dir.path(),
    )?;
    let oracle_plan = oracle_plan(
        options.cargo_check_mode,
        &target_selection,
        &check_phase.selected_packages,
        &check_phase.output,
        options.targeted_package_cap,
    );
    let input_hash = analysis_input_set_hash(AnalysisInputSet {
        root: &root,
        metadata: metadata.as_ref(),
        cargo_args: &check_phase.cargo_args,
        selected: &check_phase.selected_packages,
        registry_hash: &registry_hash,
        compilation_environment: &compilation_environment,
        features: options.features.as_deref(),
        package_name: options.package_name.as_deref(),
        toolchain: &toolchain,
        cargo_check_mode: options.cargo_check_mode,
        cargo_target_dir_mode: options.cargo_target_dir_mode,
        target_paths: &target_selection.target_paths,
        targeted_package_cap: options.targeted_package_cap,
    })
    .context("failed to serialize analysis input identity")?;

    let messages = check_phase.parsed.messages();
    let artifact = build_artifact(BuildArtifactInput {
        root: &root,
        output: options.output.as_deref(),
        metadata: metadata.as_ref(),
        messages,
        stream_parse_status: check_phase.parsed.stream_parse_status(),
        invalid_json_line_count: check_phase.parsed.invalid_json_line_count(),
        check_output: &check_phase.output,
        cargo_bin: &options.cargo_bin,
        cargo_args: &check_phase.cargo_args,
        selected_packages: &check_phase.selected_packages,
        metadata_unavailable_reason,
        registry_hash: &registry_hash,
        input_hash: &input_hash,
        compilation_environment: &compilation_environment,
        features: options.features.as_deref(),
        package_name: options.package_name.as_deref(),
        toolchain: &toolchain,
        cargo_check_mode: options.cargo_check_mode,
        cargo_target_dir_mode: options.cargo_target_dir_mode,
        cargo_target_dir: cargo_target_dir.path(),
        oracle_plan,
    })?;

    if let Some(output) = options.output {
        atomic_write_json_pretty(&output, &artifact)
            .with_context(|| format!("failed to write {}", output.display()))?;
    }

    Ok(artifact)
}

fn validate_package_name(value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        if value.is_empty()
            || !value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            return Err(usage_error(
                "--package currently supports exact package names only",
            ));
        }
    }
    Ok(())
}

fn validate_package_exists(
    metadata: Option<&crate::metadata::CargoMetadata>,
    package_name: Option<&str>,
) -> Result<()> {
    let (Some(metadata), Some(package_name)) = (metadata, package_name) else {
        return Ok(());
    };
    if metadata
        .packages
        .iter()
        .any(|pkg| pkg.name == package_name || pkg.id == package_name)
    {
        return Ok(());
    }
    Err(usage_error(format!(
        "unknown --package {package_name}: no matching package name or package ID in cargo metadata"
    )))
}

fn validate_targeted_package_cap(value: usize) -> Result<()> {
    if value == 0 {
        return Err(usage_error(
            "--targeted-package-cap must be greater than zero",
        ));
    }
    Ok(())
}
