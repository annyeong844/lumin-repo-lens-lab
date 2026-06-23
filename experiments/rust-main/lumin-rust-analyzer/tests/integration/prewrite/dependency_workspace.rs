use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_dependency_lane_reads_workspace_renames_targets_and_attribute_consumers() -> Result<()>
{
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/app"]

[workspace.dependencies]
serde1 = { package = "serde", version = "1" }
"#,
    )?;
    repo.write_bytes(
        "crates/app/Cargo.toml",
        br#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = "1"
serde1 = { workspace = true }
async-trait = "0.1"

[target.'cfg(windows)'.dependencies]
windows-sys = "0.52"
"#,
    )?;
    repo.write_bytes(
        "crates/app/src/lib.rs",
        br#"use tokio::{time::sleep};

#[async_trait::async_trait]
pub trait Worker {
    async fn run(&self);
}

pub fn decode() -> serde1::Result<()> {
    Ok(())
}

pub async fn wait() {
    sleep(std::time::Duration::from_millis(1)).await;
}
"#,
    )?;

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde", "tokio", "time", "async-trait", "windows-sys"],
  "plannedTypeEscapes": []
}"#,
    )?;

    let serde = dependency_lookup(&artifact, "serde")?;
    assert_eq!(serde["result"], "DEPENDENCY_AVAILABLE");
    assert_eq!(serde["declaredIn"], "dependencies");
    assert!(citations(serde).any(|citation| {
        citation.contains("crates/app/Cargo.toml.dependencies['serde1'] declares serde")
    }));
    assert!(examples(serde).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec == "serde1::Result")
    }));

    let tokio = dependency_lookup(&artifact, "tokio")?;
    assert_eq!(tokio["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(tokio).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec.contains("tokio::time::sleep"))
    }));

    let nested_child = dependency_lookup(&artifact, "time")?;
    assert_eq!(nested_child["result"], "NEW_PACKAGE");
    assert_eq!(
        nested_child["existingImports"]["observedImportCount"],
        serde_json::json!(0)
    );

    let async_trait = dependency_lookup(&artifact, "async-trait")?;
    assert_eq!(async_trait["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(async_trait).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec == "async_trait::async_trait")
    }));

    let windows = dependency_lookup(&artifact, "windows-sys")?;
    assert_eq!(
        windows["result"],
        "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS"
    );
    assert_eq!(windows["declaredIn"], "target.cfg(windows).dependencies");
    Ok(())
}

#[test]
fn prewrite_dependency_lane_keeps_member_declarations_package_scoped() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/declared", "crates/consumer"]

[workspace.dependencies]
serde1 = { package = "serde", version = "1" }
"#,
    )?;
    repo.write_bytes(
        "crates/declared/Cargo.toml",
        br#"[package]
name = "declared"
version = "0.1.0"
edition = "2021"

[dependencies]
serde1 = { workspace = true }
"#,
    )?;
    repo.write_bytes(
        "crates/declared/src/lib.rs",
        b"pub fn declared() -> serde1::Result<()> { Ok(()) }\n",
    )?;
    repo.write_bytes(
        "crates/consumer/Cargo.toml",
        br#"[package]
name = "consumer"
version = "0.1.0"
edition = "2021"
"#,
    )?;
    repo.write_bytes(
        "crates/consumer/src/lib.rs",
        b"pub fn consumer(_value: serde::de::IgnoredAny) {}\n",
    )?;

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    let serde = dependency_lookup(&artifact, "serde")?;
    assert_eq!(serde["result"], "NEW_PACKAGE");
    assert_eq!(serde["declaredIn"], Value::Null);
    assert!(citations(serde).any(|citation| {
        citation.contains("crates/consumer/Cargo.toml")
            && citation.contains("without a matching declaration")
    }));
    assert!(examples(serde).any(|example| {
        example["file"] == "crates/consumer/src/lib.rs"
            && example["fromSpec"]
                .as_str()
                .is_some_and(|from_spec| from_spec == "serde::de::IgnoredAny")
    }));
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

fn examples(lookup: &Value) -> impl Iterator<Item = &Value> {
    lookup["existingImports"]["examples"]
        .as_array()
        .into_iter()
        .flatten()
}
