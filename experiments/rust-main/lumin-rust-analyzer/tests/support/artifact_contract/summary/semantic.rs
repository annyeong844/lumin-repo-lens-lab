use serde_json::Value;

pub(super) fn assert_semantic_summary(artifact: &Value) {
    let summary = &artifact["summary"];

    assert_eq!(summary["verifiedSemanticFindings"], 1);
    assert_eq!(summary["semanticSafeActions"], 0);
    assert_eq!(summary["semanticActionBlockedFindings"], 1);
    assert_eq!(summary["semanticReviewFindings"], 0);
    assert_eq!(summary["semanticDegradedFindings"], 0);
    assert_eq!(summary["semanticDegradedCoverageEntries"], 1);
    assert_eq!(summary["semanticCoverageUnavailableDiagnostics"], 0);
    assert_eq!(summary["semanticUnlinkedFindings"], 0);
    assert_eq!(summary["semanticUnlinkedDiagnostics"], 1);
    assert!(summary.get("semanticActionBlockersByReason").is_none());
    assert!(summary.get("semanticReviewByReason").is_none());
    assert!(summary.get("semanticDegradedByReason").is_none());

    let policy = &artifact["actionPolicy"];
    assert_eq!(
        summary["semanticSafeActions"],
        policy["semanticSafeActions"]["findings"]
    );
    assert_eq!(
        summary["semanticActionBlockedFindings"],
        policy["semanticActionBlockers"]["findings"]
    );
    assert_eq!(
        summary["semanticReviewFindings"],
        policy["semanticReview"]["findings"]
    );
    assert_eq!(
        summary["semanticDegradedFindings"],
        policy["semanticDegraded"]["findings"]
    );
    assert_eq!(
        summary["semanticDegradedCoverageEntries"],
        policy["semanticDegraded"]["coverageEntries"]
    );
    assert_eq!(
        policy["semanticActionBlockers"]["byReason"]["diagnostic-level-not-warning"],
        1
    );
    assert_eq!(
        policy["semanticDegraded"]["byReason"]["coverage-unavailable-entry"],
        1
    );
}
