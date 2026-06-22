use anyhow::{Context, Result};
use serde_json::Value;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_local_operation_sibling_is_review_only_and_does_not_enter_def_index() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": [
    {
      "name": "get_world",
      "kind": "function",
      "why": "reuse the local repository read operation",
      "ownerFile": "src/lib.rs"
    }
  ],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let lookup = lookup(&artifact, "get_world")?;
    assert_eq!(lookup["result"], "NOT_OBSERVED");
    assert!(lookup["identities"]
        .as_array()
        .context("identities")?
        .is_empty());

    let policy = &lookup["localOperationSiblingPolicy"];
    assert_eq!(policy["policyId"], "prewrite-local-operation-sibling");
    assert_eq!(
        policy["policyVersion"],
        "prewrite-local-operation-sibling-v1"
    );
    assert_eq!(policy["status"], "complete");
    assert_eq!(policy["promotedCandidateCount"], 1);
    assert_eq!(policy["mutedCandidateCount"], 1);

    let promoted = policy["promoted"]
        .as_array()
        .context("promoted local operation candidates")?
        .iter()
        .find(|entry| entry["identity"] == "src/lib.rs::create_repository#get_world")
        .context("promoted local get_world")?;
    assert_eq!(promoted["matchedField"], "preWriteLocalOperationIndex");
    assert_eq!(promoted["surfaceKind"], "nested-local-operation");
    assert_eq!(promoted["containerName"], "create_repository");
    assert_eq!(promoted["containerKind"], "function-declaration");
    assert_eq!(promoted["operationFamily"], "read-query");
    assert_eq!(promoted["domainTokens"], serde_json::json!(["world"]));
    assert_eq!(promoted["sharedDomainTokens"], serde_json::json!(["world"]));
    assert_eq!(promoted["eligibleForDeadExportRanking"], false);
    assert_eq!(promoted["eligibleForSafeFix"], false);
    assert_eq!(
        promoted["signatureSupport"],
        serde_json::json!({"status": "unavailable", "reason": "no-signature-facts"})
    );
    assert_eq!(
        promoted["supportingReasons"],
        serde_json::json!(["local-operation-same-file-domain-overlap"])
    );

    let card = card(&artifact, "src/lib.rs::create_repository#get_world")?;
    assert_eq!(card["renderTier"], "AGENT_REVIEW_CUE");
    assert!(card["cues"]
        .as_array()
        .context("local operation cues")?
        .iter()
        .all(|cue| cue["cueTier"] != "SAFE_CUE"));
    let local_cue = card["cues"]
        .as_array()
        .context("local operation cues")?
        .iter()
        .find(|cue| cue["evidenceLane"] == "local-operation-sibling")
        .context("local operation cue")?;
    assert_eq!(local_cue["claim"], "related local service operation");
    assert_eq!(
        local_cue["evidence"][0]["matchedField"],
        "lookups[].localOperationSiblingPolicy.promoted"
    );
    assert_eq!(
        local_cue["evidence"][0]["matchedFieldSource"],
        "preWriteLocalOperationIndex"
    );
    assert_eq!(
        local_cue["evidence"][0]["surfaceKind"],
        "nested-local-operation"
    );
    assert_eq!(
        local_cue["evidence"][0]["containerName"],
        "create_repository"
    );

    let muted = local_muted(
        &artifact,
        "src/lib.rs::create_repository#list_library_docs",
        "local-operation-domain-mismatch",
    )?;
    assert_eq!(muted["matchedField"], "preWriteLocalOperationIndex");
    assert_eq!(muted["surfaceKind"], "nested-local-operation");
    assert!(artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?
        .iter()
        .all(|cue| cue["candidate"]["identity"] != "src/lib.rs::create_repository#delete_world"));

    assert!(artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .all(|card| {
            card["candidate"]["identity"] != "src/lib.rs::get_world"
                && card["renderTier"] != "SAFE_CUE"
        }));
    Ok(())
}

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

fn lookup<'a>(artifact: &'a Value, name: &str) -> Result<&'a Value> {
    artifact["lookups"]
        .as_array()
        .context("lookups")?
        .iter()
        .find(|lookup| lookup["intentName"] == name)
        .with_context(|| format!("lookup {name}"))
}

fn card<'a>(artifact: &'a Value, identity: &str) -> Result<&'a Value> {
    artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .find(|card| card["candidate"]["identity"] == identity)
        .with_context(|| format!("cue card {identity}"))
}

fn local_muted<'a>(artifact: &'a Value, identity: &str, reason: &str) -> Result<&'a Value> {
    artifact["suppressedCues"]
        .as_array()
        .context("suppressed cues")?
        .iter()
        .find(|cue| {
            cue["candidate"]["identity"] == identity
                && cue["evidenceLane"] == "local-operation-sibling"
                && cue["reason"] == reason
        })
        .with_context(|| format!("local operation muted cue {identity} {reason}"))
}
