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
fn prewrite_public_reexport_alias_is_claim_only_safe_like_ts_js_export_alias() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    repo.write_bytes(
        "src/lib.rs",
        br#"mod model {
    pub struct Thing;
}

pub use model::Thing as PublicThing;
"#,
    )?;
    let artifact = repo.run_json(
        r#"{
  "names": ["PublicThing"],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let lookup = lookup(&artifact, "PublicThing")?;
    assert_eq!(lookup["result"], "EXISTS");
    assert_eq!(
        lookup["identities"][0]["identity"],
        "src/lib.rs::PublicThing"
    );
    assert_eq!(lookup["identities"][0]["matchedField"], "useTreeIndex");
    assert_eq!(lookup["identities"][0]["visibility"], "public");
    assert!(lookup["citations"]
        .as_array()
        .context("re-export alias citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains(".ast.useTrees contains 'PublicThing'"))));

    let card = card(&artifact, "src/lib.rs::PublicThing")?;
    assert_eq!(card["renderTier"], "SAFE_CUE");
    let cue = &card["cues"][0];
    assert_eq!(cue["cueTier"], "SAFE_CUE");
    assert_eq!(cue["safeMeaning"], "claim-only");
    assert_eq!(cue["evidenceLane"], "exact-symbol");
    assert_eq!(cue["claim"], "exact Rust use-tree name exists");
    assert_eq!(
        cue["notSafeFor"],
        serde_json::json!(["semantic-equivalence", "auto-reuse", "auto-fix"])
    );
    assert_eq!(cue["evidence"][0]["matchedField"], "files[].ast.useTrees[]");
    assert_eq!(
        cue["evidence"][0]["candidateIdentity"],
        "src/lib.rs::PublicThing"
    );
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

#[test]
fn prewrite_service_operation_sibling_promotes_read_query_and_mutes_mismatches() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": [
    {
      "name": "search_user",
      "kind": "function",
      "why": "search user data",
      "ownerFile": "src/user_search.rs"
    }
  ],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;
    let search_lookup = lookup(&artifact, "search_user")?;
    let policy = &search_lookup["serviceOperationSiblingPolicy"];
    assert_eq!(policy["policyId"], "prewrite-service-operation-sibling-cue");
    assert_eq!(
        policy["policyVersion"],
        "prewrite-service-operation-sibling-cue-v1"
    );

    let promoted = policy["promoted"]
        .as_array()
        .context("promoted service candidates")?
        .iter()
        .find(|entry| entry["identity"] == "src/lib.rs::fetch_user")
        .context("promoted fetch_user service sibling")?;
    assert_eq!(promoted["operationFamily"], "read-query");
    assert_eq!(promoted["matchedField"], "defIndex");
    assert_eq!(promoted["sharedDomainTokens"], serde_json::json!(["user"]));
    assert_eq!(
        promoted["signatureSupport"],
        serde_json::json!({"status": "unavailable", "reason": "no-signature-facts"})
    );
    assert!(promoted["supportingReasons"]
        .as_array()
        .context("supporting reasons")?
        .contains(&Value::String("single-non-weak-token-only".to_string())));
    assert!(promoted["supportingReasons"]
        .as_array()
        .context("supporting reasons")?
        .contains(&Value::String("near-distance-exceeded".to_string())));

    let card = card(&artifact, "src/lib.rs::fetch_user")?;
    assert_eq!(card["renderTier"], "AGENT_REVIEW_CUE");
    let service_cue = card["cues"]
        .as_array()
        .context("service cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == "service-operation-sibling")
        .context("service operation sibling cue")?;
    assert_eq!(service_cue["cueTier"], "AGENT_REVIEW_CUE");
    assert_eq!(service_cue["claim"], "related service operation sibling");
    assert_eq!(
        service_cue["evidence"][0]["matchedField"],
        "lookups[].serviceOperationSiblingPolicy.promoted"
    );
    assert_eq!(
        service_cue["evidence"][0]["policyVersion"],
        "prewrite-service-operation-sibling-cue-v1"
    );
    assert!(card["cues"]
        .as_array()
        .context("card cues")?
        .iter()
        .all(|cue| cue["cueTier"] != "SAFE_CUE"));

    let impl_muted = service_muted(
        &artifact,
        "src/lib.rs::EventDispatcher#fetch_user",
        "service-sibling-surface-kind-unsupported",
    )?;
    assert_eq!(impl_muted["matchedField"], "implMethodIndex");

    let mismatch_artifact = repo.run_json(
        r#"{
  "names": [
    {
      "name": "create_user",
      "kind": "function",
      "why": "create user data",
      "ownerFile": "src/user_create.rs"
    }
  ],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;
    let create_lookup = lookup(&mismatch_artifact, "create_user")?;
    assert!(create_lookup["serviceOperationSiblingPolicy"]["promoted"]
        .as_array()
        .context("promoted service candidates")?
        .iter()
        .all(|entry| entry["identity"] != "src/lib.rs::fetch_user"));
    service_muted(
        &mismatch_artifact,
        "src/lib.rs::fetch_user",
        "service-sibling-operation-family-mismatch",
    )?;
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

fn lookup<'a>(artifact: &'a Value, name: &str) -> Result<&'a Value> {
    artifact["lookups"]
        .as_array()
        .context("lookups")?
        .iter()
        .find(|lookup| lookup["intentName"] == name)
        .with_context(|| format!("lookup {name}"))
}

fn service_muted<'a>(artifact: &'a Value, identity: &str, reason: &str) -> Result<&'a Value> {
    artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?
        .iter()
        .find(|cue| {
            cue["candidate"]["identity"] == identity
                && cue["evidenceLane"] == "service-operation-sibling"
                && cue["reason"] == reason
        })
        .with_context(|| format!("service muted cue {identity} {reason}"))
}
