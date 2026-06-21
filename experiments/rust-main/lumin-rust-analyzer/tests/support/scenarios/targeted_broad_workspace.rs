use anyhow::Result;
use serde_json::Value;
use tempfile::TempDir;

use crate::support::fixtures::targeted_workspace;
use crate::support::scenarios::run;

pub fn analyze_targeted_broad_workspace(package_count: usize) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    targeted_workspace::write_broad_targeted_workspace(&root, package_count)?;
    run::run_analyzer(&root, Some("targeted-cargo-check"))
}
