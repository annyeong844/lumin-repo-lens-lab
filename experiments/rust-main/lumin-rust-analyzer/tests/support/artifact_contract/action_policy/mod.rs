use anyhow::Result;
use serde_json::Value;

mod evidence;
mod semantic;
mod summary;

pub(super) fn assert_action_policy_projection(artifact: &Value) -> Result<()> {
    summary::assert_action_policy_summary(artifact);
    evidence::assert_evidence_projection(artifact);
    semantic::assert_semantic_projection(artifact)?;
    Ok(())
}
