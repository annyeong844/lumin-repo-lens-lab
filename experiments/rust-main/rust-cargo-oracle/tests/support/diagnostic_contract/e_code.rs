use anyhow::Result;

use crate::support::{confidence, diagnostics, findings::single, real_cargo_env::RealCargoEnv};

pub fn assert_type_error_is_verified_e_code() -> Result<()> {
    let env = RealCargoEnv::type_error()?;
    let artifact = env.run()?;
    let finding = single::single_finding(&artifact)?;

    confidence::assert_tier_and_claim(finding, "verified", "verified.rust.rustc-error-diagnostic");
    confidence::assert_first_authority(finding, "rust.rustc.error-diagnostic")?;
    diagnostics::assert_first_diagnostic_code(&artifact, "E0308")
}
