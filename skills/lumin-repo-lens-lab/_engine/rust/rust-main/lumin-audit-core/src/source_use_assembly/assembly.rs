use anyhow::{bail, Result};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::relative_source_resolver::{normalize_path_text, RelativeSourceResolver};

use super::glob::{expand_import_meta_glob, ImportMetaGlobExpansion};
use super::input::{
    normalize_record, record_inputs_from_request, source_files_from_request,
    SourceUseAssemblyRecord, SourceUseAssemblyTables,
};
use super::namespace::NamespaceReExportResolver;
use super::path::root_relative;
use super::protocol::{
    DependencyImportConsumerAddition, DirectConsumerAddition, NamespaceReExportDiagnosticAddition,
    NamespaceUserAddition, ResolvedInternalEdge, ResolvedRecordTarget, SkippedSourceUseRecord,
    SourceUseAssemblyCounters, SourceUseAssemblyRequest, SourceUseAssemblyResponse,
    SourceUseAssemblySummary, SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
    SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
};

#[derive(Clone, Copy)]
struct SourceUseAssemblyBuildOptions {
    emit_standalone_transport: bool,
    relative_target_missing_is_unresolved: bool,
}

const STANDALONE_BUILD_OPTIONS: SourceUseAssemblyBuildOptions = SourceUseAssemblyBuildOptions {
    emit_standalone_transport: true,
    relative_target_missing_is_unresolved: false,
};

const EMBEDDED_BUILD_OPTIONS: SourceUseAssemblyBuildOptions = SourceUseAssemblyBuildOptions {
    emit_standalone_transport: false,
    relative_target_missing_is_unresolved: true,
};

pub fn build_source_use_assembly_response(
    request: SourceUseAssemblyRequest,
) -> Result<SourceUseAssemblyResponse> {
    build_source_use_assembly_response_with_options(request, STANDALONE_BUILD_OPTIONS, None)
}

pub fn build_embedded_source_use_assembly_response(
    request: SourceUseAssemblyRequest,
) -> Result<SourceUseAssemblyResponse> {
    build_source_use_assembly_response_with_options(request, EMBEDDED_BUILD_OPTIONS, None)
}

pub(crate) fn build_embedded_source_use_assembly_response_with_path_table(
    request: SourceUseAssemblyRequest,
    inherited_path_table: &[String],
) -> Result<SourceUseAssemblyResponse> {
    build_source_use_assembly_response_with_options(
        request,
        EMBEDDED_BUILD_OPTIONS,
        Some(inherited_path_table),
    )
}

fn build_source_use_assembly_response_with_options(
    request: SourceUseAssemblyRequest,
    options: SourceUseAssemblyBuildOptions,
    inherited_path_table: Option<&[String]>,
) -> Result<SourceUseAssemblyResponse> {
    if request.schema_version != SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION {
        bail!(
            "source-use-assembly-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    let records = record_inputs_from_request(
        request.records,
        request.record_row_fields,
        request.record_rows,
    )?;
    let record_count = records.len();

    let mut response = SourceUseAssemblyResponse {
        schema_version: SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
        root: request.root.clone(),
        summary: SourceUseAssemblySummary {
            record_count,
            ..SourceUseAssemblySummary::default()
        },
        handled_record_ids: Vec::new(),
        resolved_record_targets: Vec::new(),
        external_record_ids: Vec::new(),
        skipped_records: Vec::new(),
        counters: SourceUseAssemblyCounters::default(),
        branch_counts: BTreeMap::new(),
        resolved_internal_edges: Vec::new(),
        dependency_import_consumers: Vec::new(),
        unresolved_internal_by_prefix: BTreeMap::new(),
        prefix_examples: BTreeMap::new(),
        unresolved_internal_specifiers: BTreeSet::new(),
        unresolved_internal_specifier_records: Vec::new(),
        direct_consumers: Vec::new(),
        namespace_users: Vec::new(),
        namespace_re_export_diagnostics: Vec::new(),
        generated_virtual_surfaces: Vec::new(),
        generated_virtual_import_consumers: Vec::new(),
    };

    let root = normalize_path_text(&request.root);
    let import_meta_glob_cap = request.import_meta_glob_cap;
    let path_table = if request.path_table.is_empty() {
        inherited_path_table.unwrap_or(&request.path_table)
    } else {
        &request.path_table
    };
    let source_files =
        source_files_from_request(path_table, request.source_files, request.source_file_ids)?;
    let resolver = RelativeSourceResolver::from_rooted_paths(&root, source_files);
    let namespace_resolver =
        NamespaceReExportResolver::new(request.namespace_re_exports, request.named_re_exports);
    let mut namespace_users_seen = BTreeSet::new();

    for (index, record) in records.into_iter().enumerate() {
        let record = normalize_record(
            record,
            index,
            SourceUseAssemblyTables {
                path_table,
                kind_table: &request.kind_table,
                resolver_stage_table: &request.resolver_stage_table,
                consumer_source_table: &request.consumer_source_table,
                specifier_table: &request.specifier_table,
                name_table: &request.name_table,
            },
        )?;
        let resolver_stage = record.resolver_stage.as_deref();
        let has_pre_resolved_file = record
            .resolved_file
            .as_deref()
            .is_some_and(|path| !path.is_empty());
        if resolver_stage == Some("external") {
            handle_external_record(&mut response, &root, record, options);
            continue;
        }
        if resolver_stage == Some("generated-virtual") {
            handle_generated_virtual_record(&mut response, &root, record, options);
            continue;
        }
        if resolver_stage == Some("non-source-asset") {
            handle_non_source_asset_record(&mut response, record, options);
            continue;
        }
        let track_unresolved_prefix = resolver_stage == Some("unresolved-internal");
        if matches!(
            resolver_stage,
            Some("unresolved-internal" | "unresolved-relative")
        ) {
            handle_unresolved_record(
                &mut response,
                &root,
                record,
                track_unresolved_prefix,
                options,
            );
            continue;
        }
        let kind = record.kind.as_deref().unwrap_or("import");
        let supported_stage = match resolver_stage {
            Some("relative") => true,
            Some("resolved-internal") => has_pre_resolved_file,
            Some("import-meta-glob") if kind == "import-meta-glob" => true,
            Some(_) => false,
            None => true,
        };
        if !supported_stage {
            skip(
                &mut response,
                options,
                record.record_id,
                "non-relative-resolver-stage",
            );
            continue;
        }
        let from_spec = record.from_spec.as_deref().unwrap_or_default();
        if kind == "import-meta-glob" {
            handle_import_meta_glob_record(
                &mut response,
                &root,
                &resolver,
                &mut namespace_users_seen,
                record,
                import_meta_glob_cap,
                options,
            );
            continue;
        }
        if is_projection_only_consumer_source(record.consumer_source.as_deref())
            && has_pre_resolved_file
        {
            let resolved_file = record.resolved_file.clone().unwrap_or_default();
            let record_id = record.record_id;
            increment_branch(&mut response.branch_counts, "projectionOnlyTarget");
            push_resolved_record_target(&mut response, &record_id, &resolved_file);
            mark_handled(&mut response, options, record_id);
            continue;
        }
        if !has_pre_resolved_file && !is_relative_spec(from_spec) {
            skip(
                &mut response,
                options,
                record.record_id,
                "non-relative-specifier",
            );
            continue;
        }
        if looks_like_non_source_asset(from_spec) {
            skip(
                &mut response,
                options,
                record.record_id,
                "non-source-asset-specifier",
            );
            continue;
        }

        let resolved_file = record
            .resolved_file
            .as_deref()
            .filter(|path| !path.is_empty())
            .map(ToString::to_string)
            .or_else(|| resolver.resolve(&record.consumer_file, from_spec));
        let Some(resolved_file) = resolved_file else {
            if options.relative_target_missing_is_unresolved {
                handle_relative_target_missing(&mut response, &root, record, options);
            } else {
                skip(
                    &mut response,
                    options,
                    record.record_id,
                    "relative-target-missing",
                );
            }
            continue;
        };

        if is_projection_only_consumer_source(record.consumer_source.as_deref()) {
            let record_id = record.record_id;
            increment_branch(&mut response.branch_counts, "projectionOnlyTarget");
            push_resolved_record_target(&mut response, &record_id, &resolved_file);
            mark_handled(&mut response, options, record_id);
            continue;
        }

        if is_namespace_reexport_use(kind) {
            let Some(exported_name) = record
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .map(ToString::to_string)
            else {
                skip(
                    &mut response,
                    options,
                    record.record_id,
                    "missing-symbol-name",
                );
                continue;
            };
            let from = root_relative(&root, &record.consumer_file);
            let import_file = root_relative(&root, &resolved_file);
            let record_id = record.record_id;
            let source = record.from_spec.clone().unwrap_or_default();
            let line = record.line;
            increment_branch(&mut response.branch_counts, "namespaceReExport");
            if options.emit_standalone_transport {
                push_resolved_record_target(&mut response, &record_id, &resolved_file);
            }
            mark_handled(&mut response, options, record_id);

            let Some(re_export) = namespace_resolver.resolve(&root, &resolved_file, &exported_name)
            else {
                increment_branch(&mut response.branch_counts, "namespaceReExportMiss");
                continue;
            };

            let target = root_relative(&root, &re_export.target_file);
            response.counters.total_uses += 1;
            response.counters.resolved_internal_uses += 1;
            if resolver_stage == Some("relative") {
                response.counters.rust_resolved_relative_uses += 1;
            }
            increment_out_of_band_consumer_counter(
                &mut response.counters,
                record.consumer_source.as_deref(),
            );
            response.resolved_internal_edges.push(ResolvedInternalEdge {
                from: from.clone(),
                to: target.clone(),
                kind: edge_kind_for_use(kind).to_string(),
                source: Some(source.clone()),
                type_only: record.type_only,
                line,
                sfc_language: record.sfc_language,
            });

            if kind == "imported-namespace-escape" {
                increment_branch(&mut response.branch_counts, "namespaceReExportEscape");
                response.namespace_re_export_diagnostics.push(
                    NamespaceReExportDiagnosticAddition {
                        kind: "opaque-namespace-escape",
                        reason: "namespace-object-escaped",
                        consumer_file: from.clone(),
                        import_file,
                        exported_name,
                        target_file: target.clone(),
                        source,
                        line,
                        chain: re_export.chain,
                    },
                );
                if namespace_users_seen.insert((target.clone(), from.clone())) {
                    response.namespace_users.push(NamespaceUserAddition {
                        def_file: target,
                        consumer_file: from,
                    });
                }
            } else if let Some(member_name) = record
                .member_name
                .as_deref()
                .filter(|name| !name.is_empty())
            {
                increment_branch(&mut response.branch_counts, "namespaceReExportMember");
                response.direct_consumers.push(DirectConsumerAddition {
                    def_file: target,
                    symbol: member_name.to_string(),
                    consumer_file: from,
                    space: if record.type_only { "type" } else { "value" },
                });
            }
            continue;
        }
        if requires_symbol_name(kind) && record.name.as_deref().map(str::is_empty).unwrap_or(true) {
            skip(
                &mut response,
                options,
                record.record_id,
                "missing-symbol-name",
            );
            continue;
        }

        let from = root_relative(&root, &record.consumer_file);
        let to = root_relative(&root, &resolved_file);
        let record_id = record.record_id;
        let source = record.from_spec.clone();

        if options.emit_standalone_transport {
            push_resolved_record_target(&mut response, &record_id, &resolved_file);
        }
        mark_handled(&mut response, options, record_id);
        response.counters.total_uses += 1;
        response.counters.resolved_internal_uses += 1;
        if resolver_stage == Some("relative") {
            response.counters.rust_resolved_relative_uses += 1;
        }
        increment_out_of_band_consumer_counter(
            &mut response.counters,
            record.consumer_source.as_deref(),
        );
        increment_branch(&mut response.branch_counts, "resolvedInternal");
        response.resolved_internal_edges.push(ResolvedInternalEdge {
            from: from.clone(),
            to: to.clone(),
            kind: edge_kind_for_use(kind).to_string(),
            source,
            type_only: record.type_only,
            line: record.line,
            sfc_language: record.sfc_language,
        });

        if kind == "cjs-side-effect-only" || kind == "import-side-effect" {
            increment_branch(&mut response.branch_counts, "sideEffectOnly");
            continue;
        }
        if kind == "sfc-script-src" {
            increment_branch(&mut response.branch_counts, "sfcScriptSrcReachability");
            continue;
        }
        if kind == "reExportNamespace" {
            increment_branch(&mut response.branch_counts, "reExportNamespaceSkip");
            continue;
        }
        if is_broad_namespace_use(kind) {
            increment_branch(&mut response.branch_counts, "broadNamespace");
            if namespace_users_seen.insert((to.clone(), from.clone())) {
                response.namespace_users.push(NamespaceUserAddition {
                    def_file: to,
                    consumer_file: from,
                });
            }
            continue;
        }

        let symbol = record.name.unwrap_or_default();
        increment_branch(&mut response.branch_counts, "directConsumer");
        response.direct_consumers.push(DirectConsumerAddition {
            def_file: to,
            symbol,
            consumer_file: from,
            space: if record.type_only { "type" } else { "value" },
        });
    }

    Ok(response)
}

fn handle_external_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    record: SourceUseAssemblyRecord,
    options: SourceUseAssemblyBuildOptions,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id;
    let projection_only = is_projection_only_consumer_source(record.consumer_source.as_deref());
    increment_branch(&mut response.branch_counts, "external");
    if options.emit_standalone_transport || projection_only {
        response.external_record_ids.push(record_id.clone());
    }
    mark_handled(response, options, record_id);
    if projection_only {
        return;
    }
    if is_namespace_reexport_use(&kind) {
        increment_branch(&mut response.branch_counts, "skippedNamespaceAlias");
        return;
    }

    response.counters.external_uses += 1;
    response.counters.unresolved_uses += 1;
    let from_spec = record.from_spec.unwrap_or_default();
    let Some(dep_root) = package_root_from_spec(&from_spec) else {
        return;
    };
    response
        .dependency_import_consumers
        .push(DependencyImportConsumerAddition {
            file: root_relative(root, &record.consumer_file),
            from_spec,
            dep_root,
            kind,
            source: record
                .consumer_source
                .unwrap_or_else(|| "source-import".to_string()),
            type_only: record.type_only_present.then_some(record.type_only),
        });
}

fn handle_non_source_asset_record(
    response: &mut SourceUseAssemblyResponse,
    record: SourceUseAssemblyRecord,
    options: SourceUseAssemblyBuildOptions,
) {
    increment_branch(&mut response.branch_counts, "asset");
    let projection_only = is_projection_only_consumer_source(record.consumer_source.as_deref());
    mark_handled(response, options, record.record_id);
    if !projection_only {
        response.counters.non_source_asset_uses += 1;
    }
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

fn handle_unresolved_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    record: SourceUseAssemblyRecord,
    track_prefix: bool,
    options: SourceUseAssemblyBuildOptions,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id.clone();
    increment_branch(&mut response.branch_counts, "unresolved");
    mark_handled(response, options, record_id);
    if is_namespace_reexport_use(&kind) {
        increment_branch(&mut response.branch_counts, "skippedNamespaceAlias");
        return;
    }

    response.counters.unresolved_uses += 1;
    response.counters.unresolved_internal_uses += 1;

    let from_spec = record.from_spec.clone().unwrap_or_default();
    if from_spec.is_empty() {
        return;
    }
    if track_prefix {
        let prefix = prefix_of(&from_spec);
        *response
            .unresolved_internal_by_prefix
            .entry(prefix.clone())
            .or_insert(0) += 1;
        response
            .prefix_examples
            .entry(prefix)
            .or_insert_with(|| from_spec.clone());
    }

    response
        .unresolved_internal_specifiers
        .insert(from_spec.clone());
    push_unresolved_specifier_record(response, root, &record, &from_spec, &kind);
}

fn handle_relative_target_missing(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    mut record: SourceUseAssemblyRecord,
    options: SourceUseAssemblyBuildOptions,
) {
    record.resolver_stage = Some("unresolved-relative".to_string());
    record.unresolved_evidence = Some(relative_target_missing_evidence(
        record.unresolved_evidence.take(),
    ));
    handle_unresolved_record(response, root, record, false, options);
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

fn push_unresolved_specifier_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    record: &SourceUseAssemblyRecord,
    from_spec: &str,
    kind: &str,
) {
    let consumer_file = root_relative(root, &record.consumer_file);
    let mut object = Map::new();
    object.insert("specifier".to_string(), json!(from_spec));
    object.insert("consumerFile".to_string(), json!(consumer_file));
    object.insert(
        "fromHint".to_string(),
        json!(root_relative(root, &record.consumer_file)),
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
    response
        .unresolved_internal_specifier_records
        .push(Value::Object(object));
}

fn handle_import_meta_glob_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    resolver: &RelativeSourceResolver,
    namespace_users_seen: &mut BTreeSet<(String, String)>,
    record: SourceUseAssemblyRecord,
    cap: usize,
    options: SourceUseAssemblyBuildOptions,
) {
    let record_id = record.record_id.clone();
    match expand_import_meta_glob(root, resolver, &record, cap) {
        ImportMetaGlobExpansion::Resolved { targets } => {
            increment_branch(&mut response.branch_counts, "importMetaGlobResolved");
            mark_handled(response, options, record_id);
            let from = root_relative(root, &record.consumer_file);
            let source = record.from_spec.clone();
            for target in targets {
                let to = root_relative(root, &target);
                response.counters.total_uses += 1;
                response.counters.resolved_internal_uses += 1;
                response.resolved_internal_edges.push(ResolvedInternalEdge {
                    from: from.clone(),
                    to: to.clone(),
                    kind: "dynamic-import-meta-glob".to_string(),
                    source: source.clone(),
                    type_only: false,
                    line: record.line,
                    sfc_language: record.sfc_language.clone(),
                });
                if namespace_users_seen.insert((to.clone(), from.clone())) {
                    response.namespace_users.push(NamespaceUserAddition {
                        def_file: to,
                        consumer_file: from.clone(),
                    });
                }
            }
        }
        ImportMetaGlobExpansion::Unsupported { evidence } => {
            increment_branch(&mut response.branch_counts, "importMetaGlobUnsupported");
            increment_branch(&mut response.branch_counts, "unresolved");
            mark_handled(response, options, record_id);
            response.counters.unresolved_uses += 1;
            response.counters.unresolved_internal_uses += 1;
            let from_spec = record.from_spec.clone().unwrap_or_default();
            if !from_spec.is_empty() {
                response
                    .unresolved_internal_specifiers
                    .insert(from_spec.clone());
                let mut diagnostic = record;
                diagnostic.unresolved_evidence = Some(Value::Object(evidence));
                push_unresolved_specifier_record(
                    response,
                    root,
                    &diagnostic,
                    &from_spec,
                    "import-meta-glob",
                );
            }
        }
    }
}

fn handle_generated_virtual_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    record: SourceUseAssemblyRecord,
    options: SourceUseAssemblyBuildOptions,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id.clone();
    increment_branch(&mut response.branch_counts, "generatedVirtual");
    mark_handled(response, options, record_id);
    if is_namespace_reexport_use(&kind) {
        increment_branch(&mut response.branch_counts, "skippedNamespaceAlias");
        return;
    }

    let Some(surface) = record.generated_virtual_surface.clone() else {
        skip(
            response,
            options,
            record.record_id,
            "generated-virtual-surface-missing",
        );
        return;
    };
    add_generated_virtual_surface(response, surface.clone());

    let Some(exported) = generated_virtual_export_for_use(&surface, &record, &kind) else {
        increment_branch(&mut response.branch_counts, "generatedVirtualUnresolved");
        response.counters.unresolved_uses += 1;
        response.counters.unresolved_internal_uses += 1;
        let from_spec = record.from_spec.clone().unwrap_or_default();
        if !from_spec.is_empty() {
            response
                .unresolved_internal_specifiers
                .insert(from_spec.clone());
            push_unresolved_specifier_record(response, root, &record, &from_spec, &kind);
        }
        return;
    };

    response.counters.total_uses += 1;
    response.counters.resolved_internal_uses += 1;
    response.counters.resolved_generated_virtual_uses += 1;

    let mut object = Map::new();
    object.insert(
        "consumerFile".to_string(),
        json!(root_relative(root, &record.consumer_file)),
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
    response
        .generated_virtual_import_consumers
        .push(Value::Object(object));
}

fn add_generated_virtual_surface(response: &mut SourceUseAssemblyResponse, surface: Value) {
    let Some(id) = surface.get("id").and_then(Value::as_str) else {
        response.generated_virtual_surfaces.push(surface);
        return;
    };
    if response
        .generated_virtual_surfaces
        .iter()
        .any(|value| value.get("id").and_then(Value::as_str) == Some(id))
    {
        return;
    }
    response.generated_virtual_surfaces.push(surface);
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

fn prefix_of(spec: &str) -> String {
    spec.find('/')
        .filter(|slash| *slash > 0)
        .map(|slash| spec[..=slash].to_string())
        .unwrap_or_else(|| spec.to_string())
}

fn mark_handled(
    response: &mut SourceUseAssemblyResponse,
    options: SourceUseAssemblyBuildOptions,
    record_id: String,
) {
    response.summary.handled_count += 1;
    if options.emit_standalone_transport {
        response.handled_record_ids.push(record_id);
    }
}

fn skip(
    response: &mut SourceUseAssemblyResponse,
    options: SourceUseAssemblyBuildOptions,
    record_id: String,
    reason: &'static str,
) {
    response.summary.skipped_count += 1;
    if options.emit_standalone_transport {
        response
            .skipped_records
            .push(SkippedSourceUseRecord { record_id, reason });
    }
}

fn push_resolved_record_target(
    response: &mut SourceUseAssemblyResponse,
    record_id: &str,
    resolved_file: &str,
) {
    response.resolved_record_targets.push(ResolvedRecordTarget {
        record_id: record_id.to_string(),
        resolved_file: resolved_file.to_string(),
    });
}

fn increment_branch(branch_counts: &mut BTreeMap<String, usize>, name: &str) {
    *branch_counts.entry(name.to_string()).or_insert(0) += 1;
}

fn increment_out_of_band_consumer_counter(
    counters: &mut SourceUseAssemblyCounters,
    consumer_source: Option<&str>,
) {
    match consumer_source {
        Some("mdx-import") => counters.mdx_consumer_uses += 1,
        Some("sfc-script-import") => counters.sfc_script_consumer_uses += 1,
        Some("sfc-script-src") => counters.sfc_script_src_reachability_uses += 1,
        _ => {}
    }
}

fn is_projection_only_consumer_source(consumer_source: Option<&str>) -> bool {
    matches!(
        consumer_source,
        Some(
            "sfc-template-component-ref"
                | "sfc-global-component-registration"
                | "sfc-generated-component-manifest"
        )
    )
}

fn is_namespace_reexport_use(kind: &str) -> bool {
    kind == "imported-namespace-member" || kind == "imported-namespace-escape"
}

fn is_relative_spec(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../")
}

fn looks_like_non_source_asset(spec: &str) -> bool {
    let stripped = strip_resource_query(spec);
    has_extension(stripped) && !js_source_extension(stripped)
}

fn strip_resource_query(spec: &str) -> &str {
    let query = spec.find('?');
    let fragment = spec.find('#').filter(|index| *index > 0);
    match (query, fragment) {
        (Some(left), Some(right)) => &spec[..left.min(right)],
        (Some(index), None) | (None, Some(index)) => &spec[..index],
        (None, None) => spec,
    }
}

fn has_extension(spec: &str) -> bool {
    let file_name = spec.rsplit('/').next().unwrap_or(spec);
    file_name
        .rfind('.')
        .is_some_and(|index| index > 0 && index + 1 < file_name.len())
}

fn js_source_extension(spec: &str) -> bool {
    let lower = spec.to_ascii_lowercase();
    [
        ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
    ]
    .iter()
    .any(|ext| lower.ends_with(ext))
}

fn is_broad_namespace_use(kind: &str) -> bool {
    matches!(
        kind,
        "namespace"
            | "reExportAll"
            | "dynamic"
            | "cjs-namespace-escape"
            | "cjs-reexport-broad"
            | "dynamic-import-meta-glob"
    )
}

fn requires_symbol_name(kind: &str) -> bool {
    !matches!(
        kind,
        "cjs-side-effect-only"
            | "import-side-effect"
            | "reExportNamespace"
            | "sfc-script-src"
            | "namespace"
            | "reExportAll"
            | "dynamic"
            | "import-meta-glob"
            | "dynamic-import-meta-glob"
            | "cjs-namespace-escape"
            | "cjs-reexport-broad"
    )
}

fn edge_kind_for_use(kind: &str) -> &str {
    match kind {
        "import" => "import-named",
        "default" => "import-default",
        "namespace" | "namespace-member" => "import-namespace",
        "import-side-effect" => "import-side-effect",
        "reExport" => "reexport-named",
        "reExportAll" => "reexport-broad",
        "reExportNamespace" => "reexport-namespace",
        "imported-namespace-member" => "reexport-namespace-member",
        "imported-namespace-escape" => "reexport-namespace-escape",
        "dynamic" | "dynamic-member" => "dynamic-literal",
        "cjs-side-effect-only" => "cjs-side-effect",
        "cjs-require-exact" => "cjs-require-exact",
        "cjs-namespace-member" => "cjs-namespace-member",
        "cjs-namespace-escape" => "cjs-namespace-escape",
        "cjs-reexport-broad" => "cjs-reexport-broad",
        "sfc-script-src" => "sfc-script-src",
        other => other,
    }
}

#[cfg(test)]
#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
