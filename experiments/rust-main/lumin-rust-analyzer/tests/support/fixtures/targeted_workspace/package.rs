use anyhow::Result;
use std::fs;
use std::path::Path;

pub(super) fn write_package(root: &Path, name: &str, lib_rs: &str) -> Result<()> {
    let package_root = root.join(name);
    fs::create_dir_all(package_root.join("src"))?;
    fs::write(
        package_root.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
    )?;
    fs::write(package_root.join("src").join("lib.rs"), lib_rs)?;
    Ok(())
}
