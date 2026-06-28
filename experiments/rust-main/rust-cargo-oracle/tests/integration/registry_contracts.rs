use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::protocol::ClaimKind;

use crate::support::registry::{
    array_contains_string, cargo_check_oracle, serialized_string, strings,
};
use crate::{
    absence_claim_sets_contract, claim_vocabulary_contract, classifier_rules_declared_contract,
    classifier_rules_known_contract, coverage_kinds_contract, diagnostic_schema,
    registry_coverage_contract,
};

#[test]
fn absence_clean_claim_sets_match_registry() -> Result<()> {
    absence_claim_sets_contract::assert_absence_clean_claim_sets_match_registry()
}

#[test]
fn classifier_claim_kinds_are_declared_in_registry() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let registry_claims = strings(&cargo_check["claimKinds"]);
    for claim_kind in ClaimKind::EMITTED_BY_CLASSIFIER {
        let claim_kind = serialized_string(claim_kind)?;
        assert!(
            registry_claims.contains(&claim_kind),
            "classifier claim kind {claim_kind} is not declared in oracle-registry.json"
        );
    }
    Ok(())
}

#[test]
fn uses_cargo_diagnostic_claim_vocabulary() -> Result<()> {
    claim_vocabulary_contract::assert_cargo_diagnostic_claim_vocabulary()
}

#[test]
fn classifier_rules_are_declared_in_registry() -> Result<()> {
    classifier_rules_declared_contract::assert_classifier_rules_are_declared_in_registry()
}

#[test]
fn registry_diagnostic_rules_are_known_to_rust_code() -> Result<()> {
    classifier_rules_known_contract::assert_registry_diagnostic_rules_are_known_to_rust_code()
}

#[test]
fn keeps_clean_scoped_to_verified_rustc_error_absence() -> Result<()> {
    registry_coverage_contract::assert_clean_scope_is_verified_rustc_error_absence()
}

#[test]
fn coverage_kinds_are_declared_in_registry() -> Result<()> {
    coverage_kinds_contract::assert_coverage_kinds_are_declared_in_registry()
}

#[test]
fn does_not_promote_footer_or_non_user_diagnostics_to_user_findings() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let rules = cargo_check["diagnosticClassification"]
        .as_array()
        .context("diagnosticClassification array")?
        .iter()
        .filter_map(|entry| entry["rule"].as_str())
        .collect::<Vec<_>>();

    assert!(rules.contains(&"note-help-failure-note-are-not-findings"));
    assert!(rules.contains(&"non-user-primary-error-makes-absence-clean-unavailable"));
    assert!(rules.contains(&"non-user-primary-diagnostics-are-not-user-facing-findings"));
    Ok(())
}

#[test]
fn documents_normalized_cargo_code_derivation() -> Result<()> {
    diagnostic_schema::assert_normalized_cargo_code_derivation()
}

#[test]
fn requires_input_identity_to_include_local_rust_source_bytes() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    assert!(array_contains_string(
        &cargo_check["analysisInputSet"],
        "local package Rust source bytes"
    ));
    Ok(())
}
