mod analyze;
mod pre_write;
mod usage;

use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use lumin_rust_cargo_oracle::{CargoCheckMode, CargoTargetDirMode};
use lumin_rust_common::CliAction;
use lumin_rust_source_health::protocol::{DEFAULT_WORKER_STACK_BYTES, MIN_WORKER_STACK_BYTES};
use serde::Serialize;

#[derive(Debug)]
pub(crate) struct Options {
    pub(crate) root: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) source_commit: String,
    pub(crate) cargo_bin: String,
    pub(crate) features: Option<String>,
    pub(crate) package_name: Option<String>,
    pub(crate) repo_root: PathBuf,
    pub(crate) thread_count: Option<usize>,
    pub(crate) worker_stack_bytes: usize,
    pub(crate) semantic_mode: CargoCheckMode,
    pub(crate) cargo_target_dir_mode: CargoTargetDirMode,
    pub(crate) calibration_adjudication: Option<PathBuf>,
    pub(crate) source_health_profile: SourceHealthProfile,
    pub(crate) source_health_cache_root: Option<PathBuf>,
    pub(crate) source_health_incremental_enabled: bool,
    pub(crate) source_health_clear_incremental_cache: bool,
}

#[derive(Debug)]
pub(crate) enum Command {
    Analyze(Options),
    PreWrite(PreWriteOptions),
}

#[derive(Debug)]
pub(crate) struct PreWriteOptions {
    pub(crate) root: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) source_commit: String,
    pub(crate) intent: PathBuf,
    pub(crate) thread_count: Option<usize>,
    pub(crate) worker_stack_bytes: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceHealthProfile {
    Compact,
    Full,
}

impl FromStr for SourceHealthProfile {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "compact" => Ok(Self::Compact),
            "full" => Ok(Self::Full),
            _ => Err(()),
        }
    }
}

pub(crate) fn parse_args() -> Result<CliAction<Command>> {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(command) if command == "pre-write" => pre_write::parse(args),
        Some(first) => analyze::parse(std::iter::once(first).chain(args)),
        None => analyze::parse(std::iter::empty()),
    }
}
