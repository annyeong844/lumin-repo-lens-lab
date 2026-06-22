use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_dependency_lane_reports_declared_consumed_zero_and_new_packages() -> Result<()> {
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

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["anyhow", "tracing-subscriber", "serde_json", "pretty_assertions", "cc", "serde_yaml"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert_eq!(artifact["coverage"]["dependencies"], "ran");
    let anyhow = dependency_lookup(&artifact, "anyhow")?;
    assert_eq!(anyhow["result"], "DEPENDENCY_AVAILABLE");
    assert_eq!(anyhow["declaredIn"], "dependencies");
    assert_eq!(anyhow["existingImports"]["countConfidence"], "grounded");
    assert!(
        anyhow["existingImports"]["observedImportCount"]
            .as_u64()
            .context("anyhow observed count")?
            > 0
    );
    assert!(citations(anyhow).any(|citation| {
        citation.contains("Cargo.toml.dependencies['anyhow'] declares anyhow")
    }));

    let tracing = dependency_lookup(&artifact, "tracing-subscriber")?;
    assert_eq!(tracing["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(tracing).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec.contains("tracing_subscriber"))
    }));

    let serde_json = dependency_lookup(&artifact, "serde_json")?;
    assert_eq!(serde_json["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(serde_json).any(|example| {
        example["fromSpec"]
            .as_str()
            .is_some_and(|from_spec| from_spec.contains("serde_json::json"))
    }));

    let pretty = dependency_lookup(&artifact, "pretty_assertions")?;
    assert_eq!(pretty["result"], "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS");
    assert_eq!(pretty["declaredIn"], "dev-dependencies");
    assert_eq!(pretty["existingImports"]["observedImportCount"], 0);
    assert!(citations(pretty)
        .all(|citation| { !citation.contains("unused") && !citation.contains("cleanup") }));

    let build_dependency = dependency_lookup(&artifact, "cc")?;
    assert_eq!(
        build_dependency["result"],
        "DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS"
    );
    assert_eq!(build_dependency["declaredIn"], "build-dependencies");

    let new_package = dependency_lookup(&artifact, "serde_yaml")?;
    assert_eq!(new_package["result"], "NEW_PACKAGE");
    assert_eq!(new_package["declaredIn"], Value::Null);
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

#[test]
fn prewrite_dependency_lane_hard_stops_on_malformed_manifest() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("Cargo.toml", b"[dependencies\nanyhow = \"1\"\n")?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["anyhow"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("Cargo.toml"));
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
