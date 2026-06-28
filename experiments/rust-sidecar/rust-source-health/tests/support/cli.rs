#![allow(dead_code)]

use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn run_sidecar(request: Value) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_lumin-rust-source-health");
    let mut child = match Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => panic!("spawn sidecar: {error}"),
    };
    let Some(stdin) = child.stdin.as_mut() else {
        panic!("sidecar stdin was not piped");
    };
    if let Err(error) = stdin.write_all(request.to_string().as_bytes()) {
        panic!("write request: {error}");
    }
    match child.wait_with_output() {
        Ok(output) => output,
        Err(error) => panic!("sidecar output: {error}"),
    }
}

pub fn stdout_json(output: std::process::Output) -> Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(error) => panic!("stdout json: {error}"),
    }
}

pub fn run_cli(args: &[String]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_lumin-rust-source-health");
    match Command::new(bin).args(args).output() {
        Ok(output) => output,
        Err(error) => panic!("run rust source health cli: {error}"),
    }
}
