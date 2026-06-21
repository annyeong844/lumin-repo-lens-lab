use std::ffi::OsStr;

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

pub fn analyze_targeted_broad_workspace_with_cap(
    package_count: usize,
    targeted_package_cap: usize,
) -> Result<Value> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    targeted_workspace::write_broad_targeted_workspace(&root, package_count)?;
    let cap = targeted_package_cap.to_string();
    let args = [
        OsStr::new("--targeted-package-cap"),
        OsStr::new(cap.as_str()),
    ];
    run::run_analyzer_with_args(&root, Some("targeted-cargo-check"), &args)
}
