use std::process::Output;

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::support::fixtures::package;
use crate::support::root_command;

pub struct UsageErrorRun {
    pub output: Output,
    pub artifact_exists: bool,
}

pub fn run_multi_package_argument_usage_error() -> Result<UsageErrorRun> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", "pub fn demo() {}\n")?;

    let output_path = temp.path().join("rust-analyzer-health.json");
    let mut command = root_command::unified_analyzer_command_for(&root, &output_path)?;
    let output = command
        .arg("--package")
        .arg("app,util")
        .output()
        .context("run unified rust analyzer")?;

    Ok(UsageErrorRun {
        output,
        artifact_exists: output_path.exists(),
    })
}

pub fn run_unknown_package_argument_usage_error() -> Result<UsageErrorRun> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    package::write_single_package_crate(&root, "app", "pub fn demo() {}\n")?;

    let output_path = temp.path().join("rust-analyzer-health.json");
    let mut command = root_command::unified_analyzer_command_for(&root, &output_path)?;
    let output = command
        .arg("--package")
        .arg("missing-app")
        .output()
        .context("run unified rust analyzer")?;

    Ok(UsageErrorRun {
        output,
        artifact_exists: output_path.exists(),
    })
}
