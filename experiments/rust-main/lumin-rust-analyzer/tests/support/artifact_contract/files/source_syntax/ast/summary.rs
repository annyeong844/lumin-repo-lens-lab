use serde_json::Value;

pub(super) fn assert_ast_summary(merged_file: &Value) {
    assert!(merged_file["syntax"]["astSummary"]
        .get("pathRefs")
        .is_none());
    assert!(merged_file["syntax"]["astSummary"]
        .get("methodCallSites")
        .is_none());
    assert_eq!(
        merged_file["syntax"]["astSummary"]["reviewOpaqueSurfaces"],
        2
    );
    assert!(merged_file["syntax"]["astSummary"]
        .get("mutedOpaqueSurfaces")
        .is_none());
}
