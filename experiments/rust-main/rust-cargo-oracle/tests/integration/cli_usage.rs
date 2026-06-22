use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tempfile::TempDir;

use crate::package_scope_usage;
use crate::support::{
    cli::{assert_usage_error, oracle_command},
    paths::repo_root,
};

#[test]
fn cli_missing_flag_value_exits_2_before_writing_artifact() -> Result<()> {
    let output = oracle_command()
        .arg("--root")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(&output, "--root requires a value");
    Ok(())
}

#[test]
fn cli_invalid_root_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");
    let missing_root = temp.path().join("missing-root");

    let output = oracle_command()
        .arg("--root")
        .arg(&missing_root)
        .arg("--output")
        .arg(&output_path)
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(&output, "invalid --root");
    assert!(!output_path.exists());
    Ok(())
}

#[test]
fn cli_invalid_repo_root_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");
    let analysis_root = temp.path().join("external-rust-repo");
    let invalid_repo_root = temp.path().join("external-rust-repo");
    std::fs::create_dir_all(&analysis_root)?;

    let output = oracle_command()
        .arg("--root")
        .arg(&analysis_root)
        .arg("--repo-root")
        .arg(&invalid_repo_root)
        .arg("--output")
        .arg(&output_path)
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(&output, "invalid --repo-root");
    assert_usage_error(&output, "missing canonical/oracle-registry.json");
    assert!(!output_path.exists());
    Ok(())
}

#[test]
fn cli_invalid_cargo_check_mode_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--cargo-check-mode")
        .arg("cargoish")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(&output, "invalid --cargo-check-mode value: cargoish");
    assert!(!output_path.exists());
    Ok(())
}

#[test]
fn cli_invalid_cargo_target_dir_mode_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--cargo-target-dir-mode")
        .arg("repo-target")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(
        &output,
        "invalid --cargo-target-dir-mode value: repo-target",
    );
    assert!(!output_path.exists());
    Ok(())
}

#[test]
fn cli_default_cargo_target_dir_mode_is_isolated_temp_without_repo_target() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("workspace");
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;
    fs::write(root.join("src").join("lib.rs"), "pub fn app() {}\n")?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(&root)
        .arg("--repo-root")
        .arg(repo_root()?)
        .arg("--output")
        .arg(&output_path)
        .arg("--cargo-check-mode")
        .arg("cargo-check")
        .output()
        .context("run rust cargo oracle")?;

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.join("target").exists());
    let artifact: Value =
        serde_json::from_slice(&fs::read(&output_path)?).context("semantic artifact JSON")?;
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirMode"],
        "isolated-temp"
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["repoTargetDirUsed"],
        false
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["ownedTempTargetDir"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["incrementalDisabled"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["debugSymbolsDisabled"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleCleanupOwnedTempTargetDirs"],
        true
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleIsolatedTargetDirMaxAgeSeconds"],
        86_400
    );
    assert_eq!(
        artifact["meta"]["input"]["cargoTargetDirPolicy"]["staleReusableTargetDirMaxAgeSeconds"],
        604_800
    );
    let target_dir = PathBuf::from(
        artifact["meta"]["input"]["cargoTargetDir"]
            .as_str()
            .context("cargoTargetDir")?,
    );
    assert!(target_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("lumin-rust-cargo-oracle-target-")));
    assert!(
        !target_dir.exists(),
        "isolated cargo target directory should be removed after CLI exit: {}",
        target_dir.display()
    );
    Ok(())
}

#[test]
fn cli_package_scope_usage_error_exits_2_without_artifact() -> Result<()> {
    package_scope_usage::assert_package_scope_usage_error_exits_2_without_artifact()
}

#[test]
fn cli_unknown_package_scope_usage_error_exits_2_without_artifact() -> Result<()> {
    package_scope_usage::assert_unknown_package_scope_usage_error_exits_2_without_artifact()
}
