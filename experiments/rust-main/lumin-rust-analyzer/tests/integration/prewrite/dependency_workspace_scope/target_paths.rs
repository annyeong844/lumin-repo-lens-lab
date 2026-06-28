use anyhow::Result;

use crate::support::prewrite::{citations, dependency_lookup, examples, PreWriteRepo};

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
