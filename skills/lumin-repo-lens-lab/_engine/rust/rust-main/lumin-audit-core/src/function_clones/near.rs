use super::facts::FunctionFact;
use super::projection::{line_json, sort_near_candidates, sorted_unique};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashSet};

const FUNCTION_CLONE_NEAR_POLICY_ID: &str = "function-clone-near-policy";
const FUNCTION_CLONE_NEAR_POLICY_VERSION: &str = "function-clone-near-policy-v1";
const FUNCTION_CLONE_NEAR_POLICY_HASH: &str =
    "sha256:e9a00930929d31f6f15c473af90ded820942b855e6f860a8004c888b9bbdf2ec";
const FUNCTION_CLONE_NEAR_THRESHOLD_HASH: &str =
    "sha256:ba963d4a06d50a37633a99576aeda79230ad8870878802ac66942d82cf9459da";

pub(super) const MIN_BODY_LOC_FOR_GROUPING: usize = 3;
pub(super) const MIN_STATEMENTS_FOR_GROUPING: usize = 2;
pub(super) const MIN_GROUP_SIZE: usize = 2;
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

pub(super) fn build_near_function_candidates(
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

pub(super) fn function_clone_near_policy_summary() -> Value {
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
