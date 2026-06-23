use anyhow::Result;
use serde_json::Value;

use crate::support::prewrite::{citations, dependency_lookup, examples, PreWriteRepo};

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

#[test]
fn prewrite_dependency_lane_uses_manifest_target_paths_for_scope() -> Result<()> {
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

[lib]
path = "../../src/app.rs"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "src/app.rs",
        b"pub fn decode(_: serde::de::IgnoredAny) {}\n",
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
    assert!(citations(serde).any(|citation| {
        citation.contains("crates/app/Cargo.toml.dependencies['serde'] declares serde")
    }));
    assert!(examples(serde).any(|example| {
        example["file"] == "src/app.rs"
            && example["fromSpec"]
                .as_str()
                .is_some_and(|from_spec| from_spec == "serde::de::IgnoredAny")
    }));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_scopes_renamed_crate_roots_to_declaring_manifest() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/a", "crates/b"]

[workspace.dependencies]
serde1 = { package = "serde", version = "1" }
"#,
    )?;
    repo.write_bytes(
        "crates/a/Cargo.toml",
        br#"[package]
name = "a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde1 = { workspace = true }
"#,
    )?;
    repo.write_bytes(
        "crates/a/src/lib.rs",
        b"pub fn declared() -> serde1::Result<()> { Ok(()) }\n",
    )?;
    repo.write_bytes(
        "crates/b/Cargo.toml",
        br#"[package]
name = "b"
version = "0.1.0"
edition = "2021"
"#,
    )?;
    repo.write_bytes(
        "crates/b/src/lib.rs",
        b"pub fn local_alias(_: serde1::local::Thing) {}\n",
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
        citation.contains("crates/b/Cargo.toml")
            && citation.contains("without a matching declaration")
    }));
    assert!(examples(serde).any(|example| {
        example["file"] == "crates/b/src/lib.rs"
            && example["fromSpec"]
                .as_str()
                .is_some_and(|from_spec| from_spec == "serde1::local::Thing")
    }));
    Ok(())
}

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
