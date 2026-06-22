mod benchmark;
mod matching;
mod policy;
mod stats;

use crate::calibration::CalibrationAdjudication;
use crate::policy::{ActionPolicyTier, CleanupCandidate};

use super::super::projection::{
    OracleBridgeCalibrationCandidateCounts, OracleBridgeCalibrationGate,
    OracleBridgeCalibrationReadiness, OracleBridgeCalibrationReadinessPolicy,
    OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationSeverity,
};

use self::benchmark::{
    corpus_has_unknown_fp_denominator, enough_corpus, every_corpus_has_enough_adjudication,
};
use self::matching::{adjudication_is_in_scope, matched_adjudication_entries};
use self::policy::{
    REVIEW_VISIBLE_FP_GREEN_THRESHOLD, REVIEW_VISIBLE_FP_RED_THRESHOLD, SAFE_FIX_FP_RED_THRESHOLD,
};
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
    let mut reasons = Vec::new();
    let safe_needs_adjudication = candidate_counts.safe_fix > 0 && safe_fix.fp_rate.is_none();
    let review_needs_adjudication =
        candidate_counts.review_visible_cleanup > 0 && review_visible_cleanup.fp_rate.is_none();
    if has_readiness_evidence
        && calibration_adjudication
            .is_some_and(|adjudication| !adjudication.candidate_counts().is_available())
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::CandidateCountsUnavailable,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "fix-plan.json missing or candidate counts unavailable".into(),
        });
    }
    if entries.is_empty()
        || safe_needs_adjudication
        || review_needs_adjudication
        || corpus_has_unknown_fp_denominator(calibration_adjudication, entries)
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::FpRateUnknown,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "adjudication denominator is empty or incomplete".into(),
        });
    }
    if !has_readiness_evidence
        && !entries.is_empty()
        && matched_entries == 0
        && candidate_counts.review_visible_cleanup > 0
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::AdjudicationCandidateMismatch,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "adjudication entries did not match observed Rust cleanup candidates".into(),
        });
    }
    if let Some(fp_rate) = safe_fix
        .fp_rate
        .filter(|fp_rate| *fp_rate >= SAFE_FIX_FP_RED_THRESHOLD)
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::SafeFixFpThreshold,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: format!("SAFE_FIX FP rate {fp_rate}").into(),
        });
    }
    if let Some(fp_rate) = review_visible_cleanup
        .fp_rate
        .filter(|fp_rate| *fp_rate > REVIEW_VISIBLE_FP_RED_THRESHOLD)
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::ReviewVisibleFpThreshold,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: format!("review-visible cleanup FP rate {fp_rate}").into(),
        });
    }
    if let Some(adjudication) = calibration_adjudication.filter(|_| has_readiness_evidence) {
        match adjudication.schema_round_trip() {
            Some(schema_round_trip) => {
                if !schema_round_trip.attempted() {
                    reasons.push(OracleBridgeCalibrationReason {
                        code: OracleBridgeCalibrationReasonCode::SchemaRoundtripNotAttempted,
                        severity: OracleBridgeCalibrationSeverity::Red,
                        detail: "P3/P5 schema round-trip was not attempted".into(),
                    });
                }
                if schema_round_trip.has_known_schema_drift_bugs() {
                    reasons.push(OracleBridgeCalibrationReason {
                        code: OracleBridgeCalibrationReasonCode::SchemaDriftKnown,
                        severity: OracleBridgeCalibrationSeverity::Red,
                        detail: "known P3/P5 schema drift bug present".into(),
                    });
                }
            }
            None => reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::SchemaRoundtripNotAttempted,
                severity: OracleBridgeCalibrationSeverity::Red,
                detail: "P3/P5 schema round-trip was not attempted".into(),
            }),
        }
        for corpus in adjudication.corpus() {
            if !corpus.has_immutable_identity() {
                reasons.push(OracleBridgeCalibrationReason {
                    code: OracleBridgeCalibrationReasonCode::CorpusIdentityMissing,
                    severity: OracleBridgeCalibrationSeverity::Red,
                    detail: format!("{} lacks commit/snapshotId", corpus.display_name()).into(),
                });
            }
            if !corpus.dirty_state_known() {
                reasons.push(OracleBridgeCalibrationReason {
                    code: OracleBridgeCalibrationReasonCode::DirtyWorktreeUnknown,
                    severity: OracleBridgeCalibrationSeverity::Red,
                    detail: format!("{} dirty state unknown", corpus.display_name()).into(),
                });
            } else if !corpus.dirty_state_captured() {
                reasons.push(OracleBridgeCalibrationReason {
                    code: OracleBridgeCalibrationReasonCode::DirtyWorktreeWithoutSnapshot,
                    severity: OracleBridgeCalibrationSeverity::Red,
                    detail: format!(
                        "{} dirty state lacks snapshot/contentHash",
                        corpus.display_name()
                    )
                    .into(),
                });
            }
        }
        if adjudication.unresolved_high_findings() > 0 {
            reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::UnresolvedHighFinding,
                severity: OracleBridgeCalibrationSeverity::Red,
                detail: format!(
                    "{} unresolved HIGH finding(s)",
                    adjudication.unresolved_high_findings()
                )
                .into(),
            });
        }
    }

    let gate = if reasons
        .iter()
        .any(|reason| reason.severity == OracleBridgeCalibrationSeverity::Red)
    {
        OracleBridgeCalibrationGate::Red
    } else {
        let enough_corpus = calibration_adjudication.is_some_and(enough_corpus);
        let enough_adjudication = calibration_adjudication.is_some_and(|adjudication| {
            every_corpus_has_enough_adjudication(adjudication, entries)
        });
        if candidate_counts.safe_fix == 0 {
            reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::SafeFixPopulationEmpty,
                severity: OracleBridgeCalibrationSeverity::Yellow,
                detail: "SAFE_FIX population is measured zero; autonomous cleanup precision is not measured"
                    .into(),
            });
        }
        if !enough_corpus || !enough_adjudication {
            reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::BenchmarkIncomplete,
                severity: OracleBridgeCalibrationSeverity::Yellow,
                detail: "Green corpus/adjudication thresholds not met".into(),
            });
        }
        let green = candidate_counts.safe_fix > 0
            && safe_fix
                .fp_rate
                .is_some_and(|fp_rate| fp_rate < SAFE_FIX_FP_RED_THRESHOLD)
            && review_visible_cleanup
                .fp_rate
                .is_some_and(|fp_rate| fp_rate < REVIEW_VISIBLE_FP_GREEN_THRESHOLD)
            && enough_corpus
            && enough_adjudication;
        if green {
            OracleBridgeCalibrationGate::Green
        } else {
            OracleBridgeCalibrationGate::Yellow
        }
    };

    OracleBridgeCalibrationReadiness {
        gate,
        reasons,
        safe_fix,
        review_visible_cleanup,
    }
}
