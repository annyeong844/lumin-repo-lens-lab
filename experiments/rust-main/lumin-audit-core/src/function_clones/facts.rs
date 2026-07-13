use serde_json::{json, Value};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub(super) struct FunctionFact {
    pub(super) value: Value,
    pub(super) identity: String,
    pub(super) owner_file: String,
    pub(super) exported_name: String,
    pub(super) visibility: String,
    pub(super) line: i64,
    pub(super) body_loc: i64,
    pub(super) statement_count: i64,
    pub(super) normalized_exact_hash: String,
    pub(super) normalized_structure_hash: String,
    pub(super) normalized_signature_hash: Option<String>,
    pub(super) signature: Option<Value>,
    pub(super) generated_file: bool,
    pub(super) call_tokens: Vec<String>,
    pub(super) generator: bool,
    pub(super) async_value: bool,
    pub(super) param_count: i64,
}

impl FunctionFact {
    pub(super) fn from_value(value: Value) -> Self {
        Self {
            identity: string_field(&value, "identity"),
            owner_file: string_field(&value, "ownerFile"),
            exported_name: string_field(&value, "exportedName"),
            visibility: string_field(&value, "visibility")
                .if_empty_then("exported")
                .to_string(),
            line: i64_field(&value, "line"),
            body_loc: i64_field(&value, "bodyLoc"),
            statement_count: i64_field(&value, "statementCount"),
            normalized_exact_hash: string_field(&value, "normalizedExactHash"),
            normalized_structure_hash: string_field(&value, "normalizedStructureHash"),
            normalized_signature_hash: optional_string_field(&value, "normalizedSignatureHash"),
            signature: value.get("signature").cloned(),
            generated_file: truthy_field(&value, "generatedFile"),
            call_tokens: string_array_field(&value, "callTokens"),
            generator: bool_field(&value, "generator"),
            async_value: bool_field(&value, "async"),
            param_count: i64_field(&value, "paramCount"),
            value,
        }
    }
}

trait EmptyStringDefault {
    fn if_empty_then<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl EmptyStringDefault for String {
    fn if_empty_then<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.is_empty() {
            fallback
        } else {
            self
        }
    }
}

pub(super) fn stamp_observed_at(mut value: Value, observed_at: &str) -> Value {
    if let Value::Object(object) = &mut value {
        object.insert("observedAt".to_string(), json!(observed_at));
    }
    value
}

pub(super) fn compare_facts(left: &FunctionFact, right: &FunctionFact) -> Ordering {
    left.owner_file
        .cmp(&right.owner_file)
        .then_with(|| left.line.cmp(&right.line))
        .then_with(|| left.exported_name.cmp(&right.exported_name))
        .then_with(|| left.identity.cmp(&right.identity))
}

pub(super) fn string_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn optional_string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(super) fn string_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        Some(Value::Null) | None => String::new(),
        Some(value) => value.to_string(),
    }
}

fn i64_field(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

pub(super) fn usize_field(value: &Value, key: &str) -> usize {
    value.get(key).and_then(Value::as_u64).unwrap_or(0) as usize
}

pub(super) fn bool_field(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn truthy_field(value: &Value, key: &str) -> bool {
    match value.get(key) {
        Some(Value::Null) | None => false,
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => value.as_i64().unwrap_or(1) != 0,
        Some(Value::String(value)) => !value.is_empty(),
        Some(Value::Array(_)) | Some(Value::Object(_)) => true,
    }
}

pub(super) fn f64_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn string_array_field(value: &Value, key: &str) -> Vec<String> {
    let mut values = value
        .get(key)
        .and_then(Value::as_array)
        .map(|tokens| {
            tokens
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    values.sort();
    values
}
