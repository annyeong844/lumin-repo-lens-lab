#[path = "../support/review_policy/action_blocker/mod.rs"]
mod action_blocker_policy;
#[path = "../support/review_policy/codeless_error/mod.rs"]
mod codeless_error_policy;

use anyhow::Result;

use crate::support::scenarios::single_package::analyze_cargo_check_single_package;

#[test]
fn unified_cli_keeps_blocked_rustc_suggestion_in_review_fix() -> Result<()> {
    let artifact = analyze_cargo_check_single_package(
        "macro_rules! make_mut { () => { let mut value = 1; let _ = value; }; }\npub fn demo() { make_mut!(); }\n"
    )?;
    action_blocker_policy::assert_review_fix(&artifact)
}

#[test]
fn unified_cli_keeps_rustc_codeless_error_in_review_fix() -> Result<()> {
    let artifact = analyze_cargo_check_single_package("pub fn demo() { panic!(\"{}\") }\n")?;
    codeless_error_policy::assert_review_fix(&artifact)
}
