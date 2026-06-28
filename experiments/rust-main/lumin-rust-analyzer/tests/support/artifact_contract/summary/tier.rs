use serde_json::Value;

pub(super) fn assert_tier_summary(artifact: &Value) {
    let summary = &artifact["summary"];
    let policy = &artifact["actionPolicy"];

    assert_eq!(summary["actionTierSummary"], policy["summary"]);
    assert_eq!(
        summary["evidenceTierSummary"],
        policy["evidenceTierSummary"]
    );
    assert_eq!(summary["actionTierSummary"]["SAFE_FIX"], 0);
    assert_eq!(summary["actionTierSummary"]["REVIEW_FIX"], 1);
    assert_eq!(summary["actionTierSummary"]["DEGRADED"], 0);
    assert_eq!(summary["actionTierSummary"]["MUTED"], 0);
    assert_eq!(summary["actionTierSummary"]["UNAVAILABLE"], 0);
    assert_eq!(summary["actionTierSummary"]["total"], 1);
    assert_eq!(summary["evidenceTierSummary"]["review"], 3);
    assert_eq!(summary["evidenceTierSummary"]["degraded"], 1);
    assert_eq!(summary["evidenceTierSummary"]["muted"], 3);
    assert_eq!(summary["evidenceTierSummary"]["unavailable"], 0);
    assert_eq!(summary["evidenceTierSummary"]["total"], 7);
}
