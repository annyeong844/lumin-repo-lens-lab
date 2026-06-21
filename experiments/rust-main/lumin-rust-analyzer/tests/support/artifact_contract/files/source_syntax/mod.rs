use anyhow::Result;
use serde_json::Value;

mod ast;
mod signals;

pub(super) fn assert_source_syntax_projection(merged_file: &Value) -> Result<()> {
    signals::assert_review_signal_projection(merged_file);
    ast::assert_ast_projection(merged_file)?;
    assert_eq!(merged_file["syntax"]["parse"]["ok"], true);
    assert!(merged_file["syntax"]["parse"].get("errorCount").is_none());
    assert!(merged_file["syntax"]["parse"].get("sampleLimit").is_none());
    assert!(merged_file["syntax"]["parse"]["errors"].is_null());
    assert!(merged_file["syntax"]["parse"]
        .get("errorExamples")
        .is_none());
    assert!(merged_file["syntax"]["signalProjection"].is_null());
    assert!(merged_file["syntax"]["astProjection"].is_null());
    assert!(merged_file["syntax"].get("mutedSignals").is_none());
    Ok(())
}
