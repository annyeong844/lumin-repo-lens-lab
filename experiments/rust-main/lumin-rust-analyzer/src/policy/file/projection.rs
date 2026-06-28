use serde::Serialize;

use crate::policy::{
    evidence::{SupportEvidence, TaintEvidence},
    FileParseStatus, OracleConfidence,
};

use super::ProductFileSemanticSummary;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductFileOracleBridgeProjection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) parse_status: Option<FileParseStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) oracle_confidence: Option<OracleConfidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) supported_by: Vec<SupportEvidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) tainted_by: Vec<TaintEvidence<'static>>,
    #[serde(skip_serializing_if = "FileBridgeSyntaxProjection::is_empty")]
    pub(super) syntax: FileBridgeSyntaxProjection,
    #[serde(skip_serializing_if = "FileBridgeSemanticProjection::is_empty")]
    pub(super) semantic: FileBridgeSemanticProjection,
}

impl ProductFileOracleBridgeProjection {
    pub(crate) fn is_empty(&self) -> bool {
        self.parse_status.is_none()
            && self.oracle_confidence.is_none()
            && self.supported_by.is_empty()
            && self.tainted_by.is_empty()
            && self.syntax.is_empty()
            && self.semantic.is_empty()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileBridgeSyntaxProjection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) parse_errors: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) review_signals: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) muted_signals: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) review_opaque_surfaces: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) muted_opaque_surfaces: Option<usize>,
}

impl FileBridgeSyntaxProjection {
    pub(super) fn new(
        parse_errors: usize,
        review_signals: usize,
        muted_signals: usize,
        review_opaque_surfaces: usize,
        muted_opaque_surfaces: usize,
    ) -> Self {
        let has_review_or_parse =
            parse_errors > 0 || review_signals > 0 || review_opaque_surfaces > 0;
        Self {
            parse_errors: (parse_errors > 0).then_some(parse_errors),
            review_signals: (review_signals > 0).then_some(review_signals),
            muted_signals: (has_review_or_parse && muted_signals > 0).then_some(muted_signals),
            review_opaque_surfaces: (review_opaque_surfaces > 0).then_some(review_opaque_surfaces),
            muted_opaque_surfaces: (has_review_or_parse && muted_opaque_surfaces > 0)
                .then_some(muted_opaque_surfaces),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.parse_errors.is_none()
            && self.review_signals.is_none()
            && self.muted_signals.is_none()
            && self.review_opaque_surfaces.is_none()
            && self.muted_opaque_surfaces.is_none()
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FileBridgeSemanticProjection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) findings: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) diagnostics: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) safe_actions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) action_blocked_findings: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) review_findings: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) candidate_findings: Option<usize>,
}

impl FileBridgeSemanticProjection {
    pub(super) fn from_summary(summary: ProductFileSemanticSummary) -> Self {
        Self {
            findings: (summary.findings() > 0).then_some(summary.findings()),
            diagnostics: (summary.diagnostics() > 0).then_some(summary.diagnostics()),
            safe_actions: (summary.safe_actions() > 0).then_some(summary.safe_actions()),
            action_blocked_findings: (summary.action_blocked_findings() > 0)
                .then_some(summary.action_blocked_findings()),
            review_findings: (summary.review_findings() > 0).then_some(summary.review_findings()),
            candidate_findings: (summary.candidate_findings() > 0)
                .then_some(summary.candidate_findings()),
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.findings.is_none()
            && self.diagnostics.is_none()
            && self.safe_actions.is_none()
            && self.action_blocked_findings.is_none()
            && self.review_findings.is_none()
            && self.candidate_findings.is_none()
    }
}
