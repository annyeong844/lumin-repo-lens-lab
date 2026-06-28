use anyhow::Result;

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
