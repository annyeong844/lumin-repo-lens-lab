use super::super::super::super::resolve_target_triple;
use super::super::super::support::{empty_environment, fallback_toolchain};
use crate::protocol::OracleTargetTripleSource;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[test]
fn target_triple_prefers_deeper_project_config_over_parent() -> Result<()> {
    let temp = TempDir::new()?;
    let repo = temp.path().join("repo");
    let crate_root = repo.join("crate");
    fs::create_dir_all(repo.join(".cargo"))?;
    fs::create_dir_all(crate_root.join(".cargo"))?;
    fs::write(
        repo.join(".cargo").join("config.toml"),
        "[build]\ntarget = \"parent-target\"\n",
    )?;
    fs::write(
        crate_root.join(".cargo").join("config.toml"),
        "[build]\ntarget = \"project-target\"\n",
    )?;

    let (target, targets, source) =
        resolve_target_triple(&crate_root, &fallback_toolchain(), &empty_environment());

    assert_eq!(target, "project-target");
    assert_eq!(targets, vec!["project-target"]);
    assert!(matches!(
        source,
        OracleTargetTripleSource::CargoConfig(path) if path.contains(".cargo")
    ));
    Ok(())
}
