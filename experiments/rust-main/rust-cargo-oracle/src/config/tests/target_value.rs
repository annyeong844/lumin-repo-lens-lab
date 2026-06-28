use super::super::read_build_target_from_config;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[test]
fn malformed_quoted_build_value_is_ignored_without_panic() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join(".cargo"))?;
    let config = root.join(".cargo").join("config.toml");
    fs::write(&config, "[build]\ntarget = \"\n")?;

    assert!(read_build_target_from_config(&config).is_none());
    Ok(())
}

#[test]
fn build_target_reads_multiline_toml_array() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join(".cargo"))?;
    let config = root.join(".cargo").join("config.toml");
    fs::write(
        &config,
        "[build]\ntarget = [\n  \"x86_64-unknown-linux-gnu\",\n  \"wasm32-unknown-unknown\",\n]\n",
    )?;

    assert_eq!(
        read_build_target_from_config(&config),
        Some(vec![
            "x86_64-unknown-linux-gnu".to_string(),
            "wasm32-unknown-unknown".to_string()
        ])
    );
    Ok(())
}
