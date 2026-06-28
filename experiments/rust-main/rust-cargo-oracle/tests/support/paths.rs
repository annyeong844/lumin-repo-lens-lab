use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .context("repo root")
}
