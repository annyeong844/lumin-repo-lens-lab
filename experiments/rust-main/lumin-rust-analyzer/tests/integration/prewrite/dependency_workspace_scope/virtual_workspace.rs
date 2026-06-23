use anyhow::Result;

use crate::support::prewrite::{dependency_lookup, examples, PreWriteRepo};

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
