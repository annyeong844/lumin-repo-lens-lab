use std::collections::HashMap;

use super::protocol::{BlockCloneToken, TokenizedFile};

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
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let rank_by_value = sorted
        .into_iter()
        .enumerate()
        .map(|(index, value)| (value, index))
        .collect::<HashMap<_, _>>();
    values
        .iter()
        .map(|value| *rank_by_value.get(value).unwrap_or(&0))
        .collect()
}

pub(super) fn build_suffix_array(values: &[i64]) -> Vec<usize> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let mut suffix_array = (0..n).collect::<Vec<_>>();
    let mut rank = compression_ranks(values)
        .into_iter()
        .map(|value| value as i64)
        .collect::<Vec<_>>();
    let mut next_rank = vec![0i64; n];
    let mut width = 1usize;

    while width < n {
        suffix_array.sort_by(|left, right| {
            rank[*left]
                .cmp(&rank[*right])
                .then_with(|| {
                    rank.get(*left + width)
                        .unwrap_or(&-1)
                        .cmp(rank.get(*right + width).unwrap_or(&-1))
                })
                .then_with(|| left.cmp(right))
        });

        next_rank[suffix_array[0]] = 0;
        for i in 1..n {
            let prev = suffix_array[i - 1];
            let current = suffix_array[i];
            let same = rank[prev] == rank[current]
                && rank.get(prev + width).unwrap_or(&-1)
                    == rank.get(current + width).unwrap_or(&-1);
            next_rank[current] = if same {
                next_rank[prev]
            } else {
                next_rank[prev] + 1
            };
        }
        rank.clone_from(&next_rank);
        if rank[suffix_array[n - 1]] == (n - 1) as i64 {
            break;
        }
        width *= 2;
    }

    suffix_array
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
