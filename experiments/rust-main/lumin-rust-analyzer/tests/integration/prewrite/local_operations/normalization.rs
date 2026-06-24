use anyhow::{Context, Result};

use crate::prewrite::local_operations::support::{card, local_muted, lookup};
use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_local_operation_sibling_normalizes_owner_file_and_plural_domains() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"pub fn create_repository() {
    fn list_worlds() {}
    list_worlds();
}
"#,
    )?;
    let artifact = repo.run_json(
        r#"{
  "names": [
    {
      "name": "search_world",
      "kind": "function",
      "why": "reuse the local plural read operation",
      "ownerFile": "src\\lib.rs"
    }
  ],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let lookup = lookup(&artifact, "search_world")?;
    let policy = &lookup["localOperationSiblingPolicy"];
    assert_eq!(policy["status"], "complete");
    assert_eq!(policy["promotedCandidateCount"], 1);
    assert_eq!(policy["mutedCandidateCount"], 0);

    let promoted = policy["promoted"]
        .as_array()
        .context("promoted local operation candidates")?
        .iter()
        .find(|entry| entry["identity"] == "src/lib.rs::create_repository#list_worlds")
        .context("promoted local list_worlds")?;
    assert_eq!(promoted["ownerFile"], "src/lib.rs");
    assert_eq!(promoted["domainTokens"], serde_json::json!(["world"]));
    assert_eq!(promoted["sharedDomainTokens"], serde_json::json!(["world"]));
    assert_eq!(promoted["locality"]["sameFile"], true);
    assert_eq!(
        promoted["supportingReasons"],
        serde_json::json!(["local-operation-same-file-domain-overlap"])
    );

    let card = card(&artifact, "src/lib.rs::create_repository#list_worlds")?;
    assert_eq!(card["renderTier"], "AGENT_REVIEW_CUE");
    assert!(local_muted(
        &artifact,
        "src/lib.rs::create_repository#list_worlds",
        "local-operation-domain-mismatch",
    )
    .is_err());
    Ok(())
}
