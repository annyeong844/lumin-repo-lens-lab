use anyhow::Result;

use crate::support::{
    diagnostic_contract::{
        codeless::assert_panic_format_error_is_verified_codeless,
        e_code::assert_type_error_is_verified_e_code,
    },
    findings::single::single_finding,
    real_cargo_env::RealCargoEnv,
};

#[test]
fn e_code_user_error_is_verified_rustc_error_diagnostic() -> Result<()> {
    assert_type_error_is_verified_e_code()
}

#[test]
fn codeless_user_error_is_verified_rustc_codeless_error_diagnostic() -> Result<()> {
    assert_panic_format_error_is_verified_codeless()
}

#[test]
fn provenance_records_the_actual_cargo_binary() -> Result<()> {
    let env = RealCargoEnv::type_error()?;
    let artifact = env.run()?;
    let finding = single_finding(&artifact)?;

    assert_eq!(artifact["meta"]["input"]["cargoBin"], "cargo");
    assert_eq!(artifact["meta"]["input"]["cargoArgs"][0], "check");
    assert_eq!(finding["source"]["commandArgs"][0], "cargo");
    assert!(finding["source"]["command"]
        .as_str()
        .unwrap_or_default()
        .starts_with("cargo "));
    Ok(())
}
