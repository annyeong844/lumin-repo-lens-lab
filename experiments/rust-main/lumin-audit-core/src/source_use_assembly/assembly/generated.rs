use serde_json::{json, Map, Value};

use super::super::input::SourceUseAssemblyRecord;
use super::super::path::root_relative;
use super::support::{
    increment_branch, is_namespace_reexport_use, is_projection_only_consumer_source, mark_handled,
    skip, AssemblyState,
};
use super::terminal::push_unresolved_specifier_record;

pub(super) fn handle_generated_virtual_record(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id.clone();
    let projection_only = is_projection_only_consumer_source(record.consumer_source.as_deref());
    increment_branch(state, "generatedVirtual");
    if state.options.emit_standalone_transport || projection_only {
        state
            .response
            .generated_virtual_record_ids
            .push(record_id.clone());
    }
    mark_handled(state, record_id);
    if is_namespace_reexport_use(&kind) {
        increment_branch(state, "skippedNamespaceAlias");
        return;
    }

    let Some(surface) = record.generated_virtual_surface.clone() else {
        skip(state, record.record_id, "generated-virtual-surface-missing");
        return;
    };
    add_generated_virtual_surface(state, surface.clone());

    let Some(exported) = generated_virtual_export_for_use(&surface, &record, &kind) else {
        increment_branch(state, "generatedVirtualUnresolved");
        state.response.counters.unresolved_uses += 1;
        state.response.counters.unresolved_internal_uses += 1;
        let from_spec = record.from_spec.clone().unwrap_or_default();
        if !from_spec.is_empty() {
            state
                .response
                .unresolved_internal_specifiers
                .insert(from_spec.clone());
            push_unresolved_specifier_record(state, &record, &from_spec, &kind);
        }
        return;
    };

    state.response.counters.total_uses += 1;
    state.response.counters.resolved_internal_uses += 1;
    state.response.counters.resolved_generated_virtual_uses += 1;

    let mut object = Map::new();
    object.insert(
        "consumerFile".to_string(),
        json!(root_relative(&state.root, &record.consumer_file)),
    );
    object.insert(
        "specifier".to_string(),
        json!(record.from_spec.unwrap_or_default()),
    );
    object.insert("kind".to_string(), json!(kind));
    if let Some(surface_id) = surface.get("id").and_then(Value::as_str) {
        object.insert("surfaceId".to_string(), json!(surface_id));
    }
    if let Some(source) = surface.get("source").and_then(Value::as_str) {
        object.insert("source".to_string(), json!(source));
    }
    if let Some(name) = exported.get("name").and_then(Value::as_str) {
        object.insert("name".to_string(), json!(name));
    }
    if let Some(spaces) = exported.get("spaces").and_then(Value::as_array) {
        if !spaces.is_empty() {
            object.insert("spaces".to_string(), Value::Array(spaces.clone()));
        }
    }
    if record.type_only_present {
        object.insert("typeOnly".to_string(), json!(record.type_only));
    }
    state
        .response
        .generated_virtual_import_consumers
        .push(Value::Object(object));
}

fn add_generated_virtual_surface(state: &mut AssemblyState, surface: Value) {
    let Some(id) = surface.get("id").and_then(Value::as_str) else {
        state.response.generated_virtual_surfaces.push(surface);
        return;
    };
    if state
        .response
        .generated_virtual_surfaces
        .iter()
        .any(|value| value.get("id").and_then(Value::as_str) == Some(id))
    {
        return;
    }
    state.response.generated_virtual_surfaces.push(surface);
}

fn generated_virtual_export_for_use(
    surface: &Value,
    record: &SourceUseAssemblyRecord,
    kind: &str,
) -> Option<Value> {
    if kind == "import-side-effect" {
        return None;
    }
    if kind == "namespace" {
        return Some(json!({"name": "*", "spaces": ["value", "type"]}));
    }
    let exports = surface.get("exports").and_then(Value::as_array)?;
    let name = record
        .name
        .as_deref()
        .filter(|name| !name.is_empty() && *name != "*")?;
    let wanted_space = if record.type_only { "type" } else { "value" };
    exports
        .iter()
        .find(|value| {
            value.get("name").and_then(Value::as_str) == Some(name)
                && value
                    .get("spaces")
                    .and_then(Value::as_array)
                    .is_some_and(|spaces| has_string(spaces, wanted_space))
        })
        .cloned()
}

fn has_string(values: &[Value], expected: &str) -> bool {
    values.iter().any(|value| value.as_str() == Some(expected))
}
