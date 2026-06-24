use anyhow::Result;
use serde_json::Value;

use crate::support::prewrite::{dependency_lookup, PreWriteRepo};

#[test]
fn prewrite_dependency_lane_ignores_file_only_workspace_glob_matches() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "Cargo.toml",
        br#"[workspace]
members = ["crates/*"]
"#,
    )?;
    repo.write_bytes("crates/README.md", b"placeholder\n")?;

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
    Ok(())
}
