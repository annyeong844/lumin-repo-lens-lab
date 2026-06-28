use anyhow::Result;

use crate::support::prewrite::{citations, dependency_lookup, examples, PreWriteRepo};

#[test]
fn prewrite_dependency_lane_ignores_unowned_files_in_virtual_workspace() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/app"]
"#,
    )?;
    repo.write_bytes(
        "crates/app/Cargo.toml",
        br#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/app/src/lib.rs",
        b"pub fn declared(_: serde::de::IgnoredAny) {}\n",
    )?;
    repo.write_bytes(
        "scratch.rs",
        b"pub fn scratch(_: serde::de::IgnoredAny) {}\n",
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
    assert_eq!(serde["result"], "DEPENDENCY_AVAILABLE");
    assert!(examples(serde).any(|example| example["file"] == "crates/app/src/lib.rs"));
    assert!(!examples(serde).any(|example| example["file"] == "scratch.rs"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_marks_counts_unavailable_for_declared_unowned_only_consumers(
) -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/app"]
"#,
    )?;
    repo.write_bytes(
        "crates/app/Cargo.toml",
        br#"[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes("crates/app/src/lib.rs", b"pub fn app() {}\n")?;
    repo.write_bytes(
        "scratch.rs",
        b"pub fn scratch(_: serde::de::IgnoredAny) {}\n",
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
    assert_eq!(serde["result"], "DEPENDENCY_SCOPE_UNAVAILABLE");
    assert_eq!(serde["declaredIn"], "dependencies");
    assert_eq!(
        serde["existingImports"]["observedImportCount"],
        serde_json::Value::Null
    );
    assert_eq!(serde["existingImports"]["countConfidence"], "unavailable");
    assert!(citations(serde).any(|citation| {
        citation.contains("crates/app/Cargo.toml.dependencies['serde'] declares serde")
    }));
    assert!(citations(serde).any(|citation| {
        citation.contains("omitted 1 Rust path consumer(s) outside Cargo package scopes")
    }));
    assert!(!examples(serde).any(|example| example["file"] == "scratch.rs"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_refuses_package_advice_from_unowned_files() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/app"]
"#,
    )?;
    repo.write_bytes(
        "crates/app/Cargo.toml",
        br#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
    )?;
    repo.write_bytes("crates/app/src/lib.rs", b"pub fn app() {}\n")?;
    repo.write_bytes(
        "scratch.rs",
        b"pub fn scratch(_: serde::de::IgnoredAny) {}\n",
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
    assert_eq!(serde["result"], "DEPENDENCY_SCOPE_UNAVAILABLE");
    assert_eq!(serde["declaredIn"], serde_json::Value::Null);
    assert_eq!(
        serde["existingImports"]["observedImportCount"],
        serde_json::Value::Null
    );
    assert_eq!(serde["existingImports"]["countConfidence"], "unavailable");
    assert!(citations(serde).any(|citation| {
        citation.contains("outside Cargo package manifest scopes")
            && citation.contains("no package manifest can be selected safely")
    }));
    assert!(!examples(serde).any(|example| example["file"] == "scratch.rs"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_keeps_new_package_advice_when_owned_consumers_exist() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/app"]
"#,
    )?;
    repo.write_bytes(
        "crates/app/Cargo.toml",
        br#"[package]
name = "app"
version = "0.1.0"
edition = "2021"
"#,
    )?;
    repo.write_bytes(
        "crates/app/src/lib.rs",
        b"pub fn app(_: serde::de::IgnoredAny) {}\n",
    )?;
    repo.write_bytes(
        "scratch.rs",
        b"pub fn scratch(_: serde::de::IgnoredAny) {}\n",
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
    assert!(citations(serde)
        .any(|citation| { citation.contains("Cargo manifest scope does not declare 'serde'") }));
    assert!(!citations(serde).any(|citation| citation.contains("only in files outside")));
    assert!(examples(serde).any(|example| example["file"] == "crates/app/src/lib.rs"));
    assert!(!examples(serde).any(|example| example["file"] == "scratch.rs"));
    Ok(())
}
