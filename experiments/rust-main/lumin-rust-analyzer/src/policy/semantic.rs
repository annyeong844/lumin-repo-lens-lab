use std::collections::BTreeMap;

use super::{evidence::TaintEvidence, DegradedReason, OracleConfidence};

pub(super) use action::{
    finding_action, finding_action_summary, finding_actions, FindingActionRecord,
};
pub(super) use cleanup::cleanup_candidates;

mod action;
mod cleanup;

pub(super) fn finding_oracle_confidence(tainted_by: &[TaintEvidence<'_>]) -> OracleConfidence {
    if tainted_by
        .iter()
        .any(TaintEvidence::lowers_finding_confidence_to_low)
    {
        return OracleConfidence::Low;
    }
    if tainted_by.is_empty() {
        OracleConfidence::High
    } else {
        OracleConfidence::Medium
    }
}

pub(super) fn degraded_by_reason(
    candidate_findings: usize,
    unavailable_coverage_entries: usize,
) -> BTreeMap<DegradedReason, usize> {
    let mut by_reason = BTreeMap::new();
    if candidate_findings != 0 {
        by_reason.insert(DegradedReason::SemanticCandidateFinding, candidate_findings);
    }
    if unavailable_coverage_entries != 0 {
        by_reason.insert(
            DegradedReason::CoverageUnavailableEntry,
            unavailable_coverage_entries,
        );
    }
    by_reason
}
