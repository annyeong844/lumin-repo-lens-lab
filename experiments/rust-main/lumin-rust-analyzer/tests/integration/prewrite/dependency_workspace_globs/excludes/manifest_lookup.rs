use anyhow::Result;

use crate::support::prewrite::{citations, dependency_lookup, PreWriteRepo};

#[test]
fn prewrite_dependency_lane_skips_excluded_glob_matches_without_manifests() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/*"]
exclude = ["crates/bad"]
"#,
    )?;
    repo.write_bytes("crates/bad/src/lib.rs", b"pub fn bad() {}\n")?;
    repo.write_bytes(
        "crates/foo/Cargo.toml",
        br#"[package]
name = "foo"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/foo/src/lib.rs",
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
        citation.contains("crates/foo/Cargo.toml.dependencies['serde'] declares serde")
    }));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_normalizes_dotted_workspace_excludes_before_manifest_lookup(
) -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/*"]
exclude = ["./crates/bad"]
"#,
    )?;
    repo.write_bytes("crates/bad/src/lib.rs", b"pub fn bad() {}\n")?;
    repo.write_bytes(
        "crates/good/Cargo.toml",
        br#"[package]
name = "good"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/good/src/lib.rs",
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
        citation.contains("crates/good/Cargo.toml.dependencies['serde'] declares serde")
    }));
    Ok(())
}
