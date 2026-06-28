use anyhow::{Context, Result};

pub fn serialized_string<T: serde::Serialize>(value: T) -> Result<String> {
    Ok(serde_json::to_value(value)?
        .as_str()
        .context("serialized enum string")?
        .to_string())
}
