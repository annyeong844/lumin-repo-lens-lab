use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::root_command::unified_analyzer_command_for;

pub fn run_unified_analyzer(
    root: &Path,
    output_path: &Path,
    semantic_mode: Option<&str>,
) -> Result<Value> {
    run_unified_analyzer_with_args(root, output_path, semantic_mode, &[])
}

pub fn run_unified_analyzer_with_args(
    root: &Path,
    output_path: &Path,
    semantic_mode: Option<&str>,
    extra_args: &[&OsStr],
) -> Result<Value> {
    let mut command = unified_analyzer_command_for(root, output_path)?;
    if let Some(mode) = semantic_mode {
        command.arg("--semantic-mode").arg(mode);
    }
    command.args(extra_args);

    let output = command.output().context("run unified rust analyzer")?;
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&fs::read(output_path)?).context("read unified analyzer artifact")
}
