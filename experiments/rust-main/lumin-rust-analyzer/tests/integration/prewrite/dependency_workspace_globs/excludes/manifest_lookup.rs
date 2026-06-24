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
fn prewrite_dependency_lane_does_not_descend_into_excluded_recursive_member_roots() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/**"]
exclude = ["crates/app/src", "crates/generated"]
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

    repo.write_bytes(
        "crates/generated/blocked/src/lib.rs",
        b"pub fn generated() {}\n",
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
    Ok(())
}

#[cfg(unix)]
#[test]
fn prewrite_dependency_lane_skips_excluded_terminal_recursive_root_before_reading_children(
) -> Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/app", "target/generated/**"]
exclude = ["target/generated"]
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
    repo.write_bytes("target/generated/locked/note.txt", b"excluded\n")?;

    let generated = repo.root_path().join("target/generated");
    fs::set_permissions(&generated, fs::Permissions::from_mode(0o000))?;
    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;
    fs::set_permissions(&generated, fs::Permissions::from_mode(0o700))?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact = repo.read_json_output()?;
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
