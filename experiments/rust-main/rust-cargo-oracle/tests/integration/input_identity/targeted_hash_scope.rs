use anyhow::{Context, Result};

use crate::support::real_cargo_env::RealCargoEnv;

#[test]
fn targeted_analysis_input_hash_tracks_selected_package_local_dependency_source() -> Result<()> {
    let env = RealCargoEnv::workspace_with_selected_local_dependency_and_unselected_member()?;
    let target_paths = vec!["app/src/lib.rs".to_string()];
    let before = env.run_targeted(target_paths.clone())?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("before targeted analysisInputSetHash")?
        .to_string();

    env.write_file("local_dep/src/lib.rs", "pub fn value() -> u32 { 3 }\n")?;
    let after_dependency_change = env.run_targeted(target_paths.clone())?["meta"]
        ["analysisInputSetHash"]
        .as_str()
        .context("targeted dependency change analysisInputSetHash")?
        .to_string();
    assert_ne!(before, after_dependency_change);

    env.write_file("unused_member/src/lib.rs", "pub fn unused() -> u32 { 3 }\n")?;
    let after_unused_member_change = env.run_targeted(target_paths)?["meta"]
        ["analysisInputSetHash"]
        .as_str()
        .context("targeted unused member change analysisInputSetHash")?
        .to_string();
    assert_eq!(after_dependency_change, after_unused_member_change);
    Ok(())
}

#[test]
fn targeted_analysis_input_hash_uses_normalized_target_path_set() -> Result<()> {
    let env = RealCargoEnv::workspace_with_selected_local_dependency_and_unselected_member()?;
    let first = env.run_targeted(vec![
        "app/src/lib.rs".to_string(),
        "local_dep/src/lib.rs".to_string(),
    ])?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("first targeted analysisInputSetHash")?
        .to_string();
    let second = env.run_targeted(vec![
        "local_dep\\src\\lib.rs".to_string(),
        "app/src/lib.rs".to_string(),
        "app/src/lib.rs".to_string(),
    ])?["meta"]["analysisInputSetHash"]
        .as_str()
        .context("second targeted analysisInputSetHash")?
        .to_string();

    assert_eq!(first, second);
    Ok(())
}
