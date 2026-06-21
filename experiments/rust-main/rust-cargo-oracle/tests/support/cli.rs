use std::process::{Command, Output};

pub fn oracle_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_lumin-rust-cargo-oracle"))
}

pub fn assert_usage_error(output: &Output, expected_stderr: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains(expected_stderr));
}
