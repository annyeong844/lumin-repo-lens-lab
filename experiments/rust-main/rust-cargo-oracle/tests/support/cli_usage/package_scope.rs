use std::fs;

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::support::{
    cli::{assert_usage_error, oracle_command},
    paths::repo_root,
};

pub fn assert_package_scope_usage_error_exits_2_without_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--repo-root")
        .arg(repo_root()?)
        .arg("--package")
        .arg("app,util")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(
        &output,
        "--package currently supports exact package names only",
    );
    assert!(!output_path.exists());
    Ok(())
}

pub fn assert_unknown_package_scope_usage_error_exits_2_without_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;
    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn app() { let mut value = 1; let _ = value; }\n",
    )?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(&root)
        .arg("--output")
        .arg(&output_path)
        .arg("--repo-root")
        .arg(repo_root()?)
        .arg("--package")
        .arg("missing-app")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(
        &output,
        "unknown --package missing-app: no matching package name or package ID in cargo metadata",
    );
    assert!(!output_path.exists());
    Ok(())
}
