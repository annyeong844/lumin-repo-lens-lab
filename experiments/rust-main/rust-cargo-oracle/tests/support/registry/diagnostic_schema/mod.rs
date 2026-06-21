mod code_namespace;
mod derivation;

use anyhow::Result;

use crate::support::registry::cargo_check_oracle;

pub fn assert_normalized_cargo_code_derivation() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let schema = &cargo_check["normalizedDiagnosticSchema"];
    code_namespace::assert_code_namespace(schema);
    derivation::assert_code_namespace_derivation(schema);
    Ok(())
}
