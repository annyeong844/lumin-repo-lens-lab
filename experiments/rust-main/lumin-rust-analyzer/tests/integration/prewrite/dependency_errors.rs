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
members = ["crates/*", "!crates/ignored"]
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
