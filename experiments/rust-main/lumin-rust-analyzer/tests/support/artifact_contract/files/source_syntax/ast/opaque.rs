use serde_json::Value;

pub(super) fn assert_review_opaque_example(merged_file: &Value) {
    assert!(
        merged_file["syntax"]["astExamples"]["reviewOpaqueSurfaces"][0]
            .get("visibility")
            .is_none()
    );
    assert_eq!(
        merged_file["syntax"]["astExamples"]["reviewOpaqueSurfaces"][0]["reason"],
        "cfg-condition-not-evaluated"
    );
}
