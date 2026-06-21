mod emitted;
mod reserved;

use anyhow::Result;

use crate::support::registry::cargo_check_oracle;

pub fn assert_cargo_diagnostic_claim_vocabulary() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    emitted::assert_emitted_claim_kinds(&cargo_check);
    reserved::assert_reserved_claim_kinds(&cargo_check);
    Ok(())
}
