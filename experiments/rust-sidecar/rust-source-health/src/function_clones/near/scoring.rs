use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::RUST_FUNCTION_CLONE_NEAR_CALL_IDF_SATURATION;

use super::model::NearFact;

pub(super) fn call_token_idfs(facts: &[NearFact<'_>]) -> BTreeMap<String, f64> {
    let total_functions = facts.len() as f64;
    let mut document_frequency = BTreeMap::<String, usize>::new();
    for fact in facts {
        for token in &fact.significant_call_tokens {
            *document_frequency.entry(token.clone()).or_default() += 1;
        }
    }

    document_frequency
        .into_iter()
        .map(|(token, count)| {
            let idf = ((total_functions + 1.0) / (count as f64 + 1.0)).ln();
            (token, idf)
        })
        .collect()
}

pub(super) fn sorted_intersection(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    left.iter()
        .filter(|entry| right.contains(entry.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn jaccard(left: &[String], right: &[String]) -> f64 {
    let left = left.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let union = left.union(&right).count();
    if union == 0 {
        return 0.0;
    }
    left.intersection(&right).count() as f64 / union as f64
}

pub(super) fn shared_token_idf_sum(
    shared_tokens: &[String],
    token_idfs: &BTreeMap<String, f64>,
) -> f64 {
    shared_tokens
        .iter()
        .map(|token| token_idf(token, token_idfs))
        .sum()
}

pub(super) fn saturated_call_token_idf_score(shared_idf_sum: f64) -> f64 {
    (shared_idf_sum / RUST_FUNCTION_CLONE_NEAR_CALL_IDF_SATURATION).min(1.0)
}

pub(super) fn token_idf(token: &str, token_idfs: &BTreeMap<String, f64>) -> f64 {
    token_idfs.get(token).copied().unwrap_or(0.0)
}

pub(super) fn range_similarity(left: usize, right: usize) -> f64 {
    let max = left.max(right);
    if max == 0 {
        return 0.0;
    }
    1.0 - (left.abs_diff(right) as f64 / max as f64)
}

pub(super) fn round_score(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub(super) fn format_score(value: f64) -> String {
    let rounded = round_score(value);
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        rounded.to_string()
    }
}
