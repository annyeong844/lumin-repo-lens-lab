use anyhow::{Context, Result};

use crate::support::real_cargo_env::RealCargoEnv;

#[test]
fn artifact_marks_analysis_input_set_as_incomplete_for_reuse() -> Result<()> {
    let env = RealCargoEnv::success()?;
    let artifact = env.run()?;

    assert_eq!(artifact["meta"]["analysisInputSetComplete"], false);
    assert!(artifact["meta"]["missingInfluenceKinds"]
        .as_array()
        .context("missingInfluenceKinds")?
        .iter()
        .any(|kind| kind == "build-script-runtime-inputs"));
    assert_eq!(artifact["summary"]["cacheReuse"]["status"], "not-reusable");
    assert_eq!(
        artifact["summary"]["cacheReuse"]["policy"],
        "no-reuse-unless-complete-influence-set-is-captured"
    );
    Ok(())
}

#[test]
fn artifact_reports_build_script_target_as_cache_reuse_blocker() -> Result<()> {
    let env = RealCargoEnv::with_build_script()?;
    let artifact = env.run()?;

    assert_eq!(artifact["summary"]["cacheReuse"]["blockingTargetCount"], 1);
    let blocking_target = artifact["meta"]["cacheReuse"]["blockingTargets"]
        .as_array()
        .context("cacheReuse.blockingTargets")?
        .first()
        .context("first blocking target")?;
    assert_eq!(blocking_target["targetName"], "build-script-build");
    assert_eq!(
        blocking_target["targetKinds"]
            .as_array()
            .context("targetKinds")?,
        &["custom-build"]
    );
    Ok(())
}
