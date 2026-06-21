use std::ffi::OsStr;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::cli;

pub fn run_analyzer(root: &Path, semantic_mode: Option<&str>) -> Result<Value> {
    run_analyzer_with_args(root, semantic_mode, &[])
}

pub fn run_analyzer_with_args(
    root: &Path,
    semantic_mode: Option<&str>,
    extra_args: &[&OsStr],
) -> Result<Value> {
    let output_path = root
        .parent()
        .context("test repo should live under a temp directory")?
        .join("rust-analyzer-health.json");
    cli::run_unified_analyzer_with_args(root, &output_path, semantic_mode, extra_args)
}
