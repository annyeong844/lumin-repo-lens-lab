use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{CargoCheckMode, CargoTargetDirMode};
use lumin_rust_common::{
    find_repo_root_with_fallback, parse_enum, parse_min_usize, parse_nonzero_usize, take_path,
    take_string, usage_error, CliAction,
};

use super::{usage, Command, Options, DEFAULT_WORKER_STACK_BYTES};

pub(super) fn parse(mut args: impl Iterator<Item = String>) -> Result<CliAction<Command>> {
    let mut root: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut source_commit: Option<String> = None;
    let mut cargo_bin = "cargo".to_string();
    let mut features: Option<String> = None;
    let mut package_name: Option<String> = None;
    let mut repo_root: Option<PathBuf> = None;
    let mut thread_count: Option<usize> = None;
    let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;
    let mut semantic_mode = CargoCheckMode::MetadataOnly;
    let mut cargo_target_dir_mode = CargoTargetDirMode::IsolatedTemp;
    let mut calibration_adjudication: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(take_path(&mut args, "--root")?),
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--source-commit" | "--sidecar-source-commit" => {
                source_commit = Some(take_string(&mut args, "--source-commit")?)
            }
            "--cargo-bin" => cargo_bin = take_string(&mut args, "--cargo-bin")?,
            "--features" => features = Some(take_string(&mut args, "--features")?),
            "--package" => package_name = Some(take_string(&mut args, "--package")?),
            "--repo-root" => repo_root = Some(take_path(&mut args, "--repo-root")?),
            "--semantic-mode" => {
                let value = take_string(&mut args, "--semantic-mode")?;
                semantic_mode = parse_enum(&value, "--semantic-mode")?;
            }
            "--cargo-target-dir-mode" => {
                let value = take_string(&mut args, "--cargo-target-dir-mode")?;
                cargo_target_dir_mode = parse_enum(&value, "--cargo-target-dir-mode")?;
            }
            "--calibration-adjudication" => {
                calibration_adjudication = Some(take_path(&mut args, "--calibration-adjudication")?)
            }
            "--cargo-check" => semantic_mode = CargoCheckMode::CargoCheck,
            "--targeted-cargo-check" => semantic_mode = CargoCheckMode::TargetedCargoCheck,
            "--threads" => {
                let value = take_string(&mut args, "--threads")?;
                thread_count = Some(parse_nonzero_usize(&value, "--threads")?);
            }
            "--worker-stack-bytes" => {
                let value = take_string(&mut args, "--worker-stack-bytes")?;
                worker_stack_bytes =
                    parse_min_usize(&value, "--worker-stack-bytes", DEFAULT_WORKER_STACK_BYTES)?;
            }
            "--help" | "-h" => {
                usage::print_analyze();
                return Ok(CliAction::Help);
            }
            unknown => return Err(usage_error(format!("unknown argument: {unknown}"))),
        }
    }

    let root = root.unwrap_or(env::current_dir().context("failed to read current directory")?);
    let output = output.unwrap_or_else(|| root.join("rust-analyzer-health.json"));
    let repo_root = match repo_root {
        Some(path) => path,
        None => default_repo_root(&root),
    };

    Ok(CliAction::Run(Command::Analyze(Options {
        root,
        output: Some(output),
        source_commit: source_commit.ok_or_else(|| usage_error("--source-commit is required"))?,
        cargo_bin,
        features,
        package_name,
        repo_root,
        thread_count,
        worker_stack_bytes,
        semantic_mode,
        cargo_target_dir_mode,
        calibration_adjudication,
    })))
}

fn default_repo_root(root: &Path) -> PathBuf {
    find_repo_root_with_fallback(root, Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|| PathBuf::from("."))
}
