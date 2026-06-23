use lumin_rust_cargo_oracle::protocol::{
    CacheReusePolicy, CacheReuseReason, CacheReuseSummary, CacheReuseSummaryStatus, CleanKind,
    CleanScope, CoverageStatus, CoverageUnavailableReasons, PrimarySpan, PrimarySpanClass,
    SemanticCleanSummary,
};
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProductCoverageUnavailableReason {
    message: String,
}

impl ProductCoverageUnavailableReason {
    pub(crate) fn from_reason(reason: &CoverageUnavailableReasons) -> Self {
        Self {
            message: reason.message(),
        }
    }
}

impl Serialize for ProductCoverageUnavailableReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.message)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductSemanticCleanSummaryProjection {
    status: CoverageStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    clean: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clean_kind: Option<CleanKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clean_scope: Option<CleanScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<ProductCoverageUnavailableReason>,
}

impl ProductSemanticCleanSummaryProjection {
    pub(crate) fn from_summary(summary: &SemanticCleanSummary) -> Self {
        Self {
            status: summary.status,
            clean: summary.clean,
            clean_kind: summary.clean_kind,
            clean_scope: summary.clean_scope,
            reason: summary
                .reason
                .as_ref()
                .map(ProductCoverageUnavailableReason::from_reason),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductCacheReuseSummaryProjection {
    status: CacheReuseSummaryStatus,
    policy: CacheReusePolicy,
    reason: CacheReuseReason,
    blocking_target_count: usize,
}

impl ProductCacheReuseSummaryProjection {
    pub(crate) fn from_summary(summary: &CacheReuseSummary) -> Self {
        Self {
            status: summary.status,
            policy: summary.policy,
            reason: summary.reason,
            blocking_target_count: summary.blocking_target_count,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductPrimarySpanProjection<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_end: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_end: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_expansion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    macro_decl_name: Option<&'a str>,
    primary_span_class: PrimarySpanClass,
}

impl<'a> ProductPrimarySpanProjection<'a> {
    pub(crate) fn from_span(span: &'a PrimarySpan) -> Self {
        Self {
            file_name: span.file_name.as_deref(),
            line_start: span.line_start,
            line_end: span.line_end,
            column_start: span.column_start,
            column_end: span.column_end,
            has_expansion: span.has_expansion.then_some(true),
            macro_decl_name: span
                .expansion
                .as_ref()
                .and_then(|expansion| expansion.macro_decl_name.as_deref()),
            primary_span_class: span.primary_span_class,
        }
    }
}
