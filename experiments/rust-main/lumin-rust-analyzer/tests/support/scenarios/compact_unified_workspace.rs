use std::ffi::OsStr;

use anyhow::Result;
use serde_json::Value;
use tempfile::TempDir;

use crate::support::fixtures::unified;
use crate::support::scenarios::run;

pub fn analyze_metadata_only_unified_workspace() -> Result<Value> {
    analyze_metadata_only_unified_workspace_with_args(&[])
}

pub fn analyze_metadata_only_unified_workspace_with_args(extra_args: &[&OsStr]) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    unified::write_unified_cli_workspace(&root)?;
    run::run_analyzer_with_args(&root, None, extra_args)
}
