use std::borrow::Cow;

use crate::calibration::{CalibrationAdjudication, CalibrationAdjudicationEntry};

use super::super::super::projection::{
    OracleBridgeCalibrationAdjudicationStats, OracleBridgeCalibrationCandidateCounts,
    OracleBridgeCalibrationGate, OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationSeverity,
};
use super::benchmark::{enough_corpus, every_corpus_has_enough_adjudication};
use red::red_reasons;
use yellow::{is_green, yellow_reasons};

mod red;
mod yellow;

pub(super) struct ReadinessGateInput<'a> {
    pub(super) candidate_counts: OracleBridgeCalibrationCandidateCounts,
    pub(super) calibration_adjudication: Option<&'a CalibrationAdjudication>,
    pub(super) entries: &'a [CalibrationAdjudicationEntry],
    pub(super) matched_entries: usize,
    pub(super) has_readiness_evidence: bool,
    pub(super) safe_fix: OracleBridgeCalibrationAdjudicationStats,
    pub(super) review_visible_cleanup: OracleBridgeCalibrationAdjudicationStats,
}

pub(super) fn gate_and_reasons(
    input: ReadinessGateInput<'_>,
) -> (
    OracleBridgeCalibrationGate,
    Vec<OracleBridgeCalibrationReason>,
) {
    let mut reasons = red_reasons(&input);
    if reasons
        .iter()
        .any(|reason| reason.severity == OracleBridgeCalibrationSeverity::Red)
    {
        return (OracleBridgeCalibrationGate::Red, reasons);
    }
    let enough_corpus = input.calibration_adjudication.is_some_and(enough_corpus);
    let enough_adjudication = input.calibration_adjudication.is_some_and(|adjudication| {
        every_corpus_has_enough_adjudication(adjudication, input.entries)
    });
    reasons.extend(yellow_reasons(&input, enough_corpus, enough_adjudication));
    if is_green(&input, enough_corpus, enough_adjudication) {
        (OracleBridgeCalibrationGate::Green, reasons)
    } else {
        (OracleBridgeCalibrationGate::Yellow, reasons)
    }
}

fn reason(
    code: OracleBridgeCalibrationReasonCode,
    severity: OracleBridgeCalibrationSeverity,
    detail: impl Into<Cow<'static, str>>,
) -> OracleBridgeCalibrationReason {
    OracleBridgeCalibrationReason {
        code,
        severity,
        detail: detail.into(),
    }
}
