use anyhow::{Context, Result};
use serde_json::Value;

use super::bridge_core::assert_safe_action_bridge_core;

pub(super) fn assert_safe_action_bridge(artifact: &Value) -> Result<()> {
    assert_safe_action_bridge_core(artifact)?;
    assert_calibration_candidate_counts(artifact, 1, 1)?;
    let readiness = readiness(artifact);
    assert_eq!(readiness["gate"], "red");
    assert_eq!(safe_fix(readiness)["fpRate"], serde_json::Value::Null);
    assert_readiness_reason(artifact, "fp-rate-unknown", "red")?;
    Ok(())
}

pub(super) fn assert_safe_action_calibrated_bridge(artifact: &Value) -> Result<()> {
    assert_safe_action_bridge_core(artifact)?;
    assert_calibration_candidate_counts(artifact, 1, 1)?;
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibrationStatus"],
        "measured"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["status"],
        "measured"
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["reason"],
        "rust-safe-fix-calibration-corpus-measured-with-readiness-limits"
    );
    let readiness = readiness(artifact);
    let safe_fix = safe_fix(readiness);
    assert_eq!(readiness["gate"], "yellow");
    assert_eq!(safe_fix["trueDead"], 1);
    assert_eq!(safe_fix["falsePositives"], 0);
    assert_eq!(safe_fix["fpRate"], 0.0);
    assert_eq!(review_visible_cleanup(readiness)["fpRate"], 0.0);
    assert_no_readiness_reason(artifact, "candidate-counts-unavailable")?;
    assert_no_readiness_reason(artifact, "fp-rate-unknown")?;
    assert_no_readiness_reason(artifact, "schema-roundtrip-not-attempted")?;
    assert_readiness_reason(artifact, "benchmark-incomplete", "yellow")?;
    Ok(())
}

pub(super) fn assert_safe_action_green_calibrated_bridge(artifact: &Value) -> Result<()> {
    assert_safe_action_bridge_core(artifact)?;
    assert_calibration_candidate_counts(artifact, 2, 2)?;
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibrationStatus"],
        "measured"
    );
    let readiness = readiness(artifact);
    let safe_fix = safe_fix(readiness);
    assert_eq!(readiness["gate"], "green");
    assert_eq!(safe_fix["trueDead"], 2);
    assert_eq!(safe_fix["falsePositives"], 0);
    assert_eq!(safe_fix["fpRate"], 0.0);
    assert_eq!(review_visible_cleanup(readiness)["fpRate"], 0.0);
    assert_no_readiness_reason(artifact, "candidate-counts-unavailable")?;
    assert_no_readiness_reason(artifact, "fp-rate-unknown")?;
    assert_no_readiness_reason(artifact, "schema-roundtrip-not-attempted")?;
    assert_no_readiness_reason(artifact, "benchmark-incomplete")?;
    Ok(())
}

pub(super) fn assert_safe_action_missing_schema_calibrated_bridge(artifact: &Value) -> Result<()> {
    assert_safe_action_bridge_core(artifact)?;
    assert_calibration_candidate_counts(artifact, 2, 2)?;
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibrationStatus"],
        "measured"
    );
    let readiness = readiness(artifact);
    let safe_fix = safe_fix(readiness);
    assert_eq!(readiness["gate"], "red");
    assert_eq!(safe_fix["fpRate"], 0.0);
    assert_no_readiness_reason(artifact, "fp-rate-unknown")?;
    assert_readiness_reason(artifact, "schema-roundtrip-not-attempted", "red")?;
    Ok(())
}

pub(super) fn assert_safe_action_false_positive_calibrated_bridge(artifact: &Value) -> Result<()> {
    assert_safe_action_bridge_core(artifact)?;
    assert_calibration_candidate_counts(artifact, 1, 1)?;
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibrationStatus"],
        "measured"
    );
    let readiness = readiness(artifact);
    let safe_fix = safe_fix(readiness);
    assert_eq!(readiness["gate"], "red");
    assert_eq!(safe_fix["trueDead"], 0);
    assert_eq!(safe_fix["falsePositives"], 1);
    assert_eq!(safe_fix["fpRate"], 1.0);
    assert_eq!(review_visible_cleanup(readiness)["fpRate"], 1.0);
    assert_readiness_reason(artifact, "safe-fix-fp-threshold", "red")?;
    assert_readiness_reason(artifact, "review-visible-fp-threshold", "red")?;
    Ok(())
}

pub(super) fn assert_safe_action_unmatched_calibrated_bridge(artifact: &Value) -> Result<()> {
    assert_safe_action_bridge_core(artifact)?;
    assert_calibration_candidate_counts(artifact, 1, 1)?;
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibrationStatus"],
        "measured"
    );
    let readiness = readiness(artifact);
    let safe_fix = safe_fix(readiness);
    assert_eq!(readiness["gate"], "red");
    assert_eq!(safe_fix["fpRate"], serde_json::Value::Null);
    assert_eq!(safe_fix["trueDead"], 0);
    assert_eq!(safe_fix["falsePositives"], 0);
    assert_readiness_reason(artifact, "fp-rate-unknown", "red")?;
    assert_readiness_reason(artifact, "adjudication-candidate-mismatch", "red")?;
    Ok(())
}

fn readiness(artifact: &Value) -> &Value {
    &artifact["oracleBridge"]["policy"]["calibration"]["readiness"]
}

fn safe_fix(readiness: &Value) -> &Value {
    &readiness["safeFix"]
}

fn review_visible_cleanup(readiness: &Value) -> &Value {
    &readiness["reviewVisibleCleanup"]
}

fn assert_calibration_candidate_counts(
    artifact: &Value,
    safe_fix: usize,
    review_visible_cleanup: usize,
) -> Result<()> {
    let counts = &artifact["oracleBridge"]["policy"]["calibration"]["candidateCounts"];
    assert_eq!(counts["safeFix"], safe_fix);
    assert_eq!(counts["reviewVisibleCleanup"], review_visible_cleanup);
    Ok(())
}

fn readiness_reasons(artifact: &Value) -> Result<&Vec<Value>> {
    readiness(artifact)["reasons"]
        .as_array()
        .context("calibration readiness reasons")
}

fn assert_readiness_reason(artifact: &Value, code: &str, severity: &str) -> Result<()> {
    let reasons = readiness_reasons(artifact)?;
    assert!(
        reasons
            .iter()
            .any(|reason| reason["code"] == code && reason["severity"] == severity),
        "missing calibration readiness reason {code}/{severity}"
    );
    Ok(())
}

fn assert_no_readiness_reason(artifact: &Value, code: &str) -> Result<()> {
    let reasons = readiness_reasons(artifact)?;
    assert!(
        !reasons.iter().any(|reason| reason["code"] == code),
        "unexpected calibration readiness reason {code}"
    );
    Ok(())
}
