#![allow(dead_code)]

use std::fs;

use anyhow::Result;
use tempfile::TempDir;

use super::super::RealCargoEnv;

impl RealCargoEnv {
    pub fn single_package(manifest: &str, source: &str) -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src"))?;
        fs::write(root.join("Cargo.toml"), manifest)?;
        fs::write(root.join("src").join("lib.rs"), source)?;
        Self::from_root(temp, root)
    }

    pub fn success() -> Result<Self> {
        Self::single_package(
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            "pub fn app() {}\n",
        )
    }

    pub fn with_build_script() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("src"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
        )?;
        fs::write(root.join("src").join("lib.rs"), "pub fn app() {}\n")?;
        fs::write(root.join("build.rs"), "fn main() {}\n")?;
        Self::from_root(temp, root)
    }

    pub fn type_error() -> Result<Self> {
        Self::single_package(
            "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            "pub fn app() { let _typed: u32 = \"wrong\"; }\n",
        )
    }

    pub fn targeted_timeout_workspace() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        fs::create_dir_all(root.join("aaa-slow").join("src"))?;
        fs::create_dir_all(root.join("bbb-error").join("src"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"aaa-slow\", \"bbb-error\"]\nresolver = \"2\"\n",
        )?;
        fs::write(
            root.join("aaa-slow").join("Cargo.toml"),
            "[package]\nname = \"aaa-slow\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
        )?;
        fs::write(
            root.join("aaa-slow").join("src").join("lib.rs"),
            "pub fn slow() {}\n",
        )?;
        fs::write(
            root.join("aaa-slow").join("build.rs"),
            "fn main() { std::thread::sleep(std::time::Duration::from_secs(30)); }\n",
        )?;
        fs::write(
            root.join("bbb-error").join("Cargo.toml"),
            "[package]\nname = \"bbb-error\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("bbb-error").join("src").join("lib.rs"),
            "pub fn error() { let _typed: u32 = \"wrong\"; }\n",
        )?;
        Self::from_root(temp, root)
    }

    pub fn targeted_local_dependency_ranking_workspace() -> Result<Self> {
        let temp = TempDir::new()?;
        let root = temp.path().join("workspace");
        for package in ["a-dep", "b-plain", "z-local-dep"] {
            fs::create_dir_all(root.join(package).join("src"))?;
        }
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"a-dep\", \"b-plain\", \"z-local-dep\"]\nresolver = \"2\"\n",
        )?;
        fs::write(
            root.join("a-dep").join("Cargo.toml"),
            "[package]\nname = \"a-dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nz-local-dep = { path = \"../z-local-dep\" }\n",
        )?;
        fs::write(
            root.join("a-dep").join("src").join("lib.rs"),
            "pub fn dep() { z_local_dep::helper(); }\n",
        )?;
        fs::write(
            root.join("b-plain").join("Cargo.toml"),
            "[package]\nname = \"b-plain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("b-plain").join("src").join("lib.rs"),
            "pub fn plain() {}\n",
        )?;
        fs::write(
            root.join("z-local-dep").join("Cargo.toml"),
            "[package]\nname = \"z-local-dep\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(
            root.join("z-local-dep").join("src").join("lib.rs"),
            "pub fn helper() {}\n",
        )?;
        Self::from_root(temp, root)
    }
}
