use std::collections::BTreeMap;

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
const SAFE_FIX_FP_RED_THRESHOLD: f64 = 0.05;
const REVIEW_VISIBLE_FP_RED_THRESHOLD: f64 = 0.25;
const REVIEW_VISIBLE_FP_GREEN_THRESHOLD: f64 = 0.10;
const MIN_NON_TRIVIAL_CORPUS: usize = 2;
const DEFAULT_MIN_ADJUDICATED_PER_CORPUS: usize = 50;

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
        let observed_candidate_counts = OracleBridgeCalibrationCandidateCounts {
            available: true,
            safe_fix: actions.safe_actions(),
            review_fix: actions.review_fix(),
            review_visible_cleanup: actions.review_visible_cleanup(),
            degraded: actions.degraded_findings(),
            muted: 0,
            syntax_muted_evidence: action_policy.syntax_muted_evidence_count(),
            unavailable: actions.coverage_unavailable_diagnostics(),
        };
        let candidate_counts =
            calibration_candidate_counts(observed_candidate_counts, calibration_adjudication);
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

fn calibration_candidate_counts(
    observed: OracleBridgeCalibrationCandidateCounts,
    calibration_adjudication: Option<&CalibrationAdjudication>,
) -> OracleBridgeCalibrationCandidateCounts {
    let Some(adjudication) = calibration_adjudication else {
        return observed;
    };
    if !adjudication.has_readiness_evidence() {
        return observed;
    }
    let counts = adjudication.candidate_counts();
    let safe_fix = counts.safe_fix().unwrap_or(0);
    let review_fix = counts.review_fix().unwrap_or_else(|| {
        counts
            .review_visible_cleanup()
            .unwrap_or(0)
            .saturating_sub(safe_fix)
    });
    let review_visible_cleanup = counts
        .review_visible_cleanup()
        .unwrap_or(safe_fix + review_fix);
    OracleBridgeCalibrationCandidateCounts {
        available: counts.is_available(),
        safe_fix,
        review_fix,
        review_visible_cleanup,
        degraded: counts.degraded().unwrap_or(0),
        muted: counts.muted().unwrap_or(0),
        syntax_muted_evidence: observed.syntax_muted_evidence,
        unavailable: observed.unavailable,
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
    let has_readiness_evidence =
        calibration_adjudication.is_some_and(CalibrationAdjudication::has_readiness_evidence);
    let matched_entries =
        matched_adjudication_entries(entries, safe_action_candidates, has_readiness_evidence);
    let safe_fix = summarize_adjudication(entries, |entry| {
        entry.tier == Some(ActionPolicyTier::SafeFix)
            && adjudication_is_in_scope(entry, safe_action_candidates, has_readiness_evidence)
    });
    let review_visible_cleanup = summarize_adjudication(entries, |entry| {
        matches!(
            entry.tier,
            Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
        ) && adjudication_is_in_scope(entry, safe_action_candidates, has_readiness_evidence)
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
            detail: "fix-plan.json missing or candidate counts unavailable",
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
            detail: "adjudication denominator is empty or incomplete",
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
            detail: "adjudication entries did not match observed Rust cleanup candidates",
        });
    }
    if safe_fix
        .fp_rate
        .is_some_and(|fp_rate| fp_rate >= SAFE_FIX_FP_RED_THRESHOLD)
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::SafeFixFpThreshold,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "SAFE_FIX FP rate is above the JS/TS readiness threshold",
        });
    }
    if review_visible_cleanup
        .fp_rate
        .is_some_and(|fp_rate| fp_rate > REVIEW_VISIBLE_FP_RED_THRESHOLD)
    {
        reasons.push(OracleBridgeCalibrationReason {
            code: OracleBridgeCalibrationReasonCode::ReviewVisibleFpThreshold,
            severity: OracleBridgeCalibrationSeverity::Red,
            detail: "review-visible cleanup FP rate is above the JS/TS readiness threshold",
        });
    }
    if let Some(adjudication) = calibration_adjudication.filter(|_| has_readiness_evidence) {
        match adjudication.schema_round_trip() {
            Some(schema_round_trip) => {
                if !schema_round_trip.attempted() {
                    reasons.push(OracleBridgeCalibrationReason {
                        code: OracleBridgeCalibrationReasonCode::SchemaRoundtripNotAttempted,
                        severity: OracleBridgeCalibrationSeverity::Red,
                        detail: "P3/P5 schema round-trip was not attempted",
                    });
                }
                if schema_round_trip.has_known_schema_drift_bugs() {
                    reasons.push(OracleBridgeCalibrationReason {
                        code: OracleBridgeCalibrationReasonCode::SchemaDriftKnown,
                        severity: OracleBridgeCalibrationSeverity::Red,
                        detail: "known P3/P5 schema drift bug present",
                    });
                }
            }
            None => reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::SchemaRoundtripNotAttempted,
                severity: OracleBridgeCalibrationSeverity::Red,
                detail: "P3/P5 schema round-trip was not attempted",
            }),
        }
        for corpus in adjudication.corpus() {
            if !corpus.has_immutable_identity() {
                reasons.push(OracleBridgeCalibrationReason {
                    code: OracleBridgeCalibrationReasonCode::CorpusIdentityMissing,
                    severity: OracleBridgeCalibrationSeverity::Red,
                    detail: "calibration corpus lacks commit or snapshot identity",
                });
            }
            if !corpus.dirty_state_known() {
                reasons.push(OracleBridgeCalibrationReason {
                    code: OracleBridgeCalibrationReasonCode::DirtyWorktreeUnknown,
                    severity: OracleBridgeCalibrationSeverity::Red,
                    detail: "calibration corpus dirty state is unknown",
                });
            } else if !corpus.dirty_state_captured() {
                reasons.push(OracleBridgeCalibrationReason {
                    code: OracleBridgeCalibrationReasonCode::DirtyWorktreeWithoutSnapshot,
                    severity: OracleBridgeCalibrationSeverity::Red,
                    detail: "dirty calibration corpus lacks snapshot or content hash",
                });
            }
        }
        if adjudication.unresolved_high_findings() > 0 {
            reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::UnresolvedHighFinding,
                severity: OracleBridgeCalibrationSeverity::Red,
                detail: "unresolved HIGH calibration finding present",
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
                detail: "SAFE_FIX population is measured zero; autonomous cleanup precision is not measured",
            });
        }
        if !enough_corpus || !enough_adjudication {
            reasons.push(OracleBridgeCalibrationReason {
                code: OracleBridgeCalibrationReasonCode::BenchmarkIncomplete,
                severity: OracleBridgeCalibrationSeverity::Yellow,
                detail: "Green corpus/adjudication thresholds not met",
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

fn enough_corpus(adjudication: &CalibrationAdjudication) -> bool {
    adjudication
        .corpus()
        .iter()
        .filter(|corpus| corpus.is_non_trivial())
        .count()
        >= MIN_NON_TRIVIAL_CORPUS
}

fn every_corpus_has_enough_adjudication(
    adjudication: &CalibrationAdjudication,
    entries: &[CalibrationAdjudicationEntry],
) -> bool {
    if adjudication.corpus().is_empty() {
        return false;
    }
    let counts = adjudicated_counts_by_corpus(entries);
    let corpus_total = adjudication.corpus().len();
    let min_adjudicated = adjudication
        .min_adjudicated_per_corpus()
        .unwrap_or(DEFAULT_MIN_ADJUDICATED_PER_CORPUS);
    adjudication.corpus().iter().all(|corpus| {
        let Some(name) = corpus.name() else {
            return false;
        };
        let count = counts.get(name).copied().unwrap_or(0);
        if count >= min_adjudicated {
            return true;
        }
        adjudication
            .candidate_counts()
            .expected_review_visible_for_corpus(name, corpus_total)
            .is_some_and(|expected| expected < min_adjudicated && count >= expected)
    })
}

fn corpus_has_unknown_fp_denominator(
    adjudication: Option<&CalibrationAdjudication>,
    entries: &[CalibrationAdjudicationEntry],
) -> bool {
    let Some(adjudication) = adjudication else {
        return false;
    };
    let denominators = review_visible_denominators_by_corpus(entries);
    let corpus_total = adjudication.corpus().len();
    adjudication.corpus().iter().any(|corpus| {
        let Some(name) = corpus.name() else {
            return false;
        };
        adjudication
            .candidate_counts()
            .expected_review_visible_for_corpus(name, corpus_total)
            .is_some_and(|expected| {
                expected > 0 && denominators.get(name).copied().unwrap_or(0) == 0
            })
    })
}

fn adjudicated_counts_by_corpus(entries: &[CalibrationAdjudicationEntry]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries {
        let Some(corpus_name) = entry.corpus_name.as_deref() else {
            continue;
        };
        *counts.entry(corpus_name).or_insert(0) += 1;
    }
    counts
}

fn review_visible_denominators_by_corpus(
    entries: &[CalibrationAdjudicationEntry],
) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries.iter().filter(|entry| {
        matches!(
            entry.tier,
            Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
        ) && matches!(
            entry.verdict,
            CalibrationVerdict::TrueDead | CalibrationVerdict::FalsePositive
        )
    }) {
        let Some(corpus_name) = entry.corpus_name.as_deref() else {
            continue;
        };
        *counts.entry(corpus_name).or_insert(0) += 1;
    }
    counts
}

fn matched_adjudication_entries(
    entries: &[CalibrationAdjudicationEntry],
    safe_action_candidates: &[SafeActionCandidate<'_>],
    has_readiness_evidence: bool,
) -> usize {
    entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.tier,
                Some(ActionPolicyTier::SafeFix | ActionPolicyTier::ReviewFix)
            ) && adjudication_is_in_scope(entry, safe_action_candidates, has_readiness_evidence)
        })
        .count()
}

fn adjudication_is_in_scope(
    entry: &CalibrationAdjudicationEntry,
    safe_action_candidates: &[SafeActionCandidate<'_>],
    has_readiness_evidence: bool,
) -> bool {
    has_readiness_evidence || adjudication_matches_candidate(entry, safe_action_candidates)
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
