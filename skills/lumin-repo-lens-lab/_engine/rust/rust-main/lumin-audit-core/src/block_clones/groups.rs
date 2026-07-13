use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

use super::policy::{
    thresholds_policy_payload, Thresholds, BLOCK_CLONE_NORMALIZATION_POLICY_ID,
    BLOCK_CLONE_POLICY_VERSION,
};
use super::protocol::BlockCloneToken;
use super::suffix_array::{build_lcp_array, build_suffix_array};

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

fn span_for(
    meta: &[Option<BlockCloneToken>],
    start: usize,
    token_count: usize,
) -> Option<Instance> {
    let entries = meta.get(start..start + token_count)?;
    if entries.len() != token_count || entries.iter().any(Option::is_none) {
        return None;
    }
    let tokens = entries
        .iter()
        .filter_map(Option::as_ref)
        .collect::<Vec<_>>();
    let file = tokens.first()?.file.clone();
    if !tokens.iter().all(|token| token.file == file) {
        return None;
    }
    let start_entry = tokens.first()?;
    let end_entry = tokens.last()?;
    Some(Instance {
        file,
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

fn contains_span(outer: &Instance, inner: &Instance) -> bool {
    outer.file == inner.file
        && outer.start_token <= inner.start_token
        && outer.end_token >= inner.end_token
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
    let mut by_signature = BTreeMap::<String, (usize, BTreeSet<usize>)>::new();

    for i in 1..suffix_array.len() {
        let token_count = lcp[i];
        if token_count < thresholds.min_tokens {
            continue;
        }
        let starts = [suffix_array[i - 1], suffix_array[i]];
        let Some(signature_values) = values.get(starts[0]..starts[0] + token_count) else {
            continue;
        };
        let signature = signature_values
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let entry = by_signature
            .entry(signature)
            .or_insert_with(|| (token_count, BTreeSet::new()));
        entry.0 = entry.0.max(token_count);
        for start in starts {
            entry.1.insert(start);
        }
    }

    let mut groups = Vec::<BlockCloneGroup>::new();
    for (signature, (token_count, starts)) in by_signature {
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
        groups.push(BlockCloneGroup {
            id: format!("block-clone:{}", group_hash(thresholds, &signature)),
            claim: "repeated normalized token region".to_string(),
            confidence: "heuristic-review".to_string(),
            token_count,
            line_count,
            occurrence_count: kept.len(),
            normalization_mode: "alpha-identifier".to_string(),
            reasons: vec![
                "suffix-array-lcp-repeat".to_string(),
                "line-threshold-met".to_string(),
            ],
            instances: kept,
            review_only: true,
            eligible_for_safe_fix: false,
            visibility: None,
            mute_reason: None,
        });
    }

    prune_contained_block_clone_groups(groups, thresholds.max_candidate_groups + 1)
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

fn prune_contained_block_clone_groups(
    mut groups: Vec<BlockCloneGroup>,
    max_groups: usize,
) -> Vec<BlockCloneGroup> {
    if max_groups == 0 {
        return Vec::new();
    }
    groups.sort_by(compare_groups);
    let mut kept = Vec::<BlockCloneGroup>::new();
    let mut containment_index = HashMap::<String, Vec<(usize, Instance)>>::new();

    for group in groups {
        let instances = &group.instances;
        let probe = instances
            .iter()
            .min_by_key(|instance| {
                containment_index
                    .get(&instance.file)
                    .map(Vec::len)
                    .unwrap_or(0)
            })
            .cloned();
        let contained = probe
            .as_ref()
            .and_then(|probe| {
                containment_index
                    .get(&probe.file)
                    .map(|entries| (probe, entries))
            })
            .is_some_and(|(probe, entries)| {
                entries.iter().any(|(kept_index, kept_instance)| {
                    contains_span(kept_instance, probe)
                        && group_contains_instances(&kept[*kept_index], instances)
                })
            });
        if contained {
            continue;
        }

        let kept_index = kept.len();
        for instance in &group.instances {
            containment_index
                .entry(instance.file.clone())
                .or_default()
                .push((kept_index, instance.clone()));
        }
        kept.push(group);
        if kept.len() >= max_groups {
            break;
        }
    }

    kept
}

fn group_contains_instances(group: &BlockCloneGroup, instances: &[Instance]) -> bool {
    instances.iter().all(|instance| {
        group
            .instances
            .iter()
            .any(|other| contains_span(other, instance))
    })
}
