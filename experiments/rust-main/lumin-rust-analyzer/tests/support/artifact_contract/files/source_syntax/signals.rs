use serde_json::Value;

pub(super) fn assert_review_signal_projection(merged_file: &Value) {
    assert_eq!(
        merged_file["syntax"]["reviewSignals"][0]["kind"],
        "unwrap-call"
    );
    assert!(merged_file["syntax"]["reviewSignals"][0]
        .get("confidenceTier")
        .is_none());
    assert!(merged_file["syntax"]["reviewSignals"][0]
        .get("claim")
        .is_none());
    assert!(merged_file["syntax"]["reviewSignals"][0]
        .get("visibility")
        .is_none());
    assert_eq!(merged_file["syntax"]["signalSummary"]["review"], 1);
}
