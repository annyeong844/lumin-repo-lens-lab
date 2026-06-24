use anyhow::{Context, Result};
use lumin_rust_source_health::protocol::AstCallableKind;
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

pub(super) struct SignatureFixture {
    pub(super) artifact: Value,
    pub(super) public_hash: String,
    pub(super) private_hash: String,
    pub(super) impl_hash: String,
}

pub(super) fn run_signature_fixture() -> Result<SignatureFixture> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"pub fn parse_user(input: &str, limit: usize) -> usize {
    input.len() + limit
}

pub async fn parse_user_async(input: &str, limit: usize) -> usize {
    input.len() + limit
}

pub unsafe fn parse_user_unsafe(input: &str, limit: usize) -> usize {
    input.len() + limit
}

pub fn parse_user_bounded<T>(input: T, limit: usize) -> usize
where
    T: AsRef<str>,
{
    input.as_ref().len() + limit
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

    Ok(SignatureFixture {
        artifact,
        public_hash,
        private_hash,
        impl_hash,
    })
}

pub(super) fn card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

pub(super) fn shape_lookup<'a>(artifact: &'a Value, hash: &str) -> Result<&'a Value> {
    artifact["shapeLookups"]
        .as_array()
        .context("shape lookups")?
        .iter()
        .find(|lookup| lookup["shapeHash"] == hash)
        .with_context(|| format!("shape lookup {hash}"))
}

pub(super) fn cue<'a>(card: &'a Value, evidence_lane: &str) -> Result<&'a Value> {
    card["cues"]
        .as_array()
        .context("card cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == evidence_lane)
        .with_context(|| format!("cue {evidence_lane}"))
}
