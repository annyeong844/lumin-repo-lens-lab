use serde_json::{json, Value};

pub fn assert_reserved_claim_kinds(cargo_check: &Value) {
    assert_eq!(
        cargo_check["reservedClaimKinds"],
        json!([
            "verified.rust.type-diagnostic",
            "verified.rust.borrow-diagnostic",
            "verified.rust.name-resolution-diagnostic",
            "verified.rust.cfg-expanded-diagnostic"
        ])
    );
}
