mod bridge;
mod support;
mod taint;

use lumin_rust_cargo_oracle::protocol::{
    ActionBlockerReason, ClaimKind, ConfidenceTier, CoverageId, Finding, FindingConfidence,
    OracleId, PrimarySpan, SafeAction, SafeActionEdit, SafeActionKind, SafeActionProof,
};
use lumin_rust_source_health::protocol::FileHealth;
use serde::Serialize;

use super::{
    evidence::{CoverageEvidence, SupportEvidence, TaintEvidence},
    semantic, span_overlap, ActionTier, FileParseStatus, OracleConfidence,
    ProductFileSemanticSummary, ProductPrimarySpanProjection, ProductSyntaxFileSummary,
};
use bridge::FindingOracleBridgeProjection;
use support::finding_support_evidence;
use taint::finding_taint_evidence;

const FINDING_ORACLE_BRIDGE_SCHEMA_VERSION: &str = "rust-finding-oracle-bridge.v1";
const FINDING_SPAN_SAMPLE_LIMIT: usize = 3;

pub(crate) fn product_semantic_finding<'a>(
    finding: &'a Finding,
    syntax_file: Option<&'a FileHealth>,
    coverage: &CoverageEvidence<'_>,
) -> ProductSemanticFinding<'a> {
    let syntax_summary = syntax_file
        .map(ProductSyntaxFileSummary::from_file)
        .unwrap_or_else(ProductSyntaxFileSummary::missing);
    let parse_status = syntax_summary.parse_status();
    let parse_errors = syntax_summary.parse_errors();
    let action = semantic::finding_action(finding);
    let action_tier = action.tier;
    let has_safe_action = action.has_safe_action;
    let has_action_blockers = !action.action_blockers.is_empty();
    let is_review = action.is_review;
    let action_blockers = action.action_blockers;
    let is_candidate = action.is_candidate;
    let summary = ProductFileSemanticSummary::from_finding_action(
        has_safe_action,
        has_action_blockers,
        is_review,
        is_candidate,
    );
    let local_review_opaque_surfaces = span_overlap::review_opaque_surfaces_touching_span(
        syntax_file.map(|file| &file.ast),
        finding.span.as_ref(),
        &finding.primary_spans,
    );
    let supported_by = finding_support_evidence(
        finding,
        syntax_summary.is_present(),
        parse_status,
        parse_errors,
        coverage,
        has_safe_action,
    );
    let tainted_by = finding_taint_evidence(
        parse_status,
        parse_errors,
        has_safe_action,
        is_candidate,
        &local_review_opaque_surfaces,
        &action_blockers,
        coverage,
    );

    let oracle_confidence = semantic::finding_oracle_confidence(&tainted_by);

    ProductSemanticFinding {
        summary,
        projection: ProductSemanticFindingProjection {
            oracle_id: finding.oracle_id,
            confidence: ProductFindingConfidenceProjection::from_confidence(&finding.confidence),
            diagnostic_code: finding.diagnostic_code.as_deref(),
            message: finding.message.as_deref(),
            span: finding
                .span
                .as_ref()
                .map(ProductPrimarySpanProjection::from_span),
            primary_span_count: finding.primary_spans.len(),
            macro_expansion_span_count: macro_expansion_span_count(&finding.primary_spans),
            macro_expansion_span_examples: macro_expansion_span_examples(&finding.primary_spans),
            coverage_ref: finding.coverage_ref,
            action_blockers: &finding.action_blockers,
            safe_action: finding
                .safe_action
                .as_ref()
                .map(ProductSafeActionProjection::from_action),
            parse_status,
            supported_by,
            tainted_by,
            oracle_confidence,
            action_tier,
            oracle_bridge: FindingOracleBridgeProjection::new(
                parse_status,
                coverage,
                local_review_opaque_surfaces.len(),
            ),
        },
    }
}

pub(crate) struct ProductSemanticFinding<'a> {
    pub(crate) summary: ProductFileSemanticSummary,
    pub(crate) projection: ProductSemanticFindingProjection<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProductSemanticFindingProjection<'a> {
    oracle_id: OracleId,
    confidence: ProductFindingConfidenceProjection<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostic_code: Option<&'a str>,
    message: Option<&'a str>,
    span: Option<ProductPrimarySpanProjection<'a>>,
    primary_span_count: usize,
    macro_expansion_span_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    macro_expansion_span_examples: Vec<ProductPrimarySpanProjection<'a>>,
    coverage_ref: CoverageId,
    action_blockers: &'a [ActionBlockerReason],
    #[serde(skip_serializing_if = "Option::is_none")]
    safe_action: Option<ProductSafeActionProjection<'a>>,
    parse_status: FileParseStatus,
    supported_by: Vec<SupportEvidence>,
    tainted_by: Vec<TaintEvidence<'a>>,
    oracle_confidence: OracleConfidence,
    action_tier: ActionTier,
    oracle_bridge: FindingOracleBridgeProjection,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProductFindingConfidenceProjection<'a> {
    tier: ConfidenceTier,
    authority_ids: &'a [&'static str],
    rule_ids: &'a [&'static str],
    claim_kind: ClaimKind,
}

impl<'a> ProductFindingConfidenceProjection<'a> {
    fn from_confidence(confidence: &'a FindingConfidence) -> Self {
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
struct ProductSafeActionProjection<'a> {
    kind: SafeActionKind,
    proof_complete: bool,
    edit_count: usize,
    edits: &'a [SafeActionEdit],
    proof: &'a SafeActionProof,
}

impl<'a> ProductSafeActionProjection<'a> {
    fn from_action(action: &'a SafeAction) -> Self {
        Self {
            kind: action.kind,
            proof_complete: action.proof_complete,
            edit_count: action.edits.len(),
            edits: &action.edits,
            proof: &action.proof,
        }
    }
}

fn macro_expansion_span_count(spans: &[PrimarySpan]) -> usize {
    spans.iter().filter(|span| span.has_expansion).count()
}

fn macro_expansion_span_examples(spans: &[PrimarySpan]) -> Vec<ProductPrimarySpanProjection<'_>> {
    spans
        .iter()
        .filter(|span| span.has_expansion)
        .map(ProductPrimarySpanProjection::from_span)
        .take(FINDING_SPAN_SAMPLE_LIMIT)
        .collect()
}
