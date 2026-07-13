use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn non_generated_array(artifact: &Value, key: &str) -> Vec<Value> {
    artifact
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    !item
                        .get("generatedOnly")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn generated_only_count(artifact: &Value, key: &str) -> usize {
    artifact
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    item.get("generatedOnly")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

pub(super) fn unique_sorted_strings<I>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    items
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn text_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub(super) fn number_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

pub(super) fn identities_key(value: &Value) -> String {
    value
        .get("identities")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("|")
        })
        .unwrap_or_default()
}

pub(super) fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    Some(cursor)
}

pub(super) fn as_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .or_else(|| {
            let number = value.as_f64()?;
            if number.is_finite() && number >= 0.0 {
                Some(number.floor() as usize)
            } else {
                None
            }
        })
}

pub(super) fn parse_percent(value: &str) -> Option<f64> {
    let prefix = value.split_once('%').map_or(value, |(prefix, _)| prefix);
    prefix.parse::<f64>().ok().map(|number| number / 100.0)
}

pub(super) fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}
