use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_skip(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["summary"]["syntaxReviewSignals"], 1);
    assert_eq!(artifact["summary"]["syntaxReviewOpaqueSurfaces"], 0);
    assert_eq!(artifact["oraclePlan"]["status"], "not-run");
    assert_eq!(artifact["oraclePlan"]["targetPathCount"], 0);
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 0);
    assert!(artifact["coverage"][0]["exitCode"].is_null());
    assert_eq!(
        artifact["files"]["src/lib.rs"]["syntax"]["reviewSignals"][0]["kind"],
        "unwrap-call"
    );
    assert!(artifact["semanticFindings"]
        .as_array()
        .context("semantic findings")?
        .is_empty());
    Ok(())
}
