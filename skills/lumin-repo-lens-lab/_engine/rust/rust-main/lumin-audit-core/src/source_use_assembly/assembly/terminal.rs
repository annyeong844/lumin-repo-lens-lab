use serde_json::{json, Map, Value};

use super::super::input::SourceUseAssemblyRecord;
use super::super::path::root_relative;
use super::super::protocol::{DependencyImportConsumerAddition, ResolvedRecordTarget};
use super::support::{
    increment_branch, is_namespace_reexport_use, is_projection_only_consumer_source, mark_handled,
    AssemblyState,
};

pub(super) fn handle_external_record(state: &mut AssemblyState, record: SourceUseAssemblyRecord) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id;
    let projection_only = is_projection_only_consumer_source(record.consumer_source.as_deref());
    increment_branch(state, "external");
    if state.options.emit_standalone_transport || projection_only {
        state.response.external_record_ids.push(record_id.clone());
    }
    mark_handled(state, record_id);
    if projection_only {
        return;
    }
    if is_namespace_reexport_use(&kind) {
        increment_branch(state, "skippedNamespaceAlias");
        return;
    }

    state.response.counters.external_uses += 1;
    state.response.counters.unresolved_uses += 1;
    let from_spec = record.from_spec.unwrap_or_default();
    let Some(dep_root) = package_root_from_spec(&from_spec) else {
        return;
    };
    state
        .response
        .dependency_import_consumers
        .push(DependencyImportConsumerAddition {
            file: root_relative(&state.root, &record.consumer_file),
            from_spec,
            dep_root,
            kind,
            source: record
                .consumer_source
                .unwrap_or_else(|| "source-import".to_string()),
            type_only: record.type_only_present.then_some(record.type_only),
        });
}

pub(super) fn handle_non_source_asset_record(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
) {
    increment_branch(state, "asset");
    let projection_only = is_projection_only_consumer_source(record.consumer_source.as_deref());
    if state.options.emit_standalone_transport || projection_only {
        state
            .response
            .non_source_asset_record_ids
            .push(record.record_id.clone());
        if let Some(resolved_file) = record
            .resolved_file
            .as_deref()
            .filter(|path| !path.is_empty())
        {
            state
                .response
                .non_source_asset_record_targets
                .push(ResolvedRecordTarget {
                    record_id: record.record_id.clone(),
                    resolved_file: resolved_file.to_string(),
                });
        }
    }
    mark_handled(state, record.record_id);
    if !projection_only {
        state.response.counters.non_source_asset_uses += 1;
    }
}

pub(super) fn handle_unresolved_record(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
    track_prefix: bool,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id.clone();
    increment_branch(state, "unresolved");
    mark_handled(state, record_id);
    if is_namespace_reexport_use(&kind) {
        increment_branch(state, "skippedNamespaceAlias");
        return;
    }

    state.response.counters.unresolved_uses += 1;
    state.response.counters.unresolved_internal_uses += 1;

    let from_spec = record.from_spec.clone().unwrap_or_default();
    if from_spec.is_empty() {
        return;
    }
    if track_prefix {
        let prefix = prefix_of(&from_spec);
        *state
            .response
            .unresolved_internal_by_prefix
            .entry(prefix.clone())
            .or_insert(0) += 1;
        state
            .response
            .prefix_examples
            .entry(prefix)
            .or_insert_with(|| from_spec.clone());
    }

    state
        .response
        .unresolved_internal_specifiers
        .insert(from_spec.clone());
    push_unresolved_specifier_record(state, &record, &from_spec, &kind);
}

pub(super) fn handle_relative_target_missing(
    state: &mut AssemblyState,
    mut record: SourceUseAssemblyRecord,
) {
    record.resolver_stage = Some("unresolved-relative".to_string());
    record.unresolved_evidence = Some(relative_target_missing_evidence(
        record.unresolved_evidence.take(),
    ));
    handle_unresolved_record(state, record, false);
}

fn relative_target_missing_evidence(existing: Option<Value>) -> Value {
    let mut object = match existing {
        Some(Value::Object(object)) => object,
        _ => Map::new(),
    };
    object
        .entry("reason".to_string())
        .or_insert_with(|| json!("relative-target-missing"));
    object
        .entry("resolverStage".to_string())
        .or_insert_with(|| json!("relative"));
    Value::Object(object)
}

pub(super) fn push_unresolved_specifier_record(
    state: &mut AssemblyState,
    record: &SourceUseAssemblyRecord,
    from_spec: &str,
    kind: &str,
) {
    let consumer_file = root_relative(&state.root, &record.consumer_file);
    let mut object = Map::new();
    object.insert("specifier".to_string(), json!(from_spec));
    object.insert("consumerFile".to_string(), json!(consumer_file));
    object.insert(
        "fromHint".to_string(),
        json!(root_relative(&state.root, &record.consumer_file)),
    );
    object.insert("kind".to_string(), json!(kind));
    if record.type_only_present {
        object.insert("typeOnly".to_string(), json!(record.type_only));
    }
    if let Some(Value::Object(evidence)) = record.unresolved_evidence.as_ref() {
        for (key, value) in evidence {
            object.insert(key.clone(), value.clone());
        }
    }
    state
        .response
        .unresolved_internal_specifier_records
        .push(Value::Object(object));
}

fn package_root_from_spec(spec: &str) -> Option<String> {
    if spec.is_empty() || spec.starts_with('.') || spec.starts_with('/') || spec.starts_with('#') {
        return None;
    }
    if spec.starts_with('@') {
        let mut parts = spec.split('/');
        let scope = parts.next()?;
        let package = parts.next()?;
        if package.is_empty() {
            return None;
        }
        return Some(format!("{scope}/{package}"));
    }
    spec.split('/').next().map(ToString::to_string)
}

fn prefix_of(spec: &str) -> String {
    spec.find('/')
        .filter(|slash| *slash > 0)
        .map(|slash| spec[..=slash].to_string())
        .unwrap_or_else(|| spec.to_string())
}
