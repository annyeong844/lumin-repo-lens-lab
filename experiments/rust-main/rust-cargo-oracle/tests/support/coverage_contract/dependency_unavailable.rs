use anyhow::{Context, Result};

use crate::support::{coverage, findings::empty, real_cargo_env::RealCargoEnv};

pub fn assert_dependency_primary_error_is_not_user_finding() -> Result<()> {
    let env = RealCargoEnv::workspace_with_dependency_error()?;
    let artifact = env.run()?;

    empty::assert_no_findings(&artifact)?;
    let absence = coverage::coverage(&artifact, "cov.cargo-check.absence-clean")?;
    assert_eq!(absence["status"], "unavailable");
    assert!(absence["reason"]
        .as_str()
        .is_some_and(|reason| reason.contains("non-user-code primary error diagnostic")));
    assert_eq!(
        artifact["summary"]["semanticClean"]["status"],
        "unavailable"
    );
    assert!(!artifact["summary"]["semanticClean"]
        .as_object()
        .context("semanticClean summary object")?
        .contains_key("clean"));
    Ok(())
}
