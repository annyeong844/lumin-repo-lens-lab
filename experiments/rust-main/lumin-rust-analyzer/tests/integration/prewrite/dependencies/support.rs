use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

pub(super) fn run_dependency_fixture() -> Result<Value> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[package]
name = "prewrite-case"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
tracing-subscriber = "0.3"
serde_json = "1"

[dev-dependencies]
pretty_assertions = "1"

[build-dependencies]
cc = "1"
"#,
    )?;
    repo.write_bytes(
        "src/lib.rs",
        br#"use anyhow::{Context, Result};
use tracing_subscriber::fmt;

pub fn load(raw: &str) -> Result<()> {
    let _ = fmt;
    let _ = raw.parse::<usize>().context("parse usize")?;
    let _ = serde_json::json!({"ok": true});
    Ok(())
}
"#,
    )?;
    for index in 0..10 {
        repo.write_bytes(
            format!("src/anyhow_consumer_{index}.rs"),
            format!("use anyhow::Result;\n\npub fn dep_hub_{index}() -> Result<()> {{ Ok(()) }}\n")
                .as_bytes(),
        )?;
    }

    repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["anyhow", "tracing-subscriber", "serde_json", "pretty_assertions", "cc", "serde_yaml"],
  "plannedTypeEscapes": []
}"#,
    )
}

pub(super) fn dependency_lookup<'a>(artifact: &'a Value, dependency: &str) -> Result<&'a Value> {
    artifact["dependencyLookups"]
        .as_array()
        .context("dependencyLookups array")?
        .iter()
        .find(|lookup| lookup["depName"] == dependency)
        .with_context(|| format!("dependency lookup {dependency}"))
}

pub(super) fn citations(lookup: &Value) -> impl Iterator<Item = &str> {
    lookup["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}

pub(super) fn examples(lookup: &Value) -> impl Iterator<Item = &Value> {
    lookup["existingImports"]["examples"]
        .as_array()
        .into_iter()
        .flatten()
}

pub(super) fn cue_card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cueCards array")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}
