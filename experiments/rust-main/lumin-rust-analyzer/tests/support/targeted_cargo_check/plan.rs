use anyhow::Result;
use serde_json::Value;

use super::{coverage, mode, selection};

pub fn assert_targeted_plan(artifact: &Value) -> Result<()> {
    let plan = &artifact["oraclePlan"];
    let coverage = &artifact["coverage"][0];
    assert_eq!(artifact["summary"]["oracleBridgeStatus"], "oracle-partial");
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-partial");
    assert_eq!(coverage["commandArgCount"], 5);
    assert!(coverage.get("commandArgs").is_none());
    mode::assert_targeted_mode(artifact, plan);
    selection::assert_selected_package(plan)?;
    coverage::assert_coverage_scope(coverage)?;
    Ok(())
}
