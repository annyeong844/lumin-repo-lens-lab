use anyhow::Result;

use crate::support::scenarios::multi_package_usage_error::{
    run_multi_package_argument_usage_error, run_unknown_package_argument_usage_error,
};

#[test]
fn unified_cli_propagates_oracle_usage_errors_as_exit_2() -> Result<()> {
    let run = run_multi_package_argument_usage_error()?;

    assert_eq!(run.output.status.code(), Some(2));
    assert!(run.output.stdout.is_empty());
    assert!(!run.artifact_exists);
    assert!(String::from_utf8_lossy(&run.output.stderr)
        .contains("--package currently supports exact package names only"));
    Ok(())
}

#[test]
fn unified_cli_unknown_package_exits_2_before_writing_artifact() -> Result<()> {
    let run = run_unknown_package_argument_usage_error()?;

    assert_eq!(run.output.status.code(), Some(2));
    assert!(run.output.stdout.is_empty());
    assert!(!run.artifact_exists);
    assert!(String::from_utf8_lossy(&run.output.stderr).contains(
        "unknown --package missing-app: no matching package name or package ID in cargo metadata"
    ));
    Ok(())
}
