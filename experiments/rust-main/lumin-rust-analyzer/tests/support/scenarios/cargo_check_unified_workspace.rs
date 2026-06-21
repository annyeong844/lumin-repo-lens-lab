use anyhow::Result;
use serde_json::Value;
use tempfile::TempDir;

use crate::support::fixtures::unified;
use crate::support::scenarios::run;

pub fn analyze_cargo_check_unified_workspace() -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    unified::write_unified_cli_workspace(&root)?;
    run::run_analyzer(&root, Some("cargo-check"))
}
