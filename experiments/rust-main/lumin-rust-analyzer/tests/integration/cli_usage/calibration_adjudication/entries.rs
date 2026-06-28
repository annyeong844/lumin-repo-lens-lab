use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::scenarios::invalid_calibration_adjudication::{
    run_invalid_calibration_adjudication, run_malformed_calibration_container_fields,
    run_malformed_calibration_entry_fields, run_unknown_calibration_tier,
    run_unknown_calibration_verdict,
};

use super::helpers::{assert_no_readiness_reason, assert_readiness_reason, calibration_readiness};

#[test]
fn unified_cli_invalid_calibration_adjudication_exits_1_before_writing_artifact() -> Result<()> {
    let run = run_invalid_calibration_adjudication()?;

    assert_eq!(run.output.status.code(), Some(1));
    assert!(run.output.stdout.is_empty());
    assert!(!run.artifact_exists);
    assert!(run.artifact.is_none());
    assert!(String::from_utf8_lossy(&run.output.stderr)
        .contains("failed to parse calibration adjudication"));
    Ok(())
}

#[test]
fn unified_cli_unknown_calibration_tier_is_ignored_like_js_ts_measurement() -> Result<()> {
    let run = run_unknown_calibration_tier()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_eq!(readiness["safeFix"]["trueDead"], 0);
    assert_eq!(readiness["reviewVisibleCleanup"]["trueDead"], 0);
    Ok(())
}

#[test]
fn unified_cli_unknown_calibration_verdict_becomes_inconclusive_like_js_ts_measurement(
) -> Result<()> {
    let run = run_unknown_calibration_verdict()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_eq!(readiness["safeFix"]["inconclusive"], 1);
    assert_eq!(readiness["safeFix"]["fpRate"], Value::Null);
    Ok(())
}

#[test]
fn unified_cli_malformed_entry_fields_do_not_drop_object_like_js_ts_measurement() -> Result<()> {
    let run = run_malformed_calibration_entry_fields()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_eq!(readiness["safeFix"]["trueDead"], 1);
    assert_eq!(readiness["safeFix"]["fpRate"], 0.0);
    assert_no_readiness_reason(readiness, "fp-rate-unknown");
    Ok(())
}

#[test]
fn unified_cli_malformed_calibration_container_fields_degrade_like_js_ts_measurement() -> Result<()>
{
    let run = run_malformed_calibration_container_fields()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_readiness_reason(readiness, "candidate-counts-unavailable", "red");
    assert_readiness_reason(readiness, "fp-rate-unknown", "red");
    assert_no_readiness_reason(readiness, "schema-roundtrip-not-attempted");
    assert_readiness_reason(readiness, "schema-drift-known", "red");
    Ok(())
}
