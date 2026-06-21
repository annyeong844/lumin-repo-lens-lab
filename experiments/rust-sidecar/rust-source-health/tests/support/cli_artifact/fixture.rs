use std::fs;
use std::path::Path;

use anyhow::Result;

pub fn write_cli_fixture(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("target"))?;
    fs::create_dir_all(root.join("vendor"))?;
    fs::write(
        root.join("src").join("lib.rs"),
        "pub fn main() { let value = Some(1); let _ = value.unwrap(); }\n",
    )?;
    fs::write(root.join("src").join("bad.rs"), [0xff, 0xfe, 0xfd])?;
    fs::write(
        root.join("target").join("generated.rs"),
        "pub fn hidden() { panic!(\"nope\"); }\n",
    )?;
    fs::write(
        root.join("vendor").join("vendored.rs"),
        "pub fn hidden() { panic!(\"nope\"); }\n",
    )?;
    Ok(())
}
