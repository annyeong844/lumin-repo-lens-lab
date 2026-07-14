use super::ordering::{padded_line, value_string};
use super::paths::rel_path;
use crate::symbol_graph::prepare::FileDataRecord;
use serde_json::{json, Map, Value};

fn sort_class_method_records(values: &[Value]) -> Vec<Value> {
    let mut records = values.to_vec();
    records.sort_by_key(|value| {
        format!(
            "{}|{}|{}|{}",
            value_string(value, "className"),
            value_string(value, "name"),
            padded_line(value),
            value_string(value, "identity")
        )
    });
    records
}

pub(in crate::symbol_graph) fn build_class_method_index(
    root: &str,
    file_data: &[FileDataRecord],
) -> Value {
    let mut out = Map::new();
    for file in file_data {
        if file.class_methods.is_empty() {
            continue;
        }
        let rel = rel_path(root, &file.file_path);
        let mut by_name = Map::<String, Value>::new();
        for method in sort_class_method_records(&file.class_methods) {
            let name = value_string(&method, "name");
            let name = if name.is_empty() {
                value_string(&method, "methodName")
            } else {
                name
            };
            if name.is_empty() {
                continue;
            }
            let class_name = value_string(&method, "className");
            let record = json!({
                "identity": method.get("identity").cloned().unwrap_or_else(|| json!(format!("{rel}::{class_name}#{name}"))),
                "ownerFile": method.get("ownerFile").cloned().unwrap_or_else(|| json!(rel)),
                "className": method.get("className").cloned().unwrap_or(Value::Null),
                "name": name,
                "methodName": method.get("methodName").cloned().unwrap_or_else(|| json!(name)),
                "kind": method.get("kind").cloned().unwrap_or_else(|| json!("ClassMethod")),
                "memberKind": method.get("memberKind").cloned().unwrap_or_else(|| json!("method")),
                "visibility": method.get("visibility").cloned().unwrap_or_else(|| json!("public")),
                "static": method.get("static").and_then(Value::as_bool).unwrap_or(false),
                "computed": method.get("computed").and_then(Value::as_bool).unwrap_or(false),
                "line": method.get("line").cloned().unwrap_or(Value::Null),
            });
            let mut record = record.as_object().cloned().unwrap_or_default();
            if let Some(end_line) = method.get("endLine") {
                record.insert("endLine".to_string(), end_line.clone());
            }
            let method_group = by_name
                .entry(name)
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Value::Array(methods) = method_group {
                methods.push(Value::Object(record));
            }
        }
        if !by_name.is_empty() {
            out.insert(rel, Value::Object(by_name));
        }
    }
    Value::Object(out)
}

fn sort_local_operation_records(values: &[Value]) -> Vec<Value> {
    let mut records = values.to_vec();
    records.sort_by_key(|value| {
        format!(
            "{}|{}|{}|{}",
            value_string(value, "containerName"),
            value_string(value, "name"),
            padded_line(value),
            value_string(value, "identity")
        )
    });
    records
}

pub(in crate::symbol_graph) fn build_pre_write_local_operation_index(
    root: &str,
    file_data: &[FileDataRecord],
) -> Value {
    let mut by_owner_file = Map::new();
    let mut operation_count = 0usize;
    for file in file_data {
        let operations = sort_local_operation_records(&file.local_operations);
        if operations.is_empty() {
            continue;
        }
        let rel = rel_path(root, &file.file_path);
        let projected = operations
            .into_iter()
            .map(|operation| {
                json!({
                    "identity": operation.get("identity").cloned().unwrap_or(Value::Null),
                    "name": operation.get("name").cloned().unwrap_or(Value::Null),
                    "ownerFile": operation.get("ownerFile").cloned().unwrap_or_else(|| json!(rel)),
                    "containerName": operation.get("containerName").cloned().unwrap_or(Value::Null),
                    "containerKind": operation.get("containerKind").cloned().unwrap_or(Value::Null),
                    "scopeKind": operation.get("scopeKind").cloned().unwrap_or_else(|| json!("nested-function")),
                    "matchedField": operation.get("matchedField").cloned().unwrap_or_else(|| json!("preWriteLocalOperationIndex")),
                    "line": operation.get("line").cloned().unwrap_or(Value::Null),
                    "operationFamily": operation.get("operationFamily").cloned().unwrap_or(Value::Null),
                    "domainTokens": sorted_value_strings(operation.get("domainTokens")),
                    "visibility": operation.get("visibility").cloned().unwrap_or_else(|| json!("local-only")),
                    "eligibleForDeadExportRanking": false,
                    "eligibleForSafeFix": false,
                })
            })
            .collect::<Vec<_>>();
        operation_count += projected.len();
        by_owner_file.insert(rel, Value::Array(projected));
    }
    json!({
        "schemaVersion": "pre-write-local-operations.v1",
        "status": "complete",
        "meta": {
            "supports": {
                "nestedLocalOperationIndex": true,
            },
        },
        "byOwnerFile": by_owner_file,
        "summary": {
            "ownerFileCount": by_owner_file.len(),
            "operationCount": operation_count,
        },
    })
}

fn sorted_value_strings(value: Option<&Value>) -> Vec<String> {
    let mut strings = value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    strings.sort();
    strings
}

pub(in crate::symbol_graph) fn build_re_exports_by_file(
    root: &str,
    file_data: &[FileDataRecord],
) -> Value {
    let mut out = Map::new();
    for file in file_data {
        if file.re_exports.is_empty() {
            continue;
        }
        let records = file
            .re_exports
            .iter()
            .map(|item| {
                let mut object = Map::new();
                if let Some(source) = item.get("source") {
                    object.insert("source".to_string(), source.clone());
                }
                if let Some(line) = item.get("line") {
                    object.insert("line".to_string(), line.clone());
                }
                if let Some(namespace) = item.get("namespace") {
                    object.insert("namespace".to_string(), namespace.clone());
                }
                Value::Object(object)
            })
            .collect::<Vec<_>>();
        out.insert(rel_path(root, &file.file_path), Value::Array(records));
    }
    Value::Object(out)
}
