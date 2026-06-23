use anyhow::{Context, Result};

use crate::support::prewrite::PreWriteRepo;

const SHAPE_HASH: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn prewrite_not_observed_keeps_opaque_taint_and_file_lane_visible() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(&format!(
        r#"{{
  "taskId": "TASK-42",
  "names": ["totally_missing_name"],
  "shapes": [{{"fields": ["id"]}}, {{"hash": "{SHAPE_HASH}"}}],
  "files": ["src/new.rs"]
}}"#
    ))?;

    assert_eq!(artifact["intent"]["taskId"], "TASK-42");
    assert_eq!(artifact["coverage"]["names"], "ran");
    assert_eq!(artifact["coverage"]["shapes"], "ran");
    assert_eq!(artifact["coverage"]["files"], "ran");
    assert_eq!(artifact["coverage"]["dependencies"], "not-requested");
    assert_eq!(artifact["coverage"]["plannedTypeEscapes"], "ran");
    assert_eq!(artifact["lookups"][0]["result"], "NOT_OBSERVED");
    let shape_lookups = artifact["shapeLookups"]
        .as_array()
        .context("shape lookups")?;
    assert_eq!(shape_lookups.len(), 2);
    assert_eq!(shape_lookups[0]["kind"], "shape");
    assert_eq!(shape_lookups[0]["result"], "UNAVAILABLE");
    assert_eq!(
        shape_lookups[0]["shape"]["fields"],
        serde_json::json!(["id"])
    );
    assert!(shape_lookups[0]["citations"]
        .as_array()
        .context("fields-only shape citations")?
        .iter()
        .any(|citation| citation.as_str().is_some_and(|text| {
            text.contains("field names alone are not structural equality evidence")
        })));
    assert_eq!(shape_lookups[1]["shapeHash"], SHAPE_HASH);
    assert!(shape_lookups[1]["citations"]
        .as_array()
        .context("hash shape citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains("do not yet make complete absence claims"))));

    let unavailable = artifact["unavailableEvidence"]
        .as_array()
        .context("unavailable evidence")?;
    assert_eq!(unavailable.len(), 2);
    assert!(unavailable.iter().all(|entry| {
        entry["evidenceLane"] == "shape-hash"
            && entry["status"] == "UNAVAILABLE"
            && entry["reason"] == "lookup-unavailable"
            && entry["artifact"] == "rust-source-health"
    }));
    assert!(artifact["cueCards"]
        .as_array()
        .context("cue cards")?
        .iter()
        .all(|card| card["cues"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|cue| cue["evidenceLane"] != "shape-hash")));
    assert_eq!(artifact["fileLookups"][0]["intentFile"], "src/new.rs");
    assert_eq!(artifact["fileLookups"][0]["result"], "NEW_FILE");
    assert_eq!(
        artifact["fileLookups"][0]["boundary"]["status"],
        "NOT_EVALUATED"
    );
    assert!(
        artifact["lookups"][0]["taintedBy"]["reviewOpaqueSurfaces"]
            .as_u64()
            .context("review opaque surfaces")?
            > 0
    );
    assert!(artifact["lookups"][0]["citations"]
        .as_array()
        .context("citations")?
        .iter()
        .any(|citation| citation
            .as_str()
            .is_some_and(|text| text.contains("not an absence claim"))));
    assert_eq!(
        artifact["intentWarnings"]
            .as_array()
            .context("warnings")?
            .len(),
        2
    );
    Ok(())
}

#[test]
fn prewrite_meta_exposes_js_ts_lookup_policy_constants() -> Result<()> {
    let repo = PreWriteRepo::new()?;
    let artifact = repo.run_json(
        r#"{
  "names": ["load_task"],
  "shapes": [],
  "files": ["src/new_task.rs"],
  "dependencies": [],
  "plannedTypeEscapes": []
}"#,
    )?;

    let policy = &artifact["meta"]["lookupPolicy"];
    assert_eq!(
        policy["jsTsPrecedent"],
        serde_json::json!([
            "_lib/pre-write-intent.mjs",
            "_lib/pre-write-cue-tiers.mjs",
            "_lib/pre-write-lookup-name.mjs",
            "_lib/pre-write-lookup-file.mjs",
            "_lib/pre-write-lookup-shape.mjs",
            "_lib/pre-write-lookup-dep.mjs",
            "_lib/pre-write-lookup-inline-patterns.mjs"
        ])
    );
    assert_eq!(policy["nearName"]["maxLengthDelta"], 2);
    assert_eq!(policy["nearName"]["sharedPrefixMin"], 4);
    assert_eq!(policy["nearName"]["maxDistance"], 2);
    assert_eq!(policy["nearName"]["maxResults"], 5);
    assert_eq!(policy["semanticHint"]["minScore"], 2);
    assert_eq!(policy["semanticHint"]["maxResults"], 5);
    assert_eq!(
        policy["serviceOperationSibling"]["policyId"],
        "prewrite-service-operation-sibling-cue"
    );
    assert_eq!(
        policy["serviceOperationSibling"]["policyVersion"],
        "prewrite-service-operation-sibling-cue-v1"
    );
    assert_eq!(policy["serviceOperationSibling"]["maxResults"], 5);
    assert_eq!(
        policy["localOperationSibling"]["policyId"],
        "prewrite-local-operation-sibling"
    );
    assert_eq!(
        policy["localOperationSibling"]["policyVersion"],
        "prewrite-local-operation-sibling-v1"
    );
    assert_eq!(policy["localOperationSibling"]["maxResults"], 5);
    assert_eq!(policy["fileDomainCluster"]["minMatches"], 2);
    assert_eq!(policy["fileDomainCluster"]["maxExamples"], 8);
    assert_eq!(policy["fileDomainCluster"]["minPrefixLen"], 4);
    assert_eq!(policy["dependencyHub"]["exampleLimit"], 5);
    assert_eq!(policy["dependencyHub"]["watchForThreshold"], 10);
    assert_eq!(
        artifact["lookups"][0]["serviceOperationSiblingPolicy"]["policyId"],
        policy["serviceOperationSibling"]["policyId"]
    );
    assert_eq!(
        artifact["lookups"][0]["localOperationSiblingPolicy"]["policyVersion"],
        policy["localOperationSibling"]["policyVersion"]
    );
    Ok(())
}
