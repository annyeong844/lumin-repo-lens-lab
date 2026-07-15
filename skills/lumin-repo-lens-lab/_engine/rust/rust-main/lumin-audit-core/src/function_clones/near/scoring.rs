use std::collections::BTreeMap;

use super::model::NearFact;
use super::CALL_IDF_SATURATION;

pub(super) fn call_token_idfs<'a>(facts: &[NearFact<'a>]) -> BTreeMap<&'a str, f64> {
    let total_functions = facts.len() as f64;
    let mut document_frequency = BTreeMap::<&'a str, usize>::new();
    for fact in facts {
        for token in &fact.significant_call_tokens {
            *document_frequency.entry(*token).or_default() += 1;
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

pub(super) struct TokenOverlap {
    pub(super) shared_tokens: Vec<String>,
    pub(super) jaccard: f64,
}

pub(super) fn token_overlap<T: AsRef<str>>(left: &[T], right: &[T]) -> TokenOverlap {
    debug_assert_sorted_unique(left);
    debug_assert_sorted_unique(right);

    let mut shared_tokens = Vec::new();
    let mut union = 0usize;
    let mut left_index = 0usize;
    let mut right_index = 0usize;
    while left_index < left.len() && right_index < right.len() {
        union += 1;
        match left[left_index].as_ref().cmp(right[right_index].as_ref()) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                shared_tokens.push(left[left_index].as_ref().to_string());
                left_index += 1;
                right_index += 1;
            }
        }
    }
    union += left.len().saturating_sub(left_index);
    union += right.len().saturating_sub(right_index);

    TokenOverlap {
        jaccard: if union == 0 {
            0.0
        } else {
            shared_tokens.len() as f64 / union as f64
        },
        shared_tokens,
    }
}

pub(super) fn shared_token_idf_sum(
    shared_tokens: &[String],
    token_idfs: &BTreeMap<&str, f64>,
) -> f64 {
    shared_tokens
        .iter()
        .map(|token| token_idf(token, token_idfs))
        .sum()
}

pub(super) fn saturated_call_token_idf_score(shared_idf_sum: f64) -> f64 {
    (shared_idf_sum / CALL_IDF_SATURATION).min(1.0)
}

pub(super) fn token_idf(token: &str, token_idfs: &BTreeMap<&str, f64>) -> f64 {
    token_idfs.get(token).copied().unwrap_or(0.0)
}

pub(super) fn range_similarity(left: i64, right: i64) -> f64 {
    let max = left.max(right);
    if max <= 0 {
        return 0.0;
    }
    1.0 - ((left - right).abs() as f64 / max as f64)
}

pub(super) fn round_score(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

pub(super) fn number_string(value: f64) -> String {
    let rounded = round_score(value);
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        rounded.to_string()
    }
}

fn debug_assert_sorted_unique<T: AsRef<str>>(tokens: &[T]) {
    debug_assert!(tokens
        .windows(2)
        .all(|pair| pair[0].as_ref() < pair[1].as_ref()));
}
