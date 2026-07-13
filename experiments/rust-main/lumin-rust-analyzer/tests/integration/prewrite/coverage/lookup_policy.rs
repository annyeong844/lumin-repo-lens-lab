use anyhow::Result;

use crate::support::prewrite::PreWriteRepo;

#[test]
fn prewrite_meta_exposes_current_write_gate_policy_owners() -> Result<()> {
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
        policy["writeGatePolicyOwners"],
        serde_json::json!([
            "experiments/rust-main/lumin-audit-core/src/pre_write_intent.rs",
            "experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/cues.rs",
            "experiments/rust-main/lumin-audit-core/src/pre_write_lifecycle/js_native/lookup.rs",
            "experiments/rust-main/lumin-audit-core/src/js_ts_extract/function_signature.rs",
            "experiments/rust-main/lumin-audit-core/src/js_ts_extract/inline_patterns.rs"
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
    assert_eq!(policy["inlinePattern"]["policyId"], "inline-pattern-policy");
    assert_eq!(
        policy["inlinePattern"]["policyVersion"],
        "inline-pattern-policy-v1"
    );
    assert_eq!(policy["inlinePattern"]["minOccurrences"], 3);
    assert_eq!(policy["inlinePattern"]["maxPatternStatements"], 2);
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
