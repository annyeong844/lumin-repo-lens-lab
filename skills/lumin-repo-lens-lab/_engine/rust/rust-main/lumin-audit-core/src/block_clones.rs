use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub const BLOCK_CLONES_REQUEST_SCHEMA_VERSION: &str = "lumin-block-clones-producer-request.v1";

const BLOCK_CLONE_SCHEMA_VERSION: &str = "block-clones.v1";
const BLOCK_CLONE_POLICY_VERSION: &str = "block-clone-review-policy-v1";
const BLOCK_CLONE_NORMALIZATION_POLICY_ID: &str = "block-clone-normalization-v1";
const BLOCK_CLONE_THRESHOLD_POLICY_ID: &str = "block-clone-threshold-policy-v2";
const BLOCK_CLONE_NOISE_POLICY_ID: &str = "block-clone-noise-policy-v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockClonesRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<Value>,
    #[serde(default)]
    pub files: Vec<TokenizedFile>,
    #[serde(default)]
    pub thresholds: Option<Value>,
    #[serde(default)]
    pub incremental: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizedFile {
    pub rel_file: String,
    #[serde(default)]
    pub tokens: Vec<BlockCloneToken>,
    #[serde(default)]
    pub skipped: Option<Value>,
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    #[serde(default)]
    pub token_limit_exceeded: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockCloneToken {
    pub value: String,
    pub file: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub end_line: usize,
    #[serde(default)]
    pub container: Option<Value>,
}

#[derive(Debug, Clone)]
struct Thresholds {
    min_tokens: usize,
    min_lines: usize,
    min_occurrences: usize,
    max_instances_per_group: usize,
    max_candidate_groups: usize,
    max_review_groups: usize,
    max_muted_groups: usize,
    max_tokens_per_file: usize,
    max_groups: Option<usize>,
}

#[derive(Debug, Clone)]
struct Instance {
    file: String,
    start_line: usize,
    end_line: usize,
    start_token: usize,
    end_token: usize,
    container: Option<Value>,
}

#[derive(Debug, Clone)]
struct BlockCloneGroup {
    id: String,
    claim: String,
    confidence: String,
    token_count: usize,
    line_count: usize,
    occurrence_count: usize,
    normalization_mode: String,
    reasons: Vec<String>,
    instances: Vec<Instance>,
    review_only: bool,
    eligible_for_safe_fix: bool,
    visibility: Option<String>,
    mute_reason: Option<String>,
}

#[derive(Debug, Clone)]
struct CompressedTokens {
    values: Vec<i64>,
    meta: Vec<Option<BlockCloneToken>>,
}

#[derive(Debug)]
struct NoisePolicyResult {
    groups: Vec<BlockCloneGroup>,
    review_group_count: usize,
    muted_group_count: usize,
    muted_by_reason: BTreeMap<String, usize>,
    candidate_cap_saturated: bool,
    review_cap_saturated: bool,
    muted_cap_saturated: bool,
}

pub fn build_block_clones_artifact(request: BlockClonesRequest) -> Result<Value> {
    if request.schema_version != BLOCK_CLONES_REQUEST_SCHEMA_VERSION {
        bail!(
            "block-clones-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let thresholds = normalize_thresholds(request.thresholds.as_ref());
    let mut tokenized_files = Vec::<TokenizedFile>::new();
    let mut skipped = Vec::<Value>::new();
    let mut diagnostics = Vec::<Value>::new();
    let mut unavailable_file_count = 0usize;

    for file in request.files {
        if let Some(skipped_file) = file.skipped {
            skipped.push(skipped_file);
            continue;
        }
        if file.token_limit_exceeded {
            skipped.push(json!({
                "file": file.rel_file,
                "reason": "max-tokens-per-file",
                "evidence": "threshold:maxTokensPerFile",
            }));
            continue;
        }
        if !file.diagnostics.is_empty() {
            diagnostics.extend(file.diagnostics);
            unavailable_file_count += 1;
            continue;
        }
        tokenized_files.push(file);
    }

    let compressed = compress_token_values(&tokenized_files);
    let extracted = extract_groups(&compressed.values, &compressed.meta, &thresholds);
    let noise_policy = apply_noise_policy(extracted, &thresholds);
    let status = if diagnostics.is_empty() && skipped.is_empty() {
        "complete"
    } else {
        "confidence-limited"
    };
    let token_count: usize = tokenized_files.iter().map(|file| file.tokens.len()).sum();
    let instance_count: usize = noise_policy
        .groups
        .iter()
        .map(|group| group.instances.len())
        .sum();

    let mut artifact = Map::new();
    artifact.insert(
        "schemaVersion".to_string(),
        json!(BLOCK_CLONE_SCHEMA_VERSION),
    );
    artifact.insert(
        "policyVersion".to_string(),
        json!(BLOCK_CLONE_POLICY_VERSION),
    );
    artifact.insert("status".to_string(), json!(status));
    artifact.insert("generated".to_string(), json!(request.generated));
    artifact.insert("root".to_string(), json!(request.root));
    artifact.insert(
        "scanRange".to_string(),
        json!({
            "includeTests": request.include_tests,
            "exclude": request.exclude,
        }),
    );
    artifact.insert(
        "normalization".to_string(),
        json!({
            "policyId": BLOCK_CLONE_NORMALIZATION_POLICY_ID,
            "mode": "alpha-identifier",
            "preservePropertyNames": true,
            "preserveImportSpecifiers": true,
            "literalPolicy": "classify",
            "importDeclarationPolicy": "skip",
        }),
    );
    artifact.insert("thresholds".to_string(), thresholds_json(&thresholds));
    artifact.insert(
        "summary".to_string(),
        json!({
            "fileCount": tokenized_files.len(),
            "tokenCount": token_count,
            "groupCount": noise_policy.groups.len(),
            "instanceCount": instance_count,
            "skippedFileCount": skipped.len(),
            "unavailableFileCount": unavailable_file_count,
            "reviewGroupCount": noise_policy.review_group_count,
            "mutedGroupCount": noise_policy.muted_group_count,
        }),
    );
    artifact.insert(
        "noisePolicy".to_string(),
        json!({
            "policyId": BLOCK_CLONE_NOISE_POLICY_ID,
            "reviewGroupCount": noise_policy.review_group_count,
            "mutedGroupCount": noise_policy.muted_group_count,
            "mutedByReason": noise_policy.muted_by_reason,
            "candidateCapSaturated": noise_policy.candidate_cap_saturated,
            "reviewCapSaturated": noise_policy.review_cap_saturated,
            "mutedCapSaturated": noise_policy.muted_cap_saturated,
        }),
    );
    artifact.insert(
        "groups".to_string(),
        Value::Array(noise_policy.groups.iter().map(group_json).collect()),
    );
    artifact.insert("skipped".to_string(), Value::Array(skipped));
    artifact.insert("diagnostics".to_string(), Value::Array(diagnostics));
    artifact.insert(
        "meta".to_string(),
        json!({
            "generated": request.generated,
            "root": request.root,
            "incremental": request.incremental,
        }),
    );

    Ok(Value::Object(artifact))
}

fn normalize_thresholds(value: Option<&Value>) -> Thresholds {
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

fn thresholds_json(thresholds: &Thresholds) -> Value {
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

fn thresholds_policy_payload(thresholds: &Thresholds) -> Value {
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

fn compress_token_values(files: &[TokenizedFile]) -> CompressedTokens {
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

fn build_suffix_array(values: &[i64]) -> Vec<usize> {
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

fn build_lcp_array(values: &[i64], suffix_array: &[usize]) -> Vec<usize> {
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

fn extract_groups(
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

fn compare_groups(left: &BlockCloneGroup, right: &BlockCloneGroup) -> Ordering {
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

fn is_test_file(file: &str) -> bool {
    let rel = slash_path(file).to_lowercase();
    let base = posix_basename(&rel);
    rel.starts_with("tests/")
        || rel.contains("/tests/")
        || base.starts_with("test-")
        || has_test_or_spec_extension(base)
}

fn has_test_or_spec_extension(base: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        "js", "jsx", "ts", "tsx", "mjs", "mjsx", "mts", "mtsx", "cjs", "cjsx", "cts", "ctsx",
    ];
    EXTENSIONS.iter().any(|ext| {
        base.ends_with(&format!(".test.{ext}")) || base.ends_with(&format!(".spec.{ext}"))
    })
}

fn test_mirror_entry(file: &str) -> Option<(String, &'static str)> {
    let rel = slash_path(file).to_lowercase();
    if !is_test_file(&rel) {
        return None;
    }
    let base = posix_basename(&rel);
    let dir = posix_dirname(&rel);
    if let Some(stripped) = base.strip_prefix("test-") {
        return Some((format!("{dir}/{}", strip_js_extension(stripped)), "node"));
    }
    if let Some(stem) = strip_test_spec_extension(base) {
        return Some((format!("{dir}/{stem}"), "vitest"));
    }
    None
}

fn has_node_vitest_mirror_pair(files: &[String]) -> bool {
    let mut kinds_by_key = BTreeMap::<String, HashSet<&'static str>>::new();
    for file in files {
        let Some((key, kind)) = test_mirror_entry(file) else {
            continue;
        };
        kinds_by_key.entry(key).or_default().insert(kind);
    }
    kinds_by_key
        .values()
        .any(|kinds| kinds.contains("node") && kinds.contains("vitest"))
}

fn classify_noise(group: &BlockCloneGroup) -> (&'static str, Option<&'static str>) {
    let mut files = group
        .instances
        .iter()
        .map(|instance| slash_path(&instance.file))
        .filter(|file| !file.is_empty())
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    if files.is_empty() {
        return ("review", None);
    }
    let all_test = files.iter().all(|file| is_test_file(file));
    if all_test && has_node_vitest_mirror_pair(&files) {
        return ("muted", Some("node-vitest-mirror-pair"));
    }
    if files.len() == 1 {
        return ("muted", Some("same-file-repeat"));
    }
    if all_test {
        return ("muted", Some("test-scaffold-repeat"));
    }
    ("review", None)
}

fn apply_noise_policy(groups: Vec<BlockCloneGroup>, thresholds: &Thresholds) -> NoisePolicyResult {
    let mut ranked_candidates = groups;
    ranked_candidates.sort_by(compare_groups);
    let candidate_cap_saturated = ranked_candidates.len() > thresholds.max_candidate_groups;
    ranked_candidates.truncate(thresholds.max_candidate_groups);

    let mut classified = ranked_candidates
        .into_iter()
        .map(|mut group| {
            let (visibility, mute_reason) = classify_noise(&group);
            group.visibility = Some(visibility.to_string());
            group.mute_reason = mute_reason.map(str::to_string);
            group
        })
        .collect::<Vec<_>>();

    let mut review = classified
        .iter()
        .filter(|group| group.visibility.as_deref() != Some("muted"))
        .cloned()
        .collect::<Vec<_>>();
    let mut muted = classified
        .drain(..)
        .filter(|group| group.visibility.as_deref() == Some("muted"))
        .collect::<Vec<_>>();
    review.sort_by(compare_groups);
    muted.sort_by(compare_groups);

    let review_cap_saturated = review.len() > thresholds.max_review_groups;
    let muted_cap_saturated = muted.len() > thresholds.max_muted_groups;
    review.truncate(thresholds.max_review_groups);
    muted.truncate(thresholds.max_muted_groups);

    if let Some(max_groups) = thresholds.max_groups {
        review.truncate(max_groups);
        let remaining = max_groups.saturating_sub(review.len());
        muted.truncate(remaining);
    }

    let mut muted_by_reason = BTreeMap::<String, usize>::new();
    for group in &muted {
        if let Some(reason) = &group.mute_reason {
            *muted_by_reason.entry(reason.clone()).or_insert(0) += 1;
        }
    }

    let review_group_count = review.len();
    let muted_group_count = muted.len();
    review.extend(muted);
    NoisePolicyResult {
        groups: review,
        review_group_count,
        muted_group_count,
        muted_by_reason,
        candidate_cap_saturated,
        review_cap_saturated,
        muted_cap_saturated,
    }
}

fn group_json(group: &BlockCloneGroup) -> Value {
    let mut object = Map::new();
    object.insert("id".to_string(), json!(group.id));
    object.insert("claim".to_string(), json!(group.claim));
    object.insert("confidence".to_string(), json!(group.confidence));
    object.insert("tokenCount".to_string(), json!(group.token_count));
    object.insert("lineCount".to_string(), json!(group.line_count));
    object.insert("occurrenceCount".to_string(), json!(group.occurrence_count));
    object.insert(
        "normalizationMode".to_string(),
        json!(group.normalization_mode),
    );
    object.insert("reasons".to_string(), json!(group.reasons));
    object.insert(
        "instances".to_string(),
        Value::Array(group.instances.iter().map(instance_json).collect()),
    );
    object.insert("reviewOnly".to_string(), json!(group.review_only));
    object.insert(
        "eligibleForSafeFix".to_string(),
        json!(group.eligible_for_safe_fix),
    );
    if let Some(visibility) = &group.visibility {
        object.insert("visibility".to_string(), json!(visibility));
    }
    if let Some(reason) = &group.mute_reason {
        object.insert("muteReason".to_string(), json!(reason));
    }
    Value::Object(object)
}

fn instance_json(instance: &Instance) -> Value {
    json!({
        "file": instance.file,
        "startLine": instance.start_line,
        "endLine": instance.end_line,
        "startToken": instance.start_token,
        "endToken": instance.end_token,
        "container": instance.container.clone().unwrap_or(Value::Null),
    })
}

fn slash_path(value: &str) -> String {
    value.replace('\\', "/")
}

fn posix_basename(path: &str) -> &str {
    path.rsplit_once('/').map(|(_, base)| base).unwrap_or(path)
}

fn posix_dirname(path: &str) -> &str {
    path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or(".")
}

fn strip_js_extension(base: &str) -> String {
    const EXTENSIONS: &[&str] = &[
        ".mjsx", ".mtsx", ".cjsx", ".ctsx", ".jsx", ".tsx", ".mjs", ".mts", ".cjs", ".cts", ".js",
        ".ts",
    ];
    for extension in EXTENSIONS {
        if let Some(stripped) = base.strip_suffix(extension) {
            return stripped.to_string();
        }
    }
    base.to_string()
}

fn strip_test_spec_extension(base: &str) -> Option<String> {
    for marker in [".test.", ".spec."] {
        let Some(index) = base.rfind(marker) else {
            continue;
        };
        let extension = &base[index + marker.len()..];
        if is_js_extension(extension) {
            return Some(base[..index].to_string());
        }
    }
    None
}

fn is_js_extension(extension: &str) -> bool {
    matches!(
        extension,
        "js" | "jsx"
            | "ts"
            | "tsx"
            | "mjs"
            | "mjsx"
            | "mts"
            | "mtsx"
            | "cjs"
            | "cjsx"
            | "cts"
            | "ctsx"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(value: &str, file: &str, start: usize, line: usize) -> BlockCloneToken {
        BlockCloneToken {
            value: value.to_string(),
            file: file.to_string(),
            start,
            end: start + 1,
            line,
            end_line: line,
            container: None,
        }
    }

    fn file(rel_file: &str, values: &[&str]) -> TokenizedFile {
        TokenizedFile {
            rel_file: rel_file.to_string(),
            tokens: values
                .iter()
                .enumerate()
                .map(|(index, value)| token(value, rel_file, index, index + 1))
                .collect(),
            skipped: None,
            diagnostics: vec![],
            token_limit_exceeded: false,
        }
    }

    fn request(files: Vec<TokenizedFile>, thresholds: Value) -> BlockClonesRequest {
        BlockClonesRequest {
            schema_version: BLOCK_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-04T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            files,
            thresholds: Some(thresholds),
            incremental: Some(json!({
                "enabled": false,
                "reason": "disabled-by-flag",
            })),
        }
    }

    #[test]
    fn builds_review_group_from_js_tokenized_files() -> Result<()> {
        let artifact = build_block_clones_artifact(request(
            vec![
                file("src/a.ts", &["A", "B", "C", "D", "E", "F"]),
                file("src/b.ts", &["A", "B", "C", "D", "E", "F"]),
            ],
            json!({
                "minTokens": 3,
                "minLines": 1,
                "minOccurrences": 2,
                "maxInstancesPerGroup": 20,
                "maxCandidateGroups": 100,
                "maxReviewGroups": 100,
                "maxMutedGroups": 100,
                "maxTokensPerFile": 200000,
            }),
        ))?;

        assert_eq!(artifact["schemaVersion"], "block-clones.v1");
        assert_eq!(artifact["policyVersion"], "block-clone-review-policy-v1");
        assert_eq!(artifact["summary"]["reviewGroupCount"], 1);
        assert_eq!(artifact["summary"]["mutedGroupCount"], 0);
        assert_eq!(artifact["groups"][0]["visibility"], "review");
        assert_eq!(
            artifact["groups"][0]["instances"]
                .as_array()
                .map_or(0, Vec::len),
            2
        );
        Ok(())
    }

    #[test]
    fn mutes_same_file_repeats_without_deleting_group() -> Result<()> {
        let artifact = build_block_clones_artifact(request(
            vec![file(
                "src/a.ts",
                &[
                    "A", "B", "C", "D", "E", "F", "X", "A", "B", "C", "D", "E", "F",
                ],
            )],
            json!({
                "minTokens": 3,
                "minLines": 1,
                "minOccurrences": 2,
                "maxInstancesPerGroup": 20,
                "maxCandidateGroups": 100,
                "maxReviewGroups": 100,
                "maxMutedGroups": 100,
                "maxTokensPerFile": 200000,
            }),
        ))?;

        assert_eq!(artifact["summary"]["reviewGroupCount"], 0);
        assert_eq!(artifact["summary"]["mutedGroupCount"], 1);
        assert_eq!(artifact["groups"][0]["visibility"], "muted");
        assert_eq!(artifact["groups"][0]["muteReason"], "same-file-repeat");
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() {
        let mut request = request(vec![], json!({}));
        request.schema_version = "block-clones.future".to_string();
        let error = match build_block_clones_artifact(request) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("unsupported schemaVersion"));
    }
}
