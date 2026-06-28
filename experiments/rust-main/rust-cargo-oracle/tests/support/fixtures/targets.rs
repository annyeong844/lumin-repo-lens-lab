#![allow(dead_code)]

use std::fs;

use anyhow::Result;
use tempfile::TempDir;

use super::super::RealCargoEnv;

impl RealCargoEnv {
    pub fn multi_target_success() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"app_cli\"\npath = \"src/main.rs\"\n",
        )?;
        fs::write(root.join("src").join("lib.rs"), "pub fn app() {}\n")?;
        fs::write(root.join("src").join("main.rs"), "fn main() {}\n")?;
        Self::from_root(temp, root)
    }
}
