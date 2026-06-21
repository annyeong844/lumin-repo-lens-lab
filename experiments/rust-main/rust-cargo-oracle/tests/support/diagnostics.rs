#![allow(dead_code)]

use anyhow::{Context, Result};
use serde_json::Value;

fn first_diagnostic(artifact: &Value) -> Result<&Value> {
    Ok(&artifact["diagnostics"]
        .as_array()
        .context("diagnostics array")?[0])
}

pub fn assert_first_diagnostic_code(artifact: &Value, code: &str) -> Result<()> {
    let diagnostic = first_diagnostic(artifact)?;
    assert_eq!(diagnostic["rawCode"]["code"], code);
    assert_eq!(diagnostic["normalized"]["codeValue"], code);
    Ok(())
}

pub fn assert_first_diagnostic_is_codeless(artifact: &Value) -> Result<()> {
    let diagnostic = first_diagnostic(artifact)?;
    assert!(diagnostic["rawCode"].is_null());
    assert_eq!(diagnostic["normalized"]["codePresence"], "present-null");
    Ok(())
}
