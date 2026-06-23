use super::super::super::super::projection::{
    OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationSeverity,
};
use super::super::benchmark::corpus_has_unknown_fp_denominator;
use super::super::policy::{REVIEW_VISIBLE_FP_RED_THRESHOLD, SAFE_FIX_FP_RED_THRESHOLD};
use super::{reason, ReadinessGateInput};
use adjudication::push_adjudication_reasons;

mod adjudication;

pub(super) fn red_reasons(input: &ReadinessGateInput<'_>) -> Vec<OracleBridgeCalibrationReason> {
    let mut reasons = Vec::new();
    let safe_needs_adjudication =
        input.candidate_counts.safe_fix > 0 && input.safe_fix.fp_rate.is_none();
    let review_needs_adjudication = input.candidate_counts.review_visible_cleanup > 0
        && input.review_visible_cleanup.fp_rate.is_none();
    if input.has_readiness_evidence
        && input
            .calibration_adjudication
            .is_some_and(|adjudication| !adjudication.candidate_counts().is_available())
    {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::CandidateCountsUnavailable,
            OracleBridgeCalibrationSeverity::Red,
            "fix-plan.json missing or candidate counts unavailable",
        ));
    }
    if input.entries.is_empty()
        || safe_needs_adjudication
        || review_needs_adjudication
        || corpus_has_unknown_fp_denominator(input.calibration_adjudication, input.entries)
    {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::FpRateUnknown,
            OracleBridgeCalibrationSeverity::Red,
            "adjudication denominator is empty or incomplete",
        ));
    }
    if !input.has_readiness_evidence
        && !input.entries.is_empty()
        && input.matched_entries == 0
        && input.candidate_counts.review_visible_cleanup > 0
    {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::AdjudicationCandidateMismatch,
            OracleBridgeCalibrationSeverity::Red,
            "adjudication entries did not match observed Rust cleanup candidates",
        ));
    }
    if let Some(fp_rate) = input
        .safe_fix
        .fp_rate
        .filter(|fp_rate| *fp_rate >= SAFE_FIX_FP_RED_THRESHOLD)
    {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::SafeFixFpThreshold,
            OracleBridgeCalibrationSeverity::Red,
            format!("SAFE_FIX FP rate {fp_rate}"),
        ));
    }
    if let Some(fp_rate) = input
        .review_visible_cleanup
        .fp_rate
        .filter(|fp_rate| *fp_rate > REVIEW_VISIBLE_FP_RED_THRESHOLD)
    {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::ReviewVisibleFpThreshold,
            OracleBridgeCalibrationSeverity::Red,
            format!("review-visible cleanup FP rate {fp_rate}"),
        ));
    }
    if let Some(adjudication) = input
        .calibration_adjudication
        .filter(|_| input.has_readiness_evidence)
    {
        push_adjudication_reasons(adjudication, &mut reasons);
    }
    reasons
}
