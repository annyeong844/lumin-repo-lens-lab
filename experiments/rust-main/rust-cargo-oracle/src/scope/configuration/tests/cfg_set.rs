use super::super::resolve_cfg_set;
use crate::environment::CompilationEnvironment;
use crate::protocol::OracleCfgSetSource;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

#[test]
fn cfg_set_uses_injected_rustflags_without_process_env() -> Result<()> {
    let temp = TempDir::new()?;
    let root = temp.path().join("crate");
    fs::create_dir_all(&root)?;
    let environment = CompilationEnvironment::from_vars([
        (
            "RUSTFLAGS",
            "--cfg lumin_direct --check-cfg cfg(lumin_direct)",
        ),
        ("CARGO_BUILD_RUSTFLAGS", "--cfg=lumin_build"),
        ("CARGO_ENCODED_RUSTFLAGS", "--cfg\u{1f}lumin_encoded"),
    ]);

    let (cfgs, source) = resolve_cfg_set(&root, &environment);

    assert_eq!(cfgs, vec!["lumin_build", "lumin_direct", "lumin_encoded"]);
    assert_eq!(source, OracleCfgSetSource::EnvRustflagsBestEffort);
    Ok(())
}
