use anyhow::{Context, Result};
use serde_json::json;

use crate::support::registry::cargo_check_oracle;

pub fn assert_clean_scope_is_verified_rustc_error_absence() -> Result<()> {
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
