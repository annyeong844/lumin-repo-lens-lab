use anyhow::Result;
use serde_json::Value;

mod finding;
mod policy;
mod summary;

pub fn assert_review_fix(artifact: &Value) -> Result<()> {
    summary::assert_summary(artifact);
    policy::assert_policy(artifact)?;
    finding::assert_finding(artifact)?;
    Ok(())
}
