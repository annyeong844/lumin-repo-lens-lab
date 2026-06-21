use std::collections::BTreeMap;

use lumin_rust_cargo_oracle::protocol::Summary as SemanticSummary;
use lumin_rust_source_health::protocol::{AstOpaqueMuteReason, HealthResponse, SignalMuteReason};
use serde::Serialize;

use crate::policy::{
    syntax_review_opaque_surface_examples, syntax_review_signal_examples, ActionPolicyProjection,
    ActionTierSummary, EvidenceTierSummary, OracleBridgeProjection, OracleBridgeStatus,
    ProductCacheReuseSummaryProjection, ProductSemanticCleanSummaryProjection,
    SyntaxReviewOpaqueSurfaceExample, SyntaxReviewSignalExample,
};
use crate::product_files::{ProductFilesProjection, SemanticRefCounts};

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductSemanticActionSummary {
    semantic_safe_actions: usize,
    semantic_action_blocked_findings: usize,
    semantic_review_findings: usize,
    semantic_degraded_findings: usize,
    semantic_degraded_coverage_entries: usize,
}

impl ProductSemanticActionSummary {
    fn from_action_policy_projection(action_policy: &ActionPolicyProjection<'_>) -> Self {
        Self {
            semantic_safe_actions: action_policy.semantic_safe_actions(),
            semantic_action_blocked_findings: action_policy.semantic_action_blocked_findings(),
            semantic_review_findings: action_policy.semantic_review_findings(),
            semantic_degraded_findings: action_policy.semantic_degraded_findings(),
            semantic_degraded_coverage_entries: action_policy.semantic_degraded_coverage_entries(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductSyntaxSummary<'a> {
    syntax_parse_error_files: usize,
    syntax_parse_errors: usize,
    syntax_review_signals: usize,
    syntax_muted_signals: usize,
    syntax_muted_signals_by_reason: &'a BTreeMap<SignalMuteReason, usize>,
    syntax_review_signal_examples: Vec<SyntaxReviewSignalExample<'a>>,
    syntax_definitions: usize,
    syntax_use_trees: usize,
    syntax_path_refs: usize,
    syntax_method_call_sites: usize,
    syntax_method_calls: usize,
    syntax_macro_calls: usize,
    syntax_cfg_gates: usize,
    syntax_opaque_surfaces: usize,
    syntax_review_opaque_surfaces: usize,
    syntax_muted_opaque_surfaces: usize,
    syntax_muted_opaque_surfaces_by_reason: &'a BTreeMap<AstOpaqueMuteReason, usize>,
    syntax_review_opaque_surface_examples: Vec<SyntaxReviewOpaqueSurfaceExample<'a>>,
}

impl<'a> ProductSyntaxSummary<'a> {
    fn from_syntax(response: &'a HealthResponse) -> Self {
        let summary = &response.summary;
        Self {
            syntax_parse_error_files: summary.parse_error_files,
            syntax_parse_errors: summary.parse_errors,
            syntax_review_signals: summary.review_signals,
            syntax_muted_signals: summary.muted_signals,
            syntax_muted_signals_by_reason: &summary.muted_signals_by_reason,
            syntax_review_signal_examples: syntax_review_signal_examples(response),
            syntax_definitions: summary.definitions,
            syntax_use_trees: summary.use_trees,
            syntax_path_refs: summary.path_refs,
            syntax_method_call_sites: summary.method_call_sites,
            syntax_method_calls: summary.method_calls,
            syntax_macro_calls: summary.macro_calls,
            syntax_cfg_gates: summary.cfg_gates,
            syntax_opaque_surfaces: summary.opaque_surfaces,
            syntax_review_opaque_surfaces: summary.review_opaque_surfaces,
            syntax_muted_opaque_surfaces: summary.muted_opaque_surfaces,
            syntax_muted_opaque_surfaces_by_reason: &summary.muted_opaque_surfaces_by_reason,
            syntax_review_opaque_surface_examples: syntax_review_opaque_surface_examples(response),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductSemanticSummary {
    verified_semantic_findings: usize,
    rule_backed_semantic_findings: usize,
    candidate_semantic_findings: usize,
    semantic_coverage_unavailable_diagnostics: usize,
    #[serde(flatten)]
    semantic_unlinked_refs: ProductSemanticUnlinkedRefs,
    semantic_clean: ProductSemanticCleanSummaryProjection,
    cache_reuse: ProductCacheReuseSummaryProjection,
}

impl ProductSemanticSummary {
    fn from_semantic(summary: &SemanticSummary, unlinked_refs: SemanticRefCounts) -> Self {
        Self {
            verified_semantic_findings: summary.verified_findings,
            rule_backed_semantic_findings: summary.rule_backed_findings,
            candidate_semantic_findings: summary.candidate_findings,
            semantic_coverage_unavailable_diagnostics: summary.coverage_unavailable_diagnostics,
            semantic_unlinked_refs: ProductSemanticUnlinkedRefs::from_counts(unlinked_refs),
            semantic_clean: ProductSemanticCleanSummaryProjection::from_summary(
                &summary.semantic_clean,
            ),
            cache_reuse: ProductCacheReuseSummaryProjection::from_summary(&summary.cache_reuse),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductSemanticUnlinkedRefs {
    semantic_unlinked_findings: usize,
    semantic_unlinked_diagnostics: usize,
}

impl ProductSemanticUnlinkedRefs {
    fn from_counts(counts: SemanticRefCounts) -> Self {
        Self {
            semantic_unlinked_findings: counts.findings(),
            semantic_unlinked_diagnostics: counts.diagnostics(),
        }
    }

    fn counts(&self) -> SemanticRefCounts {
        SemanticRefCounts::new(
            self.semantic_unlinked_findings,
            self.semantic_unlinked_diagnostics,
        )
    }
}
