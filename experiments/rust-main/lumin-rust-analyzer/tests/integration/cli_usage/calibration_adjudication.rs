use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::scenarios::invalid_calibration_adjudication::{
    run_empty_corpus_identity_adjudication, run_invalid_calibration_adjudication,
    run_malformed_calibration_container_fields, run_malformed_calibration_entry_fields,
    run_malformed_candidate_counts_fields, run_malformed_corpus_fields,
    run_unknown_calibration_tier, run_unknown_calibration_verdict,
    run_unknown_corpus_name_adjudication, run_unnamed_corpus_adjudication,
};

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

#[test]
fn unified_cli_malformed_corpus_fields_do_not_hard_stop_like_js_ts_measurement() -> Result<()> {
    let run = run_malformed_corpus_fields()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_readiness_reason(readiness, "corpus-identity-missing", "red");
    assert_readiness_reason(readiness, "dirty-worktree-unknown", "red");
    assert_eq!(
        readiness_reason(readiness, "corpus-identity-missing")?["detail"],
        "(unnamed) lacks commit/snapshotId"
    );
    assert_eq!(
        readiness_reason(readiness, "dirty-worktree-unknown")?["detail"],
        "(unnamed) dirty state unknown"
    );
    Ok(())
}

#[test]
fn unified_cli_unnamed_corpus_keeps_fp_denominator_unknown_like_js_ts_measurement() -> Result<()> {
    let run = run_unnamed_corpus_adjudication()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_eq!(readiness["safeFix"]["fpRate"], 0.0);
    assert_readiness_reason(readiness, "fp-rate-unknown", "red");
    Ok(())
}

#[test]
fn unified_cli_missing_entry_corpus_name_uses_unknown_bucket_like_js_ts_measurement() -> Result<()>
{
    let run = run_unknown_corpus_name_adjudication()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_eq!(readiness["safeFix"]["fpRate"], 0.0);
    assert_no_readiness_reason(readiness, "fp-rate-unknown");
    Ok(())
}

#[test]
fn unified_cli_empty_corpus_identity_is_missing_like_js_ts_measurement() -> Result<()> {
    let run = run_empty_corpus_identity_adjudication()?;

    assert_eq!(run.output.status.code(), Some(0));
    assert!(run.artifact_exists);
    let artifact = run.artifact.as_ref().context("artifact written")?;
    let readiness = calibration_readiness(artifact);
    assert_readiness_reason(readiness, "corpus-identity-missing", "red");
    assert_readiness_reason(readiness, "dirty-worktree-without-snapshot", "red");
    assert_readiness_reason(readiness, "unresolved-high-finding", "red");
    assert_eq!(
        readiness_reason(readiness, "corpus-identity-missing")?["detail"],
        "cal lacks commit/snapshotId"
    );
    assert_eq!(
        readiness_reason(readiness, "dirty-worktree-without-snapshot")?["detail"],
        "cal dirty state lacks snapshot/contentHash"
    );
    assert_eq!(
        readiness_reason(readiness, "unresolved-high-finding")?["detail"],
        "2 unresolved HIGH finding(s)"
    );
    Ok(())
}

fn calibration_readiness(artifact: &Value) -> &Value {
    &artifact["oracleBridge"]["policy"]["calibration"]["readiness"]
}

fn assert_readiness_reason(readiness: &Value, code: &str, severity: &str) {
    assert!(
        readiness["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons
                .iter()
                .any(|reason| reason["code"] == code && reason["severity"] == severity)),
        "missing {code}/{severity} readiness reason: {}",
        readiness["reasons"]
    );
}

fn assert_no_readiness_reason(readiness: &Value, code: &str) {
    assert!(
        readiness["reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().all(|reason| reason["code"] != code)),
        "unexpected {code} readiness reason: {}",
        readiness["reasons"]
    );
}

fn readiness_reason<'a>(readiness: &'a Value, code: &str) -> Result<&'a Value> {
    readiness["reasons"]
        .as_array()
        .context("calibration readiness reasons")?
        .iter()
        .find(|reason| reason["code"] == code)
        .with_context(|| format!("missing {code} readiness reason"))
}
