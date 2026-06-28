use lumin_rust_cargo_oracle::protocol::{
    ActionBlockerReason, ClaimKind, ConfidenceTier, CoverageId, FindingConfidence, OracleId,
    PrimarySpan, SafeAction, SafeActionEdit, SafeActionKind, SafeActionProof,
};
use serde::Serialize;

use crate::policy::{
    ActionTier, FileParseStatus, OracleConfidence, ProductPrimarySpanProjection,
    SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT,
};

use super::bridge::FindingOracleBridgeProjection;
use crate::policy::evidence::{SupportEvidence, TaintEvidence};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductSemanticFindingProjection<'a> {
    pub(super) oracle_id: OracleId,
    pub(super) confidence: ProductFindingConfidenceProjection<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) diagnostic_code: Option<&'a str>,
    pub(super) message: Option<&'a str>,
    pub(super) span: Option<ProductPrimarySpanProjection<'a>>,
    pub(super) primary_span_count: usize,
    pub(super) macro_expansion_span_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) macro_expansion_span_examples: Vec<ProductPrimarySpanProjection<'a>>,
    pub(super) coverage_ref: CoverageId,
    pub(super) action_blockers: &'a [ActionBlockerReason],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) safe_action: Option<ProductSafeActionProjection<'a>>,
    pub(super) parse_status: FileParseStatus,
    pub(super) supported_by: Vec<SupportEvidence>,
    pub(super) tainted_by: Vec<TaintEvidence<'a>>,
    pub(super) oracle_confidence: OracleConfidence,
    pub(super) action_tier: ActionTier,
    pub(super) oracle_bridge: FindingOracleBridgeProjection,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductFindingConfidenceProjection<'a> {
    tier: ConfidenceTier,
    authority_ids: &'a [&'static str],
    rule_ids: &'a [&'static str],
    claim_kind: ClaimKind,
}

impl<'a> ProductFindingConfidenceProjection<'a> {
    pub(super) fn from_confidence(confidence: &'a FindingConfidence) -> Self {
        Self {
            tier: confidence.tier,
            authority_ids: confidence.authority_ids.as_slice(),
            rule_ids: confidence.rule_ids.as_slice(),
            claim_kind: confidence.claim_kind,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductSafeActionProjection<'a> {
    kind: SafeActionKind,
    proof_complete: bool,
    edit_count: usize,
    edits: &'a [SafeActionEdit],
    proof: &'a SafeActionProof,
}

impl<'a> ProductSafeActionProjection<'a> {
    pub(super) fn from_action(action: &'a SafeAction) -> Self {
        Self {
            kind: action.kind,
            proof_complete: action.proof_complete,
            edit_count: action.edits.len(),
            edits: &action.edits,
            proof: &action.proof,
        }
    }
}

pub(super) fn macro_expansion_span_count(spans: &[PrimarySpan]) -> usize {
    spans.iter().filter(|span| span.has_expansion).count()
}

pub(super) fn macro_expansion_span_examples(
    spans: &[PrimarySpan],
) -> Vec<ProductPrimarySpanProjection<'_>> {
    spans
        .iter()
        .filter(|span| span.has_expansion)
        .map(ProductPrimarySpanProjection::from_span)
        .take(SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT)
        .collect()
}
