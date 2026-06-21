use lumin_rust_cargo_oracle::protocol::ActionBlockerReason;
use lumin_rust_source_health::protocol::AstOpaqueSurface;

use crate::policy::{
    evidence::{push_parse_taint, CoverageEvidence, TaintEffect, TaintEvidence},
    FileParseStatus, ACTION_SAMPLE_LIMIT,
};

pub(super) fn finding_taint_evidence<'a>(
    parse_status: FileParseStatus,
    parse_errors: usize,
    has_safe_action: bool,
    is_candidate: bool,
    local_review_opaque_surfaces: &[&'a AstOpaqueSurface],
    action_blockers: &[ActionBlockerReason],
    coverage: &CoverageEvidence<'_>,
) -> Vec<TaintEvidence<'a>> {
    let mut tainted_by = Vec::new();
    push_parse_taint(
        &mut tainted_by,
        parse_status,
        parse_errors,
        TaintEffect::FindingFileParseErrorsMayBeIncomplete,
        TaintEffect::FindingNoSyntaxProjection,
    );
    if !has_safe_action && !local_review_opaque_surfaces.is_empty() {
        tainted_by.push(TaintEvidence::rust_ast_review_opaque_surface_near_finding(
            local_review_opaque_surfaces.len(),
            local_review_opaque_surfaces
                .iter()
                .take(ACTION_SAMPLE_LIMIT)
                .copied()
                .collect::<Vec<_>>(),
            TaintEffect::FindingOverlapsReviewOpaqueSurface,
        ));
    }
    if !action_blockers.is_empty() {
        tainted_by.push(TaintEvidence::semantic_action_blocker(
            action_blockers.to_vec(),
            TaintEffect::SafeFixBlockedByActionBlocker,
        ));
    }
    if is_candidate {
        tainted_by.push(TaintEvidence::semantic_candidate_finding(
            TaintEffect::CandidateDiagnosticNotVerified,
        ));
    }
    coverage.push_tainted_by(
        &mut tainted_by,
        TaintEffect::FindingCargoEventCoverageUnavailable,
        TaintEffect::ScopeAbsenceCleanUnavailable,
    );
    tainted_by
}
