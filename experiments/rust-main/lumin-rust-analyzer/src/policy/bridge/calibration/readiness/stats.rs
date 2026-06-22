use std::collections::BTreeMap;

use crate::calibration::{CalibrationAdjudicationEntry, CalibrationVerdict};
use crate::policy::ActionPolicyTier;

use super::super::super::projection::OracleBridgeCalibrationAdjudicationStats;

const UNKNOWN_CORPUS_NAME: &str = "(unknown)";

pub(super) fn adjudicated_counts_by_corpus(
    entries: &[CalibrationAdjudicationEntry],
) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::new();
    for entry in entries {
        let corpus_name = entry_corpus_name(entry);
        *counts.entry(corpus_name).or_insert(0) += 1;
    }
    counts
}

pub(super) fn review_visible_denominators_by_corpus(
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

pub(super) fn summarize_adjudication(
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
