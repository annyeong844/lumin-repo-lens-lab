use super::ordering::{padded_line, sort_values_by_key, value_string};
use super::paths::{rel_path, resolve_prefix_target};
use crate::symbol_graph::prepare::FileDataRecord;
use serde_json::{json, Map, Value};

pub(in crate::symbol_graph) fn build_dynamic_import_opacity(
    root: &str,
    file_data: &[FileDataRecord],
) -> Vec<Value> {
    let mut records = Vec::new();
    for file in file_data {
        let consumer_file = rel_path(root, &file.file_path);
        for item in &file.dynamic_import_opacity {
            let mut object = Map::new();
            object.insert("consumerFile".to_string(), json!(consumer_file));
            if let Some(line) = item.get("line") {
                object.insert("line".to_string(), line.clone());
            }
            if let Some(kind) = item.get("kind") {
                object.insert("kind".to_string(), kind.clone());
            }
            if let Some(prefix) = item.get("prefix").and_then(Value::as_str) {
                let target = resolve_prefix_target(&file.file_path, prefix);
                object.insert("prefix".to_string(), json!(prefix));
                object.insert(
                    "targetDir".to_string(),
                    json!(format!(
                        "{}/",
                        rel_path(root, &target).trim_end_matches('/')
                    )),
                );
            }
            records.push(Value::Object(object));
        }
    }
    sort_values_by_key(records, dynamic_opacity_key)
}

fn dynamic_opacity_key(value: &Value) -> String {
    format!(
        "{}|{}|{}",
        value_string(value, "consumerFile"),
        padded_line(value),
        value_string(value, "prefix")
    )
}

pub(in crate::symbol_graph) fn build_cjs_export_surface_by_file(
    root: &str,
    file_data: &[FileDataRecord],
) -> Value {
    let mut out = Map::new();
    for file in file_data {
        let Some(surface) = file.cjs_export_surface.as_ref().and_then(Value::as_object) else {
            continue;
        };
        let exact = surface
            .get("exact")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let opaque = surface
            .get("opaque")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if exact.is_empty() && opaque.is_empty() {
            continue;
        }
        out.insert(
            rel_path(root, &file.file_path),
            json!({
                "exact": sort_cjs_surface_list(exact),
                "opaque": sort_cjs_surface_list(opaque),
            }),
        );
    }
    Value::Object(out)
}

fn sort_cjs_surface_list(values: Vec<Value>) -> Vec<Value> {
    sort_values_by_key(values, |value| {
        format!(
            "{}|{}|{}",
            value_string(value, "name"),
            value_string(value, "kind"),
            padded_line(value)
        )
    })
}

pub(in crate::symbol_graph) fn build_cjs_require_opacity(
    root: &str,
    file_data: &[FileDataRecord],
) -> Vec<Value> {
    let mut records = Vec::new();
    for file in file_data {
        for item in &file.cjs_require_opacity {
            records.push(json!({
                "consumerFile": rel_path(root, &file.file_path),
                "line": item.get("line").cloned().unwrap_or(Value::Null),
                "kind": item.get("kind").cloned().unwrap_or(Value::Null),
            }));
        }
    }
    sort_values_by_key(records, |value| {
        format!(
            "{}|{}|{}",
            value_string(value, "consumerFile"),
            padded_line(value),
            value_string(value, "kind")
        )
    })
}

pub(in crate::symbol_graph) fn files_with_parse_errors(
    root: &str,
    entries: &[String],
) -> Vec<String> {
    let mut files = entries
        .iter()
        .map(|file| rel_path(root, file))
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}
