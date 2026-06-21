use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::CargoCheckMode;

use crate::support::real_cargo_env::{
    process_env::{lock_process_env, with_rustflags},
    RealCargoEnv,
};

#[test]
fn analysis_input_hash_changes_when_rustflags_changes() -> Result<()> {
    let _guard = lock_process_env();
    let env = with_rustflags(None, RealCargoEnv::success)?;
    let before = with_rustflags(None, || analysis_input_hash(&env, "before"))?;
    let after = with_rustflags(Some("--cfg lumin_env_hash_test"), || {
        analysis_input_hash(&env, "after")
    })?;

    assert_ne!(before, after);
    Ok(())
}

fn analysis_input_hash(env: &RealCargoEnv, label: &str) -> Result<String> {
    Ok(
        env.run_unlocked(CargoCheckMode::CargoCheck)?["meta"]["analysisInputSetHash"]
            .as_str()
            .with_context(|| format!("{label} analysisInputSetHash"))?
            .to_string(),
    )
}
