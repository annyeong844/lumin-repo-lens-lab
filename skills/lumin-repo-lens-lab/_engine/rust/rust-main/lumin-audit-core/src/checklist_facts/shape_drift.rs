use super::{
    as_usize, identities_key, number_field, round3, text_field, unavailable, unique_sorted_strings,
    value_at,
};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn shape_drift(shape_index: Option<&Value>) -> Value {
    let Some(shape_index) = shape_index else {
        return unavailable(
            "shape-index.json missing — run full profile or build-shape-index.mjs first",
        );
    };
    let facts = shape_index
        .get("facts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let facts_by_identity = facts
        .iter()
        .filter_map(|fact| {
            let identity = fact.get("identity")?.as_str()?;
            Some((identity.to_string(), fact.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut groups = Vec::<Value>::new();
    if let Some(groups_by_hash) = shape_index.get("groupsByHash").and_then(Value::as_object) {
        for (hash, identities) in groups_by_hash {
            let Some(identities) = identities.as_array() else {
                continue;
            };
            if identities.len() < 2 {
                continue;
            }
            let identity_strings = identities
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>();
            if let Some(group) = summarize_shape_group(hash, &identity_strings, &facts_by_identity)
            {
                groups.push(group);
            }
        }
    }

    groups.sort_by(shape_group_rank);
    let non_generated_groups = groups
        .iter()
        .filter(|group| {
            !group
                .get("generatedOnly")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();
    let near_shape_candidates = collect_near_shape_candidates(&facts);
    let gate = if !non_generated_groups.is_empty() || !near_shape_candidates.is_empty() {
        "watch"
    } else {
        "ok"
    };
    let duplicate_identity_count = non_generated_groups
        .iter()
        .map(|group| group.get("size").and_then(as_usize).unwrap_or(0))
        .sum::<usize>();

    json!({
        "gate": gate,
        "available": true,
        "exactDuplicateGroups": non_generated_groups.len(),
        "nearShapeCandidateCount": near_shape_candidates.len(),
        "generatedOnlyGroups": groups.len().saturating_sub(non_generated_groups.len()),
        "duplicateIdentityCount": duplicate_identity_count,
        "totalShapeFacts": facts.len(),
        "shapeIndexComplete": value_at(shape_index, &["meta", "complete"]).and_then(Value::as_bool).unwrap_or(true),
        "topGroups": non_generated_groups.into_iter().take(10).collect::<Vec<_>>(),
        "nearShapeCandidates": near_shape_candidates.iter().take(10).cloned().collect::<Vec<_>>(),
        "generatedOnlySummary": groups
            .iter()
            .filter(|group| group.get("generatedOnly").and_then(Value::as_bool).unwrap_or(false))
            .take(5)
            .map(|group| json!({
                "hash": group.get("hash").cloned().unwrap_or(Value::Null),
                "size": group.get("size").cloned().unwrap_or(Value::Null),
                "ownerFiles": group.get("ownerFiles").cloned().unwrap_or_else(|| json!([])),
            }))
            .collect::<Vec<_>>(),
        "note": "Exact and near exported type-shape matches only. Treat as review cues, not proof of duplicated implementation or an automatic refactor.",
    })
}

fn summarize_shape_group(
    hash: &str,
    identities: &[String],
    facts_by_identity: &BTreeMap<String, Value>,
) -> Option<Value> {
    let mut members = identities
        .iter()
        .filter_map(|identity| facts_by_identity.get(identity))
        .cloned()
        .collect::<Vec<_>>();
    if members.len() < 2 {
        return None;
    }
    members.sort_by(|a, b| {
        text_field(a, "ownerFile")
            .cmp(&text_field(b, "ownerFile"))
            .then_with(|| text_field(a, "exportedName").cmp(&text_field(b, "exportedName")))
    });
    let owner_files = unique_sorted_strings(members.iter().map(|m| text_field(m, "ownerFile")));
    let exported_names =
        unique_sorted_strings(members.iter().map(|m| text_field(m, "exportedName")));
    let generated_members = members
        .iter()
        .filter(|member| {
            member
                .get("generatedFile")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let field_names = members.first().map(shape_field_names).unwrap_or_default();
    Some(json!({
        "hash": hash,
        "size": members.len(),
        "ownerFiles": owner_files,
        "exportedNames": exported_names,
        "generatedMembers": generated_members,
        "generatedOnly": generated_members == members.len(),
        "fieldNames": field_names,
        "identities": members.iter().filter_map(|m| m.get("identity").and_then(Value::as_str)).collect::<Vec<_>>(),
    }))
}

fn shape_group_rank(a: &Value, b: &Value) -> Ordering {
    let a_non_generated = !a
        .get("generatedOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let b_non_generated = !b
        .get("generatedOnly")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    b_non_generated
        .cmp(&a_non_generated)
        .then_with(|| {
            b.get("size")
                .and_then(as_usize)
                .unwrap_or(0)
                .cmp(&a.get("size").and_then(as_usize).unwrap_or(0))
        })
        .then_with(|| text_field(a, "hash").cmp(&text_field(b, "hash")))
}

fn collect_near_shape_candidates(facts: &[Value]) -> Vec<Value> {
    let mut usable = facts
        .iter()
        .filter(|fact| {
            fact.get("identity").and_then(Value::as_str).is_some()
                && !fact
                    .get("generatedFile")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                && shape_field_names(fact).len() >= 2
        })
        .cloned()
        .collect::<Vec<_>>();
    usable.sort_by_key(|fact| text_field(fact, "identity"));

    let mut candidates = Vec::<Value>::new();
    for i in 0..usable.len() {
        for j in (i + 1)..usable.len() {
            if let Some(candidate) = summarize_near_shape_candidate(&usable[i], &usable[j]) {
                candidates.push(candidate);
            }
        }
    }
    candidates.sort_by(|a, b| {
        number_field(b, "score")
            .partial_cmp(&number_field(a, "score"))
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                number_field(b, "fieldJaccard")
                    .partial_cmp(&number_field(a, "fieldJaccard"))
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| identities_key(a).cmp(&identities_key(b)))
    });
    candidates.truncate(20);
    candidates
}

fn summarize_near_shape_candidate(a: &Value, b: &Value) -> Option<Value> {
    let a_fields = shape_field_names(a);
    let b_fields = shape_field_names(b);
    let shared_fields = set_intersection(&a_fields, &b_fields);
    if same_hash_pair(a, b) || shared_fields.len() < 2 {
        return None;
    }
    let field_jaccard = jaccard(&a_fields, &b_fields);
    let a_name_tokens = tokenize_shape_name(&text_field(a, "exportedName"));
    let b_name_tokens = tokenize_shape_name(&text_field(b, "exportedName"));
    let shared_name_tokens = set_intersection(&a_name_tokens, &b_name_tokens);
    let name_token_jaccard = jaccard(&a_name_tokens, &b_name_tokens);
    let same_directory =
        owner_dir(&text_field(a, "ownerFile")) == owner_dir(&text_field(b, "ownerFile"));
    let domain_cue = same_directory || !shared_name_tokens.is_empty();
    if !domain_cue {
        return None;
    }
    let nearly_same_fields = field_jaccard >= 0.5 && shared_fields.len() >= 2;
    let same_named_concept = !shared_name_tokens.is_empty() && field_jaccard >= 0.4;
    if !nearly_same_fields && !same_named_concept {
        return None;
    }
    let score = round3(
        (field_jaccard * 0.75)
            + (name_token_jaccard * 0.2)
            + if same_directory { 0.05 } else { 0.0 },
    );
    Some(json!({
        "score": score,
        "fieldJaccard": round3(field_jaccard),
        "nameTokenJaccard": round3(name_token_jaccard),
        "sameDirectory": same_directory,
        "identities": [text_field(a, "identity"), text_field(b, "identity")],
        "ownerFiles": [text_field(a, "ownerFile"), text_field(b, "ownerFile")],
        "exportedNames": [text_field(a, "exportedName"), text_field(b, "exportedName")],
        "sharedFieldNames": shared_fields,
        "leftOnlyFieldNames": set_diff(&a_fields, &b_fields),
        "rightOnlyFieldNames": set_diff(&b_fields, &a_fields),
        "sharedNameTokens": shared_name_tokens,
        "reason": "near exported type-shape review cue only; field/name overlap is not proof of duplication",
    }))
}

fn shape_field_names(fact: &Value) -> Vec<String> {
    let mut fields = fact
        .get("fields")
        .and_then(Value::as_array)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|field| field.get("name").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>();
    fields.sort();
    fields
}

fn tokenize_shape_name(name: &str) -> Vec<String> {
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
        .replace(['_', '-', '.'], " ")
        .to_ascii_lowercase()
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| token.len() >= 3 && !SHAPE_NAME_STOP_TOKENS.contains(token))
        .map(str::to_string)
        .collect()
}

const SHAPE_NAME_STOP_TOKENS: &[&str] = &[
    "type",
    "types",
    "interface",
    "interfaces",
    "model",
    "models",
    "state",
    "view",
    "data",
    "dto",
    "payload",
    "props",
    "options",
    "config",
    "request",
    "response",
    "result",
    "event",
    "item",
];

fn owner_dir(file: &str) -> String {
    let slash = file.replace('\\', "/");
    slash
        .rsplit_once('/')
        .map_or(String::new(), |(dir, _)| dir.to_string())
}

fn same_hash_pair(a: &Value, b: &Value) -> bool {
    a.get("hash").and_then(Value::as_str).is_some()
        && a.get("hash").and_then(Value::as_str) == b.get("hash").and_then(Value::as_str)
}

fn set_intersection(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|item| right.contains(item))
        .cloned()
        .collect()
}

fn set_diff(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|item| !right.contains(item))
        .cloned()
        .collect()
}

fn jaccard(left: &[String], right: &[String]) -> f64 {
    let left_set = left.iter().collect::<BTreeSet<_>>();
    let right_set = right.iter().collect::<BTreeSet<_>>();
    let union_len = left_set.union(&right_set).count();
    if union_len == 0 {
        return 0.0;
    }
    let intersection_len = left_set.intersection(&right_set).count();
    intersection_len as f64 / union_len as f64
}
