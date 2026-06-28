use anyhow::Result;
use std::fs;
use std::path::Path;

use super::package::write_package;

pub fn write_two_package_targeted_workspace(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("app").join("src"))?;
    fs::create_dir_all(root.join("util").join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\", \"util\"]\nresolver = \"2\"\n",
    )?;
    write_package(
        root,
        "app",
        "pub fn app() { let value = Some(1); let _ = value.unwrap(); custom_macro!(); }\n",
    )?;
    write_package(
        root,
        "util",
        "pub fn util() -> i32 { \"util should not be checked\" }\n",
    )?;
    fs::write(
        root.join("loose.rs"),
        "pub fn loose() { custom_macro!(); }\n",
    )?;
    Ok(())
}
