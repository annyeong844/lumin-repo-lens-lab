use anyhow::Result;
use serde_json::Value;

use crate::support::prewrite::{citations, dependency_lookup, PreWriteRepo};

#[test]
fn prewrite_dependency_lane_honors_member_globs_and_workspace_excludes() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/*"]
exclude = ["crates/ignored", "crates/excluded"]

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
serde1 = { workspace = true }
"#,
    )?;
    repo.write_bytes(
        "crates/app/src/lib.rs",
        b"pub fn app() -> serde1::Result<()> { Ok(()) }\n",
    )?;
    repo.write_bytes(
        "crates/ignored/Cargo.toml",
        br#"[package]
name = "ignored"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/ignored/src/lib.rs",
        b"pub fn ignored() -> anyhow::Result<()> { Ok(()) }\n",
    )?;
    repo.write_bytes(
        "crates/excluded/Cargo.toml",
        br#"[package]
name = "excluded"
version = "0.1.0"
edition = "2021"

[dependencies]
regex = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/excluded/src/lib.rs",
        b"pub fn excluded(_: regex::Regex) {}\n",
    )?;

    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde", "anyhow", "regex"],
  "plannedTypeEscapes": []
}"#,
    )?;

    let serde = dependency_lookup(&artifact, "serde")?;
    assert_eq!(serde["result"], "DEPENDENCY_AVAILABLE");
    assert!(citations(serde).any(|citation| {
        citation.contains("crates/app/Cargo.toml.dependencies['serde1'] declares serde")
    }));

    let ignored = dependency_lookup(&artifact, "anyhow")?;
    assert_eq!(ignored["result"], "NEW_PACKAGE");
    assert_eq!(ignored["declaredIn"], Value::Null);
    assert!(!citations(ignored)
        .any(|citation| citation.contains("crates/ignored/Cargo.toml.dependencies")));

    let excluded = dependency_lookup(&artifact, "regex")?;
    assert_eq!(excluded["result"], "NEW_PACKAGE");
    assert_eq!(excluded["declaredIn"], Value::Null);
    assert!(!citations(excluded)
        .any(|citation| citation.contains("crates/excluded/Cargo.toml.dependencies")));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_honors_cargo_member_globs_with_middle_wildcards() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/*/app"]
"#,
    )?;
    repo.write_bytes(
        "crates/foo/app/Cargo.toml",
        br#"[package]
name = "foo-app"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1"
"#,
    )?;
    repo.write_bytes(
        "crates/foo/app/src/lib.rs",
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
        citation.contains("crates/foo/app/Cargo.toml.dependencies['serde'] declares serde")
    }));
    Ok(())
}
