mod benchmark;
mod matching;
mod policy;
mod reasons;
mod stats;

use crate::calibration::CalibrationAdjudication;
use crate::policy::{ActionPolicyTier, CleanupCandidate};

use super::super::projection::{
    OracleBridgeCalibrationCandidateCounts, OracleBridgeCalibrationReadiness,
    OracleBridgeCalibrationReadinessPolicy,
};

use self::matching::{adjudication_is_in_scope, matched_adjudication_entries};
use self::reasons::{gate_and_reasons, ReadinessGateInput};
use self::stats::summarize_adjudication;

pub(super) fn readiness_policy() -> OracleBridgeCalibrationReadinessPolicy {
    self::policy::readiness_policy()
}

pub(super) fn readiness(
    candidate_counts: OracleBridgeCalibrationCandidateCounts,
    cleanup_candidates: &[CleanupCandidate<'_>],
    calibration_adjudication: Option<&CalibrationAdjudication>,
) -> OracleBridgeCalibrationReadiness {
    let entries = calibration_adjudication
        .map(CalibrationAdjudication::entries)
        .unwrap_or(&[]);
    let has_readiness_evidence =
        calibration_adjudication.is_some_and(CalibrationAdjudication::has_readiness_evidence);
    let matched_entries =
        matched_adjudication_entries(entries, cleanup_candidates, has_readiness_evidence);
    let safe_fix = summarize_adjudication(entries, |entry| {
        entry.tier == Some(ActionPolicyTier::SafeFix)
            && adjudication_is_in_scope(entry, cleanup_candidates, has_readiness_evidence)
    });
    let review_visible_cleanup = summarize_adjudication(entries, |entry| {
        matches!(
            entry.tier,
            Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
        ) && adjudication_is_in_scope(entry, cleanup_candidates, has_readiness_evidence)
    });
    let (gate, reasons) = gate_and_reasons(ReadinessGateInput {
        candidate_counts,
        calibration_adjudication,
        entries,
        matched_entries,
        has_readiness_evidence,
        safe_fix,
        review_visible_cleanup,
    });

    OracleBridgeCalibrationReadiness {
        gate,
        reasons,
        safe_fix,
        review_visible_cleanup,
    }
}
