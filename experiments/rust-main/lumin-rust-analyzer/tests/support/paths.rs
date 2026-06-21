use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .context("repo root ancestor")
        .map(Path::to_path_buf)
}
