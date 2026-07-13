use serde_json::{json, Map, Value};

pub(super) const BLOCK_CLONE_SCHEMA_VERSION: &str = "block-clones.v1";
pub(super) const BLOCK_CLONE_POLICY_VERSION: &str = "block-clone-review-policy-v1";
pub(super) const BLOCK_CLONE_NORMALIZATION_POLICY_ID: &str = "block-clone-normalization-v1";
pub(super) const BLOCK_CLONE_THRESHOLD_POLICY_ID: &str = "block-clone-threshold-policy-v2";
pub(super) const BLOCK_CLONE_NOISE_POLICY_ID: &str = "block-clone-noise-policy-v1";

#[derive(Debug, Clone)]
pub(super) struct Thresholds {
    pub(super) min_tokens: usize,
    pub(super) min_lines: usize,
    pub(super) min_occurrences: usize,
    pub(super) max_instances_per_group: usize,
    pub(super) max_candidate_groups: usize,
    pub(super) max_review_groups: usize,
    pub(super) max_muted_groups: usize,
    pub(super) max_tokens_per_file: usize,
    pub(super) max_groups: Option<usize>,
}

pub(super) fn normalize_thresholds(value: Option<&Value>) -> Thresholds {
    let input = value.and_then(Value::as_object);
    Thresholds {
        min_tokens: non_negative_integer(input, "minTokens", 50),
        min_lines: non_negative_integer(input, "minLines", 5),
        min_occurrences: non_negative_integer(input, "minOccurrences", 2),
        max_instances_per_group: non_negative_integer(input, "maxInstancesPerGroup", 20),
        max_candidate_groups: non_negative_integer(input, "maxCandidateGroups", 1000),
        max_review_groups: non_negative_integer(input, "maxReviewGroups", 100),
        max_muted_groups: non_negative_integer(input, "maxMutedGroups", 100),
        max_tokens_per_file: non_negative_integer(input, "maxTokensPerFile", 200000),
        max_groups: optional_non_negative_integer(input, "maxGroups"),
    }
}

fn non_negative_integer(input: Option<&Map<String, Value>>, key: &str, fallback: usize) -> usize {
    optional_non_negative_integer(input, key).unwrap_or(fallback)
}

fn optional_non_negative_integer(input: Option<&Map<String, Value>>, key: &str) -> Option<usize> {
    input?
        .get(key)?
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
}

pub(super) fn thresholds_json(thresholds: &Thresholds) -> Value {
    let mut object = Map::new();
    object.insert(
        "policyId".to_string(),
        json!(BLOCK_CLONE_THRESHOLD_POLICY_ID),
    );
    for (key, value) in thresholds_policy_entries(thresholds) {
        object.insert(key.to_string(), value);
    }
    Value::Object(object)
}

pub(super) fn thresholds_policy_payload(thresholds: &Thresholds) -> Value {
    let mut object = Map::new();
    for (key, value) in thresholds_policy_entries(thresholds) {
        object.insert(key.to_string(), value);
    }
    Value::Object(object)
}

fn thresholds_policy_entries(thresholds: &Thresholds) -> Vec<(&'static str, Value)> {
    let mut entries = vec![
        ("minTokens", json!(thresholds.min_tokens)),
        ("minLines", json!(thresholds.min_lines)),
        ("minOccurrences", json!(thresholds.min_occurrences)),
        (
            "maxInstancesPerGroup",
            json!(thresholds.max_instances_per_group),
        ),
        ("maxCandidateGroups", json!(thresholds.max_candidate_groups)),
        ("maxReviewGroups", json!(thresholds.max_review_groups)),
        ("maxMutedGroups", json!(thresholds.max_muted_groups)),
        ("maxTokensPerFile", json!(thresholds.max_tokens_per_file)),
    ];
    if let Some(max_groups) = thresholds.max_groups {
        entries.push(("maxGroups", json!(max_groups)));
    }
    entries
}
