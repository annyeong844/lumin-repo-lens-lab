use super::protocol::Record;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub(super) fn compact_object(entries: Vec<(&str, Option<Value>)>) -> Value {
    let mut object = Map::new();
    for (key, value) in entries {
        let Some(value) = value else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        object.insert(key.to_string(), value);
    }
    Value::Object(object)
}

pub(super) fn non_empty_string_array(values: Vec<String>) -> Option<Value> {
    (!values.is_empty()).then(|| json!(sort_strings(values)))
}

pub(super) fn sort_strings(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn unique_sorted(values: Vec<&str>) -> Vec<&str> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn sort_by_key(values: Vec<Value>, key_fn: impl Fn(&Value) -> String) -> Vec<Value> {
    let mut values = values;
    values.sort_by_key(|value| key_fn(value));
    values
}

pub(super) fn dedupe_by_key(values: Vec<Value>, key_fn: impl Fn(&Value) -> String) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for value in values {
        let key = key_fn(&value);
        if seen.insert(key) {
            out.push(value);
        }
    }
    out
}

pub(super) fn array_field<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

pub(super) fn slash(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

pub(super) fn value_string(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(super) fn value_usize(value: &Value, field: &str) -> usize {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .unwrap_or_default()
}

pub(super) fn unresolved_import_key(item: &Value) -> String {
    let item = Record::new(item);
    [
        item.str("importer").unwrap_or_default(),
        item.str("specifier").unwrap_or_default(),
        item.str("kind").unwrap_or_default(),
        item.str("reason").unwrap_or_default(),
    ]
    .join("|")
}

pub(super) fn blind_zone_key(item: &Value) -> String {
    let item = Record::new(item);
    [
        item.str("family").unwrap_or_default(),
        item.str("reason").unwrap_or_default(),
        item.str("importer").unwrap_or_default(),
        item.str("specifier").unwrap_or_default(),
        item.str("affectedPackageScope").unwrap_or_default(),
        item.str("candidatePath").unwrap_or_default(),
    ]
    .join("|")
}

pub(super) fn candidate_target_key(item: &Value) -> String {
    let item = Record::new(item);
    let mut parts = vec![
        item.str("importer").unwrap_or_default().to_string(),
        item.str("specifier").unwrap_or_default().to_string(),
        item.str("family").unwrap_or_default().to_string(),
        item.str("notResolvedBecause")
            .unwrap_or_default()
            .to_string(),
    ];
    if let Some(paths) = item.get("candidatePaths").and_then(Value::as_array) {
        parts.extend(
            paths
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned),
        );
    }
    parts.join("|")
}

pub(super) fn blocked_candidate_hint_key(item: &Value) -> String {
    let item = Record::new(item);
    [
        item.str("family").unwrap_or_default(),
        item.str("reason").unwrap_or_default(),
        item.str("importer").unwrap_or_default(),
        item.str("specifier").unwrap_or_default(),
        item.str("affectedPackageScope").unwrap_or_default(),
        item.str("candidatePath").unwrap_or_default(),
        item.str("relevance").unwrap_or_default(),
    ]
    .join("|")
}
