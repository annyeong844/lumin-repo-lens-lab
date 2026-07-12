use std::path::PathBuf;

use anyhow::Result;
use lumin_rust_common::{parse_min_usize, parse_nonzero_usize, take_path, take_string, CliAction};

use crate::protocol::{DEFAULT_WORKER_STACK_BYTES, MIN_WORKER_STACK_BYTES};
use crate::usage_error;

#[derive(Debug)]
pub(super) struct WrapperOptions {
    pub(super) root: PathBuf,
    pub(super) output: PathBuf,
    pub(super) source_commit: String,
    pub(super) thread_count: Option<usize>,
    pub(super) worker_stack_bytes: usize,
    pub(super) artifact_profile: ArtifactProfile,
    pub(super) cache_root: Option<PathBuf>,
    pub(super) incremental_enabled: bool,
    pub(super) clear_incremental_cache: bool,
}

impl WrapperOptions {
    pub(super) fn parse(args: Vec<String>) -> Result<CliAction<Self>> {
        let mut root = None;
        let mut output = None;
        let mut source_commit = None;
        let mut thread_count = None;
        let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;
        let mut artifact_profile = ArtifactProfile::Compact;
        let mut cache_root = None;
        let mut incremental_enabled = true;
        let mut clear_incremental_cache = false;

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--root" => root = Some(take_path(&mut iter, "--root")?),
                "--output" => output = Some(take_path(&mut iter, "--output")?),
                "--source-commit" | "--sidecar-source-commit" => {
                    source_commit = Some(take_string(&mut iter, "--source-commit")?)
                }
                "--threads" => {
                    let raw = take_string(&mut iter, "--threads")?;
                    thread_count = Some(parse_nonzero_usize(&raw, "--threads")?);
                }
                "--worker-stack-bytes" => {
                    let raw = take_string(&mut iter, "--worker-stack-bytes")?;
                    worker_stack_bytes =
                        parse_min_usize(&raw, "--worker-stack-bytes", MIN_WORKER_STACK_BYTES)?;
                }
                "--artifact-profile" => {
                    let raw = take_string(&mut iter, "--artifact-profile")?;
                    artifact_profile = ArtifactProfile::parse(&raw)?;
                }
                "--cache-root" => cache_root = Some(take_path(&mut iter, "--cache-root")?),
                "--no-incremental" => incremental_enabled = false,
                "--clear-incremental-cache" => clear_incremental_cache = true,
                "--help" | "-h" => {
                    print_usage();
                    return Ok(CliAction::Help);
                }
                unknown => return Err(usage_error(format!("unknown argument: {unknown}"))),
            }
        }

        Ok(CliAction::Run(Self {
            root: root.ok_or_else(|| usage_error("--root is required"))?,
            output: output.ok_or_else(|| usage_error("--output is required"))?,
            source_commit: source_commit
                .ok_or_else(|| usage_error("--source-commit is required"))?,
            thread_count,
            worker_stack_bytes,
            artifact_profile,
            cache_root,
            incremental_enabled,
            clear_incremental_cache,
        }))
    }
}

fn print_usage() {
    eprintln!(
        "Usage: lumin-rust-source-health --root <path> --output <path> --source-commit <sha> [--artifact-profile compact|full] [--threads <n>] [--worker-stack-bytes <bytes>] [--cache-root <path>] [--no-incremental] [--clear-incremental-cache]"
    );
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) enum ArtifactProfile {
    Compact,
    Full,
}

impl ArtifactProfile {
    fn parse(raw: &str) -> Result<Self> {
        match raw {
            "compact" => Ok(Self::Compact),
            "full" => Ok(Self::Full),
            _ => Err(usage_error(format!(
                "invalid --artifact-profile: expected compact or full, got {raw}"
            ))),
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Full => "full",
        }
    }
}
