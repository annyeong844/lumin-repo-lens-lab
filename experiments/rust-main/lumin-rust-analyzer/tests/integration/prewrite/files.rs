use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_file_lane_reports_existing_new_and_unavailable_rust_files() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes("src/broken.rs", &[0xff])?;
    let artifact = repo.run_json(
        r#"{
  "names": [],
  "shapes": [],
  "files": [
    "src/lib.rs",
    "src/new_module.rs",
    "src/broken.rs",
    "target/generated.rs",
    "README.md"
  ],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    assert_eq!(artifact["coverage"]["files"], "ran");
    let existing = file_lookup(&artifact, "src/lib.rs")?;
    assert_eq!(existing["kind"], "file");
    assert_eq!(existing["result"], "FILE_EXISTS");
    assert_eq!(existing["boundary"]["status"], "NOT_EVALUATED");
    assert!(citations(existing)
        .any(|citation| { citation.contains("rust-source-health.files['src/lib.rs'] present") }));

    let new_file = file_lookup(&artifact, "src/new_module.rs")?;
    assert_eq!(new_file["result"], "NEW_FILE");
    assert!(citations(new_file).any(|citation| {
        citation.contains("rust-source-health.files does not contain 'src/new_module.rs'")
    }));

    let skipped = file_lookup(&artifact, "src/broken.rs")?;
    assert_eq!(skipped["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(skipped).any(|citation| citation.contains("invalid-utf8")));

    let excluded = file_lookup(&artifact, "target/generated.rs")?;
    assert_eq!(excluded["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(excluded).any(|citation| citation.contains("target/vendor excluded")));

    let non_rust = file_lookup(&artifact, "README.md")?;
    assert_eq!(non_rust["result"], "FILE_STATUS_UNKNOWN");
    assert!(citations(non_rust).any(|citation| citation.contains("Rust .rs files only")));
    Ok(())
}

fn file_lookup<'a>(artifact: &'a Value, intent_file: &str) -> Result<&'a Value> {
    artifact["fileLookups"]
        .as_array()
        .context("fileLookups array")?
        .iter()
        .find(|lookup| lookup["intentFile"] == intent_file)
        .with_context(|| format!("lookup for {intent_file}"))
}

fn citations(lookup: &Value) -> impl Iterator<Item = &str> {
    lookup["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
}
