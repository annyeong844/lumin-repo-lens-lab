use std::collections::BTreeSet;

use anyhow::{Context, Result};
use lumin_rust_cargo_oracle::protocol::ClaimKind;

use crate::support::registry::{cargo_check_oracle, serialized_string, strings};

pub fn assert_absence_clean_claim_sets_match_registry() -> Result<()> {
    let cargo_check = cargo_check_oracle()?;
    let coverage_kinds = cargo_check["coverageKinds"]
        .as_array()
        .context("coverageKinds array")?;
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
