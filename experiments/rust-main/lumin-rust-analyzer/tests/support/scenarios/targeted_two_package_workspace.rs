use anyhow::Result;
use serde_json::Value;
use tempfile::TempDir;

use crate::support::fixtures::targeted_workspace;
use crate::support::scenarios::run;

pub fn analyze_targeted_two_package_workspace() -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    targeted_workspace::write_two_package_targeted_workspace(&root)?;
    run::run_analyzer(&root, Some("targeted-cargo-check"))
}
