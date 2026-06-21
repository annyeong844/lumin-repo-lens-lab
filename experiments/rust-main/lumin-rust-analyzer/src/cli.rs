use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::{
    CargoCheckMode, CargoTargetDirMode, DEFAULT_TARGETED_CARGO_CHECK_PACKAGES,
};
use lumin_rust_common::{
    find_repo_root_with_fallback, parse_enum, parse_min_usize, parse_nonzero_usize, parse_u64,
    take_path, take_string, usage_error, CliAction,
};
use lumin_rust_source_health::protocol::DEFAULT_WORKER_STACK_BYTES;

#[derive(Debug)]
pub(crate) struct Options {
    pub(crate) root: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) source_commit: String,
    pub(crate) cargo_bin: String,
    pub(crate) timeout_ms: u64,
    pub(crate) features: Option<String>,
    pub(crate) package_name: Option<String>,
    pub(crate) repo_root: PathBuf,
    pub(crate) thread_count: Option<usize>,
    pub(crate) worker_stack_bytes: usize,
    pub(crate) semantic_mode: CargoCheckMode,
    pub(crate) cargo_target_dir_mode: CargoTargetDirMode,
    pub(crate) targeted_package_cap: usize,
    pub(crate) calibration_adjudication: Option<PathBuf>,
}

pub(crate) fn parse_args() -> Result<CliAction<Options>> {
    let mut root: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut source_commit: Option<String> = None;
    let mut cargo_bin = "cargo".to_string();
    let mut timeout_ms = 60_000_u64;
    let mut features: Option<String> = None;
    let mut package_name: Option<String> = None;
    let mut repo_root: Option<PathBuf> = None;
    let mut thread_count: Option<usize> = None;
    let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;
    let mut semantic_mode = CargoCheckMode::MetadataOnly;
    let mut cargo_target_dir_mode = CargoTargetDirMode::IsolatedTemp;
    let mut targeted_package_cap = DEFAULT_TARGETED_CARGO_CHECK_PACKAGES;
    let mut calibration_adjudication: Option<PathBuf> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(take_path(&mut args, "--root")?),
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--source-commit" | "--sidecar-source-commit" => {
                source_commit = Some(take_string(&mut args, "--source-commit")?)
            }
            "--cargo-bin" => cargo_bin = take_string(&mut args, "--cargo-bin")?,
            "--timeout-ms" => {
                let value = take_string(&mut args, "--timeout-ms")?;
                timeout_ms = parse_u64(&value, "--timeout-ms")?;
            }
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
            "--targeted-package-cap" => {
                let value = take_string(&mut args, "--targeted-package-cap")?;
                targeted_package_cap = parse_nonzero_usize(&value, "--targeted-package-cap")?;
            }
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
                print_usage();
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

    Ok(CliAction::Run(Options {
        root,
        output: Some(output),
        source_commit: source_commit.ok_or_else(|| usage_error("--source-commit is required"))?,
        cargo_bin,
        timeout_ms,
        features,
        package_name,
        repo_root,
        thread_count,
        worker_stack_bytes,
        semantic_mode,
        cargo_target_dir_mode,
        targeted_package_cap,
        calibration_adjudication,
    }))
}

fn default_repo_root(root: &Path) -> PathBuf {
    find_repo_root_with_fallback(root, Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-analyzer --root <path> --source-commit <sha> [--output <path>] [--cargo-bin <path>] [--timeout-ms <ms>] [--features <csv>] [--package <name>] [--repo-root <path>] [--semantic-mode metadata-only|cargo-check|targeted-cargo-check] [--cargo-target-dir-mode isolated-temp|reusable-temp] [--targeted-package-cap <n>] [--calibration-adjudication <path>] [--cargo-check] [--targeted-cargo-check] [--threads <n>] [--worker-stack-bytes <bytes>]"
    );
}
