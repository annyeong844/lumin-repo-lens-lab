use super::super::super::projection::{
    OracleBridgeCalibrationPrecedentRef, OracleBridgeCalibrationReadinessPolicy,
};

// Mirrors _lib/p6-measurement.mjs::computeReadiness. Change the JS/TS owner first.
pub(super) const SAFE_FIX_FP_RED_THRESHOLD: f64 = 0.05;
pub(super) const REVIEW_VISIBLE_FP_RED_THRESHOLD: f64 = 0.25;
pub(super) const REVIEW_VISIBLE_FP_GREEN_THRESHOLD: f64 = 0.10;
pub(super) const MIN_NON_TRIVIAL_CORPUS: usize = 2;
pub(super) const DEFAULT_MIN_ADJUDICATED_PER_CORPUS: usize = 50;

pub(super) fn readiness_policy() -> OracleBridgeCalibrationReadinessPolicy {
    OracleBridgeCalibrationReadinessPolicy {
        source: OracleBridgeCalibrationPrecedentRef::ReadinessGateOwner,
        safe_fix_fp_red_threshold: SAFE_FIX_FP_RED_THRESHOLD,
        review_visible_fp_red_threshold: REVIEW_VISIBLE_FP_RED_THRESHOLD,
        review_visible_fp_green_threshold: REVIEW_VISIBLE_FP_GREEN_THRESHOLD,
        min_non_trivial_corpus: MIN_NON_TRIVIAL_CORPUS,
        default_min_adjudicated_per_corpus: DEFAULT_MIN_ADJUDICATED_PER_CORPUS,
    }
}
