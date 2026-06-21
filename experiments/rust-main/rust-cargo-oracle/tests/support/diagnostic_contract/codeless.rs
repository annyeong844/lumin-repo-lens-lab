use anyhow::Result;

use crate::support::{confidence, diagnostics, findings::single, real_cargo_env::RealCargoEnv};

pub fn assert_panic_format_error_is_verified_codeless() -> Result<()> {
    let env = RealCargoEnv::single_package(
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        "pub fn app() { panic!(\"{}\") }\n",
    )?;
    let artifact = env.run()?;
    let finding = single::single_finding(&artifact)?;

    confidence::assert_tier_and_claim(
        finding,
        "verified",
        "verified.rust.rustc-codeless-error-diagnostic",
    );
    diagnostics::assert_first_diagnostic_is_codeless(&artifact)
}
