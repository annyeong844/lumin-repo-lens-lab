use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::support::command::unified_analyzer_command;

#[test]
fn unified_cli_invalid_root_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("rust-analyzer.json");
    let missing_root = temp.path().join("missing-root");

    let output = unified_analyzer_command()
        .arg("--root")
        .arg(&missing_root)
        .arg("--output")
        .arg(&output_path)
        .arg("--source-commit")
        .arg("test-source")
        .output()
        .context("run unified rust analyzer")?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output_path.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid --root"));
    Ok(())
}

#[test]
fn unified_cli_invalid_repo_root_exits_2_before_writing_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let output_path = temp.path().join("rust-analyzer.json");
    let analysis_root = temp.path().join("external-rust-repo");
    let invalid_repo_root = temp.path().join("external-rust-repo");
    std::fs::create_dir_all(&analysis_root)?;

    let output = unified_analyzer_command()
        .arg("--root")
        .arg(&analysis_root)
        .arg("--repo-root")
        .arg(&invalid_repo_root)
        .arg("--output")
        .arg(&output_path)
        .arg("--source-commit")
        .arg("test-source")
        .output()
        .context("run unified rust analyzer")?;

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output_path.exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid --repo-root"));
    assert!(stderr.contains("missing canonical/oracle-registry.json"));
    Ok(())
}
