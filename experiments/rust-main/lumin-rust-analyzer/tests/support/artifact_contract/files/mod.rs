use anyhow::Result;
use serde_json::Value;

mod muted;
mod source_oracle_bridge;
mod source_semantic;
mod source_syntax;

pub(super) fn assert_source_file_projection(artifact: &Value) -> Result<()> {
    let merged_file = &artifact["files"]["src/lib.rs"];
    source_syntax::assert_source_syntax_projection(merged_file)?;
    source_semantic::assert_source_semantic_projection(artifact, merged_file)?;
    source_oracle_bridge::assert_source_oracle_bridge_projection(artifact, merged_file)?;
    Ok(())
}

pub(super) fn assert_muted_file_projections(artifact: &Value) -> Result<()> {
    muted::assert_muted_file_projections(artifact)
}
