use anyhow::Result;

use crate::support::prewrite::{citations, dependency_lookup, PreWriteRepo};

#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

#[cfg(unix)]
struct PermissionRestore {
    path: PathBuf,
}

#[cfg(unix)]
impl Drop for PermissionRestore {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.path, fs::Permissions::from_mode(0o700));
    }
}

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
#[cfg(unix)]
fn prewrite_dependency_lane_does_not_descend_into_excluded_recursive_member_roots() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/**"]
exclude = ["crates/generated"]
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
        b"pub fn decode(_: serde::de::IgnoredAny) {}\n",
    )?;

    let blocked = repo.root_path().join("crates/generated/blocked");
    fs::create_dir_all(blocked.join("nested"))?;
    fs::set_permissions(&blocked, fs::Permissions::from_mode(0o000))?;
    let _restore = PermissionRestore { path: blocked };

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
