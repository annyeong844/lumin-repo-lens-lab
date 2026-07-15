const EMPTY_SUFFIX: usize = usize::MAX;

pub(super) fn build_suffix_array(ranks: &[usize]) -> Vec<usize> {
    if ranks.is_empty() {
        return Vec::new();
    }

    let mut symbols = Vec::with_capacity(ranks.len() + 1);
    symbols.extend(ranks.iter().map(|rank| rank + 1));
    symbols.push(0);
    let alphabet_len = ranks.iter().copied().max().map_or(2, |rank| rank + 2);
    let complete = sort_terminated_symbols(&symbols, alphabet_len);
    debug_assert_eq!(complete.first(), Some(&ranks.len()));
    complete.into_iter().skip(1).collect()
}

fn sort_terminated_symbols(symbols: &[usize], alphabet_len: usize) -> Vec<usize> {
    match symbols.len() {
        0 => return Vec::new(),
        1 => return vec![0],
        _ => {}
    }

    let suffix_classes = classify_suffixes(symbols);
    let buckets = BucketLayout::new(symbols, alphabet_len);
    let lms_positions = (1..symbols.len())
        .filter(|&index| is_lms(&suffix_classes, index))
        .collect::<Vec<_>>();
    let first_pass = run_induction(
        symbols,
        &suffix_classes,
        &buckets,
        lms_positions.iter().copied(),
    );
    let (names_by_position, name_count) =
        assign_reduced_names(symbols, &suffix_classes, &first_pass);
    let reduced = lms_positions
        .iter()
        .map(|&position| names_by_position[position])
        .collect::<Vec<_>>();
    let reduced_order = if name_count == reduced.len() {
        inverse_permutation(&reduced)
    } else {
        sort_terminated_symbols(&reduced, name_count)
    };
    run_induction(
        symbols,
        &suffix_classes,
        &buckets,
        reduced_order
            .into_iter()
            .map(|index| lms_positions[index])
            .rev(),
    )
}

fn classify_suffixes(symbols: &[usize]) -> Vec<bool> {
    let mut is_s = vec![false; symbols.len()];
    is_s[symbols.len() - 1] = true;
    for index in (0..symbols.len() - 1).rev() {
        is_s[index] = symbols[index] < symbols[index + 1]
            || (symbols[index] == symbols[index + 1] && is_s[index + 1]);
    }
    is_s
}

#[inline]
fn is_lms(is_s: &[bool], index: usize) -> bool {
    index > 0 && is_s[index] && !is_s[index - 1]
}

struct BucketLayout {
    starts: Vec<usize>,
    ends: Vec<usize>,
}

impl BucketLayout {
    fn new(symbols: &[usize], alphabet_len: usize) -> Self {
        let mut counts = vec![0usize; alphabet_len];
        for &symbol in symbols {
            counts[symbol] += 1;
        }
        let mut starts = Vec::with_capacity(alphabet_len);
        let mut ends = Vec::with_capacity(alphabet_len);
        let mut offset = 0usize;
        for count in counts {
            starts.push(offset);
            offset += count;
            ends.push(offset);
        }
        Self { starts, ends }
    }
}

fn run_induction(
    symbols: &[usize],
    is_s: &[bool],
    buckets: &BucketLayout,
    lms_order: impl Iterator<Item = usize>,
) -> Vec<usize> {
    let mut suffix_array = vec![EMPTY_SUFFIX; symbols.len()];
    let mut tails = buckets.ends.clone();
    for position in lms_order {
        let symbol = symbols[position];
        tails[symbol] -= 1;
        suffix_array[tails[symbol]] = position;
    }

    let mut heads = buckets.starts.clone();
    for index in 0..suffix_array.len() {
        let position = suffix_array[index];
        if position == EMPTY_SUFFIX || position == 0 {
            continue;
        }
        let preceding = position - 1;
        if !is_s[preceding] {
            let symbol = symbols[preceding];
            suffix_array[heads[symbol]] = preceding;
            heads[symbol] += 1;
        }
    }

    let mut tails = buckets.ends.clone();
    for index in (0..suffix_array.len()).rev() {
        let position = suffix_array[index];
        if position == EMPTY_SUFFIX || position == 0 {
            continue;
        }
        let preceding = position - 1;
        if is_s[preceding] {
            let symbol = symbols[preceding];
            tails[symbol] -= 1;
            suffix_array[tails[symbol]] = preceding;
        }
    }
    suffix_array
}

fn assign_reduced_names(
    symbols: &[usize],
    is_s: &[bool],
    suffix_array: &[usize],
) -> (Vec<usize>, usize) {
    let mut names = vec![EMPTY_SUFFIX; symbols.len()];
    let mut current_name = 0usize;
    let mut previous = None;
    for &position in suffix_array {
        if position == EMPTY_SUFFIX || !is_lms(is_s, position) {
            continue;
        }
        if previous.is_some_and(|prior| !same_lms_segment(symbols, is_s, prior, position)) {
            current_name += 1;
        }
        names[position] = current_name;
        previous = Some(position);
    }
    (names, current_name + 1)
}

fn same_lms_segment(symbols: &[usize], is_s: &[bool], left: usize, right: usize) -> bool {
    let mut offset = 0usize;
    loop {
        let left_index = left + offset;
        let right_index = right + offset;
        if symbols.get(left_index) != symbols.get(right_index)
            || is_s.get(left_index) != is_s.get(right_index)
        {
            return false;
        }
        let left_boundary = offset > 0 && is_lms(is_s, left_index);
        let right_boundary = offset > 0 && is_lms(is_s, right_index);
        if left_boundary || right_boundary {
            return left_boundary && right_boundary;
        }
        offset += 1;
    }
}

fn inverse_permutation(values: &[usize]) -> Vec<usize> {
    let mut inverse = vec![0usize; values.len()];
    for (index, &value) in values.iter().enumerate() {
        inverse[value] = index;
    }
    inverse
}
