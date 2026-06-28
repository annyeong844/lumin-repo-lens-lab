use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_runs_oracle(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["summary"]["syntaxReviewOpaqueSurfaces"], 1);
    assert_eq!(
        artifact["files"]["src/lib.rs"]["syntax"]["astExamples"]["reviewOpaqueSurfaces"][0]
            ["reason"],
        "cfg-condition-not-evaluated"
    );
    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(
        artifact["oraclePlan"]["reason"],
        "review-syntax-evidence-package-scope"
    );
    assert_eq!(artifact["oraclePlan"]["targetPathCount"], 1);
    assert_eq!(artifact["oraclePlan"]["selectedTargetPathCount"], 1);
    assert_eq!(artifact["oraclePlan"]["omittedTargetPathCount"], 0);
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 1);
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-covered");
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-covered");
    assert_eq!(
        artifact["oracleBridge"]["coverage"]["absenceClean"]["clean"],
        true
    );
    assert_eq!(artifact["coverage"][0]["status"], "ran");
    assert_eq!(artifact["coverage"][0]["streamParseStatus"], "complete");
    assert_eq!(
        artifact["files"]["src/lib.rs"]["oracleBridge"]["oracleConfidence"],
        "medium"
    );
    assert!(artifact["files"]["src/lib.rs"]["oracleBridge"]["taintedBy"]
        .as_array()
        .context("cfg file bridge taint")?
        .iter()
        .any(|entry| entry["kind"] == "rust-ast-review-opaque-surface"));
    Ok(())
}
