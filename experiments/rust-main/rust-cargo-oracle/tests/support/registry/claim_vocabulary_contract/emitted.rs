use serde_json::{json, Value};

use crate::support::registry::array_contains_string;

pub fn assert_emitted_claim_kinds(cargo_check: &Value) {
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
}
