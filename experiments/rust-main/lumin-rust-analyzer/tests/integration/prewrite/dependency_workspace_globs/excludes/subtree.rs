use anyhow::Result;
use serde_json::Value;

use crate::support::prewrite::{citations, dependency_lookup, PreWriteRepo};

#[test]
fn prewrite_dependency_lane_preserves_cargo_subtree_excludes() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/excluded/*"]
exclude = ["crates/excluded"]
"#,
    )?;
    repo.write_bytes(
        "crates/excluded/sub/Cargo.toml",
        br#"[package]
name = "excluded-sub"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/excluded/sub/src/lib.rs",
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
    assert_eq!(serde["result"], "NEW_PACKAGE");
    assert_eq!(serde["declaredIn"], Value::Null);
    assert!(!citations(serde)
        .any(|citation| citation.contains("crates/excluded/sub/Cargo.toml.dependencies")));
    Ok(())
}
