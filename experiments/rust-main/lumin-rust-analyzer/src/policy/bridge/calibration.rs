use crate::calibration::{
    CalibrationAdjudication, CalibrationAdjudicationEntry, CalibrationVerdict,
};
use crate::policy::{
    normalize_candidate_file, ActionPolicy, ActionPolicyTier, CalibrationStatus,
    SafeActionCandidate,
};

use super::projection::{
    OracleBridgeCalibrationAdjudicationStats, OracleBridgeCalibrationCandidateCounts,
    OracleBridgeCalibrationGate, OracleBridgeCalibrationPrecedent,
    OracleBridgeCalibrationPrecedentRef, OracleBridgeCalibrationProjection,
    OracleBridgeCalibrationReadiness, OracleBridgeCalibrationReason,
    OracleBridgeCalibrationReasonCode, OracleBridgeCalibrationRequiredEvidence,
    OracleBridgeCalibrationSeverity, OracleBridgeCalibrationStatusReason,
};

const CALIBRATION_REQUIRED_EVIDENCE: [OracleBridgeCalibrationRequiredEvidence; 3] = [
    OracleBridgeCalibrationRequiredEvidence::NonEmptySafeFixPopulation,
    OracleBridgeCalibrationRequiredEvidence::KnownSafeFixFpDenominator,
    OracleBridgeCalibrationRequiredEvidence::ReadinessGateFromRealCorpus,
];

#[derive(Debug, Clone)]
pub(super) struct OracleBridgeCalibrationPolicy {
    status: CalibrationStatus,
    reason: OracleBridgeCalibrationStatusReason,
    candidate_counts: OracleBridgeCalibrationCandidateCounts,
    readiness: OracleBridgeCalibrationReadiness,
}

impl OracleBridgeCalibrationPolicy {
    pub(super) fn from_action_policy(
        action_policy: &ActionPolicy<'_>,
        calibration_adjudication: Option<&CalibrationAdjudication>,
    ) -> Self {
        let actions = action_policy.semantic_action_counts();
        let candidate_counts = OracleBridgeCalibrationCandidateCounts {
            available: true,
            safe_fix: actions.safe_actions(),
            review_fix: actions.review_fix(),
            review_visible_cleanup: actions.review_visible_cleanup(),
            degraded: actions.degraded_findings(),
            muted: 0,
            syntax_muted_evidence: action_policy.syntax_muted_evidence_count(),
            unavailable: actions.coverage_unavailable_diagnostics(),
        };
        let status = if calibration_adjudication.is_some() {
            CalibrationStatus::Measured
        } else {
            CalibrationStatus::Pending
        };
        let reason = if calibration_adjudication.is_some() {
            OracleBridgeCalibrationStatusReason::MeasuredWithReadinessLimits
        } else {
            OracleBridgeCalibrationStatusReason::NotMeasured
        };
        Self {
            status,
            reason,
            candidate_counts,
            readiness: readiness(
                candidate_counts,
                action_policy.semantic_safe_action_candidates(),
                calibration_adjudication,
            ),
        }
    }

    pub(super) fn status(&self) -> CalibrationStatus {
        self.status
    }

    pub(super) fn into_projection(self) -> OracleBridgeCalibrationProjection {
        OracleBridgeCalibrationProjection {
            status: self.status,
            reason: self.reason,
            candidate_counts: self.candidate_counts,
            readiness: self.readiness,
            required_evidence: CALIBRATION_REQUIRED_EVIDENCE,
            js_ts_precedent: OracleBridgeCalibrationPrecedent {
                measurement_artifact: OracleBridgeCalibrationPrecedentRef::MeasurementArtifact,
                measurement_owner: OracleBridgeCalibrationPrecedentRef::MeasurementOwner,
                calibration_corpus_registry:
                    OracleBridgeCalibrationPrecedentRef::CalibrationCorpusRegistry,
                threshold_policy_metadata:
                    OracleBridgeCalibrationPrecedentRef::ThresholdPolicyMetadata,
            },
        }
    }
}

fn readiness(
    candidate_counts: OracleBridgeCalibrationCandidateCounts,
    safe_action_candidates: &[SafeActionCandidate<'_>],
    calibration_adjudication: Option<&CalibrationAdjudication>,
) -> OracleBridgeCalibrationReadiness {
    let entries = calibration_adjudication
        .map(CalibrationAdjudication::entries)
        .unwrap_or(&[]);
    let matched_entries = matched_adjudication_entries(entries, safe_action_candidates);
    let safe_fix = summarize_adjudication(entries, |entry| {
        entry.tier == Some(ActionPolicyTier::SafeFix)
            && adjudication_matches_candidate(entry, safe_action_candidates)
    });
    let review_visible_cleanup = summarize_adjudication(entries, |entry| {
        matches!(
            entry.tier,
            Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
        ) && adjudication_matches_candidate(entry, safe_action_candidates)
    });
    let mut reasons = Vec::new();
    let safe_needs_adjudication = candidate_counts.safe_fix > 0 && safe_fix.fp_rate.is_none();
    let review_needs_adjudication =
        candidate_counts.review_visible_cleanup > 0 && review_visible_cleanup.fp_rate.is_none();
    if entries.is_empty() || safe_needs_adjudication || review_needs_adjudication {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::FpRateUnknown,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "adjudication denominator is empty or incomplete",
        });
    }
    if !entries.is_empty() && matched_entries == 0 && candidate_counts.review_visible_cleanup > 0 {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::AdjudicationCandidateMismatch,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "adjudication entries did not match observed Rust cleanup candidates",
        });
    }
    if safe_fix.fp_rate.is_some_and(|fp_rate| fp_rate >= 0.05) {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::SafeFixFpThreshold,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "SAFE_FIX FP rate is above the JS/TS readiness threshold",
        });
    }
    if review_visible_cleanup
        .fp_rate
        .is_some_and(|fp_rate| fp_rate > 0.25)
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::ReviewVisibleFpThreshold,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "review-visible cleanup FP rate is above the JS/TS readiness threshold",
        });
    }

    let gate = if reasons
        .iter()
        .any(|reason| reason.severity == OracleBridgeCalibrationSeverity::Red)
    {
        OracleBridgeCalibrationGate::Red
    } else {
        if candidate_counts.safe_fix == 0 {
            reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::SafeFixPopulationEmpty,
                severity: OracleBridgeCalibrationSeverity::Yellow,
                detail: "SAFE_FIX population is measured zero; autonomous cleanup precision is not measured",
            });
        }
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::BenchmarkIncomplete,
            severity: OracleBridgeCalibrationSeverity::Yellow,
            detail: "Green corpus/adjudication thresholds not met",
        });
        OracleBridgeCalibrationGate::Yellow
    };

    OracleBridgeCalibrationReadiness {
        gate,
        reasons,
        safe_fix,
        review_visible_cleanup,
    }
}

fn matched_adjudication_entries(
    entries: &[CalibrationAdjudicationEntry],
    safe_action_candidates: &[SafeActionCandidate<'_>],
) -> usize {
    entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.tier,
                Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
            ) && adjudication_matches_candidate(entry, safe_action_candidates)
        })
        .count()
}

fn adjudication_matches_candidate(
    entry: &CalibrationAdjudicationEntry,
    safe_action_candidates: &[SafeActionCandidate<'_>],
) -> bool {
    entry.file.as_ref().is_some_and(|file| {
        let file = normalize_candidate_file(file);
        safe_action_candidates.iter().any(|candidate| {
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

fn summarize_adjudication(
    entries: &[CalibrationAdjudicationEntry],
    include: impl Fn(&CalibrationAdjudicationEntry) -> bool,
) -> OracleBridgeCalibrationAdjudicationStats {
    let mut stats = OracleBridgeCalibrationAdjudicationStats {
        false_positives: 0,
        true_dead: 0,
        inconclusive: 0,
        not_applicable: 0,
        fp_rate: None,
    };
    for entry in entries.iter().filter(|entry| include(entry)) {
        match entry.verdict {
            CalibrationVerdict::TrueDead => stats.true_dead += 1,
            CalibrationVerdict::FalsePositive => stats.false_positives += 1,
            CalibrationVerdict::NotApplicable => stats.not_applicable += 1,
            CalibrationVerdict::Inconclusive => stats.inconclusive += 1,
        }
    }
    let denominator = stats.true_dead + stats.false_positives;
    if denominator > 0 {
        stats.fp_rate = Some(stats.false_positives as f64 / denominator as f64);
    }
    stats
}
