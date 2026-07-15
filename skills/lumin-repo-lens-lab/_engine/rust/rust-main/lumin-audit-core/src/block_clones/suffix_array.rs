use std::collections::HashMap;

use super::protocol::{BlockCloneToken, TokenizedFile};

#[cfg(test)]
pub(super) mod reference;
mod sais;

pub(super) struct CompressedTokens {
    pub(super) values: Vec<i64>,
    pub(super) meta: Vec<Option<BlockCloneToken>>,
}

pub(super) fn compress_token_values(files: &[TokenizedFile]) -> CompressedTokens {
    let mut ids = HashMap::<String, i64>::new();
    let mut next = 1i64;
    let mut values = Vec::<i64>::new();
    let mut meta = Vec::<Option<BlockCloneToken>>::new();
    let mut sentinel = -1i64;

    for file in files {
        for token in &file.tokens {
            let id = *ids.entry(token.value.clone()).or_insert_with(|| {
                let id = next;
                next += 1;
                id
            });
            values.push(id);
            meta.push(Some(token.clone()));
        }
        values.push(sentinel);
        meta.push(None);
        sentinel -= 1;
    }

    CompressedTokens { values, meta }
}

fn compression_ranks(values: &[i64]) -> Vec<usize> {
    if let Some(ranks) = dense_offset_ranks(values) {
        return ranks;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    values
        .iter()
        .map(|value| sorted.partition_point(|candidate| candidate < value))
        .collect()
}

fn dense_offset_ranks(values: &[i64]) -> Option<Vec<usize>> {
    let minimum = values.iter().copied().min()?;
    let maximum = values.iter().copied().max()?;
    let alphabet_len = usize::try_from(i128::from(maximum) - i128::from(minimum) + 1).ok()?;
    if alphabet_len > values.len().saturating_mul(2).max(256) {
        return None;
    }
    values
        .iter()
        .map(|value| usize::try_from(i128::from(*value) - i128::from(minimum)).ok())
        .collect()
}

pub(super) fn build_suffix_array(values: &[i64]) -> Vec<usize> {
    sais::build_suffix_array(&compression_ranks(values))
}

pub(super) fn build_lcp_array(values: &[i64], suffix_array: &[usize]) -> Vec<usize> {
    let n = values.len();
    let mut rank = vec![0usize; n];
    for (index, suffix) in suffix_array.iter().enumerate() {
        rank[*suffix] = index;
    }

    let mut lcp = vec![0usize; n];
    let mut k = 0usize;
    for i in 0..n {
        let r = rank[i];
        if r == 0 {
            k = 0;
            continue;
        }
        let j = suffix_array[r - 1];
        while i + k < n && j + k < n && values[i + k] == values[j + k] && values[i + k] > 0 {
            k += 1;
        }
        lcp[r] = k;
        k = k.saturating_sub(1);
    }
    lcp
}
