use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const LIFECYCLE_EXIT_POLICY_REQUEST_SCHEMA_VERSION: &str =
    "lumin-lifecycle-exit-policy-request.v1";
pub const LIFECYCLE_EXIT_POLICY_RESULT_SCHEMA_VERSION: &str =
    "lumin-lifecycle-exit-policy-result.v1";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleExitPolicyRequest {
    pub schema_version: String,
    pub current_exit_code: i32,
    #[serde(default)]
    pub strict_post_write: bool,
    #[serde(default)]
    pub strict_post_write_confidence: bool,
    #[serde(default)]
    pub post_write: Option<PostWriteExitPolicyBlock>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostWriteExitPolicyBlock {
    #[serde(default)]
    pub ran: Option<bool>,
    #[serde(default)]
    pub baseline_status: Option<String>,
    #[serde(default)]
    pub scan_range_parity: Option<String>,
    #[serde(default)]
    pub type_escape_delta_status: Option<String>,
    #[serde(default)]
    pub after_complete: Option<Value>,
    #[serde(default)]
    pub file_delta_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleExitPolicyResult {
    pub schema_version: &'static str,
    pub exit_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

pub fn apply_lifecycle_exit_policy(
    request: LifecycleExitPolicyRequest,
) -> Result<LifecycleExitPolicyResult> {
    if request.schema_version != LIFECYCLE_EXIT_POLICY_REQUEST_SCHEMA_VERSION {
        bail!(
            "lifecycle-exit-policy: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut exit_code = request.current_exit_code;
    let mut stderr = String::new();

    if request.strict_post_write
        && request
            .post_write
            .as_ref()
            .is_some_and(|block| block.ran == Some(false))
        && exit_code == 0
    {
        stderr.push_str(
            "[audit-repo] --strict-post-write: post-write did not run; escalating to exit 2\n",
        );
        exit_code = 2;
    }

    if request.strict_post_write_confidence && exit_code == 0 {
        if let Some(block) = request
            .post_write
            .as_ref()
            .filter(|block| post_write_confidence_limited(block))
        {
            stderr.push_str(&format!(
                "[audit-repo] --strict-post-write-confidence: post-write delta confidence limited \
(baseline={}, scanRange={}, typeEscapeDelta={}, afterComplete={}); escalating to exit 2\n",
                block.baseline_status.as_deref().unwrap_or("unknown"),
                block.scan_range_parity.as_deref().unwrap_or("unknown"),
                block
                    .type_escape_delta_status
                    .as_deref()
                    .unwrap_or("unknown"),
                after_complete_true(block)
            ));
            exit_code = 2;
        }
    }

    Ok(LifecycleExitPolicyResult {
        schema_version: LIFECYCLE_EXIT_POLICY_RESULT_SCHEMA_VERSION,
        exit_code,
        stderr: (!stderr.is_empty()).then_some(stderr),
    })
}

fn post_write_confidence_limited(block: &PostWriteExitPolicyBlock) -> bool {
    if block.ran != Some(true) {
        return false;
    }
    if block.type_escape_delta_status.as_deref() == Some("not-applicable") {
        return block.file_delta_status.as_deref() != Some("computed");
    }
    block.baseline_status.as_deref() != Some("available")
        || block.scan_range_parity.as_deref() != Some("ok")
        || !after_complete_true(block)
}

fn after_complete_true(block: &PostWriteExitPolicyBlock) -> bool {
    matches!(block.after_complete.as_ref(), Some(Value::Bool(true)))
}
