use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_exact_definition_is_safe_and_near_impl_method_is_review() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": [
    "load_task",
    {
      "name": "handle_bulk_delete",
      "kind": "function",
      "why": "extract a bulk delete event handler"
    }
  ],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let safe = card(&artifact, "src/lib.rs::load_task")?;
    assert_eq!(safe["renderTier"], "SAFE_CUE");
    assert_eq!(safe["cues"][0]["safeMeaning"], "claim-only");
    assert_eq!(safe["cues"][0]["evidenceLane"], "exact-symbol");
    assert_eq!(
        safe["cues"][0]["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );

    let review = card(&artifact, "src/lib.rs::EventDispatcher#handle_delete")?;
    assert_eq!(review["renderTier"], "AGENT_REVIEW_CUE");
    assert_eq!(review["cues"][0]["evidenceLane"], "impl-method-name");
    assert_eq!(
        review["cues"][0]["evidence"][0]["matchedField"],
        "implMethodIndex"
    );
    assert!(review["cues"][0].get("safeMeaning").is_none());
    Ok(())
}

#[test]
fn prewrite_exact_impl_method_stays_review_and_test_path_is_muted() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": [{"name": "handle_delete", "ownerFile": "tests/helper.rs"}],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let review = card(&artifact, "src/lib.rs::EventDispatcher#handle_delete")?;
    assert_eq!(review["renderTier"], "AGENT_REVIEW_CUE");
    assert_eq!(review["cues"][0]["evidence"][0]["distance"], 0);
    assert!(artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .filter(|card| card["renderTier"] == "SAFE_CUE")
        .all(|card| card["candidate"]["identity"]
            .as_str()
            .is_none_or(|identity| !identity.contains("#handle_delete"))));

    let muted = artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?
        .iter()
        .find(|cue| cue["candidate"]["identity"] == "tests/helper.rs::TestDispatcher#handle_delete")
        .context("test impl method muted cue")?;
    assert_eq!(muted["cueTier"], "MUTED_CUE");
    assert_eq!(muted["originalCueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(muted["reason"], "policy-excluded");
    assert!(muted["pathClassifications"]
        .as_array()
        .context("path classifications")?
        .contains(&Value::String("test".to_string())));
    Ok(())
}

fn card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}
