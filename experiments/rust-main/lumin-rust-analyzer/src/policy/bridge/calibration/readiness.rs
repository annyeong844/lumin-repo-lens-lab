use std::collections::BTreeMap;

use crate::calibration::{
    CalibrationAdjudication, CalibrationAdjudicationEntry, CalibrationVerdict,
};
use crate::policy::{normalize_candidate_file, ActionPolicyTier, CleanupCandidate};

use super::super::projection::{
    OracleBridgeCalibrationAdjudicationStats, OracleBridgeCalibrationCandidateCounts,
    OracleBridgeCalibrationGate, OracleBridgeCalibrationPrecedentRef,
    OracleBridgeCalibrationReadiness, OracleBridgeCalibrationReadinessPolicy,
    OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationSeverity,
};

// Mirrors _lib/p6-measurement.mjs::computeReadiness. Change the JS/TS owner first.
const SAFE_FIX_FP_RED_THRESHOLD: f64 = 0.05;
const REVIEW_VISIBLE_FP_RED_THRESHOLD: f64 = 0.25;
const REVIEW_VISIBLE_FP_GREEN_THRESHOLD: f64 = 0.10;
const MIN_NON_TRIVIAL_CORPUS: usize = 2;
const DEFAULT_MIN_ADJUDICATED_PER_CORPUS: usize = 50;
const UNKNOWN_CORPUS_NAME: &str = "(unknown)";

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
        let count = corpus
            .name()
            .and_then(|name| counts.get(name).copied())
            .unwrap_or(0);
        if count >= min_adjudicated {
            return true;
        }
        adjudication
            .candidate_counts()
            .expected_review_visible_for_optional_corpus(corpus.name(), corpus_total)
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
        let denominator = corpus
            .name()
            .and_then(|name| denominators.get(name).copied())
            .unwrap_or(0);
        adjudication
            .candidate_counts()
            .expected_review_visible_for_optional_corpus(corpus.name(), corpus_total)
            .is_some_and(|expected| expected > 0 && denominator == 0)
    })
}

fn adjudicated_counts_by_corpus(entries: &[CalibrationAdjudicationEntry]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries {
        let corpus_name = entry_corpus_name(entry);
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
        let corpus_name = entry_corpus_name(entry);
        *counts.entry(corpus_name).or_insert(0) += 1;
    }
    counts
}

fn entry_corpus_name(entry: &CalibrationAdjudicationEntry) -> &str {
    entry.corpus_name.as_deref().unwrap_or(UNKNOWN_CORPUS_NAME)
}

fn matched_adjudication_entries(
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

fn adjudication_is_in_scope(
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
