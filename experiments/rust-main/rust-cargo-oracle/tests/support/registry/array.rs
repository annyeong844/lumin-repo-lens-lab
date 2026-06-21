use serde_json::Value;

pub fn array_contains_string(value: &Value, expected: &str) -> bool {
    value
        .as_array()
        .into_iter()
        .flatten()
        .any(|entry| entry.as_str() == Some(expected))
}
