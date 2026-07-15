use super::compression_ranks;

pub(crate) fn build_suffix_array_doubling(values: &[i64]) -> Vec<usize> {
    let n = values.len();
    if n == 0 {
        return Vec::new();
    }
    let mut suffix_array = vec![0usize; n];
    let mut scratch = vec![0usize; n];
    let mut indices = (0..n).collect::<Vec<_>>();
    let mut rank = compression_ranks(values);
    let mut next_rank = vec![0usize; n];
    let mut counts = vec![0usize; n + 1];
    let mut class_count = rank.iter().copied().max().map_or(0, |value| value + 1);
    let mut width = 1usize;

    while width < n {
        for (index, value) in indices.iter_mut().enumerate() {
            *value = index;
        }
        stable_counting_sort_by(
            &indices,
            &mut scratch,
            &mut counts,
            class_count + 1,
            |index| rank.get(index + width).map_or(0, |value| value + 1),
        );
        stable_counting_sort_by(
            &scratch,
            &mut suffix_array,
            &mut counts,
            class_count + 1,
            |index| rank[index] + 1,
        );

        next_rank[suffix_array[0]] = 0;
        for pair in suffix_array.windows(2) {
            let previous = pair[0];
            let current = pair[1];
            let same = rank[previous] == rank[current]
                && rank.get(previous + width) == rank.get(current + width);
            next_rank[current] = next_rank[previous] + usize::from(!same);
        }
        class_count = next_rank[suffix_array[n - 1]] + 1;
        rank.clone_from(&next_rank);
        if class_count == n {
            break;
        }
        width = width.checked_mul(2).unwrap_or(n);
    }
    suffix_array
}

fn stable_counting_sort_by<F>(
    input: &[usize],
    output: &mut [usize],
    counts: &mut [usize],
    key_count: usize,
    key: F,
) where
    F: Fn(usize) -> usize,
{
    counts[..key_count].fill(0);
    for &index in input {
        counts[key(index)] += 1;
    }
    let mut offset = 0usize;
    for count in &mut counts[..key_count] {
        let bucket_len = *count;
        *count = offset;
        offset += bucket_len;
    }
    for &index in input {
        let bucket = key(index);
        output[counts[bucket]] = index;
        counts[bucket] += 1;
    }
}
