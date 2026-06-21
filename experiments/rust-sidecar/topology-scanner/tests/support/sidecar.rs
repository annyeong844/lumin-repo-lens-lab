use std::io::Write;
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result};

pub fn run_with_stdin(input: &str) -> Result<Output> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lumin-topology-scanner"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn topology scanner")?;
    child
        .stdin
        .as_mut()
        .context("topology scanner stdin")?
        .write_all(input.as_bytes())
        .context("write topology scanner stdin")?;
    child.wait_with_output().context("wait topology scanner")
}
