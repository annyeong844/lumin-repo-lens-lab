use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_safe_action_bridge_core(artifact: &Value) -> Result<()> {
    assert_eq!(artifact["oracleBridge"]["status"], "oracle-covered");
    assert_eq!(
        artifact["oracleBridge"]["syntax"]["reviewOpaqueSurfaces"],
        0
    );
    assert_eq!(
        artifact["oracleBridge"]["coverage"]["absenceClean"]["clean"],
        true
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["candidateCounts"]["safeFix"],
        1
    );
    assert_eq!(
        artifact["oracleBridge"]["policy"]["calibration"]["candidateCounts"]
            ["reviewVisibleCleanup"],
        1
    );

    let file_bridge = &artifact["files"]["src/lib.rs"]["oracleBridge"];
    assert!(file_bridge.get("status").is_none());
    assert!(file_bridge.get("parseStatus").is_none());
    assert_eq!(file_bridge["oracleConfidence"], "high");
    assert_eq!(file_bridge["semantic"]["safeActions"], 1);
    assert!(file_bridge["semantic"]
        .get("actionBlockedFindings")
        .is_none());
    assert!(file_bridge["coverage"].is_null());
    assert!(file_bridge.get("taintedBy").is_none());
    Ok(())
}
