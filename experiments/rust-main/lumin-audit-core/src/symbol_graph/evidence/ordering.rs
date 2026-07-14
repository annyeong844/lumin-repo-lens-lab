use serde_json::Value;

pub(in crate::symbol_graph) fn sort_values_by_key(
    mut values: Vec<Value>,
    key_fn: fn(&Value) -> String,
) -> Vec<Value> {
    values.sort_by_key(key_fn);
    values
}

pub(in crate::symbol_graph) fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

pub(in crate::symbol_graph) fn value_string(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn value_bool_key(value: &Value, field: &str) -> &'static str {
    if value.get(field).and_then(Value::as_bool) == Some(true) {
        "1"
    } else {
        "0"
    }
}

pub(super) fn padded_line(value: &Value) -> String {
    let raw = value
        .get("line")
        .cloned()
        .unwrap_or(Value::String(String::new()));
    match raw {
        Value::Number(number) => format!("{number:0>6}"),
        Value::String(text) => format!("{text:0>6}"),
        _ => String::from("000000"),
    }
}

pub(in crate::symbol_graph) fn dependency_consumer_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "depRoot"),
        value_string(value, "fromSpec"),
        value_string(value, "file"),
        value_string(value, "kind")
    )
}

pub(in crate::symbol_graph) fn resolved_internal_edge_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "from"),
        value_string(value, "to"),
        value_string(value, "kind"),
        value_string(value, "source"),
        value_bool_key(value, "typeOnly")
    )
}

pub(in crate::symbol_graph) fn sfc_style_asset_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "fromSpec"),
        value_string(value, "source"),
        value_string(value, "status")
    )
}

pub(in crate::symbol_graph) fn sfc_template_ref_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "tagName"),
        value_string(value, "bindingName"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

pub(in crate::symbol_graph) fn sfc_global_registration_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "registrationFile"),
        value_string(value, "componentName"),
        value_string(value, "bindingName"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

pub(in crate::symbol_graph) fn sfc_generated_manifest_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "manifestFile"),
        value_string(value, "componentName"),
        value_string(value, "fromSpec"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

pub(in crate::symbol_graph) fn sfc_framework_convention_key(value: &Value) -> String {
    [
        "framework",
        "conventionKind",
        "consumerFile",
        "sourceFile",
        "configFile",
        "componentName",
        "tagName",
        "directiveName",
        "actionName",
        "subscriptionName",
        "storeName",
        "macroName",
        "fromSpec",
    ]
    .iter()
    .map(|field| value_string(value, field))
    .collect::<Vec<_>>()
    .join("|")
}

pub(in crate::symbol_graph) fn generated_blind_zone_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "scopePackageRoot"),
        value_string(value, "candidatePath"),
        value_string(value, "specifier"),
        value_string(value, "consumerFile")
    )
}

pub(in crate::symbol_graph) fn generated_import_consumer_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "specifier"),
        value_string(value, "name"),
        value_string(value, "kind"),
        value_string(value, "surfaceId")
    )
}

pub(in crate::symbol_graph) fn unresolved_record_key(value: &Value) -> String {
    format!(
        "{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "specifier"),
        value_string(value, "kind")
    )
}

pub(in crate::symbol_graph) fn namespace_re_export_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "exportedName"),
        value_string(value, "targetFile"),
        value_string(value, "kind"),
        value.get("line").map(Value::to_string).unwrap_or_default()
    )
}

pub(in crate::symbol_graph) fn sort_generated_virtual_surfaces(values: Vec<Value>) -> Vec<Value> {
    let mut surfaces = values
        .into_iter()
        .map(|surface| {
            let mut object = surface.as_object().cloned().unwrap_or_default();
            let mut exports = object
                .get("exports")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            exports.sort_by_key(|entry| {
                format!(
                    "{}|{}",
                    value_string(entry, "name"),
                    value_string(entry, "kind")
                )
            });
            object.insert("exports".to_string(), Value::Array(exports));
            Value::Object(object)
        })
        .collect::<Vec<_>>();
    surfaces.sort_by_key(|surface| value_string(surface, "id"));
    surfaces
}
