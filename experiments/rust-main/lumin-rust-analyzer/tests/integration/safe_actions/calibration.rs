use anyhow::{Context, Result};

use crate::support::scenarios::single_package::{
    analyze_cargo_check_single_package_with_adjudication,
    analyze_cargo_check_single_package_with_complete_calibration_evidence,
    analyze_cargo_check_single_package_with_missing_schema_round_trip_evidence,
};

use super::safe_action_policy_support::{
    assert_safe_action_calibrated_artifact, assert_safe_action_false_positive_calibrated_artifact,
    assert_safe_action_green_calibrated_artifact,
    assert_safe_action_missing_schema_calibrated_artifact,
    assert_safe_action_unmatched_calibrated_artifact,
};

#[test]
fn unified_cli_uses_safe_fix_adjudication_for_calibration_readiness() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "corpus": [
    {
      "name": "rust-single-package-cargo-check",
      "commit": "abc123",
      "worktreeDirty": false,
      "locBucket": "25k"
    }
  ],
  "candidateCounts": {
    "available": true,
    "reviewVisibleCleanup": 1,
    "safeFix": 1,
    "reviewFix": 0,
    "degraded": 0,
    "muted": 0,
    "byCorpus": {
      "rust-single-package-cargo-check": { "reviewVisibleCleanup": 1, "safeFix": 1, "reviewFix": 0 }
    }
  },
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "true_dead",
      "file": "src\\lib.rs",
      "diagnosticCode": "unused_mut",
      "lineStart": 1,
      "symbol": "demo"
    }
  ],
  "schemaRoundTrip": {
    "attempted": true,
    "knownSchemaDriftBugs": []
  }
}"#,
    )?;
    assert_safe_action_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_uses_review_fix_adjudication_for_calibration_readiness() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "macro_rules! make_unused { () => { let value = 1; }; }\npub fn demo() { make_unused!(); }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "REVIEW_FIX",
      "verdict": "true_dead",
      "file": "src/lib.rs",
      "diagnosticCode": "unused_variables",
      "lineStart": 1,
      "symbol": "demo"
    }
  ]
}"#,
    )?;

    let readiness = &artifact["oracleBridge"]["policy"]["calibration"]["readiness"];
    assert_eq!(artifact["summary"]["semanticSafeActions"], 0);
    assert_eq!(artifact["summary"]["semanticReviewFindings"], 0);
    assert_eq!(
        artifact["actionPolicy"]["semanticActionBlockers"]["findings"],
        1
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["candidateCounts"]
            ["reviewVisibleCleanup"],
        1
    );
    assert_eq!(
        artifact["semanticFindings"][0]["diagnosticCode"],
        "unused_variables"
    );
    assert_eq!(readiness["reviewVisibleCleanup"]["trueDead"], 1);
    assert_eq!(readiness["reviewVisibleCleanup"]["falsePositives"], 0);
    assert_eq!(readiness["reviewVisibleCleanup"]["fpRate"], 0.0);
    let reasons = readiness["reasons"]
        .as_array()
        .context("calibration readiness reasons")?;
    assert!(
        !reasons
            .iter()
            .any(|reason| reason["code"] == "adjudication-candidate-mismatch"),
        "review-fix adjudication should match review-visible cleanup candidates"
    );
    assert!(
        !reasons
            .iter()
            .any(|reason| reason["code"] == "fp-rate-unknown"),
        "review-fix adjudication should provide the review-visible denominator"
    );
    Ok(())
}

#[test]
fn unified_cli_reaches_green_with_complete_calibration_evidence() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_complete_calibration_evidence(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
    )?;
    assert_safe_action_green_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_uses_complete_calibration_evidence_independent_of_current_safe_fix_population(
) -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_complete_calibration_evidence(
        "pub fn demo() -> i32 { 1 }\n",
    )?;

    assert_eq!(artifact["summary"]["semanticSafeActions"], 0);
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["candidateCounts"]["safeFix"],
        2
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["readiness"]["gate"],
        "green"
    );
    Ok(())
}

#[test]
fn unified_cli_blocks_green_when_schema_round_trip_evidence_is_missing() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_missing_schema_round_trip_evidence(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
    )?;
    assert_safe_action_missing_schema_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_keeps_false_positive_safe_fix_adjudication_red() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "false_positive",
      "file": "src/lib.rs",
      "diagnosticCode": "unused_mut",
      "lineStart": 1,
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_false_positive_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_ignores_unmatched_safe_fix_adjudication_for_readiness() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "true_dead",
      "file": "src/other.rs",
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_unmatched_calibrated_artifact(&artifact)
}

#[test]
fn unified_cli_ignores_same_file_adjudication_with_wrong_diagnostic_code() -> Result<()> {
    let artifact = analyze_cargo_check_single_package_with_adjudication(
        "pub fn demo() { let mut value = 1; let _ = value; }\n",
        r#"{
  "entries": [
    {
      "corpusName": "rust-single-package-cargo-check",
      "tier": "SAFE_FIX",
      "verdict": "true_dead",
      "file": "src/lib.rs",
      "diagnosticCode": "dead_code",
      "lineStart": 1,
      "symbol": "demo"
    }
  ]
}"#,
    )?;
    assert_safe_action_unmatched_calibrated_artifact(&artifact)
}
