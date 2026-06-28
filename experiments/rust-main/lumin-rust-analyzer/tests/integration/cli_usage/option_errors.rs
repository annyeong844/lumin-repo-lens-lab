use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::support::command::unified_analyzer_command;

#[test]
fn unified_cli_missing_flag_value_exits_2_before_writing_artifact() -> Result<()> {
    let output = unified_analyzer_command()
        .arg("--root")
        .output()
        .context("run unified rust analyzer")?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--root requires a value"));
    Ok(())
}

#[test]
fn unified_cli_invalid_semantic_mode_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("rust-analyzer.json");

    let output = unified_analyzer_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--source-commit")
        .arg("test-source")
        .arg("--semantic-mode")
        .arg("cargoish")
        .output()
        .context("run unified rust analyzer")?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output_path.exists());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("invalid --semantic-mode value: cargoish")
    );
    Ok(())
}

#[test]
fn unified_cli_invalid_cargo_target_dir_mode_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("rust-analyzer.json");

    let output = unified_analyzer_command()
        .arg("--root")
        .arg(temp.path())
        .arg("--output")
        .arg(&output_path)
        .arg("--source-commit")
        .arg("test-source")
        .arg("--cargo-target-dir-mode")
        .arg("repo-target")
        .output()
        .context("run unified rust analyzer")?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output_path.exists());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("invalid --cargo-target-dir-mode value: repo-target"));
    Ok(())
}
