use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub const FUNCTION_CLONES_REQUEST_SCHEMA_VERSION: &str =
    "lumin-function-clones-producer-request.v1";

const FUNCTION_CLONE_SCHEMA_VERSION: &str = "function-clones.v3";
const FUNCTION_CLONE_NORMALIZED_VERSION: &str = "function-body.normalized.v1";
const FUNCTION_SIGNATURE_NORMALIZED_VERSION: &str = "function-signature.normalized.v1";
const FUNCTION_CLONE_NEAR_POLICY_ID: &str = "function-clone-near-policy";
const FUNCTION_CLONE_NEAR_POLICY_VERSION: &str = "function-clone-near-policy-v1";
const FUNCTION_CLONE_NEAR_POLICY_HASH: &str =
    "sha256:e9a00930929d31f6f15c473af90ded820942b855e6f860a8004c888b9bbdf2ec";
const FUNCTION_CLONE_NEAR_THRESHOLD_HASH: &str =
    "sha256:ba963d4a06d50a37633a99576aeda79230ad8870878802ac66942d82cf9459da";

const MIN_BODY_LOC_FOR_GROUPING: usize = 3;
const MIN_STATEMENTS_FOR_GROUPING: usize = 2;
const MIN_GROUP_SIZE: usize = 2;
const MAX_PARAM_COUNT_DELTA: i64 = 1;
const MIN_BODY_LOC_SIMILARITY: f64 = 0.34;
const MIN_STATEMENT_COUNT_SIMILARITY: f64 = 0.34;
const MIN_CALL_TOKEN_JACCARD: f64 = 0.5;
const MIN_NAME_TOKEN_JACCARD_FALLBACK: f64 = 0.34;
const MIN_NEAR_SCORE: f64 = 0.62;
const MAX_NEAR_CANDIDATES: usize = 50;
const CALL_TOKEN_JACCARD_WEIGHT: f64 = 0.45;
const NAME_TOKEN_JACCARD_WEIGHT: f64 = 0.25;
const BODY_LOC_SIMILARITY_WEIGHT: f64 = 0.15;
const STATEMENT_COUNT_SIMILARITY_WEIGHT: f64 = 0.15;

const GENERIC_CALL_TOKENS: &[&str] = &[
    "apply", "bind", "call", "catch", "filter", "find", "forEach", "format", "includes", "join",
    "map", "push", "reduce", "slice", "split", "then", "toString", "trim",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionClonesRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<Value>,
    pub scope: String,
    #[serde(default)]
    pub observed_at: Option<String>,
    pub file_count: usize,
    #[serde(default)]
    pub facts: Vec<Value>,
    #[serde(default)]
    pub diagnostics: Vec<Value>,
    #[serde(default)]
    pub files_with_parse_errors: Vec<Value>,
    #[serde(default)]
    pub files_with_read_errors: Vec<Value>,
    #[serde(default)]
    pub incremental: Option<Value>,
}

#[derive(Debug, Clone)]
struct FunctionFact {
    value: Value,
    identity: String,
    owner_file: String,
    exported_name: String,
    visibility: String,
    line: i64,
    body_loc: i64,
    statement_count: i64,
    normalized_exact_hash: String,
    normalized_structure_hash: String,
    normalized_signature_hash: Option<String>,
    signature: Option<Value>,
    generated_file: bool,
    call_tokens: Vec<String>,
    generator: bool,
    async_value: bool,
    param_count: i64,
}

pub fn build_function_clones_artifact(request: FunctionClonesRequest) -> Result<Value> {
    if request.schema_version != FUNCTION_CLONES_REQUEST_SCHEMA_VERSION {
        bail!(
            "function-clones-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let observed_at = request
        .observed_at
        .clone()
        .unwrap_or_else(|| request.generated.clone());
    let mut stamped_facts = request
        .facts
        .into_iter()
        .map(|fact| FunctionFact::from_value(stamp_observed_at(fact, &observed_at)))
        .collect::<Vec<_>>();

    stamped_facts.sort_by(compare_facts);
    let sorted_diagnostics = sort_diagnostics(request.diagnostics);

    let exact_body_groups = group_facts(
        &stamped_facts,
        GroupKey::NormalizedExact,
        1,
        1,
        MIN_GROUP_SIZE,
    );
    let structure_groups = group_facts(
        &stamped_facts,
        GroupKey::NormalizedStructure,
        MIN_BODY_LOC_FOR_GROUPING,
        MIN_STATEMENTS_FOR_GROUPING,
        MIN_GROUP_SIZE,
    );
    let signature_groups = group_signature_facts(&stamped_facts, MIN_GROUP_SIZE);
    let near_function_candidates =
        build_near_function_candidates(&stamped_facts, &exact_body_groups, &structure_groups);
    let generated_file_fact_count = stamped_facts
        .iter()
        .filter(|fact| fact.generated_file)
        .count();

    let mut meta = Map::new();
    meta.insert("tool".to_string(), json!("build-function-clone-index.mjs"));
    meta.insert("generated".to_string(), json!(request.generated));
    meta.insert("root".to_string(), json!(request.root));
    meta.insert("source".to_string(), json!("fresh-ast-pass"));
    meta.insert("scope".to_string(), json!(request.scope));
    meta.insert("observedAt".to_string(), json!(observed_at));
    meta.insert(
        "complete".to_string(),
        json!(
            request.files_with_read_errors.is_empty() && request.files_with_parse_errors.is_empty()
        ),
    );
    meta.insert("includeTests".to_string(), json!(request.include_tests));
    meta.insert("exclude".to_string(), Value::Array(request.exclude));
    meta.insert("fileCount".to_string(), json!(request.file_count));
    meta.insert("factCount".to_string(), json!(stamped_facts.len()));
    meta.insert(
        "generatedFileFactCount".to_string(),
        json!(generated_file_fact_count),
    );
    meta.insert(
        "exactBodyGroupCount".to_string(),
        json!(non_generated_count(&exact_body_groups)),
    );
    meta.insert(
        "structureGroupCount".to_string(),
        json!(non_generated_count(&structure_groups)),
    );
    meta.insert(
        "signatureGroupCount".to_string(),
        json!(non_generated_count(&signature_groups)),
    );
    meta.insert(
        "nearFunctionCandidateCount".to_string(),
        json!(non_generated_count(&near_function_candidates)),
    );
    meta.insert(
        "diagnosticCount".to_string(),
        json!(sorted_diagnostics.len()),
    );
    meta.insert(
        "filesWithParseErrors".to_string(),
        Value::Array(request.files_with_parse_errors),
    );
    meta.insert(
        "filesWithReadErrors".to_string(),
        Value::Array(request.files_with_read_errors),
    );
    meta.insert(
        "thresholdPolicies".to_string(),
        Value::Array(vec![function_clone_near_policy_summary()]),
    );
    if let Some(incremental) = request.incremental {
        meta.insert("incremental".to_string(), incremental);
    }
    meta.insert(
        "supports".to_string(),
        json!({
            "exportedTopLevelFunctions": true,
            "fileLocalTopLevelFunctions": true,
            "functionFactVisibility": true,
            "exportedConstArrowFunctions": true,
            "defaultFunctionExports": true,
            "exactBodyHash": true,
            "normalizedExactHash": true,
            "normalizedStructureHash": true,
            "normalizedVersion": FUNCTION_CLONE_NORMALIZED_VERSION,
            "normalizedFunctionSignatureHash": true,
            "functionSignatureGroups": true,
            "functionSignatureNormalizedVersion": FUNCTION_SIGNATURE_NORMALIZED_VERSION,
            "nearFunctionCandidates": true,
            "generatedFileEvidence": true,
            "semanticEquivalence": false,
        }),
    );
    meta.insert(
        "caveat".to_string(),
        json!("Function clone groups and near candidates are deterministic review cues. They do not prove semantic equivalence or justify automatic merging."),
    );

    Ok(json!({
        "schemaVersion": FUNCTION_CLONE_SCHEMA_VERSION,
        "meta": meta,
        "facts": stamped_facts.into_iter().map(|fact| fact.value).collect::<Vec<_>>(),
        "exactBodyGroups": exact_body_groups,
        "structureGroups": structure_groups,
        "signatureGroups": signature_groups,
        "nearFunctionCandidates": near_function_candidates,
        "diagnostics": sorted_diagnostics,
    }))
}

impl FunctionFact {
    fn from_value(value: Value) -> Self {
        Self {
            identity: string_field(&value, "identity"),
            owner_file: string_field(&value, "ownerFile"),
            exported_name: string_field(&value, "exportedName"),
            visibility: string_field(&value, "visibility")
                .if_empty_then("exported")
                .to_string(),
            line: i64_field(&value, "line"),
            body_loc: i64_field(&value, "bodyLoc"),
            statement_count: i64_field(&value, "statementCount"),
            normalized_exact_hash: string_field(&value, "normalizedExactHash"),
            normalized_structure_hash: string_field(&value, "normalizedStructureHash"),
            normalized_signature_hash: optional_string_field(&value, "normalizedSignatureHash"),
            signature: value.get("signature").cloned(),
            generated_file: truthy_field(&value, "generatedFile"),
            call_tokens: string_array_field(&value, "callTokens"),
            generator: bool_field(&value, "generator"),
            async_value: bool_field(&value, "async"),
            param_count: i64_field(&value, "paramCount"),
            value,
        }
    }

    fn hash_for(&self, key: GroupKey) -> &str {
        match key {
            GroupKey::NormalizedExact => &self.normalized_exact_hash,
            GroupKey::NormalizedStructure => &self.normalized_structure_hash,
        }
    }
}

trait EmptyStringDefault {
    fn if_empty_then<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl EmptyStringDefault for String {
    fn if_empty_then<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.is_empty() {
            fallback
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GroupKey {
    NormalizedExact,
    NormalizedStructure,
}

fn stamp_observed_at(mut value: Value, observed_at: &str) -> Value {
    if let Value::Object(object) = &mut value {
        object.insert("observedAt".to_string(), json!(observed_at));
    }
    value
}

fn compare_facts(left: &FunctionFact, right: &FunctionFact) -> Ordering {
    left.owner_file
        .cmp(&right.owner_file)
        .then_with(|| left.line.cmp(&right.line))
        .then_with(|| left.exported_name.cmp(&right.exported_name))
        .then_with(|| left.identity.cmp(&right.identity))
}

fn sort_diagnostics(mut diagnostics: Vec<Value>) -> Vec<Value> {
    diagnostics.sort_by(|left, right| {
        string_field(left, "file")
            .cmp(&string_field(right, "file"))
            .then_with(|| string_field(left, "code").cmp(&string_field(right, "code")))
            .then_with(|| string_value(left.get("line")).cmp(&string_value(right.get("line"))))
            .then_with(|| string_field(left, "message").cmp(&string_field(right, "message")))
    });
    diagnostics
}

fn group_facts(
    facts: &[FunctionFact],
    key: GroupKey,
    min_body_loc: usize,
    min_statements: usize,
    min_size: usize,
) -> Vec<Value> {
    let mut by_hash = BTreeMap::<String, Vec<&FunctionFact>>::new();
    for fact in facts {
        let hash = fact.hash_for(key);
        if hash.is_empty() {
            continue;
        }
        if fact.body_loc < min_body_loc as i64 || fact.statement_count < min_statements as i64 {
            continue;
        }
        by_hash.entry(hash.to_string()).or_default().push(fact);
    }

    let mut groups = Vec::<Value>::new();
    for (group_hash, members) in by_hash {
        if members.len() < min_size {
            continue;
        }
        let mut sorted = members;
        sorted.sort_by(|left, right| left.identity.cmp(&right.identity));
        let generated_only = sorted.iter().all(|fact| fact.generated_file);
        let exact_hash_count = sorted
            .iter()
            .map(|fact| fact.normalized_exact_hash.clone())
            .collect::<BTreeSet<_>>()
            .len();
        let shared_call_tokens = shared_call_tokens(&sorted);
        let body_loc_values = sorted.iter().map(|fact| fact.body_loc).collect::<Vec<_>>();

        groups.push(json!({
            "hash": group_hash,
            "size": sorted.len(),
            "generatedOnly": generated_only,
            "exactHashCount": exact_hash_count,
            "identities": sorted.iter().map(|fact| fact.identity.clone()).collect::<Vec<_>>(),
            "ownerFiles": sorted_unique(sorted.iter().map(|fact| fact.owner_file.clone())),
            "exportedNames": sorted_unique(sorted.iter().map(|fact| fact.exported_name.clone())),
            "visibilities": sorted_unique(sorted.iter().map(|fact| fact.visibility.clone())),
            "lines": sorted.iter().copied().map(line_json).collect::<Vec<_>>(),
            "bodyLocRange": [
                body_loc_values.iter().min().copied().unwrap_or(0),
                body_loc_values.iter().max().copied().unwrap_or(0),
            ],
            "sharedCallTokens": shared_call_tokens,
            "reason": match key {
                GroupKey::NormalizedExact => "same normalized function body; verify domain ownership before merging",
                GroupKey::NormalizedStructure => "same anonymized function-body structure; review cue only, not proof of semantic equivalence",
            },
        }));
    }

    sort_clone_groups(&mut groups, true);
    groups
}

fn group_signature_facts(facts: &[FunctionFact], min_size: usize) -> Vec<Value> {
    let mut by_hash = BTreeMap::<String, Vec<&FunctionFact>>::new();
    for fact in facts {
        let Some(hash) = &fact.normalized_signature_hash else {
            continue;
        };
        if hash.is_empty() {
            continue;
        }
        by_hash.entry(hash.clone()).or_default().push(fact);
    }

    let mut groups = Vec::<Value>::new();
    for (signature_hash, members) in by_hash {
        if members.len() < min_size {
            continue;
        }
        let mut sorted = members;
        sorted.sort_by(|left, right| left.identity.cmp(&right.identity));
        let generated_only = sorted.iter().all(|fact| fact.generated_file);
        let visibilities = sorted_unique(sorted.iter().map(|fact| fact.visibility.clone()));
        let has_file_local = visibilities
            .iter()
            .any(|visibility| visibility == "file-local");
        groups.push(json!({
            "kind": "function-signature-group",
            "hash": signature_hash,
            "size": sorted.len(),
            "generatedOnly": generated_only,
            "risk": "review-only",
            "signature": sorted.first().and_then(|fact| fact.signature.clone()).unwrap_or(Value::Null),
            "identities": sorted.iter().map(|fact| fact.identity.clone()).collect::<Vec<_>>(),
            "ownerFiles": sorted_unique(sorted.iter().map(|fact| fact.owner_file.clone())),
            "exportedNames": sorted_unique(sorted.iter().map(|fact| fact.exported_name.clone())),
            "visibilities": visibilities,
            "lines": sorted.iter().copied().map(line_json).collect::<Vec<_>>(),
            "reason": if has_file_local {
                "same normalized function type signature; file-local helpers are review cues only; not import/reuse proof or a merge recommendation"
            } else {
                "same normalized exported function type signature; review cue only; not proof of semantic equivalence or a merge recommendation"
            },
        }));
    }

    sort_clone_groups(&mut groups, false);
    groups
}

fn build_near_function_candidates(
    facts: &[FunctionFact],
    exact_body_groups: &[Value],
    structure_groups: &[Value],
) -> Vec<Value> {
    let grouped = grouped_identity_set([exact_body_groups, structure_groups]);
    let mut eligible = facts
        .iter()
        .filter(|fact| !grouped.contains(&fact.identity))
        .filter(|fact| !significant_call_tokens(fact).is_empty())
        .filter(|fact| !fact.generator)
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.identity.cmp(&right.identity));

    let mut by_call_token = BTreeMap::<String, Vec<&FunctionFact>>::new();
    for fact in eligible {
        for token in significant_call_tokens(fact) {
            by_call_token.entry(token).or_default().push(fact);
        }
    }

    let mut pair_keys = HashSet::<String>::new();
    let mut candidates = Vec::<Value>::new();
    for bucket in by_call_token.values_mut() {
        bucket.sort_by(|left, right| left.identity.cmp(&right.identity));
        for i in 0..bucket.len() {
            for j in (i + 1)..bucket.len() {
                let a = bucket[i];
                let b = bucket[j];
                let pair_key = ordered_pair_key(&a.identity, &b.identity);
                if !pair_keys.insert(pair_key) {
                    continue;
                }
                if a.async_value != b.async_value {
                    continue;
                }
                if (a.param_count - b.param_count).abs() > MAX_PARAM_COUNT_DELTA {
                    continue;
                }

                let a_calls = significant_call_tokens(a);
                let b_calls = significant_call_tokens(b);
                let shared_call_tokens = shared_sorted(&a_calls, &b_calls);
                if shared_call_tokens.is_empty() {
                    continue;
                }

                let call_token_jaccard = jaccard(&a_calls, &b_calls);
                let a_name_tokens = name_tokens(&a.exported_name);
                let b_name_tokens = name_tokens(&b.exported_name);
                let shared_name_tokens = shared_sorted(&a_name_tokens, &b_name_tokens);
                let name_token_jaccard = jaccard(&a_name_tokens, &b_name_tokens);
                let body_loc_similarity = range_similarity(a.body_loc, b.body_loc);
                let statement_count_similarity =
                    range_similarity(a.statement_count, b.statement_count);
                if body_loc_similarity < MIN_BODY_LOC_SIMILARITY
                    || statement_count_similarity < MIN_STATEMENT_COUNT_SIMILARITY
                {
                    continue;
                }
                if call_token_jaccard < MIN_CALL_TOKEN_JACCARD
                    && name_token_jaccard < MIN_NAME_TOKEN_JACCARD_FALLBACK
                {
                    continue;
                }

                let score = round_score(
                    (call_token_jaccard * CALL_TOKEN_JACCARD_WEIGHT)
                        + (name_token_jaccard * NAME_TOKEN_JACCARD_WEIGHT)
                        + (body_loc_similarity * BODY_LOC_SIMILARITY_WEIGHT)
                        + (statement_count_similarity * STATEMENT_COUNT_SIMILARITY_WEIGHT),
                );
                if score < MIN_NEAR_SCORE {
                    continue;
                }

                let mut sorted = [a, b];
                sorted.sort_by(|left, right| left.identity.cmp(&right.identity));
                let mut reasons = vec![
                    format!(
                        "shared significant call tokens: {}",
                        shared_call_tokens.join(", ")
                    ),
                    format!(
                        "body size similarity: {}",
                        number_string(round_score(body_loc_similarity))
                    ),
                    format!(
                        "statement-count similarity: {}",
                        number_string(round_score(statement_count_similarity))
                    ),
                ];
                if !shared_name_tokens.is_empty() {
                    reasons.push(format!(
                        "shared exported-name tokens: {}",
                        shared_name_tokens.join(", ")
                    ));
                }
                let mut exported_names = sorted
                    .iter()
                    .map(|fact| fact.exported_name.clone())
                    .collect::<Vec<_>>();
                exported_names.sort();
                candidates.push(json!({
                    "kind": "near-function-candidate",
                    "identities": sorted.iter().map(|fact| fact.identity.clone()).collect::<Vec<_>>(),
                    "ownerFiles": sorted_unique(sorted.iter().map(|fact| fact.owner_file.clone())),
                    "exportedNames": exported_names,
                    "lines": sorted.iter().copied().map(line_json).collect::<Vec<_>>(),
                    "score": score,
                    "risk": "review-only",
                    "generatedOnly": sorted.iter().all(|fact| fact.generated_file),
                    "sharedCallTokens": shared_call_tokens,
                    "sharedNameTokens": shared_name_tokens,
                    "callTokenJaccard": round_score(call_token_jaccard),
                    "nameTokenJaccard": round_score(name_token_jaccard),
                    "bodyLocRange": [
                        sorted.iter().map(|fact| fact.body_loc).min().unwrap_or(0),
                        sorted.iter().map(|fact| fact.body_loc).max().unwrap_or(0),
                    ],
                    "statementCountRange": [
                        sorted.iter().map(|fact| fact.statement_count).min().unwrap_or(0),
                        sorted.iter().map(|fact| fact.statement_count).max().unwrap_or(0),
                    ],
                    "reasons": reasons,
                    "reason": "near function cue only; source review required; not proof of semantic equivalence or an automatic merge",
                }));
            }
        }
    }

    sort_near_candidates(&mut candidates);
    candidates.truncate(MAX_NEAR_CANDIDATES);
    candidates
}

fn grouped_identity_set<'a>(
    groups_lists: impl IntoIterator<Item = &'a [Value]>,
) -> HashSet<String> {
    let mut out = HashSet::new();
    for groups in groups_lists {
        for group in groups {
            if let Some(identities) = group.get("identities").and_then(Value::as_array) {
                for identity in identities {
                    if let Some(identity) = identity.as_str() {
                        out.insert(identity.to_string());
                    }
                }
            }
        }
    }
    out
}

fn significant_call_tokens(fact: &FunctionFact) -> Vec<String> {
    let generic = GENERIC_CALL_TOKENS.iter().copied().collect::<HashSet<_>>();
    let mut tokens = fact
        .call_tokens
        .iter()
        .filter_map(|token| {
            let raw = token.as_str();
            if raw.len() >= 4 && !generic.contains(raw) {
                Some(raw.to_string())
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    tokens.sort();
    tokens
}

fn name_tokens(name: &str) -> Vec<String> {
    let mut expanded = String::new();
    let mut previous: Option<char> = None;
    for ch in name.chars() {
        if let Some(prev) = previous {
            if (prev.is_ascii_lowercase() || prev.is_ascii_digit()) && ch.is_ascii_uppercase() {
                expanded.push(' ');
            }
        }
        expanded.push(ch);
        previous = Some(ch);
    }
    expanded
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| token.len() >= 2)
        .collect()
}

fn jaccard(left: &[String], right: &[String]) -> f64 {
    let left = left.iter().collect::<HashSet<_>>();
    let right = right.iter().collect::<HashSet<_>>();
    let union = left.union(&right).count();
    if union == 0 {
        return 0.0;
    }
    let shared = left.intersection(&right).count();
    shared as f64 / union as f64
}

fn range_similarity(left: i64, right: i64) -> f64 {
    let max = left.max(right);
    if max <= 0 {
        return 0.0;
    }
    1.0 - ((left - right).abs() as f64 / max as f64)
}

fn round_score(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

fn number_string(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn ordered_pair_key(left: &str, right: &str) -> String {
    if left <= right {
        format!("{left}\0{right}")
    } else {
        format!("{right}\0{left}")
    }
}

fn shared_sorted(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<HashSet<_>>();
    let mut shared = left
        .iter()
        .filter(|entry| right.contains(entry))
        .cloned()
        .collect::<Vec<_>>();
    shared.sort();
    shared
}

fn shared_call_tokens(sorted: &[&FunctionFact]) -> Vec<String> {
    if sorted.is_empty() {
        return Vec::new();
    }
    let mut shared = sorted[0]
        .call_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    for fact in sorted.iter().skip(1) {
        let set = fact.call_tokens.iter().cloned().collect::<BTreeSet<_>>();
        shared = shared.intersection(&set).cloned().collect();
    }
    shared.into_iter().collect()
}

fn sort_clone_groups(groups: &mut [Value], body_loc_tiebreak: bool) {
    groups.sort_by(|left, right| {
        generated_rank(right)
            .cmp(&generated_rank(left))
            .then_with(|| usize_field(right, "size").cmp(&usize_field(left, "size")))
            .then_with(|| {
                if body_loc_tiebreak {
                    body_loc_range_max(right).cmp(&body_loc_range_max(left))
                } else {
                    Ordering::Equal
                }
            })
            .then_with(|| identities_join(left).cmp(&identities_join(right)))
    });
}

fn sort_near_candidates(candidates: &mut [Value]) {
    candidates.sort_by(|left, right| {
        generated_rank(right)
            .cmp(&generated_rank(left))
            .then_with(|| f64_field(right, "score").total_cmp(&f64_field(left, "score")))
            .then_with(|| identities_join(left).cmp(&identities_join(right)))
    });
}

fn generated_rank(value: &Value) -> usize {
    if bool_field(value, "generatedOnly") {
        0
    } else {
        1
    }
}

fn body_loc_range_max(value: &Value) -> i64 {
    value
        .get("bodyLocRange")
        .and_then(Value::as_array)
        .and_then(|values| values.get(1))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

fn identities_join(value: &Value) -> String {
    value
        .get("identities")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_default()
}

fn non_generated_count(groups: &[Value]) -> usize {
    groups
        .iter()
        .filter(|group| !bool_field(group, "generatedOnly"))
        .count()
}

fn line_json(fact: &FunctionFact) -> Value {
    json!({
        "identity": fact.identity,
        "file": fact.owner_file,
        "line": fact.line,
    })
}

fn sorted_unique(values: impl Iterator<Item = String>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().collect()
}

fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn optional_string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn string_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(value) => value.to_string(),
    }
}

fn i64_field(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn usize_field(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn truthy_field(value: &Value, key: &str) -> bool {
    match value.get(key) {
        Some(Value::Null) | None => false,
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => value.as_i64().unwrap_or(1) != 0,
        Some(Value::String(value)) => !value.is_empty(),
        Some(Value::Array(_)) | Some(Value::Object(_)) => true,
    }
}

fn f64_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    let mut values = value
        .get(key)
        .and_then(Value::as_array)
        .map(|tokens| {
            tokens
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    values.sort();
    values
}

fn function_clone_near_policy_summary() -> Value {
    json!({
        "schemaVersion": "threshold-policy.v1",
        "policyId": FUNCTION_CLONE_NEAR_POLICY_ID,
        "policyVersion": FUNCTION_CLONE_NEAR_POLICY_VERSION,
        "policyClass": "review",
        "policyHash": FUNCTION_CLONE_NEAR_POLICY_HASH,
        "thresholdHash": FUNCTION_CLONE_NEAR_THRESHOLD_HASH,
        "thresholds": {
            "minBodyLocForGrouping": MIN_BODY_LOC_FOR_GROUPING,
            "minStatementsForGrouping": MIN_STATEMENTS_FOR_GROUPING,
            "minGroupSize": MIN_GROUP_SIZE,
            "maxParamCountDelta": MAX_PARAM_COUNT_DELTA,
            "minBodyLocSimilarity": MIN_BODY_LOC_SIMILARITY,
            "minStatementCountSimilarity": MIN_STATEMENT_COUNT_SIMILARITY,
            "minCallTokenJaccard": MIN_CALL_TOKEN_JACCARD,
            "minNameTokenJaccardFallback": MIN_NAME_TOKEN_JACCARD_FALLBACK,
            "minNearScore": MIN_NEAR_SCORE,
            "maxNearCandidates": MAX_NEAR_CANDIDATES,
            "weights": {
                "callTokenJaccard": CALL_TOKEN_JACCARD_WEIGHT,
                "nameTokenJaccard": NAME_TOKEN_JACCARD_WEIGHT,
                "bodyLocSimilarity": BODY_LOC_SIMILARITY_WEIGHT,
                "statementCountSimilarity": STATEMENT_COUNT_SIMILARITY_WEIGHT,
            },
        },
        "calibration": {
            "corpus": "calibration-2026-05-prewrite-v1",
            "note": "agent-entry resolver calibration threshold contract",
        },
        "calibrationCorpus": {
            "schemaVersion": "calibration-corpus.v1",
            "corpusId": "calibration-2026-05-prewrite-v1",
            "purpose": "pre-write cue and threshold calibration",
            "status": "registry-anchor",
            "metrics": [
                "precisionProxy",
                "noiseRate",
                "runtimeMs",
                "suppressedCueRate",
            ],
            "entryCount": 3,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_function_clone_groups_from_js_facts() -> Result<()> {
        let artifact = build_function_clones_artifact(FunctionClonesRequest {
            schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            scope: "TS/JS including tests, top-level exported and file-local functions".to_string(),
            observed_at: None,
            file_count: 2,
            facts: vec![
                fact("src/a.ts", "alpha", 1, "exact-a", "structure-a", "sig-a"),
                fact("src/b.ts", "beta", 4, "exact-a", "structure-a", "sig-a"),
            ],
            diagnostics: vec![],
            files_with_parse_errors: vec![],
            files_with_read_errors: vec![],
            incremental: None,
        })?;

        assert_eq!(artifact["schemaVersion"], FUNCTION_CLONE_SCHEMA_VERSION);
        assert_eq!(artifact["meta"]["tool"], "build-function-clone-index.mjs");
        assert_eq!(artifact["meta"]["complete"], true);
        assert_eq!(artifact["meta"]["exactBodyGroupCount"], 1);
        assert_eq!(artifact["meta"]["structureGroupCount"], 1);
        assert_eq!(artifact["meta"]["signatureGroupCount"], 1);
        assert_eq!(
            artifact["facts"][0]["observedAt"],
            "2026-07-05T00:00:00.000Z"
        );
        assert_eq!(
            artifact["exactBodyGroups"][0]["identities"][0],
            "src/a.ts::alpha"
        );
        assert_eq!(
            artifact["signatureGroups"][0]["reason"],
            "same normalized exported function type signature; review cue only; not proof of semantic equivalence or a merge recommendation"
        );
        Ok(())
    }

    #[test]
    fn near_candidates_skip_already_grouped_facts_and_score_remaining_pairs() -> Result<()> {
        let artifact = build_function_clones_artifact(FunctionClonesRequest {
            schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            scope: "scope".to_string(),
            observed_at: None,
            file_count: 2,
            facts: vec![
                fact_with_calls(
                    "src/a.ts",
                    "loadUserAlpha",
                    1,
                    "exact-a",
                    "structure-a",
                    &["fetchUser", "parseBody"],
                ),
                fact_with_calls(
                    "src/b.ts",
                    "loadUserBeta",
                    8,
                    "exact-b",
                    "structure-b",
                    &["fetchUser", "parseBody"],
                ),
            ],
            diagnostics: vec![],
            files_with_parse_errors: vec![],
            files_with_read_errors: vec![],
            incremental: None,
        })?;

        assert_eq!(artifact["meta"]["nearFunctionCandidateCount"], 1);
        assert_eq!(
            artifact["nearFunctionCandidates"][0]["sharedCallTokens"][0],
            "fetchUser"
        );
        assert_eq!(artifact["nearFunctionCandidates"][0]["score"], 0.875);
        Ok(())
    }

    #[test]
    fn parse_or_read_errors_make_artifact_incomplete() -> Result<()> {
        let artifact = build_function_clones_artifact(FunctionClonesRequest {
            schema_version: FUNCTION_CLONES_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: false,
            exclude: vec![json!("dist")],
            scope: "scope".to_string(),
            observed_at: Some("2026-07-05T01:00:00.000Z".to_string()),
            file_count: 1,
            facts: vec![],
            diagnostics: vec![json!({
                "kind": "function-clone-diagnostic",
                "code": "parse-error",
                "severity": "error",
                "file": "bad.ts",
                "message": "bad",
            })],
            files_with_parse_errors: vec![json!({"file": "bad.ts", "message": "bad"})],
            files_with_read_errors: vec![],
            incremental: Some(json!({"enabled": true})),
        })?;

        assert_eq!(artifact["meta"]["complete"], false);
        assert_eq!(artifact["meta"]["includeTests"], false);
        assert_eq!(artifact["meta"]["exclude"][0], "dist");
        assert_eq!(artifact["meta"]["incremental"]["enabled"], true);
        assert_eq!(artifact["diagnostics"][0]["file"], "bad.ts");
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() {
        let error = match build_function_clones_artifact(FunctionClonesRequest {
            schema_version: "future".to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            scope: "scope".to_string(),
            observed_at: None,
            file_count: 0,
            facts: vec![],
            diagnostics: vec![],
            files_with_parse_errors: vec![],
            files_with_read_errors: vec![],
            incremental: None,
        }) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("unsupported schemaVersion"));
    }

    fn fact(
        file: &str,
        name: &str,
        line: i64,
        exact_hash: &str,
        structure_hash: &str,
        signature_hash: &str,
    ) -> Value {
        let mut value =
            fact_with_calls(file, name, line, exact_hash, structure_hash, &["fetchUser"]);
        if let Value::Object(object) = &mut value {
            object.insert("normalizedSignatureHash".to_string(), json!(signature_hash));
            object.insert("signature".to_string(), json!("fn(value)"));
        }
        value
    }

    fn fact_with_calls(
        file: &str,
        name: &str,
        line: i64,
        exact_hash: &str,
        structure_hash: &str,
        calls: &[&str],
    ) -> Value {
        json!({
            "kind": "function-body-fingerprint",
            "identity": format!("{file}::{name}"),
            "exportedName": name,
            "localName": name,
            "visibility": "exported",
            "exported": true,
            "ownerFile": file,
            "line": line,
            "endLine": line + 4,
            "bodyLineStart": line + 1,
            "bodyLineEnd": line + 3,
            "bodyLoc": 3,
            "declarationKind": "FunctionDeclaration",
            "functionKind": "FunctionDeclaration",
            "async": false,
            "generator": false,
            "paramCount": 1,
            "statementCount": 2,
            "exactBodyHash": format!("raw-{exact_hash}"),
            "normalizedExactHash": exact_hash,
            "normalizedStructureHash": structure_hash,
            "callTokens": calls,
            "source": "fresh-ast-pass",
            "scope": "scope",
            "confidence": "high",
        })
    }
}
