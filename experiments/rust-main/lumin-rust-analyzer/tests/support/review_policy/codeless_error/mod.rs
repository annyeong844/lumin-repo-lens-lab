use serde_json::Value;

pub fn assert_review_fix(artifact: &Value) -> anyhow::Result<()> {
    let summary = &artifact["summary"];
    assert_eq!(summary["verifiedSemanticFindings"], 1);
    assert_eq!(summary["ruleBackedSemanticFindings"], 0);
    assert_eq!(summary["candidateSemanticFindings"], 0);
    assert_eq!(summary["semanticReviewFindings"], 1);
    assert_eq!(summary["semanticDegradedFindings"], 0);
    assert_eq!(summary["semanticDegradedCoverageEntries"], 1);
    assert_eq!(summary["semanticUnlinkedFindings"], 0);
    assert_eq!(summary["semanticUnlinkedDiagnostics"], 0);
    assert_eq!(summary["actionTierSummary"]["SAFE_FIX"], 0);
    assert_eq!(summary["actionTierSummary"]["REVIEW_FIX"], 1);
    assert_eq!(summary["actionTierSummary"]["DEGRADED"], 0);
    assert_eq!(summary["actionTierSummary"]["MUTED"], 0);
    assert_eq!(summary["actionTierSummary"]["UNAVAILABLE"], 0);
    assert_eq!(summary["evidenceTierSummary"]["review"], 2);
    assert_eq!(summary["evidenceTierSummary"]["degraded"], 1);
    assert_eq!(summary["evidenceTierSummary"]["muted"], 0);
    assert_eq!(summary["evidenceTierSummary"]["unavailable"], 0);
    assert_eq!(
        artifact["actionPolicy"]["semanticReview"]["byReason"]
            ["verified.rust.rustc-codeless-error-diagnostic"],
        1
    );
    assert_eq!(summary["semanticClean"]["status"], "unavailable");
    assert_eq!(summary["oracleBridgeStatus"], "oracle-partial");

    assert_eq!(
        artifact["actionPolicy"]["reasons"]["REVIEW_FIX"]["semantic-review-finding"],
        1
    );
    assert_eq!(
        artifact["actionPolicy"]["semanticReview"]["examples"][0]["reason"],
        "verified.rust.rustc-codeless-error-diagnostic"
    );

    assert_eq!(
        artifact["semanticFindings"][0]["confidence"]["claimKind"],
        "verified.rust.rustc-codeless-error-diagnostic"
    );
    assert_eq!(artifact["semanticFindings"][0]["actionTier"], "REVIEW_FIX");
    assert_eq!(
        artifact["semanticFindings"][0]["oracleConfidence"],
        "medium"
    );

    Ok(())
}
