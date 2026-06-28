use serde::Serialize;

use super::{
    CacheReusePolicy, CacheReuseReason, CacheReuseSummaryStatus, CleanKind, CleanScope,
    CoverageStatus, CoverageUnavailableReasons,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub findings: usize,
    pub diagnostics: usize,
    pub coverage: usize,
    pub verified_findings: usize,
    pub rule_backed_findings: usize,
    pub candidate_findings: usize,
    pub coverage_unavailable_diagnostics: usize,
    pub semantic_clean: SemanticCleanSummary,
    pub cache_reuse: CacheReuseSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticCleanSummary {
    pub status: CoverageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean_kind: Option<CleanKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clean_scope: Option<CleanScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<CoverageUnavailableReasons>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheReuseSummary {
    pub status: CacheReuseSummaryStatus,
    pub policy: CacheReusePolicy,
    pub reason: CacheReuseReason,
    pub blocking_target_count: usize,
}
