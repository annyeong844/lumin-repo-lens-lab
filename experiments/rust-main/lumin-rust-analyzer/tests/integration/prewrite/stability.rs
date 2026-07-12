use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;
use crate::support::scenarios::single_package::analyze_metadata_only_single_package;

#[test]
fn prewrite_output_is_deterministic_and_does_not_change_legacy_artifact_shape() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let intent = r#"{
  "names": ["load_task"],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#;
    let first = repo.run_json(intent)?;
    let second = repo.run_json(intent)?;

    assert_eq!(first, second);
    assert!(first.get("generated").is_none());
    assert!(first["meta"].get("generated").is_none());
    assert!(first.get("definitionIndex").is_none());
    assert!(first.get("implMethodIndex").is_none());

    let legacy = analyze_metadata_only_single_package("pub fn demo() {}\n")?;
    assert!(legacy.get("preWrite").is_none());
    assert!(legacy.get("cueCards").is_none());
    assert!(legacy.get("lookups").is_none());
    Ok(())
}

#[test]
fn prewrite_accepts_intent_from_stdin_like_js_ts_prewrite() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let output = repo.run_stdin(
        r#"{
  "names": ["load_task"],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let artifact = repo.read_json_output()?;
    assert_eq!(artifact["meta"]["producer"], "lumin-rust-analyzer");
    let lookup = lookup(&artifact, "load_task")?;
    assert_eq!(lookup["result"], "EXISTS");
    Ok(())
}

fn lookup<'a>(artifact: &'a Value, name: &str) -> Result<&'a Value> {
    artifact["lookups"]
        .as_array()
        .context("lookups")?
        .iter()
        .find(|lookup| lookup["intentName"] == name)
        .with_context(|| format!("lookup {name}"))
}
