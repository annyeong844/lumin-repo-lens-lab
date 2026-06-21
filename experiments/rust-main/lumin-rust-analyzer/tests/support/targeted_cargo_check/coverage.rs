use anyhow::{Context, Result};
use serde_json::Value;

pub(super) fn assert_coverage_scope(coverage: &Value) -> Result<()> {
    assert_eq!(coverage["status"], "ran");
    assert_eq!(coverage["streamParseStatus"], "complete");
    assert_eq!(coverage["scope"]["kind"], "crate-target-configuration");
    assert_eq!(coverage["scope"]["package"], "app");
    assert!(coverage.get("command").is_none());
    assert!(coverage.get("commandArgs").is_none());
    assert_eq!(coverage["commandArgCount"], 5);
    assert!(coverage.get("analysisInputSetHash").is_none());
    assert!(coverage["scope"].get("cfgSet").is_none());
    assert_eq!(
        coverage["scope"]["featureSelection"]["defaultFeatures"],
        true
    );
    assert!(coverage["scope"].get("targets").is_none());
    assert_eq!(coverage["scope"]["targetCount"], 1);
    assert!(coverage["scope"]["targetExamples"]
        .as_array()
        .context("coverage scope target examples")?
        .iter()
        .any(|target| {
            target["targetName"] == "app"
                && target["source"] == "cargo-json-message"
                && target.get("packageId").is_none()
        }));
    Ok(())
}
