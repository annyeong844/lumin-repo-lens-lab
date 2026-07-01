use anyhow::{bail, Context, Result};
use std::io::{self, Write};

use super::io_support::{
    read_json_input, take_path, take_string, write_json_file, write_stdout_json,
};
use super::usage::USAGE;
use lumin_audit_core::canon_draft_lifecycle::{
    execute_canon_draft_lifecycle, CanonDraftLifecycleRequest,
};
use lumin_audit_core::check_canon_lifecycle::{
    execute_check_canon_lifecycle, CheckCanonLifecycleRequest,
};
use lumin_audit_core::lifecycle::summarize_lifecycle;
use lumin_audit_core::lifecycle_exit_policy::{
    apply_lifecycle_exit_policy, LifecycleExitPolicyRequest,
};
use lumin_audit_core::lifecycle_request::{
    evaluate_lifecycle_request_guard, LifecycleRequestGuardInput,
};
use lumin_audit_core::post_write_lifecycle::{
    execute_post_write_lifecycle, execute_post_write_lifecycle_streaming, PostWriteLifecycleRequest,
};
use lumin_audit_core::pre_write_lifecycle::{
    execute_rust_pre_write_lifecycle, execute_rust_pre_write_lifecycle_streaming,
    RustPreWriteLifecycleRequest,
};
use lumin_audit_core::pre_write_routing::{resolve_pre_write_route, PreWriteRoutingRequest};

pub(super) fn run_lifecycle_summary(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-summary: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-summary: missing --input <path|->")?;
    let lifecycle_json = read_json_input(&input, "lifecycle-summary")?;
    let summary = summarize_lifecycle(&lifecycle_json);
    write_stdout_json(&summary)
}

pub(super) fn run_lifecycle_exit_policy(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-exit-policy: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-exit-policy: missing --input <path|->")?;
    let json = read_json_input(&input, "lifecycle-exit-policy")?;
    let request = serde_json::from_value::<LifecycleExitPolicyRequest>(json)
        .context("lifecycle-exit-policy: invalid request shape")?;
    let result = apply_lifecycle_exit_policy(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_lifecycle_request_guard(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("lifecycle-request-guard: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("lifecycle-request-guard: missing --input <path|->")?;
    let json = read_json_input(&input, "lifecycle-request-guard")?;
    let request = serde_json::from_value::<LifecycleRequestGuardInput>(json)
        .context("lifecycle-request-guard: invalid request shape")?;
    let result = evaluate_lifecycle_request_guard(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_canon_draft(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-canon-draft: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-canon-draft: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-canon-draft")?;
    let request = serde_json::from_value::<CanonDraftLifecycleRequest>(json)
        .context("execute-canon-draft: invalid request shape")?;
    let result = execute_canon_draft_lifecycle(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_check_canon(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("execute-check-canon: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-check-canon: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-check-canon")?;
    let request = serde_json::from_value::<CheckCanonLifecycleRequest>(json)
        .context("execute-check-canon: invalid request shape")?;
    let result = execute_check_canon_lifecycle(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_pre_write_route(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            _ => bail!("pre-write-route: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("pre-write-route: missing --input <path|->")?;
    let json = read_json_input(&input, "pre-write-route")?;
    let request = serde_json::from_value::<PreWriteRoutingRequest>(json)
        .context("pre-write-route: invalid request shape")?;
    let result = resolve_pre_write_route(request)?;
    write_stdout_json(&result)
}

pub(super) fn run_execute_rust_pre_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("execute-rust-pre-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-rust-pre-write: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-rust-pre-write")?;
    let request = serde_json::from_value::<RustPreWriteLifecycleRequest>(json)
        .context("execute-rust-pre-write: invalid request shape")?;
    if let Some(result_output) = result_output {
        let mut result = execute_rust_pre_write_lifecycle_streaming(request)?;
        if let Some(stdout) = result.stdout.as_deref() {
            io::stdout()
                .write_all(stdout.as_bytes())
                .context("failed to replay rust pre-write stdout")?;
        }
        if let Some(stderr) = result.stderr.as_deref() {
            io::stderr()
                .write_all(stderr.as_bytes())
                .context("failed to replay rust pre-write stderr")?;
        }
        result.stdout = None;
        result.stderr = None;
        write_json_file(&result_output, &result)
    } else {
        let result = execute_rust_pre_write_lifecycle(request)?;
        write_stdout_json(&result)
    }
}

pub(super) fn run_execute_post_write(args: Vec<String>) -> Result<()> {
    let mut input = None;
    let mut result_output = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--input" => input = Some(take_string(&mut args, "--input")?),
            "--result-output" => result_output = Some(take_path(&mut args, "--result-output")?),
            _ => bail!("execute-post-write: unknown argument '{arg}'\n{USAGE}"),
        }
    }

    let input = input.context("execute-post-write: missing --input <path|->")?;
    let json = read_json_input(&input, "execute-post-write")?;
    let request = serde_json::from_value::<PostWriteLifecycleRequest>(json)
        .context("execute-post-write: invalid request shape")?;
    if let Some(result_output) = result_output {
        let mut result = execute_post_write_lifecycle_streaming(request)?;
        if let Some(stdout) = result.stdout.as_deref() {
            io::stdout()
                .write_all(stdout.as_bytes())
                .context("failed to replay post-write stdout")?;
        }
        if let Some(stderr) = result.stderr.as_deref() {
            io::stderr()
                .write_all(stderr.as_bytes())
                .context("failed to replay post-write stderr")?;
        }
        result.stdout = None;
        result.stderr = None;
        write_json_file(&result_output, &result)
    } else {
        let result = execute_post_write_lifecycle(request)?;
        write_stdout_json(&result)
    }
}
