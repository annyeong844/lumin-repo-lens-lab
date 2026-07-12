use std::io::Write;
use std::process::{Command, Stdio};

use super::advisory::path_string;
use super::protocol::{JsPreWriteLifecycleRequest, PreWriteLifecycleRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ChildStdio {
    Capture,
    Inherit,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ChildOutput {
    pub(super) status_success: bool,
    pub(super) exit_code: Option<i32>,
    pub(super) reason: String,
    pub(super) stdout: String,
    pub(super) stderr: String,
}

pub(super) fn run_rust_pre_write_child(
    request: &PreWriteLifecycleRequest,
    source_commit: &str,
    child_stdio: ChildStdio,
) -> ChildOutput {
    let mut args = request.analyzer_invocation.prefix_args.clone();
    args.extend([
        "pre-write".to_string(),
        "--root".to_string(),
        path_string(&request.root),
        "--source-commit".to_string(),
        source_commit.to_string(),
        "--intent".to_string(),
        "-".to_string(),
        "--output".to_string(),
        path_string(&request.rust_native_artifact_path),
    ]);
    if !request.include_tests {
        args.push("--production".to_string());
    }
    for pattern in &request.excludes {
        args.extend(["--exclude".to_string(), pattern.clone()]);
    }

    run_child(
        "lumin-rust-analyzer pre-write",
        &request.analyzer_invocation.command,
        &args,
        &request.intent_input,
        child_stdio,
    )
}

pub(super) fn run_js_pre_write_child(
    request: &JsPreWriteLifecycleRequest,
    child_stdio: ChildStdio,
) -> ChildOutput {
    let mut args = vec![
        path_string(&request.scripts_dir.join("pre-write.mjs")),
        "--root".to_string(),
        path_string(&request.root),
        "--output".to_string(),
        path_string(&request.output),
        "--intent".to_string(),
        request.child_intent_flag.clone(),
    ];
    args.extend(request.scan_args.clone());
    args.extend(request.incremental_args.clone());
    if request.no_fresh_audit {
        args.push("--no-fresh-audit".to_string());
    }
    run_child(
        "pre-write.mjs",
        &request.node_executable,
        &args,
        request.child_intent_input.as_deref().unwrap_or(""),
        child_stdio,
    )
}

pub(super) fn nonempty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn run_child(
    label: &str,
    command: &str,
    args: &[String],
    input: &str,
    child_stdio: ChildStdio,
) -> ChildOutput {
    match child_stdio {
        ChildStdio::Capture => run_child_capture(label, command, args, input),
        ChildStdio::Inherit => run_child_inherit(label, command, args, input),
    }
}

fn run_child_capture(label: &str, command: &str, args: &[String], input: &str) -> ChildOutput {
    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return child_start_error(error),
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(input.as_bytes()) {
            return ChildOutput {
                status_success: false,
                exit_code: Some(1),
                reason: format!("failed to write intent stdin: {error}"),
                stdout: String::new(),
                stderr: String::new(),
            };
        }
    }
    match child.wait_with_output() {
        Ok(output) => child_output(
            label,
            output.status.success(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ),
        Err(error) => child_start_error(error),
    }
}

fn run_child_inherit(label: &str, command: &str, args: &[String], input: &str) -> ChildOutput {
    let mut child = match Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return child_start_error(error),
    };
    if let Some(mut stdin) = child.stdin.take() {
        if let Err(error) = stdin.write_all(input.as_bytes()) {
            return ChildOutput {
                status_success: false,
                exit_code: Some(1),
                reason: format!("failed to write intent stdin: {error}"),
                stdout: String::new(),
                stderr: String::new(),
            };
        }
    }
    match child.wait() {
        Ok(status) => child_output(
            label,
            status.success(),
            status.code(),
            String::new(),
            String::new(),
        ),
        Err(error) => child_start_error(error),
    }
}

fn child_start_error(error: std::io::Error) -> ChildOutput {
    ChildOutput {
        status_success: false,
        exit_code: Some(1),
        reason: error.to_string(),
        stdout: String::new(),
        stderr: String::new(),
    }
}

fn child_output(
    label: &str,
    status_success: bool,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
) -> ChildOutput {
    let reason = exit_code
        .map(|code| format!("{label} exited {code}"))
        .unwrap_or_else(|| format!("{label} terminated by signal"));
    ChildOutput {
        status_success,
        exit_code,
        reason,
        stdout,
        stderr,
    }
}
