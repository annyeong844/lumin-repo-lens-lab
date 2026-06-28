use serde_json::Value;

pub(super) fn assert_safe_action_summary(artifact: &Value) {
    assert_eq!(artifact["summary"]["verifiedSemanticFindings"], 0);
    assert_eq!(artifact["summary"]["ruleBackedSemanticFindings"], 1);
    assert_eq!(artifact["summary"]["semanticSafeActions"], 1);
    assert_eq!(artifact["summary"]["semanticActionBlockedFindings"], 0);
    assert_eq!(artifact["summary"]["semanticReviewFindings"], 0);
    assert_eq!(artifact["summary"]["semanticDegradedFindings"], 0);
    assert_eq!(artifact["summary"]["semanticDegradedCoverageEntries"], 0);
    assert_eq!(artifact["summary"]["actionTierSummary"]["SAFE_FIX"], 1);
    assert_eq!(artifact["summary"]["actionTierSummary"]["REVIEW_FIX"], 0);
    assert_eq!(artifact["summary"]["actionTierSummary"]["DEGRADED"], 0);
    assert_eq!(artifact["summary"]["actionTierSummary"]["MUTED"], 0);
    assert_eq!(artifact["summary"]["actionTierSummary"]["UNAVAILABLE"], 0);
    assert_eq!(artifact["summary"]["actionTierSummary"]["total"], 1);
    assert_eq!(artifact["summary"]["evidenceTierSummary"]["review"], 0);
    assert_eq!(artifact["summary"]["evidenceTierSummary"]["degraded"], 0);
    assert_eq!(artifact["summary"]["evidenceTierSummary"]["muted"], 0);
    assert_eq!(artifact["summary"]["evidenceTierSummary"]["unavailable"], 0);
    assert_eq!(artifact["summary"]["evidenceTierSummary"]["total"], 0);
    assert_eq!(artifact["summary"]["semanticClean"]["status"], "ran");
    assert_eq!(artifact["summary"]["semanticClean"]["clean"], true);
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-covered");
}
