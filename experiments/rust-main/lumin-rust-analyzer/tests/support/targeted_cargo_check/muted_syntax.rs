use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_skip(artifact: &Value) -> Result<()> {
    assert_eq!(
        artifact["meta"]["input"]["semanticMode"],
        "targeted-cargo-check"
    );
    assert_eq!(artifact["summary"]["syntaxReviewSignals"], 0);
    assert_eq!(artifact["summary"]["syntaxMutedSignals"], 1);
    assert_eq!(artifact["oraclePlan"]["status"], "not-run");
    assert_eq!(
        artifact["oraclePlan"]["reason"],
        "targeted-cargo-check-selected-no-packages"
    );
    assert_eq!(artifact["oraclePlan"]["targetPathCount"], 0);
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 0);
    assert_eq!(artifact["coverage"][0]["streamParseStatus"], "not-run");
    assert_eq!(artifact["coverage"][0]["status"], "unavailable");
    assert!(artifact["coverage"][0]["exitCode"].is_null());
    assert!(artifact["coverage"][0]["reason"]
        .as_str()
        .unwrap_or_default()
        .contains("selected no packages"));
    assert_eq!(
        artifact["files"]["tests/integration.rs"]["syntax"]["mutedSignals"][0]["muteReason"],
        "test-path"
    );
    assert!(artifact["semanticFindings"]
        .as_array()
        .context("semantic findings")?
        .is_empty());
    Ok(())
}
