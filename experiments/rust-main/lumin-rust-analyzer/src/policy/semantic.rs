use std::borrow::Cow;
use std::collections::BTreeMap;

use lumin_rust_cargo_oracle::protocol::{
    ActionBlockerReason, ClaimKind, ConfidenceTier as OracleConfidenceTier, Finding,
};

use super::{
    evidence::TaintEvidence, ActionTier, CleanupCandidate, DegradedReason, OracleConfidence,
};

pub(super) struct FindingActionSummary {
    pub(super) safe_actions: usize,
    pub(super) action_blockers: usize,
    pub(super) action_blockers_by_reason: BTreeMap<ActionBlockerReason, usize>,
    pub(super) review_findings: usize,
    pub(super) review_by_reason: BTreeMap<ClaimKind, usize>,
}

pub(super) struct FindingAction<'a> {
    pub(super) tier: ActionTier,
    pub(super) has_safe_action: bool,
    pub(super) action_blockers: Cow<'a, [ActionBlockerReason]>,
    pub(super) is_review: bool,
    pub(super) is_candidate: bool,
}

pub(super) struct FindingActionRecord<'a> {
    pub(super) finding: &'a Finding,
    pub(super) action: FindingAction<'a>,
}

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

pub(super) fn finding_action(finding: &Finding) -> FindingAction<'_> {
    let action_blockers = finding_action_blockers(finding);
    let has_safe_action = finding.safe_action.as_ref().is_some_and(|action| {
        action.proof_complete && action.action_blockers.is_empty() && action_blockers.is_empty()
    });
    let is_candidate = finding.confidence_tier == OracleConfidenceTier::Candidate;
    let is_review = !has_safe_action
        && action_blockers.is_empty()
        && matches!(
            finding.confidence_tier,
            OracleConfidenceTier::Verified | OracleConfidenceTier::RuleBacked
        );
    let tier = if has_safe_action {
        ActionTier::SafeFix
    } else if !action_blockers.is_empty() || is_review {
        ActionTier::ReviewFix
    } else if is_candidate {
        ActionTier::Degraded
    } else {
        ActionTier::ReviewFix
    };
    FindingAction {
        tier,
        has_safe_action,
        action_blockers,
        is_review,
        is_candidate,
    }
}

pub(super) fn finding_actions(findings: &[Finding]) -> Vec<FindingActionRecord<'_>> {
    findings
        .iter()
        .map(|finding| FindingActionRecord {
            finding,
            action: finding_action(finding),
        })
        .collect()
}

fn finding_action_blockers(finding: &Finding) -> Cow<'_, [ActionBlockerReason]> {
    let safe_action_blockers = finding
        .safe_action
        .as_ref()
        .map(|action| action.action_blockers.as_slice())
        .unwrap_or_default();
    normalize_action_blockers(&finding.action_blockers, safe_action_blockers)
}

fn normalize_action_blockers<'a>(
    finding_blockers: &'a [ActionBlockerReason],
    safe_action_blockers: &'a [ActionBlockerReason],
) -> Cow<'a, [ActionBlockerReason]> {
    match (finding_blockers.is_empty(), safe_action_blockers.is_empty()) {
        (true, true) => Cow::Borrowed(&[]),
        (false, true) if is_sorted_unique(finding_blockers) => Cow::Borrowed(finding_blockers),
        (true, false) if is_sorted_unique(safe_action_blockers) => {
            Cow::Borrowed(safe_action_blockers)
        }
        _ => {
            let mut blockers =
                Vec::with_capacity(finding_blockers.len() + safe_action_blockers.len());
            blockers.extend(finding_blockers.iter().copied());
            blockers.extend(safe_action_blockers.iter().copied());
            blockers.sort_unstable();
            blockers.dedup();
            Cow::Owned(blockers)
        }
    }
}

fn is_sorted_unique(blockers: &[ActionBlockerReason]) -> bool {
    blockers.windows(2).all(|pair| pair[0] < pair[1])
}

pub(super) fn finding_action_summary(records: &[FindingActionRecord<'_>]) -> FindingActionSummary {
    let mut summary = FindingActionSummary {
        safe_actions: 0,
        action_blockers: 0,
        action_blockers_by_reason: BTreeMap::new(),
        review_findings: 0,
        review_by_reason: BTreeMap::new(),
    };
    for record in records {
        let action = &record.action;
        if action.has_safe_action {
            summary.safe_actions += 1;
        } else if !action.action_blockers.is_empty() {
            summary.action_blockers += 1;
            for blocker in action.action_blockers.iter().copied() {
                *summary
                    .action_blockers_by_reason
                    .entry(blocker)
                    .or_insert(0usize) += 1;
            }
        } else if action.is_review {
            summary.review_findings += 1;
            *summary
                .review_by_reason
                .entry(record.finding.claim_kind)
                .or_insert(0usize) += 1;
        }
    }
    summary
}

pub(super) fn cleanup_candidates<'a>(
    records: &[FindingActionRecord<'a>],
) -> Vec<CleanupCandidate<'a>> {
    records
        .iter()
        .filter(|record| {
            record.action.has_safe_action
                || !record.action.action_blockers.is_empty()
                || record.action.is_review
        })
        .filter_map(cleanup_candidate)
        .collect()
}

fn cleanup_candidate<'a>(record: &FindingActionRecord<'a>) -> Option<CleanupCandidate<'a>> {
    let finding = record.finding;
    let action = finding.safe_action.as_ref();
    let edit = action.and_then(|action| action.edits.first());
    let file = edit
        .map(|edit| edit.file_name.as_str())
        .or_else(|| finding.span.as_ref()?.file_name.as_deref())?;
    let line_start = edit
        .map(|edit| edit.line_start)
        .or_else(|| finding.span.as_ref().and_then(|span| span.line_start));
    Some(CleanupCandidate::new(
        file,
        action.map(|action| action.proof.diagnostic_code.as_str()),
        line_start,
    ))
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
