use anyhow::Result;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_dependency_lane_hard_stops_on_malformed_manifest() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("Cargo.toml", b"[dependencies\nanyhow = \"1\"\n")?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["anyhow"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("Cargo.toml"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_negated_workspace_member() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["!crates/ignored"]
"#,
    )?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("workspace.members does not support negated member"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_non_string_workspace_member() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = [1]
"#,
    )?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("workspace.members[0] must be a string"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_non_string_workspace_exclude() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = []
exclude = [1]
"#,
    )?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("workspace.exclude[0] must be a string"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_malformed_workspace_dependencies() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = []
dependencies = []
"#,
    )?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("workspace.dependencies must be a table"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_missing_exact_workspace_member() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/missing"]
"#,
    )?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("workspace member 'crates/missing' does not contain Cargo.toml"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_glob_member_without_manifest() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/*"]
"#,
    )?;
    repo.write_bytes("crates/app/src/lib.rs", b"pub fn app() {}\n")?;

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("does not contain Cargo.toml"));
    Ok(())
}

#[test]
fn prewrite_dependency_lane_hard_stops_on_recursive_glob_member_without_manifest() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/**"]
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

    let output = repo.run(
        r#"{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": ["serde"],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(!output.status.success());
    assert!(!repo.output_exists());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("blocked-prewrite-dependency-manifest"));
    assert!(stderr.contains("crates"));
    assert!(stderr.contains("src"));
    assert!(stderr.contains("does not contain Cargo.toml"));
    Ok(())
}
