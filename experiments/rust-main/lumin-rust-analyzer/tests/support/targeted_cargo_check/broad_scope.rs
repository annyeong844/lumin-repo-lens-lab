use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_uncapped_run(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(
        artifact["oraclePlan"]["reason"],
        "review-syntax-evidence-package-scope"
    );
    assert_eq!(artifact["oraclePlan"]["targetPathCount"], 18);
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["targetPathExamples"],
        10
    );
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["selectedPackageExamples"],
        3
    );
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["omittedPackageExamples"],
        5
    );
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["unmatchedTargetPathExamples"],
        3
    );
    assert_eq!(
        artifact["oraclePlan"]["sampleLimits"]["selectedPackageTargetPathExamples"],
        5
    );
    assert_eq!(artifact["oraclePlan"]["selectedTargetPathCount"], 18);
    assert_eq!(artifact["oraclePlan"]["omittedTargetPathCount"], 0);
    assert_eq!(artifact["oraclePlan"]["candidatePackageCount"], 17);
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 17);
    assert_eq!(artifact["oraclePlan"]["omittedPackageCount"], 0);
    assert!(artifact["oraclePlan"]["omittedPackageExamples"]
        .as_array()
        .context("omitted package examples")?
        .is_empty());
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-partial");
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-partial");
    assert!(artifact["oraclePlan"].get("selectedPackages").is_none());
    assert!(artifact["oraclePlan"].get("unmatchedTargetPaths").is_none());
    let selected_examples = artifact["oraclePlan"]["selectedPackageExamples"]
        .as_array()
        .context("selected package examples")?;
    assert!(selected_examples.len() < 17);
    assert!(selected_examples
        .iter()
        .any(|package| package["packageName"] == "pkg0"));
    assert_eq!(artifact["coverage"][0]["streamParseStatus"], "complete");
    assert_eq!(artifact["coverage"][0]["exitCode"], 101);
    assert!(!artifact["semanticFindings"]
        .as_array()
        .context("semantic findings")?
        .is_empty());
    Ok(())
}
