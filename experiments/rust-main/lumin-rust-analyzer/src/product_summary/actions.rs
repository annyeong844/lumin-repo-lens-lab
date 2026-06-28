use serde::Serialize;

use crate::policy::ActionPolicyProjection;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductSemanticActionSummary {
    semantic_safe_actions: usize,
    semantic_action_blocked_findings: usize,
    semantic_review_findings: usize,
    semantic_degraded_findings: usize,
    semantic_degraded_coverage_entries: usize,
}

impl ProductSemanticActionSummary {
    pub(super) fn from_action_policy_projection(
        action_policy: &ActionPolicyProjection<'_>,
    ) -> Self {
        Self {
            semantic_safe_actions: action_policy.semantic_safe_actions(),
            semantic_action_blocked_findings: action_policy.semantic_action_blocked_findings(),
            semantic_review_findings: action_policy.semantic_review_findings(),
            semantic_degraded_findings: action_policy.semantic_degraded_findings(),
            semantic_degraded_coverage_entries: action_policy.semantic_degraded_coverage_entries(),
        }
    }
}
