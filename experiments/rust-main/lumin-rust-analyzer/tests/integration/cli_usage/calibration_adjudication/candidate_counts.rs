use anyhow::{Context, Result};

use crate::support::scenarios::invalid_calibration_adjudication::{
    run_malformed_candidate_counts_fields, run_present_but_invalid_candidate_counts,
};

use super::helpers::{assert_no_readiness_reason, assert_readiness_reason};

#[test]
fn unified_cli_malformed_candidate_counts_fields_are_missing_like_js_ts_measurement() -> Result<()>
{
    let run = run_malformed_candidate_counts_fields()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let calibration = &artifact["oracleBridge"]["policy"]["calibration"];
    assert_eq!(calibration["candidateCounts"]["safeFix"], 0);
    assert_eq!(calibration["candidateCounts"]["reviewVisibleCleanup"], 1);
    assert_no_readiness_reason(&calibration["readiness"], "candidate-counts-unavailable");
    assert_no_readiness_reason(&calibration["readiness"], "adjudication-candidate-mismatch");
    Ok(())
}

#[test]
fn unified_cli_present_but_invalid_candidate_counts_are_unavailable_like_js_ts_measurement(
) -> Result<()> {
    let run = run_present_but_invalid_candidate_counts()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let calibration = &artifact["oracleBridge"]["policy"]["calibration"];
    assert_eq!(calibration["candidateCounts"]["available"], false);
    assert_eq!(calibration["candidateCounts"]["safeFix"], 0);
    assert_eq!(calibration["candidateCounts"]["reviewVisibleCleanup"], 0);
    assert_readiness_reason(
        &calibration["readiness"],
        "candidate-counts-unavailable",
        "red",
    );
    assert_no_readiness_reason(&calibration["readiness"], "adjudication-candidate-mismatch");
    Ok(())
}
