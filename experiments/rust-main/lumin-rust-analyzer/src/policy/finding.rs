mod bridge;
mod projection;
mod support;
mod taint;

use super::{
    evidence::CoverageEvidence, semantic, span_overlap, ProductFileSemanticSummary,
    ProductPrimarySpanProjection, ProductSyntaxFileSummary,
};
use crate::syntax_phase::SyntaxFile;
use bridge::FindingOracleBridgeProjection;
use lumin_rust_cargo_oracle::protocol::Finding;
pub(crate) use projection::ProductSemanticFindingProjection;
use projection::{
    macro_expansion_span_count, macro_expansion_span_examples, ProductFindingConfidenceProjection,
    ProductSafeActionProjection,
};
use support::finding_support_evidence;
use taint::finding_taint_evidence;

const FINDING_ORACLE_BRIDGE_SCHEMA_VERSION: &str = "rust-finding-oracle-bridge.v1";

pub(crate) fn product_semantic_finding<'a>(
    finding: &'a Finding,
    syntax_file: Option<SyntaxFile<'a>>,
    coverage: &CoverageEvidence<'_>,
) -> ProductSemanticFinding<'a> {
    let syntax_summary = syntax_summary_from_file(syntax_file);
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
        syntax_file.and_then(|file| match file {
            SyntaxFile::Full(file) => Some(&file.ast),
            SyntaxFile::Compact(_) => None,
        }),
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

fn syntax_summary_from_file(file: Option<SyntaxFile<'_>>) -> ProductSyntaxFileSummary {
    match file {
        Some(SyntaxFile::Full(file)) => ProductSyntaxFileSummary::from_file(file),
        Some(SyntaxFile::Compact(file)) => ProductSyntaxFileSummary::from_compact_file(file),
        None => ProductSyntaxFileSummary::missing(),
    }
}

pub(crate) struct ProductSemanticFinding<'a> {
    pub(crate) summary: ProductFileSemanticSummary,
    pub(crate) projection: ProductSemanticFindingProjection<'a>,
}
