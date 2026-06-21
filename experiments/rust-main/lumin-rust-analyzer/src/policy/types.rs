use std::borrow::Cow;

use lumin_rust_cargo_oracle::protocol::{
    CacheReusePolicy, CacheReuseReason, CacheReuseSummary, CacheReuseSummaryStatus, CleanKind,
    CleanScope, ConfidenceTier, CoverageStatus, CoverageUnavailableReasons, PrimarySpan,
    PrimarySpanClass, SemanticCleanSummary,
};
use lumin_rust_common::posix_path_text;
use serde::{Serialize, Serializer};

pub(crate) const POLICY_VERSION: &str = "rust-unified-analyzer.v1";
pub(crate) const SYNTAX_CONFIDENCE_TIER: ConfidenceTier = ConfidenceTier::Candidate;
pub(crate) const ACTION_SAMPLE_LIMIT: usize = 5;
pub(crate) const SIGNAL_SAMPLE_LIMIT: usize = 3;
pub(crate) const FILE_SIGNAL_SAMPLE_LIMIT: usize = 1;
pub(crate) const PARSE_ERROR_SAMPLE_LIMIT: usize = 3;
pub(crate) const SKIPPED_FILE_SAMPLE_LIMIT: usize = 3;
pub(crate) const AST_SAMPLE_LIMIT: usize = 3;
pub(crate) const FILE_AST_SAMPLE_LIMIT: usize = 1;
pub(crate) const DEFINITION_SAMPLE_LIMIT: usize = 2;
pub(crate) const USE_TREE_SAMPLE_LIMIT: usize = 2;
pub(crate) const ORACLE_SCOPE_SAMPLE_LIMIT: usize = 3;

#[derive(Debug, Copy, Clone, Default)]
pub(crate) struct RawLaneOmitted;

impl Serialize for RawLaneOmitted {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(false)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FileParseStatus {
    Ok,
    Error,
    Missing,
}

impl FileParseStatus {
    pub(crate) fn is_ok(self) -> bool {
        self == Self::Ok
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CoverageRunStatus {
    Ran,
    Unavailable,
    Missing,
}

impl CoverageRunStatus {
    pub(crate) fn from_coverage_status(status: Option<CoverageStatus>) -> Self {
        match status {
            Some(CoverageStatus::Ran) => Self::Ran,
            Some(CoverageStatus::Unavailable) => Self::Unavailable,
            None => Self::Missing,
        }
    }

    pub(crate) fn is_ran(self) -> bool {
        self == Self::Ran
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(crate) enum OracleBridgeStatus {
    #[serde(rename = "oracle-covered")]
    Covered,
    #[serde(rename = "oracle-partial")]
    Partial,
    #[serde(rename = "oracle-unavailable")]
    Unavailable,
    #[serde(rename = "oracle-missing")]
    Missing,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OracleConfidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CalibrationStatus {
    Pending,
    Measured,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ActionTier {
    SafeFix,
    ReviewFix,
    Degraded,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DegradedReason {
    SemanticCandidateFinding,
    CoverageUnavailableEntry,
}

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct SafeActionCandidate<'a> {
    pub(crate) file: Cow<'a, str>,
    pub(crate) diagnostic_code: Option<&'a str>,
    pub(crate) line_start: Option<i64>,
}

impl<'a> SafeActionCandidate<'a> {
    pub(crate) fn new(
        file: &'a str,
        diagnostic_code: Option<&'a str>,
        line_start: Option<i64>,
    ) -> Self {
        Self {
            file: normalize_candidate_file(file),
            diagnostic_code,
            line_start,
        }
    }
}

pub(crate) fn normalize_candidate_file(path: &str) -> Cow<'_, str> {
    posix_path_text(path)
}
