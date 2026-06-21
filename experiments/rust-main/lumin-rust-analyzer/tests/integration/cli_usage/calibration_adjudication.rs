use anyhow::Result;

use crate::support::scenarios::invalid_calibration_adjudication::{
    run_invalid_calibration_adjudication, run_unknown_calibration_tier,
    run_unknown_calibration_verdict,
};

#[test]
fn unified_cli_invalid_calibration_adjudication_exits_1_before_writing_artifact() -> Result<()> {
    let run = run_invalid_calibration_adjudication()?;

    assert_eq!(run.output.status.code(), Some(1));
    assert!(run.output.stdout.is_empty());
    assert!(!run.artifact_exists);
    assert!(String::from_utf8_lossy(&run.output.stderr)
        .contains("failed to parse calibration adjudication"));
    Ok(())
}

#[test]
fn unified_cli_unknown_calibration_tier_exits_1_before_writing_artifact() -> Result<()> {
    let run = run_unknown_calibration_tier()?;

    assert_eq!(run.output.status.code(), Some(1));
    assert!(run.output.stdout.is_empty());
    assert!(!run.artifact_exists);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(stderr.contains("failed to parse calibration adjudication"));
    assert!(stderr.contains("unknown variant `SAFEISH`"));
    Ok(())
}

#[test]
fn unified_cli_unknown_calibration_verdict_exits_1_before_writing_artifact() -> Result<()> {
    let run = run_unknown_calibration_verdict()?;

    assert_eq!(run.output.status.code(), Some(1));
    assert!(run.output.stdout.is_empty());
    assert!(!run.artifact_exists);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(stderr.contains("failed to parse calibration adjudication"));
    assert!(stderr.contains("unknown variant `mostly_true`"));
    Ok(())
}
