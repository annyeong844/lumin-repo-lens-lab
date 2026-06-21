use anyhow::Result;

use crate::support::scenarios::single_package::analyze_cargo_check_single_package;

use super::safe_action_policy_support::{
    assert_safe_action_artifact, assert_safe_action_artifact_with_diagnostic_code,
    assert_safe_action_artifact_with_edit, assert_safe_action_artifact_with_edits,
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
