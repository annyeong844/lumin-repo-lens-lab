use anyhow::Result;
use serde_json::Value;

mod examples;
mod opaque;
mod summary;

pub(super) fn assert_ast_projection(merged_file: &Value) -> Result<()> {
    assert!(merged_file["syntax"].get("ast").is_none());
    summary::assert_ast_summary(merged_file);
    examples::assert_ast_examples(merged_file)?;
    opaque::assert_review_opaque_example(merged_file);
    Ok(())
}
