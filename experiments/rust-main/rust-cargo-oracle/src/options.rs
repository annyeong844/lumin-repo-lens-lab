use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use lumin_rust_common::{
    find_repo_root_with_fallback, parse_enum, parse_nonzero_usize, parse_u64, take_path,
    take_string, usage_error, CliAction,
};

use crate::protocol::{CargoCheckMode, CargoTargetDirMode};
use crate::DEFAULT_TARGETED_CARGO_CHECK_PACKAGES;

#[derive(Debug, Clone)]
pub struct OracleOptions {
    pub root: PathBuf,
    pub output: Option<PathBuf>,
    pub cargo_bin: String,
    pub timeout_ms: u64,
    pub features: Option<String>,
    pub package_name: Option<String>,
    pub repo_root: PathBuf,
    pub cargo_check_mode: CargoCheckMode,
    pub cargo_target_dir_mode: CargoTargetDirMode,
    pub target_paths: Vec<String>,
    pub targeted_package_cap: usize,
}

pub fn parse_args() -> Result<CliAction<OracleOptions>> {
    let mut root: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut cargo_bin = "cargo".to_string();
    let mut timeout_ms = 60_000_u64;
    let mut features: Option<String> = None;
    let mut package_name: Option<String> = None;
    let mut repo_root: Option<PathBuf> = None;
    let mut cargo_check_mode = CargoCheckMode::CargoCheck;
    let mut cargo_target_dir_mode = CargoTargetDirMode::IsolatedTemp;
    let mut target_paths = Vec::<String>::new();
    let mut targeted_package_cap = DEFAULT_TARGETED_CARGO_CHECK_PACKAGES;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(take_path(&mut args, "--root")?),
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--cargo-bin" => cargo_bin = take_string(&mut args, "--cargo-bin")?,
            "--timeout-ms" => {
                let value = take_string(&mut args, "--timeout-ms")?;
                timeout_ms = parse_u64(&value, "--timeout-ms")?;
            }
            "--features" => features = Some(take_string(&mut args, "--features")?),
            "--package" => package_name = Some(take_string(&mut args, "--package")?),
            "--repo-root" => repo_root = Some(take_path(&mut args, "--repo-root")?),
            "--target-path" => target_paths.push(take_string(&mut args, "--target-path")?),
            "--targeted-package-cap" => {
                let value = take_string(&mut args, "--targeted-package-cap")?;
                targeted_package_cap = parse_nonzero_usize(&value, "--targeted-package-cap")?;
            }
            "--cargo-check-mode" => {
                let value = take_string(&mut args, "--cargo-check-mode")?;
                cargo_check_mode = parse_enum(&value, "--cargo-check-mode")?;
            }
            "--cargo-target-dir-mode" => {
                let value = take_string(&mut args, "--cargo-target-dir-mode")?;
                cargo_target_dir_mode = parse_enum(&value, "--cargo-target-dir-mode")?;
            }
            "--help" | "-h" => {
                print_usage();
                return Ok(CliAction::Help);
            }
            unknown => return Err(usage_error(format!("unknown argument: {unknown}"))),
        }
    }

    let root = root.unwrap_or(env::current_dir().context("failed to read current directory")?);
    let output = output.unwrap_or_else(|| root.join("semantic-health.json"));
    let repo_root = match repo_root {
        Some(path) => path,
        None => default_repo_root(&root),
    };

    Ok(CliAction::Run(OracleOptions {
        root,
        output: Some(output),
        cargo_bin,
        timeout_ms,
        features,
        package_name,
        repo_root,
        cargo_check_mode,
        cargo_target_dir_mode,
        target_paths,
        targeted_package_cap,
    }))
}

fn default_repo_root(root: &Path) -> PathBuf {
    find_repo_root_with_fallback(root, Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-cargo-oracle --root <path> [--output <path>] [--cargo-bin <path>] [--timeout-ms <ms>] [--features <csv>] [--package <name>] [--repo-root <path>] [--cargo-check-mode metadata-only|cargo-check|targeted-cargo-check] [--cargo-target-dir-mode isolated-temp|reusable-temp] [--target-path <path>...] [--targeted-package-cap <n>]"
    );
}
