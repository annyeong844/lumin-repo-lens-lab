use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) struct FindingRecord {
    pub(super) value: Value,
    pub(super) id: String,
    pub(super) key: String,
    pub(super) file: String,
    pub(super) excluded_reason: Option<String>,
}

pub(super) fn ordinary_findings(dead_classify: &Value) -> Vec<FindingRecord> {
    let mut records = Vec::new();
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_C_remove_symbol",
        "C",
    ));
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_A_demote_to_internal",
        "A",
    ));
    records.extend(flatten_bucket(dead_classify, "proposal_B_review", "B"));
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_remove_export_specifier",
        "specifier",
    ));
    records.extend(flatten_bucket(
        dead_classify,
        "proposal_DEGRADED_unprocessed",
        "unprocessed",
    ));
    records
}

fn flatten_bucket(dead_classify: &Value, field: &str, bucket: &str) -> Vec<FindingRecord> {
    dead_classify
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| finding_record(item, bucket, None))
        .collect()
}

pub(super) fn excluded_findings(dead_classify: &Value) -> Vec<FindingRecord> {
    dead_classify
        .get("excludedCandidates")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let reason = item
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let mut record = finding_record(item, "excluded", Some(reason.clone()))?;
            let object = record.value.as_object_mut()?;
            object.insert(
                "action".to_string(),
                Value::String(format!("Policy-excluded: {reason}")),
            );
            object.insert("_excludeReason".to_string(), Value::String(reason));
            if let Some(policy_evidence) = item.get("policyEvidence") {
                object.insert("policyEvidence".to_string(), policy_evidence.clone());
            }
            Some(record)
        })
        .collect()
}

fn finding_record(
    item: &Value,
    bucket: &str,
    excluded_reason: Option<String>,
) -> Option<FindingRecord> {
    let file = normalize_path(item.get("file")?);
    let symbol = item.get("symbol")?.as_str()?.to_string();
    let line = item.get("line").cloned().unwrap_or(Value::Null);
    let id = finding_id(&file, &symbol, &line);
    let key = lookup_key(&file, &symbol, &line);
    let mut object = item.as_object()?.clone();
    object.insert("id".to_string(), Value::String(id.clone()));
    object.insert("file".to_string(), Value::String(file.clone()));
    object.insert("bucket".to_string(), Value::String(bucket.to_string()));
    Some(FindingRecord {
        value: Value::Object(object),
        id,
        key,
        file,
        excluded_reason,
    })
}

pub(super) fn merge_action_evidence(
    finding: &mut FindingRecord,
    action_by_id: &BTreeMap<String, Value>,
) {
    let Some(action_record) = action_by_id.get(&finding.id) else {
        return;
    };
    let Some(object) = finding.value.as_object_mut() else {
        return;
    };
    for field in ["safeAction", "actionBlockers", "localUseProof"] {
        if let Some(value) = action_record.get(field) {
            object.insert(field.to_string(), value.clone());
        }
    }
}

pub(super) fn normalize_path(value: &Value) -> String {
    value
        .as_str()
        .unwrap_or_default()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

pub(super) fn normalize_path_text(value: &str) -> String {
    value
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn line_key(value: &Value) -> String {
    match value {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn finding_id(file: &str, symbol: &str, line: &Value) -> String {
    format!("dead-export:{file}:{symbol}:{}", line_key(line))
}

pub(super) fn lookup_key(file: &str, symbol: &str, line: &Value) -> String {
    format!("{file}|{symbol}|{}", line_key(line))
}

pub(super) fn finding_identity(file: &str, symbol: &str) -> String {
    format!("{file}::{symbol}")
}

pub(super) fn action_by_id(export_action_safety: Option<&Value>) -> BTreeMap<String, Value> {
    let mut map = BTreeMap::new();
    if let Some(by_id) = export_action_safety
        .and_then(|value| value.get("byId"))
        .and_then(Value::as_object)
    {
        for (id, record) in by_id {
            map.insert(id.clone(), record.clone());
        }
    }
    if let Some(records) = export_action_safety
        .and_then(|value| value.get("findings"))
        .and_then(Value::as_array)
    {
        for record in records {
            if let Some(id) = record.get("id").and_then(Value::as_str) {
                map.insert(id.to_string(), record.clone());
            }
        }
    }
    map
}
