use lumin_rust_cargo_oracle::protocol::Summary as SemanticSummary;
use lumin_rust_source_health::protocol::HealthResponse;
use serde::Serialize;

mod actions;
mod semantic;
mod syntax;

use crate::policy::{
    ActionPolicyProjection, ActionTierSummary, EvidenceTierSummary, OracleBridgeProjection,
    OracleBridgeStatus,
};
use crate::product_files::{ProductFilesProjection, SemanticRefCounts};
use actions::ProductSemanticActionSummary;
use semantic::ProductSemanticSummary;
use syntax::ProductSyntaxSummary;

pub(crate) fn product_summary<'a>(
    syntax_phase: &'a HealthResponse,
    files: &ProductFilesProjection<'_>,
    semantic_summary: &'a SemanticSummary,
    action_policy: &ActionPolicyProjection<'_>,
    oracle_bridge: &OracleBridgeProjection<'_>,
    unlinked_semantic_refs: SemanticRefCounts,
) -> ProductSummary<'a> {
    let action_tier_summary = action_policy.action_tier_summary();
    let evidence_tier_summary = action_policy.evidence_tier_summary();
    ProductSummary {
        files: files.len(),
        syntax: ProductSyntaxSummary::from_syntax(syntax_phase),
        semantic: ProductSemanticSummary::from_semantic(semantic_summary, unlinked_semantic_refs),
        semantic_actions: ProductSemanticActionSummary::from_action_policy_projection(
            action_policy,
        ),
        oracle_bridge_status: oracle_bridge.status(),
        action_tier_summary,
        evidence_tier_summary,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductSummary<'a> {
    files: usize,
    #[serde(flatten)]
    syntax: ProductSyntaxSummary<'a>,
    #[serde(flatten)]
    semantic: ProductSemanticSummary,
    #[serde(flatten)]
    semantic_actions: ProductSemanticActionSummary,
    oracle_bridge_status: OracleBridgeStatus,
    action_tier_summary: ActionTierSummary,
    evidence_tier_summary: EvidenceTierSummary,
}

impl ProductSummary<'_> {
    pub(crate) fn semantic_unlinked_refs(&self) -> SemanticRefCounts {
        self.semantic.semantic_unlinked_refs.counts()
    }
}
