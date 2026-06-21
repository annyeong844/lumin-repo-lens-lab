use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_finding(artifact: &Value) -> Result<()> {
    let finding = &artifact["semanticFindings"][0];
    assert!(finding.get("safeAction").is_none());
    assert!(finding["actionBlockers"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("actionBlockers"))?
        .iter()
        .any(|blocker| blocker == "macro-expansion"));
    assert_eq!(finding["macroExpansionSpanCount"], 1);
    assert_eq!(
        finding["macroExpansionSpanExamples"][0]["macroDeclName"],
        "make_mut!"
    );
    assert!(finding["macroExpansionSpanExamples"][0]
        .get("expansion")
        .is_none());
    assert!(finding.get("primarySpans").is_none());
    assert_eq!(finding["actionTier"], "REVIEW_FIX");
    assert_eq!(finding["oracleConfidence"], "medium");
    assert!(finding["taintedBy"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("finding taint"))?
        .iter()
        .any(|entry| entry["kind"] == "semantic-action-blocker"));
    Ok(())
}
