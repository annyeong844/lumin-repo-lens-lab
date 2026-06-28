use anyhow::{Context, Result};

use super::support::{card, cue, run_signature_fixture, shape_lookup};

#[test]
fn prewrite_function_signature_hash_excludes_unsupported_call_surfaces() -> Result<()> {
    let fixture = run_signature_fixture()?;
    let public_lookup = shape_lookup(&fixture.artifact, &fixture.public_hash)?;
    assert_eq!(public_lookup["result"], "SIGNATURE_MATCH");
    assert_eq!(public_lookup["shapeHashSource"], "functionSignature");
    assert!(public_lookup["matches"]
        .as_array()
        .context("public signature matches")?
        .iter()
        .any(|entry| entry["identity"] == "src/lib.rs::parse_user"));
    for unsupported_identity in [
        "src/lib.rs::parse_user_async",
        "src/lib.rs::parse_user_unsafe",
        "src/lib.rs::parse_user_bounded",
    ] {
        assert!(public_lookup["matches"]
            .as_array()
            .context("public signature matches")?
            .iter()
            .all(|entry| entry["identity"] != unsupported_identity));
        assert!(fixture.artifact["cueCards"]
            .as_array()
            .context("cue cards")?
            .iter()
            .all(|card| card["candidate"]["identity"] != unsupported_identity));
    }
    Ok(())
}

#[test]
fn prewrite_function_signature_cue_tiers_public_and_private_functions_like_ts_js() -> Result<()> {
    let fixture = run_signature_fixture()?;
    assert_eq!(fixture.public_hash, fixture.private_hash);

    let public_card = card(&fixture.artifact, "src/lib.rs::parse_user")?;
    let public_cue = cue(public_card, "function-signature")?;
    assert_eq!(public_card["renderTier"], "SAFE_CUE");
    assert_eq!(public_cue["cueTier"], "SAFE_CUE");
    assert_eq!(public_cue["safeMeaning"], "claim-only");
    assert_eq!(public_cue["claim"], "same normalized function signature");
    assert_eq!(
        public_cue["evidence"][0]["matchedField"],
        "files[].ast.functionSignatures[].hash"
    );
    assert_eq!(public_cue["evidence"][0]["visibility"], "exported");
    assert_eq!(
        public_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );

    let private_card = card(&fixture.artifact, "src/lib.rs::normalize_user")?;
    let private_cue = cue(private_card, "function-signature")?;
    assert_eq!(private_card["renderTier"], "AGENT_REVIEW_CUE");
    assert_eq!(private_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(private_cue["evidence"][0]["visibility"], "file-local");
    assert!(private_cue.get("safeMeaning").is_none());
    assert_eq!(
        private_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    Ok(())
}

#[test]
fn prewrite_function_signature_cue_keeps_impl_methods_review_only() -> Result<()> {
    let fixture = run_signature_fixture()?;
    assert_ne!(fixture.public_hash, fixture.impl_hash);

    let impl_lookup = shape_lookup(&fixture.artifact, &fixture.impl_hash)?;
    assert_eq!(impl_lookup["result"], "SIGNATURE_MATCH");
    let impl_card = card(&fixture.artifact, "src/lib.rs::Parser#parse")?;
    let impl_cue = cue(impl_card, "function-signature")?;
    assert_eq!(impl_card["renderTier"], "AGENT_REVIEW_CUE");
    assert_eq!(impl_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(impl_cue["evidence"][0]["visibility"], "unknown");
    assert!(impl_cue.get("safeMeaning").is_none());
    assert_eq!(
        impl_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    Ok(())
}
