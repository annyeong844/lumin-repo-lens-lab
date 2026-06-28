use anyhow::Result;

use crate::support::{coverage, real_cargo_env::RealCargoEnv};

pub fn assert_dependency_events_do_not_replace_selected_scope() -> Result<()> {
    let env = RealCargoEnv::workspace_with_dependency_error()?;
    let artifact = env.run()?;
    let scope = &coverage::coverage(&artifact, "cov.cargo-check.absence-clean")?["scope"];

    assert_eq!(scope["package"], "app");
    assert_eq!(scope["target"], "app");
    assert_eq!(scope["targets"][0]["packageName"], "app");
    assert_eq!(scope["targets"][0]["targetName"], "app");
    assert_eq!(
        scope["targets"][0]["source"],
        "cargo-metadata-default-selection"
    );
    Ok(())
}
