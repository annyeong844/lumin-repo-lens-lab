use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn write_single_package_crate(root: &Path, package_name: &str, lib_rs: &str) -> Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )?;
    fs::write(root.join("src").join("lib.rs"), lib_rs)?;
    Ok(())
}
