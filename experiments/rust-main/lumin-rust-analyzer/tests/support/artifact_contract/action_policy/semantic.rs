use anyhow::{anyhow, Result};
use serde_json::Value;

pub(super) fn assert_semantic_projection(artifact: &Value) -> Result<()> {
    let policy = &artifact["actionPolicy"];

    assert_eq!(
        policy["reasons"]["REVIEW_FIX"]["semantic-action-blocker"],
        1
    );
    assert_eq!(policy["semanticActionBlockers"]["findings"], 1);
    assert_eq!(
        policy["semanticActionBlockers"]["byReason"]["diagnostic-level-not-warning"],
        1
    );
    assert!(
        policy["semanticActionBlockers"]["examples"][0]["actionBlockers"]
            .as_array()
            .ok_or_else(|| anyhow!("action blocker reasons"))?
            .iter()
            .any(|reason| reason == "diagnostic-level-not-warning")
    );

    assert_eq!(policy["semanticReview"]["findings"], 0);
    assert!(policy["semanticReview"]["examples"]
        .as_array()
        .ok_or_else(|| anyhow!("semantic review examples"))?
        .is_empty());

    assert_eq!(policy["semanticDegraded"]["coverageEntries"], 1);
    assert_eq!(
        policy["semanticDegraded"]["examples"][0]["reason"],
        "coverage-unavailable-entry"
    );

    assert!(policy["semanticSafeActions"]["examples"]
        .as_array()
        .ok_or_else(|| anyhow!("safe action examples"))?
        .is_empty());

    Ok(())
}
