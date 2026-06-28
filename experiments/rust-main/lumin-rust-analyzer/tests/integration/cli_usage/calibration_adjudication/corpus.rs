use anyhow::{Context, Result};

use crate::support::scenarios::invalid_calibration_adjudication::{
    run_empty_corpus_identity_adjudication, run_malformed_corpus_fields,
    run_unknown_corpus_name_adjudication, run_unnamed_corpus_adjudication,
};

use super::helpers::{
    assert_no_readiness_reason, assert_readiness_reason, calibration_readiness, readiness_reason,
};

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
