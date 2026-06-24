use anyhow::Result;

use crate::support::prewrite::{citations, dependency_lookup, PreWriteRepo};

#[test]
fn prewrite_dependency_lane_honors_cargo_character_class_member_globs() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/[ab]"]
"#,
    )?;
    repo.write_bytes(
        "crates/a/Cargo.toml",
        br#"[package]
name = "a"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/a/src/lib.rs",
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
        citation.contains("crates/a/Cargo.toml.dependencies['serde'] declares serde")
    }));
    Ok(())
}
