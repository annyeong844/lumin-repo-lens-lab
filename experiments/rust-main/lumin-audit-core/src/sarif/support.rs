use serde_json::{json, Map, Value};

use super::rules::rule_index;

#[derive(Debug, Default)]
pub(super) struct SarifState {
    pub(super) results: Vec<Value>,
    pub(super) artifacts_used: Vec<&'static str>,
}

pub(super) fn make_result(
    rule_id: &str,
    level: &str,
    message: String,
    file: &str,
    line: Option<i64>,
    properties: Map<String, Value>,
    root: &str,
) -> Value {
    let mut result = json!({
        "ruleId": rule_id,
        "ruleIndex": rule_index(rule_id),
        "level": level,
        "message": { "text": message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": uri_for(root, file) },
                "region": { "startLine": line.unwrap_or(1).max(1) }
            }
        }]
    });
    if !properties.is_empty() {
        result["properties"] = Value::Object(properties);
    }
    result
}

fn uri_for(root: &str, file: &str) -> String {
    if file.is_empty() {
        return ".".to_string();
    }
    let file = slash_path(file);
    let abs = if path_is_absolute(&file) {
        file
    } else {
        format!("{}/{}", root.trim_end_matches('/'), file)
    };
    if abs == root {
        return ".".to_string();
    }
    if let Some(rest) = abs.strip_prefix(&format!("{}/", root.trim_end_matches('/'))) {
        return rest.to_string();
    }
    abs
}

fn path_is_absolute(path: &str) -> bool {
    path.starts_with('/') || path.as_bytes().get(1).copied() == Some(b':')
}

pub(super) fn slash_path(path: &str) -> String {
    path.replace('\\', "/")
}

pub(super) fn array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| values.iter().collect())
        .unwrap_or_default()
}

pub(super) fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn string_field(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(super) fn number_field(value: &Value, field: &str) -> Option<i64> {
    value.get(field).and_then(Value::as_i64)
}

pub(super) fn insert_string(object: &mut Map<String, Value>, key: &str, value: impl Into<String>) {
    object.insert(key.to_string(), Value::String(value.into()));
}

pub(super) fn insert_optional_string(
    object: &mut Map<String, Value>,
    key: &str,
    value: Option<String>,
) {
    if let Some(value) = value {
        insert_string(object, key, value);
    }
}

pub(super) fn insert_value(object: &mut Map<String, Value>, key: &str, value: Value) {
    object.insert(key.to_string(), value);
}

pub(super) fn copy_field(source: &Value, target: &mut Map<String, Value>, field: &str) {
    if let Some(value) = source.get(field).cloned() {
        target.insert(field.to_string(), value);
    }
}

pub(super) fn present_artifact(value: Option<&Value>) -> Option<&Value> {
    value.filter(|value| !value.is_null())
}
