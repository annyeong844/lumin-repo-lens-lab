use anyhow::Result;
use serde_json::Value;

pub(super) fn assert_muted_file_projections(artifact: &Value) -> Result<()> {
    let test_file = &artifact["files"]["tests/integration.rs"];
    assert_eq!(
        test_file["syntax"]["mutedSignals"][0]["muteReason"],
        "test-path"
    );
    assert!(test_file["syntax"]["mutedSignals"][0]
        .get("visibility")
        .is_none());
    assert!(test_file["syntax"].get("reviewSignals").is_none());
    assert!(test_file.get("oracleBridge").is_none());
    assert_eq!(test_file["syntax"]["astSummary"]["mutedOpaqueSurfaces"], 1);
    assert_eq!(
        test_file["syntax"]["astSummary"]["mutedOpaqueSurfacesByReason"]["test-path"],
        1
    );
    assert!(test_file["syntax"].get("astExamples").is_none());

    let generated_file = &artifact["files"]["generated/bindings.rs"];
    assert_eq!(
        generated_file["syntax"]["mutedSignals"][0]["muteReason"],
        "generated-path"
    );
    assert!(generated_file["syntax"]["mutedSignals"][0]
        .get("visibility")
        .is_none());
    assert!(generated_file.get("oracleBridge").is_none());
    Ok(())
}
