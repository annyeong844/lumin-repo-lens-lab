use anyhow::{Context, Result};
use serde_json::Value;

pub fn assert_no_findings(artifact: &Value) -> Result<()> {
    assert!(artifact["findings"]
        .as_array()
        .context("findings array")?
        .is_empty());
    Ok(())
}
