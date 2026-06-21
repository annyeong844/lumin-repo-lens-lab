use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_capped_run(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(
        artifact["oraclePlan"]["reason"],
        "review-syntax-evidence-package-scope"
    );
    assert_eq!(artifact["oraclePlan"]["targetPathCount"], 18);
    assert_eq!(artifact["oraclePlan"]["selectedTargetPathCount"], 16);
    assert_eq!(artifact["oraclePlan"]["omittedTargetPathCount"], 2);
    assert_eq!(artifact["oraclePlan"]["candidatePackageCount"], 17);
    assert_eq!(artifact["oraclePlan"]["selectedPackageCount"], 16);
    assert_eq!(artifact["oraclePlan"]["targetedPackageCap"], 16);
    assert_eq!(artifact["oraclePlan"]["omittedPackageCount"], 1);
    assert_eq!(artifact["oraclePlan"]["omittedPackageExamples"][0], "pkg16");
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-partial");
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-partial");
    assert!(artifact["oraclePlan"].get("selectedPackages").is_none());
    assert!(artifact["oraclePlan"].get("unmatchedTargetPaths").is_none());
    let selected_examples = artifact["oraclePlan"]["selectedPackageExamples"]
        .as_array()
        .context("selected package examples")?;
    assert!(selected_examples.len() < 16);
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

pub(super) fn assert_custom_package_cap_run(
    artifact: &Value,
    targeted_package_cap: usize,
    package_count: usize,
) -> Result<()> {
    assert_eq!(artifact["oraclePlan"]["status"], "ran");
    assert_eq!(
        artifact["oraclePlan"]["reason"],
        "review-syntax-evidence-package-scope"
    );
    assert_eq!(
        artifact["oraclePlan"]["candidatePackageCount"],
        package_count
    );
    assert_eq!(
        artifact["oraclePlan"]["selectedTargetPathCount"],
        targeted_package_cap
    );
    assert_eq!(
        artifact["oraclePlan"]["omittedTargetPathCount"],
        package_count.saturating_sub(targeted_package_cap) + 1
    );
    assert_eq!(
        artifact["oraclePlan"]["selectedPackageCount"],
        targeted_package_cap
    );
    assert_eq!(
        artifact["oraclePlan"]["targetedPackageCap"],
        targeted_package_cap
    );
    assert_eq!(
        artifact["oraclePlan"]["omittedPackageCount"],
        package_count.saturating_sub(targeted_package_cap)
    );
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-partial");
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-partial");
    assert_eq!(artifact["coverage"][0]["streamParseStatus"], "complete");
    assert_eq!(artifact["coverage"][0]["exitCode"], 101);
    assert!(!artifact["semanticFindings"]
        .as_array()
        .context("semantic findings")?
        .is_empty());
    Ok(())
}
