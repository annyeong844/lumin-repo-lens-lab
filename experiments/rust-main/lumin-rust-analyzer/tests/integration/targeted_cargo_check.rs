use anyhow::Result;

use crate::support::scenarios::single_package::{
    analyze_targeted_single_package, analyze_targeted_single_package_with_integration,
};
use crate::support::scenarios::targeted_broad_workspace::analyze_targeted_broad_workspace;
use crate::support::scenarios::targeted_two_package_workspace::analyze_targeted_two_package_workspace;
use crate::support::targeted_cargo_check;

#[test]
fn unified_cli_targeted_cargo_check_runs_only_package_with_review_syntax() -> Result<()> {
    let artifact = analyze_targeted_two_package_workspace()?;
    targeted_cargo_check::assert_targeted_package_scope(&artifact)
}

#[test]
fn unified_cli_targeted_cargo_check_runs_package_with_review_derive_macro() -> Result<()> {
    let artifact = analyze_targeted_single_package("#[derive(CustomDerive)]\npub struct Demo;\n")?;
    targeted_cargo_check::assert_review_derive_macro_run(&artifact)
}

#[test]
fn unified_cli_targeted_cargo_check_runs_current_scope_for_cfg_opacity() -> Result<()> {
    let artifact = analyze_targeted_single_package(
        "#[cfg(feature = \"fast\")]\npub fn fast() {}\npub fn demo() {}\n",
    )?;
    targeted_cargo_check::assert_cfg_opacity_runs_oracle(&artifact)
}

#[test]
fn unified_cli_targeted_cargo_check_skips_when_only_muted_syntax_exists() -> Result<()> {
    let artifact = analyze_targeted_single_package_with_integration(
        "pub fn demo() {}\n",
        "pub fn helper() { let value = Some(1); let _ = value.unwrap(); }\n",
    )?;
    targeted_cargo_check::assert_muted_syntax_skip(&artifact)
}

#[test]
fn unified_cli_targeted_cargo_check_runs_all_broad_package_scope_by_default() -> Result<()> {
    let artifact = analyze_targeted_broad_workspace(17)?;
    targeted_cargo_check::assert_broad_scope_uncapped_run(&artifact)
}

#[test]
fn unified_cli_targeted_cargo_check_skips_style_review_signals_without_compiler_opacity(
) -> Result<()> {
    let artifact = analyze_targeted_single_package(
        "pub fn demo() { let value = Some(1); let _ = value.unwrap(); }\n",
    )?;
    targeted_cargo_check::assert_style_signal_skip(&artifact)
}
