use anyhow::Result;
use serde_json::Value;

mod action_policy;
mod bridge_calibration;
mod bridge_core;
mod finding;
mod summary;

pub fn assert_safe_action_artifact(artifact: &Value) -> Result<()> {
    summary::assert_safe_action_summary(artifact);
    action_policy::assert_safe_action_policy(artifact)?;
    finding::assert_safe_action_finding(artifact)?;
    bridge_calibration::assert_safe_action_bridge(artifact)?;
    Ok(())
}

pub fn assert_safe_action_artifact_with_diagnostic_code(
    artifact: &Value,
    diagnostic_code: &str,
) -> Result<()> {
    assert_safe_action_artifact_with_edit(artifact, diagnostic_code, "")
}

pub fn assert_safe_action_artifact_with_edit(
    artifact: &Value,
    diagnostic_code: &str,
    replacement: &str,
) -> Result<()> {
    assert_safe_action_artifact_with_edits(artifact, diagnostic_code, &[replacement])
}

pub fn assert_safe_action_artifact_with_edits(
    artifact: &Value,
    diagnostic_code: &str,
    replacements: &[&str],
) -> Result<()> {
    summary::assert_safe_action_summary(artifact);
    action_policy::assert_safe_action_policy_with_edits(artifact, diagnostic_code, replacements)?;
    finding::assert_safe_action_finding_with_edits(artifact, diagnostic_code, replacements)?;
    bridge_calibration::assert_safe_action_bridge(artifact)?;
    Ok(())
}

pub fn assert_safe_action_calibrated_artifact(artifact: &Value) -> Result<()> {
    summary::assert_safe_action_summary(artifact);
    action_policy::assert_safe_action_policy(artifact)?;
    finding::assert_safe_action_finding(artifact)?;
    bridge_calibration::assert_safe_action_calibrated_bridge(artifact)?;
    Ok(())
}

pub fn assert_safe_action_false_positive_calibrated_artifact(artifact: &Value) -> Result<()> {
    summary::assert_safe_action_summary(artifact);
    action_policy::assert_safe_action_policy(artifact)?;
    finding::assert_safe_action_finding(artifact)?;
    bridge_calibration::assert_safe_action_false_positive_calibrated_bridge(artifact)?;
    Ok(())
}

pub fn assert_safe_action_unmatched_calibrated_artifact(artifact: &Value) -> Result<()> {
    summary::assert_safe_action_summary(artifact);
    action_policy::assert_safe_action_policy(artifact)?;
    finding::assert_safe_action_finding(artifact)?;
    bridge_calibration::assert_safe_action_unmatched_calibrated_bridge(artifact)?;
    Ok(())
}
