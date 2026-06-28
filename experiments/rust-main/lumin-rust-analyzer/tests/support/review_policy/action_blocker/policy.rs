use anyhow::{anyhow, Result};
use serde_json::Value;

pub(super) fn assert_policy(artifact: &Value) -> Result<()> {
    let policy = &artifact["actionPolicy"];

    assert_eq!(
        policy["reasons"]["REVIEW_FIX"]["semantic-action-blocker"],
        1
    );
    assert_eq!(
        policy["reasons"]["REVIEW_FIX"]["semantic-review-finding"],
        0
    );

    let blockers = &policy["semanticActionBlockers"];
    assert_eq!(blockers["byReason"]["macro-expansion"], 1);
    assert_eq!(
        blockers["examples"][0]["actionBlockers"][0],
        "macro-expansion"
    );
    assert_eq!(
        blockers["examples"][0]["claimKind"],
        "rule-backed.rust.rustc-lint-diagnostic"
    );
    assert!(blockers["examples"][0]["span"].get("expansion").is_none());
    assert_eq!(blockers["examples"][0]["span"]["hasExpansion"], true);
    assert_eq!(
        blockers["examples"][0]["span"]["macroDeclName"],
        "make_mut!"
    );

    assert!(policy["semanticSafeActions"]["examples"]
        .as_array()
        .ok_or_else(|| anyhow!("safe action examples"))?
        .is_empty());
    assert!(policy["semanticReview"]["examples"]
        .as_array()
        .ok_or_else(|| anyhow!("semantic review examples"))?
        .is_empty());

    Ok(())
}
