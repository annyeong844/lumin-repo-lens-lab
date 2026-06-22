use crate::calibration::CalibrationAdjudicationEntry;
use crate::policy::{normalize_candidate_file, ActionPolicyTier, CleanupCandidate};

pub(super) fn matched_adjudication_entries(
    entries: &[CalibrationAdjudicationEntry],
    cleanup_candidates: &[CleanupCandidate<'_>],
    has_readiness_evidence: bool,
) -> usize {
    entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.tier,
                Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
            ) && adjudication_is_in_scope(entry, cleanup_candidates, has_readiness_evidence)
        })
        .count()
}

pub(super) fn adjudication_is_in_scope(
    entry: &CalibrationAdjudicationEntry,
    cleanup_candidates: &[CleanupCandidate<'_>],
    has_readiness_evidence: bool,
) -> bool {
    has_readiness_evidence || adjudication_matches_candidate(entry, cleanup_candidates)
}

fn adjudication_matches_candidate(
    entry: &CalibrationAdjudicationEntry,
    cleanup_candidates: &[CleanupCandidate<'_>],
) -> bool {
    entry.file.as_ref().is_some_and(|file| {
        let file = normalize_candidate_file(file);
        cleanup_candidates.iter().any(|candidate| {
            candidate.file.as_ref() == file.as_ref()
                && entry
                    .diagnostic_code
                    .as_ref()
                    .is_none_or(|code| candidate.diagnostic_code == Some(code.as_str()))
                && entry
                    .line_start
                    .is_none_or(|line_start| candidate.line_start == Some(line_start))
        })
    })
}
