use anyhow::{Context, Result};
use serde_json::Value;

pub fn single_finding(artifact: &Value) -> Result<&Value> {
    let findings = artifact["findings"].as_array().context("findings array")?;
    assert_eq!(findings.len(), 1);
    Ok(&findings[0])
}
