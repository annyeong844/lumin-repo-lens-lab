use serde_json::Value;

pub(super) fn assert_summary(artifact: &Value) {
    let summary = &artifact["summary"];
    assert_eq!(summary["ruleBackedSemanticFindings"], 1);
    assert_eq!(summary["semanticSafeActions"], 0);
    assert_eq!(summary["semanticActionBlockedFindings"], 1);
    assert_eq!(summary["semanticReviewFindings"], 0);
    assert_eq!(summary["semanticDegradedFindings"], 0);
    assert_eq!(summary["semanticDegradedCoverageEntries"], 0);
    assert_eq!(
        artifact["actionPolicy"]["semanticActionBlockers"]["byReason"]["macro-expansion"],
        1
    );
    assert_eq!(summary["actionTierSummary"]["SAFE_FIX"], 0);
    assert_eq!(summary["actionTierSummary"]["REVIEW_FIX"], 1);
    assert_eq!(summary["actionTierSummary"]["DEGRADED"], 0);
    assert_eq!(summary["actionTierSummary"]["MUTED"], 0);
    assert_eq!(summary["actionTierSummary"]["UNAVAILABLE"], 0);
    assert_eq!(summary["evidenceTierSummary"]["review"], 1);
    assert_eq!(summary["evidenceTierSummary"]["degraded"], 0);
    assert_eq!(summary["evidenceTierSummary"]["muted"], 0);
    assert_eq!(summary["evidenceTierSummary"]["unavailable"], 0);
    assert_eq!(summary["semanticClean"]["status"], "ran");
    assert_eq!(summary["semanticClean"]["clean"], true);
}
