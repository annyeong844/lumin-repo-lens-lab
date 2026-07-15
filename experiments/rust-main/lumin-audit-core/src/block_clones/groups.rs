use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use super::policy::{
    thresholds_policy_payload, Thresholds, BLOCK_CLONE_NORMALIZATION_POLICY_ID,
    BLOCK_CLONE_POLICY_VERSION,
};
use super::protocol::BlockCloneToken;
use super::suffix_array::{build_lcp_array, build_suffix_array};

mod containment;

use containment::{ContainmentIndex, ContainmentScratch};

#[derive(Debug, Clone)]
pub(super) struct Instance {
    pub(super) file: String,
    pub(super) start_line: usize,
    pub(super) end_line: usize,
    pub(super) start_token: usize,
    pub(super) end_token: usize,
    pub(super) container: Option<Value>,
}

#[derive(Debug, Clone)]
pub(super) struct BlockCloneGroup {
    pub(super) id: String,
    pub(super) claim: String,
    pub(super) confidence: String,
    pub(super) token_count: usize,
    pub(super) line_count: usize,
    pub(super) occurrence_count: usize,
    pub(super) normalization_mode: String,
    pub(super) reasons: Vec<String>,
    pub(super) instances: Vec<Instance>,
    pub(super) review_only: bool,
    pub(super) eligible_for_safe_fix: bool,
    pub(super) visibility: Option<String>,
    pub(super) mute_reason: Option<String>,
}

struct BlockCloneCandidate {
    representative_start: usize,
    token_count: usize,
    line_count: usize,
    instances: Vec<Instance>,
}

fn span_for(
    meta: &[Option<BlockCloneToken>],
    start: usize,
    token_count: usize,
) -> Option<Instance> {
    if token_count == 0 {
        return None;
    }
    let start_entry = meta.get(start)?.as_ref()?;
    let end_entry = meta
        .get(start.checked_add(token_count.checked_sub(1)?)?)?
        .as_ref()?;
    // LCP construction stops at the non-positive sentinel between files, so a
    // matched span with token-backed endpoints cannot cross a file boundary.
    if start_entry.file != end_entry.file {
        return None;
    }
    Some(Instance {
        file: start_entry.file.clone(),
        start_line: start_entry.line,
        end_line: end_entry.end_line,
        start_token: start,
        end_token: start + token_count,
        container: start_entry.container.clone(),
    })
}

fn overlaps(left: &Instance, right: &Instance) -> bool {
    left.file == right.file
        && left.start_token < right.end_token
        && right.start_token < left.end_token
}

fn filter_non_overlapping(instances: Vec<Instance>, limit: usize) -> Vec<Instance> {
    let mut instances = instances;
    instances.sort_by(compare_instances);
    let mut kept = Vec::<Instance>::new();
    for instance in instances {
        if kept.iter().any(|other| overlaps(other, &instance)) {
            continue;
        }
        kept.push(instance);
        if kept.len() >= limit {
            break;
        }
    }
    kept
}

fn compare_instances(left: &Instance, right: &Instance) -> Ordering {
    left.file
        .cmp(&right.file)
        .then_with(|| left.start_token.cmp(&right.start_token))
        .then_with(|| left.end_token.cmp(&right.end_token))
}

pub(super) fn extract_groups(
    values: &[i64],
    meta: &[Option<BlockCloneToken>],
    thresholds: &Thresholds,
) -> Vec<BlockCloneGroup> {
    if values.is_empty() {
        return Vec::new();
    }
    let suffix_array = build_suffix_array(values);
    let lcp = build_lcp_array(values, &suffix_array);
    let interval_starts = signature_interval_starts(&lcp);
    let mut by_signature = HashMap::<(usize, usize), (usize, BTreeSet<usize>)>::new();

    for i in 1..suffix_array.len() {
        let token_count = lcp[i];
        if token_count < thresholds.min_tokens {
            continue;
        }
        let starts = [suffix_array[i - 1], suffix_array[i]];
        let entry = by_signature
            .entry((token_count, interval_starts[i]))
            .or_insert_with(|| (starts[0], BTreeSet::new()));
        for start in starts {
            entry.1.insert(start);
        }
    }

    let mut candidates = Vec::<BlockCloneCandidate>::new();
    for ((token_count, _), (representative_start, starts)) in by_signature {
        let instances = starts
            .into_iter()
            .filter_map(|start| span_for(meta, start, token_count))
            .collect::<Vec<_>>();
        let kept = filter_non_overlapping(instances, thresholds.max_instances_per_group);
        let line_count = kept
            .iter()
            .map(|span| span.end_line.saturating_sub(span.start_line) + 1)
            .max()
            .unwrap_or(0);
        if kept.len() < thresholds.min_occurrences || line_count < thresholds.min_lines {
            continue;
        }
        candidates.push(BlockCloneCandidate {
            representative_start,
            token_count,
            line_count,
            instances: kept,
        });
    }
    rank_and_prune_candidates(
        values,
        candidates,
        thresholds,
        thresholds.max_candidate_groups.saturating_add(1),
    )
}

pub(super) fn signature_interval_starts(lcp: &[usize]) -> Vec<usize> {
    let mut interval_starts = vec![0usize; lcp.len()];
    let mut increasing = Vec::<usize>::new();
    for (index, &length) in lcp.iter().enumerate() {
        while increasing
            .last()
            .is_some_and(|&previous| lcp[previous] >= length)
        {
            increasing.pop();
        }
        interval_starts[index] = increasing.last().copied().unwrap_or(0);
        increasing.push(index);
    }
    interval_starts
}

fn group_hash(thresholds: &Thresholds, signature: &str) -> String {
    let payload = json!({
        "policy": BLOCK_CLONE_POLICY_VERSION,
        "normalization": BLOCK_CLONE_NORMALIZATION_POLICY_ID,
        "thresholds": thresholds_policy_payload(thresholds),
        "signature": signature,
    });
    let stable = stable_json(&payload);
    let mut hasher = Sha256::new();
    hasher.update(stable.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn stable_json(value: &Value) -> String {
    serde_json::to_string(&stable_value(value)).unwrap_or_else(|_| "null".to_string())
}

fn stable_value(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(stable_value).collect()),
        Value::Object(object) => {
            let mut sorted = Map::new();
            for (key, value) in object {
                sorted.insert(key.clone(), stable_value(value));
            }
            Value::Object(sorted)
        }
        other => other.clone(),
    }
}

pub(super) fn compare_groups(left: &BlockCloneGroup, right: &BlockCloneGroup) -> Ordering {
    right
        .token_count
        .cmp(&left.token_count)
        .then_with(|| right.occurrence_count.cmp(&left.occurrence_count))
        .then_with(|| left.id.cmp(&right.id))
}

fn rank_and_prune_candidates(
    values: &[i64],
    mut candidates: Vec<BlockCloneCandidate>,
    thresholds: &Thresholds,
    max_groups: usize,
) -> Vec<BlockCloneGroup> {
    let max_groups = max_groups.min(candidates.len());
    if max_groups == 0 {
        return Vec::new();
    }
    candidates.sort_by(|left, right| {
        right
            .token_count
            .cmp(&left.token_count)
            .then_with(|| right.instances.len().cmp(&left.instances.len()))
    });
    let mut kept = Vec::<BlockCloneGroup>::new();
    let mut containment_index = ContainmentIndex::new(values.len());
    let mut containment_scratch = ContainmentScratch::new(max_groups);
    let mut candidates = candidates.into_iter().peekable();

    while let Some(first) = candidates.next() {
        let rank = (first.token_count, first.instances.len());
        let mut rank_bucket = vec![first];
        while candidates
            .peek()
            .is_some_and(|candidate| (candidate.token_count, candidate.instances.len()) == rank)
        {
            if let Some(candidate) = candidates.next() {
                rank_bucket.push(candidate);
            }
        }

        // Equal-rank candidates cannot strictly contain one another: equal
        // token lengths require equal intervals, and equal occurrence counts
        // then require equal instance sets. Check only higher-rank groups
        // before paying to materialize signatures and stable ids.
        let survivors = rank_bucket
            .into_iter()
            .filter(|candidate| {
                !containment_index.contains_group(&candidate.instances, &mut containment_scratch)
            })
            .collect::<Vec<_>>();
        let mut groups = survivors
            .into_iter()
            .filter_map(|candidate| materialize_candidate(values, candidate, thresholds))
            .collect::<Vec<_>>();
        groups.sort_by(compare_groups);
        for group in groups {
            let kept_index = kept.len();
            for instance in &group.instances {
                containment_index.insert(kept_index, instance);
            }
            kept.push(group);
            if kept.len() >= max_groups {
                return kept;
            }
        }
    }

    kept
}

fn materialize_candidate(
    values: &[i64],
    candidate: BlockCloneCandidate,
    thresholds: &Thresholds,
) -> Option<BlockCloneGroup> {
    let signature = values
        .get(
            candidate.representative_start..candidate.representative_start + candidate.token_count,
        )?
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    Some(BlockCloneGroup {
        id: format!("block-clone:{}", group_hash(thresholds, &signature)),
        claim: "repeated normalized token region".to_string(),
        confidence: "heuristic-review".to_string(),
        token_count: candidate.token_count,
        line_count: candidate.line_count,
        occurrence_count: candidate.instances.len(),
        normalization_mode: "alpha-identifier".to_string(),
        reasons: vec![
            "suffix-array-lcp-repeat".to_string(),
            "line-threshold-met".to_string(),
        ],
        instances: candidate.instances,
        review_only: true,
        eligible_for_safe_fix: false,
        visibility: None,
        mute_reason: None,
    })
}
