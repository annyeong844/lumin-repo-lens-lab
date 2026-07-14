use super::super::input::SourceUseAssemblyRecord;
use super::super::path::root_relative;
use super::super::protocol::{
    DirectConsumerAddition, NamespaceReExportDiagnosticAddition, NamespaceUserAddition,
    ResolvedInternalEdge,
};
use super::support::{
    edge_kind_for_use, increment_branch, increment_out_of_band_consumer_counter,
    is_broad_namespace_use, is_namespace_reexport_use, mark_handled, push_resolved_record_target,
    requires_symbol_name, skip, AssemblyState,
};

pub(super) fn handle_projection_only_target(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
    resolved_file: &str,
) {
    let record_id = record.record_id;
    increment_branch(state, "projectionOnlyTarget");
    push_resolved_record_target(state, &record_id, resolved_file);
    mark_handled(state, record_id);
}

pub(super) fn handle_resolved_internal_record(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
    resolved_file: String,
    rust_resolved_relative: bool,
) {
    let kind = record.kind.as_deref().unwrap_or("import");
    if is_namespace_reexport_use(kind) {
        handle_namespace_reexport_record(state, record, resolved_file, rust_resolved_relative);
        return;
    }
    if requires_symbol_name(kind) && record.name.as_deref().map(str::is_empty).unwrap_or(true) {
        skip(state, record.record_id, "missing-symbol-name");
        return;
    }

    let from = root_relative(&state.root, &record.consumer_file);
    let to = root_relative(&state.root, &resolved_file);
    let record_id = record.record_id;
    let source = record.from_spec.clone();

    if state.options.emit_standalone_transport {
        push_resolved_record_target(state, &record_id, &resolved_file);
    }
    mark_handled(state, record_id);
    state.response.counters.total_uses += 1;
    state.response.counters.resolved_internal_uses += 1;
    if rust_resolved_relative {
        state.response.counters.rust_resolved_relative_uses += 1;
    }
    increment_out_of_band_consumer_counter(
        &mut state.response.counters,
        record.consumer_source.as_deref(),
    );
    increment_branch(state, "resolvedInternal");
    state
        .response
        .resolved_internal_edges
        .push(ResolvedInternalEdge {
            from: from.clone(),
            to: to.clone(),
            kind: edge_kind_for_use(kind).to_string(),
            source,
            type_only: record.type_only,
            line: record.line,
            sfc_language: record.sfc_language,
        });

    if kind == "cjs-side-effect-only" || kind == "import-side-effect" {
        increment_branch(state, "sideEffectOnly");
        return;
    }
    if kind == "sfc-script-src" {
        increment_branch(state, "sfcScriptSrcReachability");
        return;
    }
    if kind == "reExportNamespace" {
        increment_branch(state, "reExportNamespaceSkip");
        return;
    }
    if is_broad_namespace_use(kind) {
        increment_branch(state, "broadNamespace");
        if state
            .namespace_users_seen
            .insert((to.clone(), from.clone()))
        {
            state.response.namespace_users.push(NamespaceUserAddition {
                def_file: to,
                consumer_file: from,
            });
        }
        return;
    }

    let symbol = record.name.unwrap_or_default();
    increment_branch(state, "directConsumer");
    state
        .response
        .direct_consumers
        .push(DirectConsumerAddition {
            def_file: to,
            symbol,
            consumer_file: from,
            space: if record.type_only { "type" } else { "value" },
        });
}

fn handle_namespace_reexport_record(
    state: &mut AssemblyState,
    record: SourceUseAssemblyRecord,
    resolved_file: String,
    rust_resolved_relative: bool,
) {
    let kind = record.kind.as_deref().unwrap_or("import");
    let Some(exported_name) = record
        .name
        .as_deref()
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
    else {
        skip(state, record.record_id, "missing-symbol-name");
        return;
    };
    let from = root_relative(&state.root, &record.consumer_file);
    let import_file = root_relative(&state.root, &resolved_file);
    let record_id = record.record_id;
    let source = record.from_spec.clone().unwrap_or_default();
    let line = record.line;
    increment_branch(state, "namespaceReExport");
    if state.options.emit_standalone_transport {
        push_resolved_record_target(state, &record_id, &resolved_file);
    }
    mark_handled(state, record_id);

    let Some(re_export) =
        state
            .namespace_resolver
            .resolve(&state.root, &resolved_file, &exported_name)
    else {
        increment_branch(state, "namespaceReExportMiss");
        return;
    };

    let target = root_relative(&state.root, &re_export.target_file);
    state.response.counters.total_uses += 1;
    state.response.counters.resolved_internal_uses += 1;
    if rust_resolved_relative {
        state.response.counters.rust_resolved_relative_uses += 1;
    }
    increment_out_of_band_consumer_counter(
        &mut state.response.counters,
        record.consumer_source.as_deref(),
    );
    state
        .response
        .resolved_internal_edges
        .push(ResolvedInternalEdge {
            from: from.clone(),
            to: target.clone(),
            kind: edge_kind_for_use(kind).to_string(),
            source: Some(source.clone()),
            type_only: record.type_only,
            line,
            sfc_language: record.sfc_language,
        });

    if kind == "imported-namespace-escape" {
        increment_branch(state, "namespaceReExportEscape");
        state
            .response
            .namespace_re_export_diagnostics
            .push(NamespaceReExportDiagnosticAddition {
                kind: "opaque-namespace-escape",
                reason: "namespace-object-escaped",
                consumer_file: from.clone(),
                import_file,
                exported_name,
                target_file: target.clone(),
                source,
                line,
                chain: re_export.chain,
            });
        if state
            .namespace_users_seen
            .insert((target.clone(), from.clone()))
        {
            state.response.namespace_users.push(NamespaceUserAddition {
                def_file: target,
                consumer_file: from,
            });
        }
    } else if let Some(member_name) = record
        .member_name
        .as_deref()
        .filter(|name| !name.is_empty())
    {
        increment_branch(state, "namespaceReExportMember");
        state
            .response
            .direct_consumers
            .push(DirectConsumerAddition {
                def_file: target,
                symbol: member_name.to_string(),
                consumer_file: from,
                space: if record.type_only { "type" } else { "value" },
            });
    }
}
