use super::super::super::resolve_target_triple;
use super::super::support::fallback_toolchain;
use crate::environment::CompilationEnvironment;
use crate::protocol::OracleTargetTripleSource;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[test]
fn target_triple_prefers_injected_environment_over_cargo_config() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(root.join(".cargo"))?;
    fs::write(
        root.join(".cargo").join("config.toml"),
        "[build]\ntarget = \"config-target\"\n",
    )?;
    let environment = CompilationEnvironment::from_vars([("CARGO_BUILD_TARGET", "env-target")]);

    let (target, targets, source) =
        resolve_target_triple(&root, &fallback_toolchain(), &environment);

    assert_eq!(target, "env-target");
    assert_eq!(targets, vec!["env-target"]);
    assert_eq!(source, OracleTargetTripleSource::EnvCargoBuildTarget);
    Ok(())
}
