use lumin_rust_cargo_oracle::protocol::Summary as SemanticSummary;
use serde::Serialize;

use crate::policy::{ProductCacheReuseSummaryProjection, ProductSemanticCleanSummaryProjection};
use crate::product_files::SemanticRefCounts;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductSemanticSummary {
    verified_semantic_findings: usize,
    rule_backed_semantic_findings: usize,
    candidate_semantic_findings: usize,
    semantic_coverage_unavailable_diagnostics: usize,
    #[serde(flatten)]
    pub(super) semantic_unlinked_refs: ProductSemanticUnlinkedRefs,
    semantic_clean: ProductSemanticCleanSummaryProjection,
    cache_reuse: ProductCacheReuseSummaryProjection,
}

impl ProductSemanticSummary {
    pub(super) fn from_semantic(
        summary: &SemanticSummary,
        unlinked_refs: SemanticRefCounts,
    ) -> Self {
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
pub(super) struct ProductSemanticUnlinkedRefs {
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

    pub(super) fn counts(&self) -> SemanticRefCounts {
        SemanticRefCounts::new(
            self.semantic_unlinked_findings,
            self.semantic_unlinked_diagnostics,
        )
    }
}
