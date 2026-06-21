#[path = "../support/safe_action_policy/mod.rs"]
mod safe_action_policy_support;

use anyhow::Result;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use tempfile::TempDir;

use crate::support::scenarios::single_package::{
    analyze_cargo_check_single_package, analyze_cargo_check_single_package_with_adjudication,
};
#[cfg(windows)]
use crate::support::{cli, fixtures::package};
use safe_action_policy_support::{
    assert_safe_action_artifact, assert_safe_action_artifact_with_diagnostic_code,
    assert_safe_action_artifact_with_edit, assert_safe_action_artifact_with_edits,
    assert_safe_action_calibrated_artifact, assert_safe_action_false_positive_calibrated_artifact,
    assert_safe_action_unmatched_calibrated_artifact,
};

#[test]
fn unified_cli_promotes_rustc_machine_applicable_warning_to_safe_fix() -> Result<()> {
    let artifact = analyze_cargo_check_single_package(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
    )?;
    assert_safe_action_artifact(&artifact)
}

#[test]
fn unified_cli_promotes_unused_import_machine_applicable_warning_to_safe_fix() -> Result<()> {
    let artifact = analyze_cargo_check_single_package("use std::fmt;\npub fn demo() {}\n")?;

    assert_safe_action_artifact_with_diagnostic_code(&artifact, "unused_imports")
}

#[test]
fn unified_cli_promotes_unused_variable_machine_applicable_rename_to_safe_fix() -> Result<()> {
    let artifact = analyze_cargo_check_single_package("pub fn demo() { let value = 1; }\n")?;

    assert_safe_action_artifact_with_edit(&artifact, "unused_variables", "_value")
}

#[test]
fn unified_cli_promotes_multi_edit_machine_applicable_warning_to_safe_fix() -> Result<()> {
    let artifact = analyze_cargo_check_single_package(
        "pub fn demo(flag: bool) -> i32 { if (flag) { 1 } else { 2 } }\n",
    )?;

    assert_safe_action_artifact_with_edits(&artifact, "unused_parens", &["", " "])
}

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

#[test]
fn unified_cli_uses_safe_fix_adjudication_for_calibration_readiness() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "true_dead",
      "file": "src\\lib.rs",
      "diagnosticCode": "unused_mut",
      "lineStart": 1,
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_keeps_false_positive_safe_fix_adjudication_red() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "false_positive",
      "file": "src/lib.rs",
      "diagnosticCode": "unused_mut",
      "lineStart": 1,
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_false_positive_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_ignores_unmatched_safe_fix_adjudication_for_readiness() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "true_dead",
      "file": "src/other.rs",
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_unmatched_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_ignores_same_file_adjudication_with_wrong_diagnostic_code() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "true_dead",
      "file": "src/lib.rs",
      "diagnosticCode": "dead_code",
      "lineStart": 1,
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_unmatched_calibrated_artifact(&artifact)
}
