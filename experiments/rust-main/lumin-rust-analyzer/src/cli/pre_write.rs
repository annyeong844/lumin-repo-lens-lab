use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};
use lumin_rust_common::{
    parse_min_usize, parse_nonzero_usize, take_path, take_string, usage_error, CliAction,
};

use super::{usage, Command, PreWriteOptions, DEFAULT_WORKER_STACK_BYTES, MIN_WORKER_STACK_BYTES};

pub(super) fn parse(mut args: impl Iterator<Item = String>) -> Result<CliAction<Command>> {
    let mut root: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut source_commit: Option<String> = None;
    let mut intent: Option<PathBuf> = None;
    let mut include_tests = true;
    let mut exclude = Vec::new();
    let mut thread_count: Option<usize> = None;
    let mut worker_stack_bytes = DEFAULT_WORKER_STACK_BYTES;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = Some(take_path(&mut args, "--root")?),
            "--output" => output = Some(take_path(&mut args, "--output")?),
            "--source-commit" | "--sidecar-source-commit" => {
                source_commit = Some(take_string(&mut args, "--source-commit")?)
            }
            "--intent" => intent = Some(take_path(&mut args, "--intent")?),
            "--production" | "--exclude-tests" | "--no-tests" | "--no-include-tests" => {
                include_tests = false;
            }
            "--include-tests" => include_tests = true,
            "--exclude" => exclude.push(take_string(&mut args, "--exclude")?),
            "--threads" => {
                let value = take_string(&mut args, "--threads")?;
                thread_count = Some(parse_nonzero_usize(&value, "--threads")?);
            }
            "--worker-stack-bytes" => {
                let value = take_string(&mut args, "--worker-stack-bytes")?;
                worker_stack_bytes =
                    parse_min_usize(&value, "--worker-stack-bytes", MIN_WORKER_STACK_BYTES)?;
            }
            "--help" | "-h" => {
                usage::print_pre_write();
                return Ok(CliAction::Help);
            }
            unknown => {
                return Err(usage_error(format!(
                    "unknown pre-write argument: {unknown}"
                )))
            }
        }
    }

    Ok(CliAction::Run(Command::PreWrite(PreWriteOptions {
        root: root.unwrap_or(env::current_dir().context("failed to read current directory")?),
        output,
        source_commit: source_commit.ok_or_else(|| usage_error("--source-commit is required"))?,
        intent: intent.ok_or_else(|| usage_error("--intent is required"))?,
        include_tests,
        exclude,
        thread_count,
        worker_stack_bytes,
    })))
}
