#![allow(dead_code)]

mod run;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tempfile::TempDir;

use super::super::paths::repo_root;

pub struct RealCargoEnv {
    _temp: TempDir,
    root: PathBuf,
    repo_root: PathBuf,
}

impl RealCargoEnv {
    pub fn write_cargo_config(&self, config: &str) -> Result<()> {
        fs::create_dir_all(self.root.join(".cargo"))?;
        fs::write(self.root.join(".cargo").join("config.toml"), config)?;
        Ok(())
    }

    pub fn write_file(&self, path: impl AsRef<Path>, contents: &str) -> Result<()> {
        fs::write(self.root.join(path), contents)?;
        Ok(())
    }

    pub fn path_exists(&self, path: impl AsRef<Path>) -> bool {
        self.root.join(path).exists()
    }

    pub(super) fn from_root(temp: TempDir, root: PathBuf) -> Result<Self> {
        let repo_root = repo_root()?;
        Ok(Self {
            _temp: temp,
            root,
            repo_root,
        })
    }
}
