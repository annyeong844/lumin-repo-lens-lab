use super::ordering::{unresolved_record_key, value_string};
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::BTreeMap;

pub(in crate::symbol_graph) fn top_unresolved_specifiers(
    counters: &BTreeMap<String, usize>,
    examples: &BTreeMap<String, String>,
) -> Vec<Value> {
    let mut entries = counters.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(key, count)| (Reverse(**count), (*key).clone()));
    entries
        .into_iter()
        .take(20)
        .map(|(key, count)| {
            let example = examples
                .get(key)
                .cloned()
                .unwrap_or_else(|| key.clone());
            let mut object = Map::new();
            object.insert("specifierPrefix".to_string(), json!(key));
            object.insert("count".to_string(), json!(count));
            object.insert("example".to_string(), json!(example));
            if likely_alias_prefix(
                object
                    .get("specifierPrefix")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            ) {
                object.insert(
                    "likelyCause".to_string(),
                    json!("possible unresolved tsconfig paths alias. Check per-app tsconfig.json for a compilerOptions.paths entry matching this prefix. See FP-36 in references/false-positive-index.md."),
                );
            }
            Value::Object(object)
        })
        .collect()
}

fn likely_alias_prefix(prefix: &str) -> bool {
    prefix.starts_with("@/")
        || prefix.starts_with("~/")
        || prefix.starts_with("#/")
        || (prefix.starts_with('@') && prefix.get(1..).is_some_and(|rest| rest.contains('/')))
}

fn compact_unresolved_example(record: &Value) -> Value {
    let mut object = Map::new();
    for field in ["specifier", "consumerFile", "kind"] {
        if let Some(value) = record.get(field) {
            object.insert(field.to_string(), value.clone());
        }
    }
    if let Some(value) = record.get("typeOnly").filter(|value| value.is_boolean()) {
        object.insert("typeOnly".to_string(), value.clone());
    }
    for field in ["resolverStage", "matchedPattern", "hint"] {
        if let Some(value) = record.get(field).filter(|value| value.is_string()) {
            object.insert(field.to_string(), value.clone());
        }
    }
    if let Some(candidates) = record.get("targetCandidates").and_then(Value::as_array) {
        if !candidates.is_empty() {
            object.insert(
                "targetCandidates".to_string(),
                Value::Array(candidates.iter().take(3).cloned().collect()),
            );
        }
    }
    Value::Object(object)
}

fn unresolved_space(record: &Value) -> &'static str {
    match record.get("typeOnly").and_then(Value::as_bool) {
        Some(true) => "type",
        Some(false) => "value",
        None => "unknown",
    }
}

#[derive(Default)]
struct UnresolvedGroup {
    count: usize,
    spaces_type: usize,
    spaces_value: usize,
    spaces_unknown: usize,
    resolver_stages: BTreeMap<String, usize>,
    hints: BTreeMap<String, usize>,
    examples: Vec<Value>,
}

pub(in crate::symbol_graph) fn unresolved_summary_by_reason(records: &[Value]) -> Value {
    let mut groups = BTreeMap::<String, UnresolvedGroup>::new();
    for record in records {
        let reason = value_string(record, "reason");
        let reason = if reason.is_empty() {
            "unknown-internal-resolution".to_string()
        } else {
            reason
        };
        let group = groups.entry(reason).or_default();
        group.count += 1;
        match unresolved_space(record) {
            "type" => group.spaces_type += 1,
            "value" => group.spaces_value += 1,
            _ => group.spaces_unknown += 1,
        }
        let resolver_stage = value_string(record, "resolverStage");
        if !resolver_stage.is_empty() {
            *group.resolver_stages.entry(resolver_stage).or_insert(0) += 1;
        }
        let hint = value_string(record, "hint");
        if !hint.is_empty() {
            *group.hints.entry(hint).or_insert(0) += 1;
        }
        group.examples.push(compact_unresolved_example(record));
    }

    let mut entries = groups.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .count
            .cmp(&left.1.count)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut out = Map::new();
    for (reason, mut group) in entries {
        group.examples.sort_by_key(unresolved_record_key);
        let mut object = Map::new();
        object.insert("count".to_string(), json!(group.count));
        object.insert(
            "spaces".to_string(),
            json!({
                "type": group.spaces_type,
                "value": group.spaces_value,
                "unknown": group.spaces_unknown,
            }),
        );
        if !group.resolver_stages.is_empty() {
            object.insert("resolverStages".to_string(), json!(group.resolver_stages));
        }
        if !group.hints.is_empty() {
            object.insert("hints".to_string(), json!(group.hints));
        }
        object.insert(
            "examples".to_string(),
            Value::Array(group.examples.into_iter().take(5).collect()),
        );
        out.insert(reason, Value::Object(object));
    }
    Value::Object(out)
}
