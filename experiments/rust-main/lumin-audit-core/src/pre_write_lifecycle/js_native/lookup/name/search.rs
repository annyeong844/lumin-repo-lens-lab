use super::*;

const NEAR_NAME_MAX_LENGTH_DELTA: usize = 2;
const NEAR_NAME_SHARED_PREFIX_MIN: usize = 4;
const NEAR_NAME_MAX_DISTANCE: usize = 2;
const SEMANTIC_HINT_MIN_SCORE: usize = 2;
const SEMANTIC_STOP_TOKENS: &[&str] = &[
    "a", "an", "and", "as", "at", "by", "for", "from", "in", "into", "of", "on", "or", "the",
    "this", "that", "to", "with", "add", "new", "helper", "function", "type", "file", "module",
    "service", "manager", "index", "main", "src", "lib", "utils", "util", "ts", "js", "mjs", "cjs",
    "tsx", "jsx",
];

const WEAK_COMMON_TOKENS: &[&str] = &[
    "action",
    "adapter",
    "api",
    "app",
    "application",
    "client",
    "command",
    "config",
    "context",
    "data",
    "domain",
    "event",
    "factory",
    "handler",
    "item",
    "manager",
    "model",
    "module",
    "option",
    "provider",
    "request",
    "response",
    "result",
    "service",
    "state",
    "store",
    "type",
    "util",
    "value",
];

#[derive(Debug, Clone)]

pub(super) struct SearchCandidate {
    name: String,
    owner_file: String,
    matched_field: &'static str,
    identity: Option<String>,
    definition_kind: Option<String>,
    class_name: Option<String>,
    member_kind: Option<String>,
    visibility: Option<String>,
    static_member: bool,
    line: Option<u64>,
}

pub(super) fn candidates(symbols: &Value) -> Vec<SearchCandidate> {
    let mut candidates = Vec::new();
    if let Some(files) = symbols.get("defIndex").and_then(Value::as_object) {
        for (file, definitions) in files {
            let Some(definitions) = definitions.as_object() else {
                continue;
            };
            for (name, definition) in definitions {
                candidates.push(SearchCandidate {
                    name: name.clone(),
                    owner_file: file.clone(),
                    matched_field: "defIndex",
                    identity: Some(format!("{file}::{name}")),
                    definition_kind: definition
                        .get("kind")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    class_name: None,
                    member_kind: None,
                    visibility: None,
                    static_member: false,
                    line: definition.get("line").and_then(Value::as_u64),
                });
            }
        }
    }
    if let Some(files) = symbols.get("classMethodIndex").and_then(Value::as_object) {
        for (file, methods) in files {
            let Some(methods) = methods.as_object() else {
                continue;
            };
            for (indexed_name, records) in methods {
                for record in records.as_array().into_iter().flatten() {
                    let name = record
                        .get("name")
                        .or_else(|| record.get("methodName"))
                        .and_then(Value::as_str)
                        .unwrap_or(indexed_name);
                    candidates.push(SearchCandidate {
                        name: name.to_string(),
                        owner_file: record
                            .get("ownerFile")
                            .and_then(Value::as_str)
                            .unwrap_or(file)
                            .to_string(),
                        matched_field: "classMethodIndex",
                        identity: record
                            .get("identity")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        definition_kind: None,
                        class_name: optional_string(record, "className"),
                        member_kind: optional_string(record, "memberKind"),
                        visibility: optional_string(record, "visibility"),
                        static_member: record.get("static").and_then(Value::as_bool) == Some(true),
                        line: record.get("line").and_then(Value::as_u64),
                    });
                }
            }
        }
    }
    candidates
}

pub(super) fn near_names(
    intent_name: &str,
    owner_hint: Option<&str>,
    candidates: &[SearchCandidate],
) -> (Vec<Value>, Vec<Value>, usize) {
    let mut matches = Vec::<(usize, Value)>::new();
    let mut suppressed = Vec::<(usize, usize, Value)>::new();
    for candidate in candidates {
        if candidate.name == intent_name && candidate.matched_field != "classMethodIndex" {
            continue;
        }
        let matched_tokens = common_tokens(intent_name, &candidate.name);
        let locality = locality(candidate, owner_hint);
        if !matched_tokens.is_empty() && matched_tokens.iter().all(|token| is_weak_token(token)) {
            let mut value = candidate_value(candidate);
            extend_object(
                &mut value,
                json!({
                    "matchedTokens": matched_tokens,
                    "reason": "domain-token-overlap",
                    "locality": locality,
                }),
            );
            suppressed.push((locality_rank(&value), usize::MAX, value));
            continue;
        }
        let prefix = shared_prefix(&candidate.name, intent_name);
        let length_delta = candidate.name.len().abs_diff(intent_name.len());
        if prefix >= NEAR_NAME_SHARED_PREFIX_MIN && length_delta <= intent_name.len() {
            let distance =
                levenshtein_capped(&candidate.name, intent_name, NEAR_NAME_MAX_DISTANCE * 4);
            let mut value = candidate_value(candidate);
            extend_object(&mut value, json!({ "distance": distance }));
            matches.push((distance, value));
            continue;
        }
        if length_delta > NEAR_NAME_MAX_LENGTH_DELTA {
            if !matched_tokens.is_empty() || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
                let mut value = candidate_value(candidate);
                extend_object(
                    &mut value,
                    json!({
                        "matchedTokens": matched_tokens,
                        "lengthDelta": length_delta,
                        "reason": "near-length-delta-exceeded",
                        "locality": locality,
                    }),
                );
                suppressed.push((locality_rank(&value), length_delta, value));
            }
            continue;
        }
        let distance = levenshtein_capped(&candidate.name, intent_name, NEAR_NAME_MAX_DISTANCE);
        if distance <= NEAR_NAME_MAX_DISTANCE {
            let mut value = candidate_value(candidate);
            extend_object(&mut value, json!({ "distance": distance }));
            matches.push((distance, value));
        } else if !matched_tokens.is_empty() || prefix >= NEAR_NAME_SHARED_PREFIX_MIN {
            let mut value = candidate_value(candidate);
            extend_object(
                &mut value,
                json!({
                    "matchedTokens": matched_tokens,
                    "distance": distance,
                    "reason": "near-distance-exceeded",
                    "locality": locality,
                }),
            );
            suppressed.push((locality_rank(&value), distance, value));
        }
    }
    matches.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| {
                string_at(&left.1, "matchedField").cmp(string_at(&right.1, "matchedField"))
            })
            .then_with(|| string_at(&left.1, "name").cmp(string_at(&right.1, "name")))
            .then_with(|| string_at(&left.1, "ownerFile").cmp(string_at(&right.1, "ownerFile")))
    });
    suppressed.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| string_at(&left.2, "name").cmp(string_at(&right.2, "name")))
            .then_with(|| string_at(&left.2, "ownerFile").cmp(string_at(&right.2, "ownerFile")))
    });
    let suppressed_count = suppressed.len();
    let capped_suppressed = suppressed
        .into_iter()
        .take(RESULT_CAP)
        .map(|(_, _, mut value)| {
            value["candidateCount"] = json!(suppressed_count);
            value
        })
        .collect();
    (
        matches
            .into_iter()
            .take(RESULT_CAP)
            .map(|(_, value)| value)
            .collect(),
        capped_suppressed,
        suppressed_count,
    )
}

pub(super) fn semantic_hints(
    query_tokens: &[String],
    owner_hint: Option<&str>,
    candidates: &[SearchCandidate],
) -> (Vec<Value>, Vec<Value>, usize) {
    let query = query_tokens.iter().cloned().collect::<BTreeSet<_>>();
    let mut matches = Vec::new();
    let mut suppressed = Vec::new();
    for candidate in candidates {
        let name_tokens = unique_tokens(&[Some(candidate.name.as_str())]);
        let support_tokens = unique_tokens(&[
            candidate.definition_kind.as_deref(),
            candidate.class_name.as_deref(),
            candidate.member_kind.as_deref(),
        ]);
        let candidate_tokens = name_tokens
            .iter()
            .chain(&support_tokens)
            .cloned()
            .collect::<BTreeSet<_>>();
        let matched = candidate_tokens
            .intersection(&query)
            .cloned()
            .collect::<Vec<_>>();
        if matched.is_empty() {
            continue;
        }
        let matched_name = name_tokens
            .iter()
            .filter(|token| query.contains(*token))
            .cloned()
            .collect::<Vec<_>>();
        let strong_name = matched_name
            .iter()
            .filter(|token| !is_weak_token(token))
            .cloned()
            .collect::<Vec<_>>();
        let strong_support = support_tokens
            .iter()
            .filter(|token| {
                query.contains(*token) && !is_weak_token(token) && !strong_name.contains(token)
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut value = candidate_value(candidate);
        let score = matched.len();
        extend_object(
            &mut value,
            json!({
                "matchedTokens": matched,
                "matchedNameTokens": matched_name,
                "matchedSupportTokens": strong_support,
                "score": score,
                "locality": locality(candidate, owner_hint),
            }),
        );
        if score < SEMANTIC_HINT_MIN_SCORE
            || !(strong_name.len() >= 2 || (strong_name.len() == 1 && !strong_support.is_empty()))
        {
            let reason = if value["matchedTokens"].as_array().is_some_and(|tokens| {
                tokens
                    .iter()
                    .all(|token| token.as_str().is_some_and(is_weak_token))
            }) {
                "domain-token-overlap"
            } else if score < SEMANTIC_HINT_MIN_SCORE {
                "single-non-weak-token-only"
            } else {
                "insufficient-non-weak-support"
            };
            value["reason"] = json!(reason);
            suppressed.push(value);
        } else {
            matches.push(value);
        }
    }
    let sort_values = |values: &mut Vec<Value>| {
        values.sort_by(|left, right| {
            locality_rank(right)
                .cmp(&locality_rank(left))
                .then_with(|| {
                    right
                        .get("score")
                        .and_then(Value::as_u64)
                        .unwrap_or(0)
                        .cmp(&left.get("score").and_then(Value::as_u64).unwrap_or(0))
                })
                .then_with(|| string_at(left, "name").cmp(string_at(right, "name")))
                .then_with(|| string_at(left, "ownerFile").cmp(string_at(right, "ownerFile")))
        });
    };
    sort_values(&mut matches);
    sort_values(&mut suppressed);
    let suppressed_count = suppressed.len();
    for value in &mut suppressed {
        value["candidateCount"] = json!(suppressed_count);
    }
    (
        matches.into_iter().take(RESULT_CAP).collect(),
        suppressed.into_iter().take(RESULT_CAP).collect(),
        suppressed_count,
    )
}

fn candidate_value(candidate: &SearchCandidate) -> Value {
    let mut object = Map::new();
    object.insert("name".to_string(), json!(candidate.name));
    object.insert("ownerFile".to_string(), json!(candidate.owner_file));
    object.insert("matchedField".to_string(), json!(candidate.matched_field));
    insert_option(&mut object, "identity", candidate.identity.as_deref());
    insert_option(
        &mut object,
        "definitionKind",
        candidate.definition_kind.as_deref(),
    );
    insert_option(&mut object, "className", candidate.class_name.as_deref());
    insert_option(&mut object, "memberKind", candidate.member_kind.as_deref());
    insert_option(&mut object, "visibility", candidate.visibility.as_deref());
    if candidate.matched_field == "classMethodIndex" {
        object.insert("exportedName".to_string(), json!(candidate.name));
    }
    if candidate.static_member {
        object.insert("static".to_string(), json!(true));
    }
    if let Some(line) = candidate.line {
        object.insert("line".to_string(), json!(line));
    }
    Value::Object(object)
}

pub(super) fn unique_tokens(parts: &[Option<&str>]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut tokens = Vec::new();
    for part in parts.iter().flatten() {
        for token in tokenize(part) {
            if token.len() >= 2
                && !SEMANTIC_STOP_TOKENS.contains(&token.as_str())
                && seen.insert(token.clone())
            {
                tokens.push(token);
            }
        }
    }
    tokens
}

fn tokenize(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars = value.chars().collect::<Vec<_>>();
    for (index, ch) in chars.iter().copied().enumerate() {
        let boundary = if index == 0 {
            false
        } else {
            let previous = chars[index - 1];
            (ch.is_ascii_uppercase()
                && (previous.is_ascii_lowercase() || previous.is_ascii_digit()))
                || (!ch.is_ascii_alphanumeric() && !current.is_empty())
        };
        if boundary && !current.is_empty() {
            tokens.push(normalize_token(&current));
            current.clear();
        }
        if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        }
    }
    if !current.is_empty() {
        tokens.push(normalize_token(&current));
    }
    tokens
        .into_iter()
        .filter(|token| !token.is_empty())
        .collect()
}

fn normalize_token(token: &str) -> String {
    match token {
        "cfg" => "config".to_string(),
        "configuration" => "config".to_string(),
        otherwise => normalize_domain_token(otherwise),
    }
}

fn common_tokens(left: &str, right: &str) -> Vec<String> {
    let left = unique_tokens(&[Some(left)])
        .into_iter()
        .collect::<BTreeSet<_>>();
    unique_tokens(&[Some(right)])
        .into_iter()
        .filter(|token| left.contains(token))
        .collect()
}

fn is_weak_token(token: &str) -> bool {
    WEAK_COMMON_TOKENS.contains(&token)
}

fn shared_prefix(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn levenshtein_capped(left: &str, right: &str, cap: usize) -> usize {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len().abs_diff(right.len()) > cap {
        return cap + 1;
    }
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_char) in left.iter().enumerate() {
        current[0] = left_index + 1;
        let mut row_min = current[0];
        for (right_index, right_char) in right.iter().enumerate() {
            let cost = usize::from(left_char != right_char);
            current[right_index + 1] = (current[right_index] + 1)
                .min(previous[right_index + 1] + 1)
                .min(previous[right_index] + cost);
            row_min = row_min.min(current[right_index + 1]);
        }
        if row_min > cap {
            return cap + 1;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn locality(candidate: &SearchCandidate, owner_hint: Option<&str>) -> Value {
    let same_file = owner_hint == Some(candidate.owner_file.as_str());
    let same_dir = owner_hint.is_some_and(|owner| dirname(owner) == dirname(&candidate.owner_file));
    json!({ "sameDir": same_dir, "sameFile": same_file })
}

pub(super) fn locality_rank(value: &Value) -> usize {
    if value.pointer("/locality/sameFile").and_then(Value::as_bool) == Some(true) {
        2
    } else if value.pointer("/locality/sameDir").and_then(Value::as_bool) == Some(true) {
        1
    } else {
        0
    }
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}

fn insert_option(object: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        object.insert(key.to_string(), json!(value));
    }
}

fn extend_object(target: &mut Value, extra: Value) {
    let Some(target) = target.as_object_mut() else {
        return;
    };
    let Some(extra) = extra.as_object() else {
        return;
    };
    target.extend(extra.clone());
}
