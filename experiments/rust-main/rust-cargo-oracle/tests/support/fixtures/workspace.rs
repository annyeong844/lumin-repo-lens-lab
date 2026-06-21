#![allow(dead_code)]

use std::fs;

use anyhow::Result;
use tempfile::TempDir;

use super::super::RealCargoEnv;

impl RealCargoEnv {
    pub fn workspace_with_selected_local_dependency_and_unselected_member() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("app").join("src"))?;
        fs::create_dir_all(root.join("local_dep").join("src"))?;
        fs::create_dir_all(root.join("unused_member").join("src"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"local_dep\", \"unused_member\"]\ndefault-members = [\"app\"]\nresolver = \"2\"\n",
        )?;
        fs::write(
            root.join("app").join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nlocal_dep = { path = \"../local_dep\" }\n",
        )?;
        fs::write(
            root.join("app").join("src").join("lib.rs"),
            "pub fn app() -> u32 { local_dep::value() }\n",
        )?;
        fs::write(
            root.join("local_dep").join("Cargo.toml"),
            "[package]\nname = \"local_dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("local_dep").join("src").join("lib.rs"),
            "pub fn value() -> u32 { 1 }\n",
        )?;
        fs::write(
            root.join("unused_member").join("Cargo.toml"),
            "[package]\nname = \"unused_member\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("unused_member").join("src").join("lib.rs"),
            "pub fn unused() -> u32 { 1 }\n",
        )?;
        Self::from_root(temp, root)
    }

    pub fn workspace_with_unselected_member() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("app").join("src"))?;
        fs::create_dir_all(root.join("unused_member").join("src"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\", \"unused_member\"]\ndefault-members = [\"app\"]\nresolver = \"2\"\n",
        )?;
        fs::write(
            root.join("app").join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("app").join("src").join("lib.rs"),
            "pub fn app() {}\n",
        )?;
        fs::write(
            root.join("unused_member").join("Cargo.toml"),
            "[package]\nname = \"unused_member\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("unused_member").join("src").join("lib.rs"),
            "pub fn unused() -> u32 { 1 }\n",
        )?;
        Self::from_root(temp, root)
    }

    pub fn workspace_with_dependency_error() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("app").join("src"))?;
        fs::create_dir_all(root.join("bad_dep").join("src"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"app\"]\ndefault-members = [\"app\"]\nresolver = \"2\"\n",
        )?;
        fs::write(
            root.join("app").join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nbad_dep = { path = \"../bad_dep\" }\n",
        )?;
        fs::write(
            root.join("app").join("src").join("lib.rs"),
            "pub fn app() { let _ = bad_dep::dep(); }\n",
        )?;
        fs::write(
            root.join("bad_dep").join("Cargo.toml"),
            "[package]\nname = \"bad_dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("bad_dep").join("src").join("lib.rs"),
            "pub fn dep() -> u32 { \"wrong\" }\n",
        )?;
        Self::from_root(temp, root)
    }
}
