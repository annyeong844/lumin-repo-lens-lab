mod confidence;
mod projection;
mod semantic_summary;
mod support;
mod taint;

use super::{
    bridge::OracleBridge, evidence::CoverageEvidence, OracleConfidence, ProductSyntaxFileSummary,
};
use confidence::file_oracle_confidence;
pub(crate) use projection::ProductFileOracleBridgeProjection;
use projection::{FileBridgeSemanticProjection, FileBridgeSyntaxProjection};
pub(crate) use semantic_summary::ProductFileSemanticSummary;
use support::file_bridge_support_evidence;
use taint::file_bridge_taint_evidence;

pub(crate) fn product_file_oracle_bridge(
    syntax_summary: ProductSyntaxFileSummary,
    semantic_summary: ProductFileSemanticSummary,
    oracle_bridge: &OracleBridge<'_>,
    coverage: &CoverageEvidence<'_>,
) -> ProductFileOracleBridgeProjection {
    let parse_status = syntax_summary.parse_status();
    let parse_errors = syntax_summary.parse_errors();
    let review_signals = syntax_summary.review_signals();
    let muted_signals = syntax_summary.muted_signals();
    let review_opaque_surfaces = syntax_summary.review_opaque_surfaces();
    let muted_opaque_surfaces = syntax_summary.muted_opaque_surfaces();
    let bridge_status = oracle_bridge.status();

    let supported_by = file_bridge_support_evidence(syntax_summary, semantic_summary);
    let tainted_by = file_bridge_taint_evidence(parse_status, parse_errors, review_opaque_surfaces);
    let oracle_confidence = file_oracle_confidence(
        parse_status,
        review_opaque_surfaces,
        bridge_status,
        coverage.cargo_event_status(),
        coverage.absence_status(),
    );

    let syntax = FileBridgeSyntaxProjection::new(
        parse_errors,
        review_signals,
        muted_signals,
        review_opaque_surfaces,
        muted_opaque_surfaces,
    );
    let semantic = FileBridgeSemanticProjection::from_summary(semantic_summary);
    let has_local_bridge_evidence = !supported_by.is_empty()
        || !tainted_by.is_empty()
        || !parse_status.is_ok()
        || !syntax.is_empty()
        || !semantic.is_empty();

    ProductFileOracleBridgeProjection {
        parse_status: (!parse_status.is_ok()).then_some(parse_status),
        oracle_confidence: (oracle_confidence == OracleConfidence::High
            || has_local_bridge_evidence)
            .then_some(oracle_confidence),
        supported_by,
        tainted_by,
        syntax,
        semantic,
    }
}
