use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_dependency_hub_cue_requires_grounded_consumer_count() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[package]
name = "prewrite-case"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
"#,
    )?;
    repo.write_bytes("src/lib.rs", b"pub fn ok() {}\n")?;
    for index in 0..10 {
        repo.write_bytes(
            format!("src/partial_anyhow_consumer_{index}.rs"),
            format!(
                "use anyhow::Result;\n\npub fn partial_dep_hub_{index}() -> Result<()> {{ Ok(()) }}\n"
            )
            .as_bytes(),
        )?;
    }
    repo.write_bytes("src/broken.rs", b"pub fn broken( {\n")?;

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["anyhow"],
  "plannedTypeEscapes": []
}"#,
    )?;

    let anyhow = dependency_lookup(&artifact, "anyhow")?;
    assert_eq!(anyhow["result"], "DEPENDENCY_AVAILABLE");
    assert_eq!(anyhow["existingImports"]["countConfidence"], "sample-only");
    assert!(
        anyhow["existingImports"]["observedImportCount"]
            .as_u64()
            .context("sample-only observed import count")?
            >= 10
    );
    assert!(cue_card(&artifact, "Cargo.toml::dependency::anyhow").is_err());
    Ok(())
}

#[test]
fn prewrite_dependency_lane_does_not_claim_zero_when_import_graph_is_partial() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[package]
name = "prewrite-case"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
"#,
    )?;
    repo.write_bytes("src/lib.rs", b"pub fn ok() {}\n")?;
    repo.write_bytes("src/broken.rs", b"pub fn broken( {\n")?;

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["anyhow"],
  "plannedTypeEscapes": []
}"#,
    )?;

    let anyhow = dependency_lookup(&artifact, "anyhow")?;
    assert_eq!(
        anyhow["result"],
        "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE"
    );
    assert_eq!(
        anyhow["existingImports"]["observedImportCount"],
        Value::Null
    );
    assert_eq!(anyhow["existingImports"]["countConfidence"], "unavailable");
    assert!(anyhow["existingImports"]["unavailableReason"]
        .as_str()
        .is_some_and(|reason| reason.contains("parse-error file")));
    assert!(citations(anyhow).any(|citation| citation.contains("not a grounded absence claim")));
    Ok(())
}

fn dependency_lookup<'a>(artifact: &'a Value, dependency: &str) -> Result<&'a Value> {
    artifact["dependencyLookups"]
        .as_array()
        .context("dependencyLookups array")?
        .iter()
        .find(|lookup| lookup["depName"] == dependency)
        .with_context(|| format!("dependency lookup {dependency}"))
}

fn citations(lookup: &Value) -> impl Iterator<Item = &str> {
    lookup["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

fn cue_card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cueCards array")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}
