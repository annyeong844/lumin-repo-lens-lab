use crate::calibration::{CalibrationAdjudication, CalibrationAdjudicationEntry};

use super::policy::{DEFAULT_MIN_ADJUDICATED_PER_CORPUS, MIN_NON_TRIVIAL_CORPUS};
use super::stats::{adjudicated_counts_by_corpus, review_visible_denominators_by_corpus};

pub(super) fn enough_corpus(adjudication: &CalibrationAdjudication) -> bool {
    adjudication
        .corpus()
        .iter()
        .filter(|corpus| corpus.is_non_trivial())
        .count()
        >= MIN_NON_TRIVIAL_CORPUS
}

pub(super) fn every_corpus_has_enough_adjudication(
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

pub(super) fn corpus_has_unknown_fp_denominator(
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
