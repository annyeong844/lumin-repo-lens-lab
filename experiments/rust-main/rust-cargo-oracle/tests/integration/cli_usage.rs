use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::package_scope_usage;
use crate::support::cli::{assert_usage_error, oracle_command};

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
fn cli_invalid_timeout_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--timeout-ms")
        .arg("soon")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(&output, "invalid --timeout-ms value: soon");
    assert!(!output_path.exists());
    Ok(())
}

#[test]
fn cli_invalid_targeted_package_cap_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("semantic-health.json");

    let output = oracle_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--targeted-package-cap")
        .arg("0")
        .output()
        .context("run rust cargo oracle")?;

    assert_usage_error(&output, "--targeted-package-cap must be greater than zero");
    assert!(!output_path.exists());
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
