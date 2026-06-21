use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_selected_package(plan: &Value) -> Result<()> {
    assert_eq!(plan["targetPathCount"], 2);
    assert_eq!(plan["selectedTargetPathCount"], 1);
    assert_eq!(plan["omittedTargetPathCount"], 0);
    assert_eq!(plan["selectedPackageCount"], 1);
    assert!(plan.get("selectedPackages").is_none());
    assert_eq!(plan["selectedPackageExamples"][0]["packageName"], "app");
    assert!(plan["selectedPackageExamples"][0]
        .get("packageId")
        .is_none());
    assert!(plan["selectedPackageExamples"][0]
        .get("manifestPath")
        .is_none());
    assert_eq!(plan["selectedPackageExamples"][0]["targetPathCount"], 1);
    assert!(plan["selectedPackageExamples"][0]["targetPathExamples"]
        .as_array()
        .context("target path examples")?
        .iter()
        .any(|path| path == "app/src/lib.rs"));
    assert_eq!(plan["unmatchedTargetPathCount"], 1);
    assert!(plan.get("unmatchedTargetPaths").is_none());
    assert_eq!(plan["unmatchedTargetPathExamples"][0], "loose.rs");
    Ok(())
}
