use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::protocol::{ClaimKind, ClassificationRule, CoverageKind};
use serde_json::{json, Value};

#[test]
fn uses_cargo_diagnostic_claim_vocabulary() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;

    assert_eq!(
        cargo_check["claimKinds"],
        json!([
            "verified.rust.rustc-error-diagnostic",
            "verified.rust.rustc-codeless-error-diagnostic",
            "rule-backed.rust.rustc-lint-diagnostic",
            "candidate.rust.unclassified-cargo-diagnostic"
        ])
    );
    assert!(!array_contains_string(
        &cargo_check["claimKinds"],
        "rule-backed.rust.rustc-lint-warning"
    ));
    assert!(!array_contains_string(
        &cargo_check["authorityIds"],
        "rust.rustc.lint-diagnostic"
    ));
    assert_eq!(
        cargo_check["reservedClaimKinds"],
        json!([
            "verified.rust.type-diagnostic",
            "verified.rust.borrow-diagnostic",
            "verified.rust.name-resolution-diagnostic",
            "verified.rust.cfg-expanded-diagnostic"
        ])
    );
    Ok(())
}

#[test]
fn documents_normalized_cargo_code_derivation() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;

    assert_eq!(
        cargo_check["normalizedDiagnosticSchema"]["codeNamespace"],
        json!([
            "rustc-codeless",
            "rustc-error",
            "rustc-non-ecode",
            "unknown"
        ])
    );
    assert_eq!(
        cargo_check["normalizedDiagnosticSchema"]["codeNamespaceDerivation"],
        json!([
            {
                "when": "message.code === null",
                "codeNamespace": "rustc-codeless",
                "codeKind": "null-error-code"
            },
            {
                "when": "message.code.code matches ^E[0-9]+$",
                "codeNamespace": "rustc-error",
                "codeKind": "rustc-error-code"
            },
            {
                "when": "message.code.code is any other non-empty string",
                "codeNamespace": "rustc-non-ecode",
                "codeKind": "non-ecode-name"
            },
            {
                "when": "message.code is missing or malformed",
                "codeNamespace": "unknown",
                "codeKind": "unknown"
            }
        ])
    );
    Ok(())
}

#[test]
fn keeps_clean_scoped_to_verified_rustc_error_absence() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let absence_clean = cargo_check["coverageKinds"]
        .as_array()
        .context("coverageKinds array")?
        .iter()
        .find(|entry| entry["coverageKind"] == "absence-clean")
        .context("absence-clean coverage kind")?;

    assert_eq!(
        absence_clean,
        &json!({
            "coverageKind": "absence-clean",
            "cleanKind": "verified-rustc-error-absence",
            "absenceOfClaimKinds": [
                "verified.rust.rustc-error-diagnostic",
                "verified.rust.rustc-codeless-error-diagnostic"
            ],
            "allowsConcurrentClaimKinds": [
                "rule-backed.rust.rustc-lint-diagnostic",
                "candidate.rust.unclassified-cargo-diagnostic"
            ]
        })
    );
    Ok(())
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
fn classifier_rules_are_declared_in_registry() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let registry_rules = cargo_check["diagnosticClassification"]
        .as_array()
        .context("diagnosticClassification array")?
        .iter()
        .filter_map(|entry| entry["rule"].as_str())
        .collect::<Vec<_>>();

    for rule in ClassificationRule::EMITTED_BY_CLASSIFIER {
        let rule = serde_json::to_value(rule)?
            .as_str()
            .context("serialized rule string")?
            .to_string();
        assert!(
            registry_rules.contains(&rule.as_str()),
            "classifier rule {rule} is not declared in oracle-registry.json"
        );
    }
    Ok(())
}

#[test]
fn registry_diagnostic_rules_are_known_to_rust_code() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let code_rules = ClassificationRule::EMITTED_BY_CLASSIFIER
        .iter()
        .map(|rule| serialized_string(*rule))
        .collect::<Result<BTreeSet<_>>>()?;
    for rule in cargo_check["diagnosticClassification"]
        .as_array()
        .context("diagnosticClassification array")?
        .iter()
        .filter_map(|entry| entry["rule"].as_str())
    {
        assert!(
            code_rules.contains(rule),
            "registry rule {rule} is not represented in ClassificationRule"
        );
    }
    Ok(())
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
fn coverage_kind_and_absence_claim_sets_match_registry() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let coverage_kinds = cargo_check["coverageKinds"]
        .as_array()
        .context("coverageKinds array")?;
    let registry_kinds = coverage_kinds
        .iter()
        .filter_map(|entry| entry["coverageKind"].as_str())
        .collect::<BTreeSet<_>>();
    for coverage_kind in CoverageKind::EMITTED_BY_ORACLE {
        let coverage_kind = serialized_string(coverage_kind)?;
        assert!(
            registry_kinds.contains(coverage_kind.as_str()),
            "coverage kind {coverage_kind} is not declared in oracle-registry.json"
        );
    }

    let absence_clean = coverage_kinds
        .iter()
        .find(|entry| entry["coverageKind"] == "absence-clean")
        .context("absence-clean coverage kind")?;
    assert_eq!(
        strings(&absence_clean["absenceOfClaimKinds"]),
        ClaimKind::ABSENCE_CLEAN_CLAIM_KINDS
            .iter()
            .map(|claim| serialized_string(*claim))
            .collect::<Result<BTreeSet<_>>>()?
    );
    assert_eq!(
        strings(&absence_clean["allowsConcurrentClaimKinds"]),
        ClaimKind::ABSENCE_CLEAN_CONCURRENT_CLAIM_KINDS
            .iter()
            .map(|claim| serialized_string(*claim))
            .collect::<Result<BTreeSet<_>>>()?
    );
    Ok(())
}

fn cargo_check_oracle() -> Result<Value> {
    let registry = fs::read_to_string(repo_root().join("canonical").join("oracle-registry.json"))?;
    let registry: Value = serde_json::from_str(&registry)?;
    registry["oracles"]
        .as_array()
        .context("registry.oracles array")?
        .iter()
        .find(|entry| entry["id"] == "rust.cargo-check")
        .cloned()
        .context("rust.cargo-check oracle")
}

fn serialized_string<T: serde::Serialize>(value: T) -> Result<String> {
    Ok(serde_json::to_value(value)?
        .as_str()
        .context("serialized enum string")?
        .to_string())
}

fn strings(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str())
        .map(str::to_string)
        .collect()
}

fn array_contains_string(value: &Value, expected: &str) -> bool {
    value
        .as_array()
        .into_iter()
        .flatten()
        .any(|entry| entry.as_str() == Some(expected))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
