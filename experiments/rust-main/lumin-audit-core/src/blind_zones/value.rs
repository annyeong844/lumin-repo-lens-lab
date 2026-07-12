use serde_json::{Map, Value};

pub(crate) fn by_language(triage: Option<&Value>) -> Option<&Value> {
    triage
        .and_then(|triage| triage.get("byLanguage"))
        .or_else(|| triage.and_then(|triage| triage.get("languages")))
        .or_else(|| get_path_opt(triage, &["summary", "byLanguage"]))
}

pub(crate) fn language_count(value: Option<&Value>) -> Option<u64> {
    value.and_then(number_u64).or_else(|| {
        value
            .and_then(|value| value.get("files"))
            .and_then(number_u64)
    })
}

pub(crate) fn first_number(values: &[Option<&Value>]) -> Option<f64> {
    values.iter().find_map(|value| value.and_then(number_f64))
}

pub(crate) fn first_u64(values: &[Option<&Value>]) -> Option<u64> {
    values.iter().find_map(|value| value.and_then(number_u64))
}

pub(crate) fn number_f64(value: &Value) -> Option<f64> {
    value.as_f64().filter(|number| number.is_finite())
}

pub(crate) fn number_u64(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|number| u64::try_from(number).ok()))
}

pub(crate) fn optional_array(value: Option<&Value>) -> Option<&Vec<Value>> {
    value.and_then(Value::as_array)
}

pub(crate) fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value, |cursor, key| cursor.get(*key))
}

pub(crate) fn get_path_opt<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    value.and_then(|value| get_path(value, path))
}

pub(crate) fn insert_optional_number(map: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(value) = value {
        map.insert(key.to_string(), serde_json::json!(value));
    }
}

pub(crate) fn insert_optional_u64(map: &mut Map<String, Value>, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        map.insert(key.to_string(), serde_json::json!(value));
    }
}

pub(crate) fn insert_existing(map: &mut Map<String, Value>, key: &str, value: Option<&Value>) {
    if let Some(value) = value {
        map.insert(key.to_string(), value.clone());
    }
}
