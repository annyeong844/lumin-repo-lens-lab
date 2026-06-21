use std::collections::BTreeSet;

use serde_json::Value;

pub fn strings(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.as_str())
        .map(str::to_string)
        .collect()
}
