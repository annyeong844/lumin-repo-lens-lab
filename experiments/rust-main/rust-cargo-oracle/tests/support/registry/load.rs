use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::Value;

pub fn cargo_check_oracle() -> Result<Value> {
    let registry = fs::read_to_string(repo_root().join("canonical").join("oracle-registry.json"))?;
    let registry: Value = serde_json::from_str(&registry)?;
    registry["oracles"]
        .as_array()
        .context("registry.oracles array")?
        .iter()
        .find(|entry| entry["id"] == "rust.cargo-check")
        .cloned()
        .context("rust.cargo-check oracle")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
