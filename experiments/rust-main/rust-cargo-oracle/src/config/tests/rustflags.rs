use super::super::read_build_rustflags_from_config;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[test]
fn build_rustflags_splits_string_value_like_environment_rustflags() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join(".cargo"))?;
    let config = root.join(".cargo").join("config.toml");
    fs::write(
        &config,
        "[build]\nrustflags = \"--cfg lumin_from_string\"\n",
    )?;

    assert_eq!(
        read_build_rustflags_from_config(&config),
        vec!["--cfg", "lumin_from_string"]
    );
    Ok(())
}
