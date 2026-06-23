use anyhow::{Context, Result};
use lumin_rust_source_health::protocol::AstCallableKind;
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_function_signature_hash_uses_ts_js_safe_and_review_tiers() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"pub fn parse_user(input: &str, limit: usize) -> usize {
    input.len() + limit
}

fn normalize_user(input: &str, limit: usize) -> usize {
    input.len() + limit
}

pub struct Parser;

impl Parser {
    pub fn parse(&self, input: &str, limit: usize) -> usize {
        input.len() + limit
    }
}
"#,
    )?;
    let health = repo.source_health()?;
    let signatures = &health
        .files
        .get("src/lib.rs")
        .context("src/lib.rs health")?
        .ast
        .function_signatures;
    let public_hash = signatures
        .iter()
        .find(|signature| {
            signature.name == "parse_user"
                && signature.callable_kind == AstCallableKind::Function
                && signature.owner.is_none()
        })
        .context("public function signature")?
        .hash
        .clone();
    let private_hash = signatures
        .iter()
        .find(|signature| signature.name == "normalize_user")
        .context("private function signature")?
        .hash
        .clone();
    let impl_hash = signatures
        .iter()
        .find(|signature| {
            signature.name == "parse"
                && signature.callable_kind == AstCallableKind::ImplMethod
                && signature
                    .owner
                    .as_ref()
                    .is_some_and(|owner| owner.target == "Parser")
        })
        .context("impl method signature")?
        .hash
        .clone();
    assert_eq!(public_hash, private_hash);
    assert_ne!(public_hash, impl_hash);

    let intent = serde_json::json!({
        "names": [],
        "shapes": [
            {"fields": [], "hash": public_hash},
            {"fields": [], "hash": impl_hash}
        ],
        "files": [],
        "dependencies": [],
        "plannedTypeEscapes": []
    })
    .to_string();
    let artifact = repo.run_json(&intent)?;

    let public_lookup = shape_lookup(&artifact, &public_hash)?;
    assert_eq!(public_lookup["result"], "SIGNATURE_MATCH");
    assert_eq!(public_lookup["shapeHashSource"], "functionSignature");
    assert!(public_lookup["matches"]
        .as_array()
        .context("public signature matches")?
        .iter()
        .any(|entry| entry["identity"] == "src/lib.rs::parse_user"));

    let public_card = card(&artifact, "src/lib.rs::parse_user")?;
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

    let private_card = card(&artifact, "src/lib.rs::normalize_user")?;
    let private_cue = cue(private_card, "function-signature")?;
    assert_eq!(private_card["renderTier"], "AGENT_REVIEW_CUE");
    assert_eq!(private_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(private_cue["evidence"][0]["visibility"], "file-local");
    assert!(private_cue.get("safeMeaning").is_none());
    assert_eq!(
        private_cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );

    let impl_lookup = shape_lookup(&artifact, &impl_hash)?;
    assert_eq!(impl_lookup["result"], "SIGNATURE_MATCH");
    let impl_card = card(&artifact, "src/lib.rs::Parser#parse")?;
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

fn card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

fn shape_lookup<'a>(artifact: &'a Value, hash: &str) -> Result<&'a Value> {
    artifact["shapeLookups"]
        .as_array()
        .context("shape lookups")?
        .iter()
        .find(|lookup| lookup["shapeHash"] == hash)
        .with_context(|| format!("shape lookup {hash}"))
}

fn cue<'a>(card: &'a Value, evidence_lane: &str) -> Result<&'a Value> {
    card["cues"]
        .as_array()
        .context("card cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == evidence_lane)
        .with_context(|| format!("cue {evidence_lane}"))
}
