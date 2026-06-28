use serde_json::Value;

pub(super) fn assert_action_policy_summary(artifact: &Value) {
    let policy = &artifact["actionPolicy"];
    assert_eq!(policy["summary"]["SAFE_FIX"], 0);
    assert_eq!(policy["summary"]["REVIEW_FIX"], 1);
    assert_eq!(policy["summary"]["DEGRADED"], 0);
    assert_eq!(policy["summary"]["MUTED"], 0);
    assert_eq!(policy["summary"]["UNAVAILABLE"], 0);
    assert_eq!(policy["summary"]["total"], 1);
    assert_eq!(policy["evidenceTierSummary"]["review"], 3);
    assert_eq!(policy["evidenceTierSummary"]["degraded"], 1);
    assert_eq!(policy["evidenceTierSummary"]["muted"], 3);
    assert_eq!(policy["evidenceTierSummary"]["unavailable"], 0);
    assert_eq!(policy["evidenceTierSummary"]["total"], 7);
    assert_eq!(policy["safeFixGate"]["status"], "strict");
}
