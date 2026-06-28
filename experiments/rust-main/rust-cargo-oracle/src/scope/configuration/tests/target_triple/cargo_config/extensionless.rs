use super::super::super::super::resolve_target_triple;
use super::super::super::support::{empty_environment, fallback_toolchain};
use crate::protocol::OracleTargetTripleSource;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[test]
fn target_triple_prefers_extensionless_config_over_config_toml() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join(".cargo"))?;
    fs::write(
        root.join(".cargo").join("config"),
        "[build]\ntarget = \"extensionless-target\"\n",
    )?;
    fs::write(
        root.join(".cargo").join("config.toml"),
        "[build]\ntarget = \"toml-target\"\n",
    )?;

    let (target, targets, source) =
        resolve_target_triple(&root, &fallback_toolchain(), &empty_environment());

    assert_eq!(target, "extensionless-target");
    assert_eq!(targets, vec!["extensionless-target"]);
    assert!(matches!(
        source,
        OracleTargetTripleSource::CargoConfig(path) if path.ends_with("config")
    ));
    Ok(())
}
