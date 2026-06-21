use anyhow::Result;
use std::fs;
use std::path::Path;

use super::package::write_package;

pub fn write_broad_targeted_workspace(root: &Path, package_count: usize) -> Result<()> {
    let package_names = (0..package_count)
        .map(|index| format!("pkg{index}"))
        .collect::<Vec<_>>();
    let members = package_names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::create_dir_all(root)?;
    fs::write(
        root.join("Cargo.toml"),
        format!("[workspace]\nmembers = [{members}]\nresolver = \"2\"\n"),
    )?;
    for name in &package_names {
        write_package(root, name, "pub fn demo() { custom_macro!(); }\n")?;
    }
    if let Some(name) = package_names.last() {
        let package_root = root.join(name);
        fs::write(
            package_root.join("src").join("lib.rs"),
            "mod extra;\npub fn demo() { custom_macro!(); }\n",
        )?;
        fs::write(
            package_root.join("src").join("extra.rs"),
            "pub fn extra() { custom_macro!(); }\n",
        )?;
    }
    Ok(())
}
