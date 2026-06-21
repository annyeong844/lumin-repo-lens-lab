use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::support::real_cargo_env::{
    process_env::{lock_process_env, with_clean_compilation_env, with_env_var},
    RealCargoEnv,
};

#[test]
fn analysis_input_hash_changes_when_cargo_config_changes() -> Result<()> {
    let env = RealCargoEnv::success()?;
    let before = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("before analysisInputSetHash")?
        .to_string();

    env.write_cargo_config("[build]\nrustflags = [\"--cfg\", \"lumin_config_hash_test\"]\n")?;
    let after = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("after analysisInputSetHash")?
        .to_string();

    assert_ne!(before, after);
    Ok(())
}

#[test]
fn analysis_input_hash_ignores_ambient_cargo_target_dir() -> Result<()> {
    let _guard = lock_process_env();
    with_clean_compilation_env(|| {
        let env = RealCargoEnv::success()?;
        let before = env.run_unlocked(lumin_rust_cargo_oracle::CargoCheckMode::CargoCheck)?["meta"]
            ["analysisInputSetHash"]
            .as_str()
            .context("before analysisInputSetHash")?
            .to_string();

        let ambient = TempDir::new()?;
        let ambient_target = ambient.path().join("shared-target");
        let after = with_env_var("CARGO_TARGET_DIR", Some(ambient_target.as_os_str()), || {
            env.run_unlocked(lumin_rust_cargo_oracle::CargoCheckMode::CargoCheck)
        })?["meta"]["analysisInputSetHash"]
            .as_str()
            .context("after analysisInputSetHash")?
            .to_string();

        assert_eq!(before, after);
        assert!(
            !ambient_target.exists(),
            "oracle cargo check leaked outputs into {}",
            ambient_target.display()
        );
        assert!(!env.path_exists("target"));
        Ok(())
    })
}

#[test]
fn analysis_input_hash_ignores_unselected_workspace_member_source() -> Result<()> {
    let env = RealCargoEnv::workspace_with_unselected_member()?;
    let before = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("before analysisInputSetHash")?
        .to_string();

    env.write_file("unused_member/src/lib.rs", "pub fn unused() -> u32 { 2 }\n")?;
    let after = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("after analysisInputSetHash")?
        .to_string();

    assert_eq!(before, after);
    Ok(())
}

#[test]
fn analysis_input_hash_tracks_selected_package_local_dependency_source() -> Result<()> {
    let env = RealCargoEnv::workspace_with_selected_local_dependency_and_unselected_member()?;
    let before = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("before analysisInputSetHash")?
        .to_string();

    env.write_file("local_dep/src/lib.rs", "pub fn value() -> u32 { 2 }\n")?;
    let after_dependency_change = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("dependency change analysisInputSetHash")?
        .to_string();
    assert_ne!(before, after_dependency_change);

    env.write_file("unused_member/src/lib.rs", "pub fn unused() -> u32 { 2 }\n")?;
    let after_unused_member_change = env.run()?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("unused member change analysisInputSetHash")?
        .to_string();
    assert_eq!(after_dependency_change, after_unused_member_change);
    Ok(())
}
