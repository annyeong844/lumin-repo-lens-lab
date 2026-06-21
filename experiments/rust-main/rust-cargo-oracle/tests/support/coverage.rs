#![allow(dead_code)]

use anyhow::{Context, Result};
use serde_json::Value;

pub fn coverage<'a>(artifact: &'a Value, id: &str) -> Result<&'a Value> {
    artifact["coverage"]
        .as_array()
        .context("coverage array")?
        .iter()
        .find(|entry| entry["id"] == id)
        .with_context(|| format!("missing coverage entry {id}"))
}
