#![allow(dead_code)]

use anyhow::{Context, Result};
use serde_json::Value;

pub fn assert_tier_and_claim(finding: &Value, tier: &str, claim_kind: &str) {
    assert_eq!(finding["confidence"]["tier"], tier);
    assert_eq!(finding["confidence"]["claimKind"], claim_kind);
}

pub fn assert_first_authority(finding: &Value, authority_id: &str) -> Result<()> {
    assert_eq!(
        finding["confidence"]["authorityIds"][0]
            .as_str()
            .context("authorityIds[0]")?,
        authority_id
    );
    Ok(())
}
