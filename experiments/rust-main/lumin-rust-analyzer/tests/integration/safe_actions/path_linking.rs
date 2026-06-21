#[cfg(windows)]
use anyhow::Result;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use tempfile::TempDir;

#[cfg(windows)]
use crate::support::{cli, fixtures::package};

#[cfg(windows)]
use super::safe_action_policy_support::assert_safe_action_artifact;

#[cfg(windows)]
#[test]
fn unified_cli_links_safe_action_when_root_path_case_differs_from_rustc_span() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("Repo");
    package::write_single_package_crate(
        &root,
        "app",
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
    )?;
    let lower_case_root = PathBuf::from(root.display().to_string().to_ascii_lowercase());
    let output_path = temp.path().join("rust-analyzer-health.json");
    let artifact = cli::run_unified_analyzer(&lower_case_root, &output_path, Some("cargo-check"))?;

    assert_safe_action_artifact(&artifact)?;
    assert_eq!(artifact["summary"]["semanticUnlinkedFindings"], 0);
    assert_eq!(artifact["summary"]["semanticUnlinkedDiagnostics"], 0);
    assert_eq!(
        artifact["files"]["src/lib.rs"]["semantic"]["findings"][0]["index"],
        0
    );
    Ok(())
}
