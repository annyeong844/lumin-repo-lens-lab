use anyhow::Result;
use tempfile::TempDir;

use crate::support::{
    action_blockers::assert_macro_expansion_blocker,
    findings::single::single_finding,
    real_cargo_env::process_env::{lock_process_env, with_env_var},
    real_cargo_env::RealCargoEnv,
    safe_action_shape::{
        assert_rule_backed_safe_action, assert_rule_backed_safe_action_contract,
        assert_rule_backed_safe_action_with_diagnostic_code,
        assert_rule_backed_safe_action_with_edit, assert_rule_backed_safe_action_with_edits,
    },
};

#[test]
fn machine_applicable_warning_from_macro_expansion_is_action_blocked() -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "macro_rules! make_mut { () => { let mut value = 1; let _ = value; }; }\npub fn app() { make_mut!(); }\n",
    )?;
    let artifact = env.run()?;
    let finding = single_finding(&artifact)?;

    assert!(finding.get("safeAction").is_none());
    assert_macro_expansion_blocker(finding, "make_mut!")
}

#[test]
fn machine_applicable_rule_backed_warning_without_allowlist_still_promotes_safe_action(
) -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn app() { let mut value = 1; let _ = value; }\n",
    )?;
    let artifact = env.run()?;

    assert!(!env.path_exists("target"));
    assert_rule_backed_safe_action(&artifact)
}

#[test]
fn unused_import_machine_applicable_edit_that_contains_primary_span_promotes_safe_action(
) -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "use std::fmt;\npub fn app() {}\n",
    )?;
    let artifact = env.run()?;

    assert_rule_backed_safe_action_with_diagnostic_code(&artifact, "unused_imports")
}

#[test]
fn unused_variable_machine_applicable_rename_promotes_safe_action() -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn app() { let value = 1; }\n",
    )?;
    let artifact = env.run()?;

    assert_rule_backed_safe_action_with_edit(&artifact, "unused_variables", "_value")
}

#[test]
fn multi_edit_machine_applicable_warning_promotes_single_safe_action() -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn app(flag: bool) -> i32 { if (flag) { 1 } else { 2 } }\n",
    )?;
    let artifact = env.run()?;

    assert_rule_backed_safe_action_with_edits(&artifact, "unused_parens", &["", " "])
}

#[test]
fn warning_lint_is_rule_backed_without_blocking_verified_error_clean() -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn app() { let mut value = 1; let _ = value; }\n",
    )?;
    let artifact = env.run()?;
    assert_rule_backed_safe_action_contract(&artifact)
}

#[test]
fn ambient_cargo_target_dir_does_not_receive_oracle_cargo_check_outputs() -> Result<()> {
    let _guard = lock_process_env();
    let ambient = TempDir::new()?;
    let ambient_target = ambient.path().join("shared-target");
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn app() { let mut value = 1; let _ = value; }\n",
    )?;

    let artifact = with_env_var("CARGO_TARGET_DIR", Some(ambient_target.as_os_str()), || {
        env.run_unlocked(lumin_rust_cargo_oracle::CargoCheckMode::CargoCheck)
    })?;

    assert!(
        !ambient_target.exists(),
        "oracle cargo check leaked outputs into {}",
        ambient_target.display()
    );
    assert!(!env.path_exists("target"));
    assert_rule_backed_safe_action(&artifact)
}
