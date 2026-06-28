use serde_json::Value;

pub(super) fn assert_evidence_projection(artifact: &Value) {
    let policy = &artifact["actionPolicy"];
    assert_eq!(
        policy["evidenceReasons"]["review"]["syntax-review-signal"],
        1
    );
    assert_eq!(
        policy["evidenceReasons"]["review"]["syntax-review-opaque-surface"],
        2
    );
    assert_eq!(
        policy["evidenceReasons"]["degraded"]["coverage-unavailable-entry"],
        1
    );
    assert_eq!(policy["evidenceReasons"]["muted"]["syntax-muted-signal"], 2);
    assert_eq!(
        policy["reasons"]["UNAVAILABLE"]["coverage-unavailable-diagnostic"],
        0
    );
}
