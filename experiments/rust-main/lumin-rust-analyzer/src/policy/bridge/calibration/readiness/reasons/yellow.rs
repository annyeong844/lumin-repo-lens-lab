use super::super::super::super::projection::{
    OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationSeverity,
};
use super::super::policy::{REVIEW_VISIBLE_FP_GREEN_THRESHOLD, SAFE_FIX_FP_RED_THRESHOLD};
use super::{reason, ReadinessGateInput};

pub(super) fn yellow_reasons(
    input: &ReadinessGateInput<'_>,
    enough_corpus: bool,
    enough_adjudication: bool,
) -> Vec<OracleBridgeCalibrationReason> {
    let mut reasons = Vec::new();
    if input.candidate_counts.safe_fix == 0 {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::SafeFixPopulationEmpty,
            OracleBridgeCalibrationSeverity::Yellow,
            "SAFE_FIX population is measured zero; autonomous cleanup precision is not measured",
        ));
    }
    if !enough_corpus || !enough_adjudication {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::BenchmarkIncomplete,
            OracleBridgeCalibrationSeverity::Yellow,
            "Green corpus/adjudication thresholds not met",
        ));
    }
    reasons
}

pub(super) fn is_green(
    input: &ReadinessGateInput<'_>,
    enough_corpus: bool,
    enough_adjudication: bool,
) -> bool {
    input.candidate_counts.safe_fix > 0
        && input
            .safe_fix
            .fp_rate
            .is_some_and(|fp_rate| fp_rate < SAFE_FIX_FP_RED_THRESHOLD)
        && input
            .review_visible_cleanup
            .fp_rate
            .is_some_and(|fp_rate| fp_rate < REVIEW_VISIBLE_FP_GREEN_THRESHOLD)
        && enough_corpus
        && enough_adjudication
}
