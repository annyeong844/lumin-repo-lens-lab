use crate::calibration::CalibrationAdjudication;

use super::super::super::super::super::projection::{
    OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationSeverity,
};
use super::super::reason;

pub(super) fn push_adjudication_reasons(
    adjudication: &CalibrationAdjudication,
    reasons: &mut Vec<OracleBridgeCalibrationReason>,
) {
    push_schema_reasons(adjudication, reasons);
    push_corpus_reasons(adjudication, reasons);
    push_unresolved_finding_reason(adjudication, reasons);
}

fn push_schema_reasons(
    adjudication: &CalibrationAdjudication,
    reasons: &mut Vec<OracleBridgeCalibrationReason>,
) {
    match adjudication.schema_round_trip() {
        Some(schema_round_trip) => {
            if !schema_round_trip.attempted() {
                reasons.push(reason(
                    OracleBridgeCalibrationReasonCode::SchemaRoundtripNotAttempted,
                    OracleBridgeCalibrationSeverity::Red,
                    "P3/P5 schema round-trip was not attempted",
                ));
            }
            if schema_round_trip.has_known_schema_drift_bugs() {
                reasons.push(reason(
                    OracleBridgeCalibrationReasonCode::SchemaDriftKnown,
                    OracleBridgeCalibrationSeverity::Red,
                    "known P3/P5 schema drift bug present",
                ));
            }
        }
        None => reasons.push(reason(
            OracleBridgeCalibrationReasonCode::SchemaRoundtripNotAttempted,
            OracleBridgeCalibrationSeverity::Red,
            "P3/P5 schema round-trip was not attempted",
        )),
    }
}

fn push_corpus_reasons(
    adjudication: &CalibrationAdjudication,
    reasons: &mut Vec<OracleBridgeCalibrationReason>,
) {
    for corpus in adjudication.corpus() {
        if !corpus.has_immutable_identity() {
            reasons.push(reason(
                OracleBridgeCalibrationReasonCode::CorpusIdentityMissing,
                OracleBridgeCalibrationSeverity::Red,
                format!("{} lacks commit/snapshotId", corpus.display_name()),
            ));
        }
        if !corpus.dirty_state_known() {
            reasons.push(reason(
                OracleBridgeCalibrationReasonCode::DirtyWorktreeUnknown,
                OracleBridgeCalibrationSeverity::Red,
                format!("{} dirty state unknown", corpus.display_name()),
            ));
        } else if !corpus.dirty_state_captured() {
            reasons.push(reason(
                OracleBridgeCalibrationReasonCode::DirtyWorktreeWithoutSnapshot,
                OracleBridgeCalibrationSeverity::Red,
                format!(
                    "{} dirty state lacks snapshot/contentHash",
                    corpus.display_name()
                ),
            ));
        }
    }
}

fn push_unresolved_finding_reason(
    adjudication: &CalibrationAdjudication,
    reasons: &mut Vec<OracleBridgeCalibrationReason>,
) {
    if adjudication.unresolved_high_findings() > 0 {
        reasons.push(reason(
            OracleBridgeCalibrationReasonCode::UnresolvedHighFinding,
            OracleBridgeCalibrationSeverity::Red,
            format!(
                "{} unresolved HIGH finding(s)",
                adjudication.unresolved_high_findings()
            ),
        ));
    }
}
