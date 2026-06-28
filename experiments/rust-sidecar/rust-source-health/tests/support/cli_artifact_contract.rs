#[path = "cli_artifact/mod.rs"]
mod cli_artifact;

use tempfile::TempDir;

use anyhow::Result;

use crate::cli::run_cli;
use cli_artifact::{assert_cli_artifact, assert_full_cli_artifact, write_cli_fixture};

pub fn assert_cli_collects_sources_and_writes_final_artifact_without_node_wrapper() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    write_cli_fixture(&root)?;

    let output_path = temp.path().join("rust-health.json");
    let output = run_cli(&[
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--output".to_string(),
        output_path.to_string_lossy().to_string(),
        "--source-commit".to_string(),
        "test-source-commit".to_string(),
        "--threads".to_string(),
        "2".to_string(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_cli_artifact(&output_path)
}

pub fn assert_cli_full_profile_preserves_raw_ast_fact_artifact() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("repo");
    write_cli_fixture(&root)?;

    let output_path = temp.path().join("rust-health-full.json");
    let output = run_cli(&[
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--output".to_string(),
        output_path.to_string_lossy().to_string(),
        "--source-commit".to_string(),
        "test-source-commit".to_string(),
        "--artifact-profile".to_string(),
        "full".to_string(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_full_cli_artifact(&output_path)
}
