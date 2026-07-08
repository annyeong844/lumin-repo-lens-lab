use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION: &str = "lumin-source-use-assembly-request.v1";
pub const SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION: &str =
    "lumin-source-use-assembly-response.v1";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyRequest {
    pub schema_version: String,
    pub root: String,
    #[serde(default = "default_import_meta_glob_cap")]
    pub import_meta_glob_cap: usize,
    #[serde(default)]
    pub source_files: Vec<String>,
    #[serde(default)]
    pub namespace_re_exports: Vec<SourceUseAssemblyReExport>,
    #[serde(default)]
    pub named_re_exports: Vec<SourceUseAssemblyReExport>,
    #[serde(default)]
    pub path_table: Vec<String>,
    #[serde(default)]
    pub kind_table: Vec<String>,
    #[serde(default)]
    pub resolver_stage_table: Vec<String>,
    #[serde(default)]
    pub consumer_source_table: Vec<String>,
    #[serde(default)]
    pub specifier_table: Vec<String>,
    #[serde(default)]
    pub records: Vec<SourceUseAssemblyRecordInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyReExport {
    pub barrel_file: String,
    pub exported_name: String,
    pub target_file: String,
    #[serde(default)]
    pub source_spec: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyRecordInput {
    pub record_id: String,
    #[serde(default)]
    pub consumer_file: Option<String>,
    #[serde(default)]
    pub consumer_file_id: Option<usize>,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub resolved_file_id: Option<usize>,
    #[serde(default)]
    pub from_spec: Option<String>,
    #[serde(default)]
    pub from_spec_id: Option<usize>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub member_name: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub kind_id: Option<usize>,
    #[serde(default)]
    pub type_only: bool,
    #[serde(default)]
    pub type_only_present: bool,
    #[serde(default)]
    pub line: Option<u64>,
    #[serde(default)]
    pub sfc_language: Option<String>,
    #[serde(default)]
    pub resolver_stage: Option<String>,
    #[serde(default)]
    pub resolver_stage_id: Option<usize>,
    #[serde(default)]
    pub consumer_source: Option<String>,
    #[serde(default)]
    pub consumer_source_id: Option<usize>,
    #[serde(default)]
    pub unresolved_evidence: Option<Value>,
    #[serde(default)]
    pub generated_virtual_surface: Option<Value>,
}

#[derive(Debug)]
pub struct SourceUseAssemblyRecord {
    pub record_id: String,
    pub consumer_file: String,
    pub resolved_file: Option<String>,
    pub from_spec: Option<String>,
    pub name: Option<String>,
    pub member_name: Option<String>,
    pub kind: Option<String>,
    pub type_only: bool,
    pub type_only_present: bool,
    pub line: Option<u64>,
    pub sfc_language: Option<String>,
    pub resolver_stage: Option<String>,
    pub consumer_source: Option<String>,
    pub unresolved_evidence: Option<Value>,
    pub generated_virtual_surface: Option<Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyResponse {
    pub schema_version: &'static str,
    pub root: String,
    pub summary: SourceUseAssemblySummary,
    pub handled_record_ids: Vec<String>,
    pub resolved_record_targets: Vec<ResolvedRecordTarget>,
    pub skipped_records: Vec<SkippedSourceUseRecord>,
    pub counters: SourceUseAssemblyCounters,
    pub branch_counts: BTreeMap<String, usize>,
    pub resolved_internal_edges: Vec<ResolvedInternalEdge>,
    pub dependency_import_consumers: Vec<DependencyImportConsumerAddition>,
    pub unresolved_internal_by_prefix: BTreeMap<String, usize>,
    pub prefix_examples: BTreeMap<String, String>,
    pub unresolved_internal_specifiers: BTreeSet<String>,
    pub unresolved_internal_specifier_records: Vec<Value>,
    pub direct_consumers: Vec<DirectConsumerAddition>,
    pub namespace_users: Vec<NamespaceUserAddition>,
    pub namespace_re_export_diagnostics: Vec<NamespaceReExportDiagnosticAddition>,
    pub generated_virtual_surfaces: Vec<Value>,
    pub generated_virtual_import_consumers: Vec<Value>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblySummary {
    pub record_count: usize,
    pub handled_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyCounters {
    pub total_uses: usize,
    pub resolved_internal_uses: usize,
    pub rust_resolved_relative_uses: usize,
    pub non_source_asset_uses: usize,
    pub mdx_consumer_uses: usize,
    pub sfc_script_consumer_uses: usize,
    pub sfc_script_src_reachability_uses: usize,
    pub external_uses: usize,
    pub unresolved_uses: usize,
    pub unresolved_internal_uses: usize,
    pub resolved_generated_virtual_uses: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedInternalEdge {
    pub from: String,
    pub to: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub type_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sfc_language: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRecordTarget {
    pub record_id: String,
    pub resolved_file: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyImportConsumerAddition {
    pub file: String,
    pub from_spec: String,
    pub dep_root: String,
    pub kind: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_only: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectConsumerAddition {
    pub def_file: String,
    pub symbol: String,
    pub consumer_file: String,
    pub space: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceUserAddition {
    pub def_file: String,
    pub consumer_file: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceReExportDiagnosticAddition {
    pub kind: &'static str,
    pub reason: &'static str,
    pub consumer_file: String,
    pub import_file: String,
    pub exported_name: String,
    pub target_file: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chain: Vec<NamespaceReExportChainEntry>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceReExportChainEntry {
    pub kind: &'static str,
    pub file: String,
    pub exported_name: String,
    pub target_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedSourceUseRecord {
    pub record_id: String,
    pub reason: &'static str,
}

fn default_import_meta_glob_cap() -> usize {
    64
}

fn source_use_path_from_table(path_table: &[String], id: usize, field: &str) -> Result<String> {
    path_table
        .get(id)
        .filter(|path| !path.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid {field} {id}"))
}

fn source_use_string_from_table(table: &[String], id: usize, field: &str) -> Result<String> {
    table
        .get(id)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("source-use-assembly-artifact: invalid {field} {id}"))
}

fn normalize_source_use_record(
    input: SourceUseAssemblyRecordInput,
    path_table: &[String],
    kind_table: &[String],
    resolver_stage_table: &[String],
    consumer_source_table: &[String],
    specifier_table: &[String],
) -> Result<SourceUseAssemblyRecord> {
    let consumer_file = match (input.consumer_file, input.consumer_file_id) {
        (Some(path), _) if !path.is_empty() => path,
        (_, Some(id)) => source_use_path_from_table(path_table, id, "consumerFileId")?,
        _ => bail!(
            "source-use-assembly-artifact: record '{}' missing consumerFile",
            input.record_id
        ),
    };
    let resolved_file = match (input.resolved_file, input.resolved_file_id) {
        (Some(path), _) if !path.is_empty() => Some(path),
        (_, Some(id)) => Some(source_use_path_from_table(
            path_table,
            id,
            "resolvedFileId",
        )?),
        _ => None,
    };
    let kind = match (input.kind, input.kind_id) {
        (Some(kind), _) if !kind.is_empty() => Some(kind),
        (_, Some(id)) => Some(source_use_string_from_table(kind_table, id, "kindId")?),
        _ => None,
    };
    let resolver_stage = match (input.resolver_stage, input.resolver_stage_id) {
        (Some(stage), _) if !stage.is_empty() => Some(stage),
        (_, Some(id)) => Some(source_use_string_from_table(
            resolver_stage_table,
            id,
            "resolverStageId",
        )?),
        _ => None,
    };
    let consumer_source = match (input.consumer_source, input.consumer_source_id) {
        (Some(source), _) if !source.is_empty() => Some(source),
        (_, Some(id)) => Some(source_use_string_from_table(
            consumer_source_table,
            id,
            "consumerSourceId",
        )?),
        _ => None,
    };
    let from_spec = match (input.from_spec, input.from_spec_id) {
        (Some(spec), _) if !spec.is_empty() => Some(spec),
        (_, Some(id)) => Some(source_use_string_from_table(
            specifier_table,
            id,
            "fromSpecId",
        )?),
        _ => None,
    };

    Ok(SourceUseAssemblyRecord {
        record_id: input.record_id,
        consumer_file,
        resolved_file,
        from_spec,
        name: input.name,
        member_name: input.member_name,
        kind,
        type_only: input.type_only,
        type_only_present: input.type_only_present,
        line: input.line,
        sfc_language: input.sfc_language,
        resolver_stage,
        consumer_source,
        unresolved_evidence: input.unresolved_evidence,
        generated_virtual_surface: input.generated_virtual_surface,
    })
}

pub fn build_source_use_assembly_response(
    request: SourceUseAssemblyRequest,
) -> Result<SourceUseAssemblyResponse> {
    if request.schema_version != SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION {
        bail!(
            "source-use-assembly-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }

    let mut response = SourceUseAssemblyResponse {
        schema_version: SOURCE_USE_ASSEMBLY_RESPONSE_SCHEMA_VERSION,
        root: request.root.clone(),
        summary: SourceUseAssemblySummary {
            record_count: request.records.len(),
            ..SourceUseAssemblySummary::default()
        },
        handled_record_ids: Vec::new(),
        resolved_record_targets: Vec::new(),
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
    let resolver = RelativeSourceResolver::new(&root, request.source_files);
    let namespace_resolver =
        NamespaceReExportResolver::new(request.namespace_re_exports, request.named_re_exports);
    let mut namespace_users_seen = BTreeSet::new();

    for record in request.records {
        let record = normalize_source_use_record(
            record,
            &request.path_table,
            &request.kind_table,
            &request.resolver_stage_table,
            &request.consumer_source_table,
            &request.specifier_table,
        )?;
        let resolver_stage = record.resolver_stage.as_deref();
        let has_pre_resolved_file = record
            .resolved_file
            .as_deref()
            .is_some_and(|path| !path.is_empty());
        if resolver_stage == Some("external") {
            handle_external_record(&mut response, &root, record);
            continue;
        }
        if resolver_stage == Some("generated-virtual") {
            handle_generated_virtual_record(&mut response, &root, record);
            continue;
        }
        if resolver_stage == Some("non-source-asset") {
            handle_non_source_asset_record(&mut response, record);
            continue;
        }
        let track_unresolved_prefix = resolver_stage == Some("unresolved-internal");
        if matches!(
            resolver_stage,
            Some("unresolved-internal" | "unresolved-relative")
        ) {
            handle_unresolved_record(&mut response, &root, record, track_unresolved_prefix);
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
            );
            continue;
        }
        if !has_pre_resolved_file && !is_relative_spec(from_spec) {
            skip(&mut response, record.record_id, "non-relative-specifier");
            continue;
        }
        if looks_like_non_source_asset(from_spec) {
            skip(
                &mut response,
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
            skip(&mut response, record.record_id, "relative-target-missing");
            continue;
        };

        if is_namespace_reexport_use(kind) {
            let Some(exported_name) = record
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .map(ToString::to_string)
            else {
                skip(&mut response, record.record_id, "missing-symbol-name");
                continue;
            };
            let from = root_relative(&root, &record.consumer_file);
            let import_file = root_relative(&root, &resolved_file);
            let record_id = record.record_id;
            let source = record.from_spec.clone().unwrap_or_default();
            let line = record.line;
            increment_branch(&mut response.branch_counts, "namespaceReExport");
            push_resolved_record_target(&mut response, &record_id, &resolved_file);
            response.handled_record_ids.push(record_id);

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
            skip(&mut response, record.record_id, "missing-symbol-name");
            continue;
        }

        let from = root_relative(&root, &record.consumer_file);
        let to = root_relative(&root, &resolved_file);
        let record_id = record.record_id;
        let source = record.from_spec.clone();

        push_resolved_record_target(&mut response, &record_id, &resolved_file);
        response.handled_record_ids.push(record_id);
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

    response.summary.handled_count = response.handled_record_ids.len();
    response.summary.skipped_count = response.skipped_records.len();
    Ok(response)
}

fn handle_external_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    record: SourceUseAssemblyRecord,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id;
    increment_branch(&mut response.branch_counts, "external");
    response.handled_record_ids.push(record_id);
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
) {
    increment_branch(&mut response.branch_counts, "asset");
    response.handled_record_ids.push(record.record_id);
    response.counters.non_source_asset_uses += 1;
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
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id.clone();
    increment_branch(&mut response.branch_counts, "unresolved");
    response.handled_record_ids.push(record_id);
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
) {
    let record_id = record.record_id.clone();
    match expand_import_meta_glob(root, resolver, &record, cap) {
        ImportMetaGlobExpansion::Resolved { targets } => {
            increment_branch(&mut response.branch_counts, "importMetaGlobResolved");
            response.handled_record_ids.push(record_id);
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
            response.handled_record_ids.push(record_id);
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

enum ImportMetaGlobExpansion {
    Resolved { targets: Vec<String> },
    Unsupported { evidence: Map<String, Value> },
}

#[derive(Debug)]
struct ParsedImportMetaGlobPattern {
    segments: Vec<String>,
    star_index: usize,
    prefix: String,
    suffix: String,
}

fn expand_import_meta_glob(
    root: &str,
    resolver: &RelativeSourceResolver,
    record: &SourceUseAssemblyRecord,
    cap: usize,
) -> ImportMetaGlobExpansion {
    let pattern = record.from_spec.as_deref().unwrap_or_default();
    let parsed = match validate_import_meta_glob_pattern(pattern) {
        Ok(parsed) => parsed,
        Err(reason) => return unsupported_import_meta_glob(reason, None, None, None),
    };

    let consumer_dir = dirname_text(&record.consumer_file);
    let base_pattern = if parsed.star_index == 0 {
        ".".to_string()
    } else {
        parsed.segments[..parsed.star_index].join("/")
    };
    let base_dir = join_relative_spec(consumer_dir, &base_pattern);
    if !is_inside_or_same(root, &base_dir) {
        return unsupported_import_meta_glob(
            "import-meta-glob-outside-root-unsupported",
            None,
            None,
            None,
        );
    }

    let mut matches = resolver
        .source_files()
        .filter(|source_file| {
            normalize_path_text(dirname_text(source_file)) == normalize_path_text(&base_dir)
                && is_inside_or_same(root, source_file)
                && basename_text(source_file).is_some_and(|basename| {
                    basename.starts_with(&parsed.prefix) && basename.ends_with(&parsed.suffix)
                })
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    matches.sort_by_key(|path| root_relative(root, path));

    if matches.is_empty() {
        return unsupported_import_meta_glob(
            "import-meta-glob-zero-matches",
            Some(0),
            None,
            Some(relative_scope(root, &base_dir)),
        );
    }
    if matches.len() > cap {
        return unsupported_import_meta_glob(
            "import-meta-glob-match-cap-exceeded",
            Some(matches.len()),
            Some(cap),
            Some(relative_scope(root, &base_dir)),
        );
    }

    ImportMetaGlobExpansion::Resolved { targets: matches }
}

fn validate_import_meta_glob_pattern(
    pattern: &str,
) -> std::result::Result<ParsedImportMetaGlobPattern, &'static str> {
    if pattern.is_empty() || pattern == "import.meta.glob(<nonliteral>)" {
        return Err("import-meta-glob-nonliteral-unsupported");
    }
    let normalized = pattern.replace('\\', "/");
    if !normalized.starts_with("./") && !normalized.starts_with("../") {
        return Err("import-meta-glob-nonrelative-unsupported");
    }
    if normalized.contains('?')
        || normalized.contains('[')
        || normalized.contains(']')
        || normalized.contains('{')
        || normalized.contains('}')
    {
        return Err("import-meta-glob-unsupported-pattern");
    }
    if normalized.chars().filter(|ch| *ch == '*').count() != 1 {
        return Err("import-meta-glob-unsupported-pattern");
    }

    let segments = normalized
        .split('/')
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let Some(star_index) = segments.iter().position(|segment| segment.contains('*')) else {
        return Err("import-meta-glob-unsupported-pattern");
    };
    let star_segment = &segments[star_index];
    if star_segment.is_empty() || star_segment.contains("**") {
        return Err("import-meta-glob-unsupported-pattern");
    }
    let Some((prefix, suffix)) = star_segment.split_once('*') else {
        return Err("import-meta-glob-unsupported-pattern");
    };
    if !is_import_meta_glob_source_suffix(suffix) {
        return Err("import-meta-glob-target-extension-unsupported");
    }
    let prefix = prefix.to_string();
    let suffix = suffix.to_string();

    Ok(ParsedImportMetaGlobPattern {
        segments,
        star_index,
        prefix,
        suffix,
    })
}

fn unsupported_import_meta_glob(
    reason: &'static str,
    match_count: Option<usize>,
    cap: Option<usize>,
    affected_package_scope: Option<String>,
) -> ImportMetaGlobExpansion {
    let mut evidence = Map::new();
    evidence.insert("reason".to_string(), json!(reason));
    evidence.insert("resolverStage".to_string(), json!("import-meta-glob"));
    evidence.insert("outputLevel".to_string(), json!("unsupported"));
    evidence.insert("unsupportedFamily".to_string(), json!("dynamic-modules"));
    evidence.insert("hint".to_string(), json!("dynamic-module-surface"));
    evidence.insert("scanPolicy".to_string(), json!("scanned-source-files"));
    if let Some(match_count) = match_count {
        evidence.insert("matchCount".to_string(), json!(match_count));
    }
    if let Some(cap) = cap {
        evidence.insert("cap".to_string(), json!(cap));
    }
    if let Some(scope) = affected_package_scope {
        evidence.insert("affectedPackageScope".to_string(), json!(scope));
    }
    ImportMetaGlobExpansion::Unsupported { evidence }
}

fn handle_generated_virtual_record(
    response: &mut SourceUseAssemblyResponse,
    root: &str,
    record: SourceUseAssemblyRecord,
) {
    let kind = record.kind.clone().unwrap_or_else(|| "import".to_string());
    let record_id = record.record_id.clone();
    increment_branch(&mut response.branch_counts, "generatedVirtual");
    response.handled_record_ids.push(record_id);
    if is_namespace_reexport_use(&kind) {
        increment_branch(&mut response.branch_counts, "skippedNamespaceAlias");
        return;
    }

    let Some(surface) = record.generated_virtual_surface.clone() else {
        skip(
            response,
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

fn skip(response: &mut SourceUseAssemblyResponse, record_id: String, reason: &'static str) {
    response
        .skipped_records
        .push(SkippedSourceUseRecord { record_id, reason });
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

fn is_import_meta_glob_source_suffix(suffix: &str) -> bool {
    let lower = suffix.to_ascii_lowercase();
    if lower.ends_with(".d.ts") || lower.ends_with(".d.mts") || lower.ends_with(".d.cts") {
        return false;
    }
    [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"]
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

fn root_relative(root: &str, path: &str) -> String {
    let normalized = normalize_path_text(path);
    let trimmed_root = root.trim_end_matches('/');
    if let Some(rest) = normalized.strip_prefix(&format!("{trimmed_root}/")) {
        return rest.to_string();
    }
    if normalized == trimmed_root {
        return ".".to_string();
    }
    normalized
}

const RESOLVE_FILE_EXTS: &[&str] = &[
    "", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".mts", ".cts", ".d.ts", ".d.mts", ".d.cts",
];

const RESOLVE_INDEX_EXTS: &[&str] = &[
    "/index.ts",
    "/index.tsx",
    "/index.js",
    "/index.jsx",
    "/index.mjs",
    "/index.cjs",
    "/index.mts",
    "/index.cts",
    "/index.d.ts",
    "/index.d.mts",
    "/index.d.cts",
];

#[derive(Clone, Debug)]
struct ReExportTarget {
    target_file: String,
    source_spec: Option<String>,
}

#[derive(Debug)]
struct ResolvedNamespaceReExport {
    target_file: String,
    chain: Vec<NamespaceReExportChainEntry>,
}

#[derive(Debug)]
struct NamespaceReExportResolver {
    namespace: BTreeMap<(String, String), ReExportTarget>,
    named: BTreeMap<(String, String), ReExportTarget>,
}

impl NamespaceReExportResolver {
    fn new(
        namespace_re_exports: Vec<SourceUseAssemblyReExport>,
        named_re_exports: Vec<SourceUseAssemblyReExport>,
    ) -> Self {
        Self {
            namespace: re_export_map(namespace_re_exports),
            named: re_export_map(named_re_exports),
        }
    }

    fn resolve(
        &self,
        root: &str,
        barrel_file: &str,
        exported_name: &str,
    ) -> Option<ResolvedNamespaceReExport> {
        let mut seen = BTreeSet::new();
        self.resolve_inner(root, barrel_file, exported_name, &mut seen)
    }

    fn resolve_inner(
        &self,
        root: &str,
        barrel_file: &str,
        exported_name: &str,
        seen: &mut BTreeSet<(String, String)>,
    ) -> Option<ResolvedNamespaceReExport> {
        let normalized_barrel = normalize_path_text(barrel_file);
        let exported = exported_name.to_string();
        if !seen.insert((normalized_barrel.clone(), exported.clone())) {
            return None;
        }

        if let Some(direct) = self
            .namespace
            .get(&(normalized_barrel.clone(), exported.clone()))
        {
            return Some(ResolvedNamespaceReExport {
                target_file: direct.target_file.clone(),
                chain: vec![NamespaceReExportChainEntry {
                    kind: "namespace-reexport",
                    file: root_relative(root, &normalized_barrel),
                    exported_name: exported,
                    target_file: root_relative(root, &direct.target_file),
                    source: direct.source_spec.clone(),
                }],
            });
        }

        let named = self
            .named
            .get(&(normalized_barrel.clone(), exported.clone()))?;
        let nested = self.resolve_inner(root, &named.target_file, exported_name, seen)?;
        let mut chain = vec![NamespaceReExportChainEntry {
            kind: "named-reexport",
            file: root_relative(root, &normalized_barrel),
            exported_name: exported,
            target_file: root_relative(root, &named.target_file),
            source: named.source_spec.clone(),
        }];
        chain.extend(nested.chain);
        Some(ResolvedNamespaceReExport {
            target_file: nested.target_file,
            chain,
        })
    }
}

fn re_export_map(
    re_exports: Vec<SourceUseAssemblyReExport>,
) -> BTreeMap<(String, String), ReExportTarget> {
    let mut out = BTreeMap::new();
    for re_export in re_exports {
        if re_export.exported_name.is_empty() {
            continue;
        }
        out.insert(
            (
                normalize_path_text(&re_export.barrel_file),
                re_export.exported_name,
            ),
            ReExportTarget {
                target_file: normalize_path_text(&re_export.target_file),
                source_spec: re_export.source_spec,
            },
        );
    }
    out
}

#[derive(Debug)]
struct RelativeSourceResolver {
    source_files: BTreeMap<String, String>,
    listed_source_files: Vec<String>,
}

impl RelativeSourceResolver {
    fn new(root: &str, source_files: Vec<String>) -> Self {
        let mut out = BTreeMap::new();
        let mut listed = BTreeSet::new();
        let root = normalize_path_text(root);
        for source_file in source_files {
            let normalized = normalize_path_text(&source_file);
            let resolved = if is_absolute_path_text(&normalized) {
                normalized.clone()
            } else {
                normalize_path_text(&format!("{}/{}", root.trim_end_matches('/'), normalized))
            };
            out.entry(normalized).or_insert(resolved.clone());
            out.entry(resolved.clone()).or_insert(resolved.clone());
            out.entry(root_relative(&root, &resolved))
                .or_insert(resolved.clone());
            listed.insert(resolved);
        }
        Self {
            source_files: out,
            listed_source_files: listed.into_iter().collect(),
        }
    }

    fn resolve(&self, from_file: &str, spec: &str) -> Option<String> {
        if !is_relative_spec(spec) {
            return None;
        }
        let base = join_relative_spec(dirname_text(from_file), spec);
        for ext in RESOLVE_FILE_EXTS {
            if let Some(resolved) = self.source_file(&format!("{base}{ext}")) {
                return Some(resolved);
            }
        }
        for ext in RESOLVE_INDEX_EXTS {
            if let Some(resolved) = self.source_file(&format!("{base}{ext}")) {
                return Some(resolved);
            }
        }
        if js_output_extension(spec) {
            for alt in js_output_source_extensions(spec) {
                if let Some(swapped) = replace_js_output_extension(spec, alt) {
                    let candidate = join_relative_spec(dirname_text(from_file), &swapped);
                    if let Some(resolved) = self.source_file(&candidate) {
                        return Some(resolved);
                    }
                }
            }
        }
        if js_output_extension(spec) {
            if let Some(stripped) = strip_js_output_extension(&base) {
                for ext in RESOLVE_INDEX_EXTS {
                    if let Some(resolved) = self.source_file(&format!("{stripped}{ext}")) {
                        return Some(resolved);
                    }
                }
            }
        }
        None
    }

    fn source_file(&self, candidate: &str) -> Option<String> {
        self.source_files
            .get(&normalize_path_text(candidate))
            .cloned()
    }

    fn source_files(&self) -> impl Iterator<Item = &str> {
        self.listed_source_files.iter().map(String::as_str)
    }
}

fn dirname_text(path: &str) -> &str {
    let normalized = path.rfind(['/', '\\']);
    normalized.map_or("", |index| &path[..index])
}

fn join_relative_spec(base: &str, spec: &str) -> String {
    let joined = if base.is_empty() {
        spec.to_string()
    } else {
        format!("{base}/{spec}")
    };
    normalize_path_text(&joined)
}

fn normalize_path_text(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let (prefix, rest) = split_path_prefix(&normalized);
    let absolute = rest.starts_with('/');
    let mut parts = Vec::new();
    for part in rest.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            if let Some(last) = parts.last() {
                if last != &".." {
                    parts.pop();
                    continue;
                }
            }
            if !absolute {
                parts.push(part);
            }
            continue;
        }
        parts.push(part);
    }

    let body = parts.join("/");
    match (prefix.is_empty(), absolute, body.is_empty()) {
        (false, _, false) => format!("{prefix}/{body}"),
        (false, _, true) => prefix.to_string(),
        (true, true, false) => format!("/{body}"),
        (true, true, true) => "/".to_string(),
        (true, false, false) => body,
        (true, false, true) => ".".to_string(),
    }
}

fn split_path_prefix(path: &str) -> (&str, &str) {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        let prefix = &path[..2];
        let rest = path.get(2..).unwrap_or_default();
        return (prefix, rest);
    }
    ("", path)
}

fn is_absolute_path_text(path: &str) -> bool {
    let (prefix, rest) = split_path_prefix(path);
    !prefix.is_empty() || rest.starts_with('/')
}

fn basename_text(path: &str) -> Option<String> {
    normalize_path_text(path)
        .rsplit('/')
        .next()
        .map(ToString::to_string)
        .filter(|value| !value.is_empty())
}

fn is_inside_or_same(parent: &str, child: &str) -> bool {
    let parent = normalize_path_text(parent);
    let child = normalize_path_text(child);
    let parent = parent.trim_end_matches('/');
    child == parent || child.starts_with(&format!("{parent}/"))
}

fn relative_scope(root: &str, path: &str) -> String {
    let root = normalize_path_text(root);
    let path = normalize_path_text(path);
    let root = root.trim_end_matches('/');
    if path == root {
        String::new()
    } else {
        path.strip_prefix(&format!("{root}/"))
            .map(ToString::to_string)
            .unwrap_or(path)
    }
}

fn js_output_extension(spec: &str) -> bool {
    [".mjs", ".cjs", ".js", ".jsx"]
        .iter()
        .any(|ext| spec.ends_with(ext))
}

fn js_output_source_extensions(spec: &str) -> &'static [&'static str] {
    if spec.ends_with(".jsx") {
        &[".tsx", ".ts"]
    } else {
        &[".ts", ".tsx", ".mts", ".cts"]
    }
}

fn replace_js_output_extension(spec: &str, alt: &str) -> Option<String> {
    for ext in [".mjs", ".cjs", ".js", ".jsx"] {
        if let Some(replaced) = replace_suffix(spec, ext, alt) {
            return Some(replaced);
        }
    }
    None
}

fn replace_suffix(value: &str, suffix: &str, replacement: &str) -> Option<String> {
    value
        .strip_suffix(suffix)
        .map(|prefix| format!("{prefix}{replacement}"))
}

fn strip_js_output_extension(spec: &str) -> Option<&str> {
    for ext in [".mjs", ".cjs", ".js", ".jsx"] {
        if let Some(prefix) = spec.strip_suffix(ext) {
            return Some(prefix);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn must_request(value: serde_json::Value) -> SourceUseAssemblyRequest {
        match serde_json::from_value(value) {
            Ok(request) => request,
            Err(error) => panic!("test request must deserialize: {error}"),
        }
    }

    fn request(records: serde_json::Value) -> SourceUseAssemblyRequest {
        must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "records": records
        }))
    }

    fn response(request: SourceUseAssemblyRequest) -> SourceUseAssemblyResponse {
        match build_source_use_assembly_response(request) {
            Ok(response) => response,
            Err(error) => panic!("test response must build: {error}"),
        }
    }

    #[test]
    fn assembles_direct_and_namespace_relative_uses() {
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "./b",
                "name": "thing",
                "kind": "import",
                "typeOnly": false,
                "line": 3,
                "resolverStage": "relative"
            },
            {
                "recordId": "src/c.ts#0",
                "consumerFile": "C:/repo/src/c.ts",
                "resolvedFile": "C:/repo/src/d.ts",
                "fromSpec": "./d",
                "kind": "namespace",
                "resolverStage": "relative"
            }
        ])));

        assert_eq!(response.summary.handled_count, 2);
        assert_eq!(response.counters.total_uses, 2);
        assert_eq!(response.counters.resolved_internal_uses, 2);
        assert_eq!(response.branch_counts["resolvedInternal"], 2);
        assert_eq!(response.branch_counts["directConsumer"], 1);
        assert_eq!(response.branch_counts["broadNamespace"], 1);
        assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
        assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
        assert_eq!(response.resolved_internal_edges[0].kind, "import-named");
        assert_eq!(response.direct_consumers[0].symbol, "thing");
        assert_eq!(response.namespace_users[0].def_file, "src/d.ts");
        assert_eq!(response.resolved_record_targets.len(), 2);
        assert_eq!(response.resolved_record_targets[0].record_id, "src/a.ts#0");
        assert_eq!(
            response.resolved_record_targets[0].resolved_file,
            "C:/repo/src/b.ts"
        );
    }

    #[test]
    fn assembles_pre_resolved_root_relative_record_paths() {
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "src/a.ts",
                "resolvedFile": "src/b.ts",
                "fromSpec": "./b",
                "name": "thing",
                "kind": "import",
                "resolverStage": "resolved-internal"
            }
        ])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
        assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
        assert_eq!(
            response.resolved_record_targets[0].resolved_file,
            "src/b.ts"
        );
        assert_eq!(response.direct_consumers[0].consumer_file, "src/a.ts");
    }

    #[test]
    fn assembles_path_table_compacted_record_paths() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "pathTable": ["src/a.ts", "src/b.ts"],
            "records": [{
                "recordId": "r0",
                "consumerFileId": 0,
                "resolvedFileId": 1,
                "fromSpec": "./b",
                "name": "thing",
                "kind": "import",
                "resolverStage": "resolved-internal"
            }]
        })));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
        assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
        assert_eq!(response.direct_consumers[0].consumer_file, "src/a.ts");
        assert_eq!(
            response.resolved_record_targets[0].resolved_file,
            "src/b.ts"
        );
    }

    #[test]
    fn assembles_enum_table_compacted_record_fields() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "pathTable": ["src/a.ts", "src/b.ts"],
            "kindTable": ["import"],
            "resolverStageTable": ["resolved-internal"],
            "consumerSourceTable": ["mdx-import"],
            "specifierTable": ["./b"],
            "records": [{
                "recordId": "r0",
                "consumerFileId": 0,
                "resolvedFileId": 1,
                "fromSpecId": 0,
                "name": "thing",
                "kindId": 0,
                "resolverStageId": 0,
                "consumerSourceId": 0
            }]
        })));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.counters.mdx_consumer_uses, 1);
        assert_eq!(response.resolved_internal_edges[0].kind, "import-named");
        assert_eq!(response.resolved_internal_edges[0].from, "src/a.ts");
        assert_eq!(
            response.resolved_internal_edges[0].source.as_deref(),
            Some("./b")
        );
        assert_eq!(response.direct_consumers[0].symbol, "thing");
    }

    #[test]
    fn exposes_relative_resolution_targets_for_js_embedding() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": ["C:/repo/src/consumer.ts", "C:/repo/src/dep.ts"],
            "records": [{
                "recordId": "src/consumer.ts#0",
                "consumerFile": "C:/repo/src/consumer.ts",
                "fromSpec": "./dep",
                "name": "value",
                "kind": "import",
                "resolverStage": "relative"
            }]
        })));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.counters.rust_resolved_relative_uses, 1);
        assert_eq!(response.resolved_record_targets.len(), 1);
        assert_eq!(
            response.resolved_record_targets[0].record_id,
            "src/consumer.ts#0"
        );
        assert_eq!(
            response.resolved_record_targets[0].resolved_file,
            "C:/repo/src/dep.ts"
        );
    }

    #[test]
    fn counts_out_of_band_consumer_sources_in_rust_projection() {
        let response = response(request(json!([
            {
                "recordId": "mdx:0:docs/page.mdx:./widget",
                "consumerFile": "C:/repo/docs/page.mdx",
                "resolvedFile": "C:/repo/src/widget.ts",
                "fromSpec": "./widget",
                "name": "Widget",
                "kind": "import",
                "resolverStage": "relative",
                "consumerSource": "mdx-import"
            },
            {
                "recordId": "sfc:0:components/App.vue:./panel",
                "consumerFile": "C:/repo/components/App.vue",
                "resolvedFile": "C:/repo/components/panel.ts",
                "fromSpec": "./panel",
                "name": "Panel",
                "kind": "import",
                "resolverStage": "relative",
                "consumerSource": "sfc-script-import"
            },
            {
                "recordId": "sfc-script-src:0:components/App.vue:./setup",
                "consumerFile": "C:/repo/components/App.vue",
                "resolvedFile": "C:/repo/components/setup.ts",
                "fromSpec": "./setup",
                "kind": "sfc-script-src",
                "resolverStage": "relative",
                "consumerSource": "sfc-script-src"
            }
        ])));

        assert_eq!(response.summary.handled_count, 3);
        assert_eq!(response.counters.mdx_consumer_uses, 1);
        assert_eq!(response.counters.sfc_script_consumer_uses, 1);
        assert_eq!(response.counters.sfc_script_src_reachability_uses, 1);
        assert_eq!(response.branch_counts["sfcScriptSrcReachability"], 1);
    }

    #[test]
    fn handles_namespace_reexport_miss_and_skips_non_relative_records() {
        let response = response(request(json!([
            {
                "recordId": "a",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "./b",
                "name": "api",
                "kind": "imported-namespace-member",
                "resolverStage": "relative"
            },
            {
                "recordId": "b",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "./b",
                "kind": "import",
                "resolverStage": "alias"
            }
        ])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.summary.skipped_count, 1);
        assert_eq!(response.branch_counts["namespaceReExport"], 1);
        assert_eq!(response.branch_counts["namespaceReExportMiss"], 1);
        assert_eq!(
            response.skipped_records[0].reason,
            "non-relative-resolver-stage"
        );
    }

    #[test]
    fn assembles_pre_resolved_non_relative_internal_uses() {
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/b.ts",
                "fromSpec": "@/b",
                "name": "thing",
                "kind": "import",
                "typeOnly": false,
                "resolverStage": "resolved-internal"
            },
            {
                "recordId": "src/c.ts#0",
                "consumerFile": "C:/repo/src/c.ts",
                "resolvedFile": "C:/repo/src/d.ts",
                "fromSpec": "@pkg/d",
                "kind": "namespace",
                "resolverStage": "resolved-internal"
            }
        ])));

        assert_eq!(response.summary.handled_count, 2);
        assert_eq!(response.counters.total_uses, 2);
        assert_eq!(response.counters.resolved_internal_uses, 2);
        assert_eq!(response.counters.rust_resolved_relative_uses, 0);
        assert_eq!(
            response.resolved_internal_edges[0].source.as_deref(),
            Some("@/b")
        );
        assert_eq!(response.resolved_internal_edges[0].to, "src/b.ts");
        assert_eq!(response.direct_consumers[0].symbol, "thing");
        assert_eq!(response.namespace_users[0].def_file, "src/d.ts");
    }

    #[test]
    fn assembles_external_dependency_consumers() {
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "C:/repo/src/a.ts",
                "fromSpec": "react/jsx-runtime",
                "kind": "import",
                "typeOnly": false,
                "typeOnlyPresent": true,
                "resolverStage": "external",
                "consumerSource": "source-import"
            },
            {
                "recordId": "src/b.ts#0",
                "consumerFile": "C:/repo/src/b.ts",
                "fromSpec": "@scope/pkg/subpath",
                "kind": "import",
                "typeOnly": true,
                "typeOnlyPresent": true,
                "resolverStage": "external",
                "consumerSource": "mdx-import"
            },
            {
                "recordId": "src/c.ts#0",
                "consumerFile": "C:/repo/src/c.ts",
                "fromSpec": "#internal",
                "kind": "import",
                "resolverStage": "external",
                "consumerSource": "source-import"
            },
            {
                "recordId": "src/d.ts#0",
                "consumerFile": "C:/repo/src/d.ts",
                "fromSpec": "pkg",
                "name": "api",
                "kind": "imported-namespace-escape",
                "resolverStage": "external",
                "consumerSource": "source-import"
            }
        ])));

        assert_eq!(response.summary.handled_count, 4);
        assert_eq!(response.counters.external_uses, 3);
        assert_eq!(response.counters.unresolved_uses, 3);
        assert_eq!(response.branch_counts["external"], 4);
        assert_eq!(response.branch_counts["skippedNamespaceAlias"], 1);
        assert_eq!(response.dependency_import_consumers.len(), 2);
        assert_eq!(response.dependency_import_consumers[0].dep_root, "react");
        assert_eq!(
            response.dependency_import_consumers[0].from_spec,
            "react/jsx-runtime"
        );
        assert_eq!(
            response.dependency_import_consumers[0].type_only,
            Some(false)
        );
        assert_eq!(
            response.dependency_import_consumers[1].dep_root,
            "@scope/pkg"
        );
        assert_eq!(response.dependency_import_consumers[1].source, "mdx-import");
        assert_eq!(
            response.dependency_import_consumers[1].type_only,
            Some(true)
        );
    }

    #[test]
    fn import_meta_glob_expansions_are_broad_namespace_consumers() {
        let response = response(request(json!([
            {
                "recordId": "src/routes.ts#0:glob:src/pages/home.ts",
                "consumerFile": "C:/repo/src/routes.ts",
                "resolvedFile": "C:/repo/src/pages/home.ts",
                "fromSpec": "./pages/*.ts",
                "kind": "dynamic-import-meta-glob",
                "resolverStage": "resolved-internal"
            }
        ])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.counters.total_uses, 1);
        assert_eq!(response.counters.resolved_internal_uses, 1);
        assert_eq!(response.resolved_internal_edges[0].from, "src/routes.ts");
        assert_eq!(response.resolved_internal_edges[0].to, "src/pages/home.ts");
        assert_eq!(
            response.resolved_internal_edges[0].kind,
            "dynamic-import-meta-glob"
        );
        assert_eq!(response.namespace_users[0].def_file, "src/pages/home.ts");
        assert_eq!(response.namespace_users[0].consumer_file, "src/routes.ts");
    }

    #[test]
    fn expands_import_meta_glob_records_from_scanned_source_files() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "importMetaGlobCap": 64,
            "sourceFiles": [
                "C:/repo/src/pages/about.ts",
                "C:/repo/src/pages/home.ts",
                "C:/repo/src/pages/home.test.ts",
                "C:/repo/src/pages/nested/ignored.ts",
                "C:/repo/src/pages/ignored.css"
            ],
            "records": [{
                "recordId": "src/routes.ts#0",
                "consumerFile": "C:/repo/src/routes.ts",
                "fromSpec": "./pages/*.ts",
                "kind": "import-meta-glob",
                "resolverStage": "relative",
                "line": 7
            }]
        })));

        assert_eq!(response.summary.handled_count, 1);
        assert!(response
            .handled_record_ids
            .contains(&"src/routes.ts#0".to_string()));
        assert_eq!(response.counters.total_uses, 3);
        assert_eq!(response.counters.resolved_internal_uses, 3);
        assert_eq!(response.branch_counts["importMetaGlobResolved"], 1);
        assert_eq!(
            response
                .resolved_internal_edges
                .iter()
                .map(|edge| edge.to.as_str())
                .collect::<Vec<_>>(),
            vec![
                "src/pages/about.ts",
                "src/pages/home.test.ts",
                "src/pages/home.ts"
            ]
        );
        assert!(response
            .namespace_users
            .iter()
            .any(|user| user.def_file == "src/pages/home.ts"
                && user.consumer_file == "src/routes.ts"));
    }

    #[test]
    fn import_meta_glob_namespace_users_are_deduplicated_with_other_broad_uses() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/pages/home.ts"
            ],
            "records": [
                {
                    "recordId": "src/routes.ts#0",
                    "consumerFile": "C:/repo/src/routes.ts",
                    "resolvedFile": "C:/repo/src/pages/home.ts",
                    "fromSpec": "./pages/home",
                    "name": "*",
                    "kind": "namespace",
                    "resolverStage": "relative"
                },
                {
                    "recordId": "src/routes.ts#1",
                    "consumerFile": "C:/repo/src/routes.ts",
                    "fromSpec": "./pages/*.ts",
                    "kind": "import-meta-glob",
                    "resolverStage": "import-meta-glob"
                }
            ]
        })));

        assert_eq!(response.summary.handled_count, 2);
        assert_eq!(response.resolved_internal_edges.len(), 2);
        let matching_namespace_users = response
            .namespace_users
            .iter()
            .filter(|user| {
                user.def_file == "src/pages/home.ts" && user.consumer_file == "src/routes.ts"
            })
            .count();
        assert_eq!(matching_namespace_users, 1);
    }

    #[test]
    fn import_meta_glob_unsupported_records_remain_unresolved_without_prefix_summary() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": ["C:/repo/src/app.ts"],
            "records": [{
                "recordId": "src/app.ts#0",
                "consumerFile": "C:/repo/src/app.ts",
                "fromSpec": "./*.md",
                "kind": "import-meta-glob",
                "resolverStage": "relative"
            }]
        })));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.counters.unresolved_uses, 1);
        assert_eq!(response.counters.unresolved_internal_uses, 1);
        assert_eq!(response.branch_counts["importMetaGlobUnsupported"], 1);
        assert!(response.unresolved_internal_by_prefix.is_empty());
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["reason"],
            "import-meta-glob-target-extension-unsupported"
        );
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["scanPolicy"],
            "scanned-source-files"
        );
    }

    #[test]
    fn non_relative_import_meta_glob_reports_unsupported_evidence() {
        let response = response(must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": ["C:/repo/src/app.ts"],
            "records": [{
                "recordId": "src/app.ts#0",
                "consumerFile": "C:/repo/src/app.ts",
                "fromSpec": "@/pages/*.ts",
                "kind": "import-meta-glob",
                "resolverStage": "import-meta-glob"
            }]
        })));

        assert_eq!(response.summary.handled_count, 1);
        assert!(response.skipped_records.is_empty());
        assert_eq!(response.counters.unresolved_uses, 1);
        assert_eq!(response.counters.unresolved_internal_uses, 1);
        assert_eq!(response.branch_counts["importMetaGlobUnsupported"], 1);
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["reason"],
            "import-meta-glob-nonrelative-unsupported"
        );
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["resolverStage"],
            "import-meta-glob"
        );
    }

    #[test]
    fn assembles_generated_virtual_consumers_and_unresolved_exports() {
        let surface = json!({
            "id": "generated-virtual:prisma-enums:@pkg/db:enums",
            "source": "generated-virtual",
            "mode": "virtual",
            "virtual": true,
            "exports": [{
                "name": "Role",
                "kind": "prisma-enum",
                "spaces": ["value", "type"]
            }]
        });
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "C:/repo/src/a.ts",
                "fromSpec": "@pkg/db/enums",
                "name": "Role",
                "kind": "import",
                "typeOnly": false,
                "typeOnlyPresent": true,
                "resolverStage": "generated-virtual",
                "generatedVirtualSurface": surface
            },
            {
                "recordId": "src/a.ts#1",
                "consumerFile": "C:/repo/src/a.ts",
                "fromSpec": "@pkg/db/enums",
                "name": "Missing",
                "kind": "import",
                "typeOnly": false,
                "typeOnlyPresent": true,
                "resolverStage": "generated-virtual",
                "generatedVirtualSurface": surface,
                "unresolvedEvidence": {
                    "reason": "workspace-generated-virtual-export-missing"
                }
            }
        ])));

        assert_eq!(response.summary.handled_count, 2);
        assert_eq!(response.counters.total_uses, 1);
        assert_eq!(response.counters.resolved_internal_uses, 1);
        assert_eq!(response.counters.resolved_generated_virtual_uses, 1);
        assert_eq!(response.counters.unresolved_uses, 1);
        assert_eq!(response.counters.unresolved_internal_uses, 1);
        assert_eq!(response.generated_virtual_surfaces.len(), 1);
        assert_eq!(
            response.generated_virtual_import_consumers[0]["surfaceId"],
            "generated-virtual:prisma-enums:@pkg/db:enums"
        );
        assert_eq!(
            response.generated_virtual_import_consumers[0]["name"],
            "Role"
        );
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["reason"],
            "workspace-generated-virtual-export-missing"
        );
    }

    #[test]
    fn assembles_unresolved_internal_specifier_records() {
        let response = response(request(json!([
            {
                "recordId": "src/a.ts#0",
                "consumerFile": "C:/repo/src/a.ts",
                "fromSpec": "@/missing",
                "kind": "import",
                "typeOnly": false,
                "typeOnlyPresent": true,
                "resolverStage": "unresolved-internal",
                "unresolvedEvidence": {
                    "reason": "tsconfig-path-target-missing",
                    "resolverStage": "tsconfig-paths",
                    "matchedPattern": "@/*",
                    "targetCandidates": ["src/missing.ts"],
                    "hint": "check-tsconfig-paths"
                }
            },
            {
                "recordId": "src/b.ts#0",
                "consumerFile": "C:/repo/src/b.ts",
                "fromSpec": "./missing",
                "kind": "import-side-effect",
                "resolverStage": "unresolved-relative",
                "unresolvedEvidence": {
                    "reason": "relative-target-missing",
                    "resolverStage": "relative"
                }
            },
            {
                "recordId": "src/c.ts#0",
                "consumerFile": "C:/repo/src/c.ts",
                "fromSpec": "@/api",
                "kind": "imported-namespace-escape",
                "resolverStage": "unresolved-internal"
            }
        ])));

        assert_eq!(response.summary.handled_count, 3);
        assert_eq!(response.counters.unresolved_uses, 2);
        assert_eq!(response.counters.unresolved_internal_uses, 2);
        assert_eq!(response.branch_counts["unresolved"], 3);
        assert_eq!(response.branch_counts["skippedNamespaceAlias"], 1);
        assert_eq!(response.unresolved_internal_by_prefix["@/"], 1);
        assert_eq!(response.prefix_examples["@/"], "@/missing");
        assert!(response
            .unresolved_internal_specifiers
            .contains("@/missing"));
        assert!(response
            .unresolved_internal_specifiers
            .contains("./missing"));
        assert_eq!(response.unresolved_internal_specifier_records.len(), 2);
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["reason"],
            "tsconfig-path-target-missing"
        );
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["typeOnly"],
            false
        );
        assert_eq!(
            response.unresolved_internal_specifier_records[1]["reason"],
            "relative-target-missing"
        );
    }

    #[test]
    fn preserves_import_meta_glob_unsupported_evidence_without_prefix_summary() {
        let response = response(request(json!([{
            "recordId": "src/app.ts#0",
            "consumerFile": "C:/repo/src/app.ts",
            "fromSpec": "./*.md",
            "kind": "import-meta-glob",
            "resolverStage": "unresolved-relative",
            "unresolvedEvidence": {
                "reason": "import-meta-glob-target-extension-unsupported",
                "resolverStage": "import-meta-glob",
                "outputLevel": "unsupported",
                "unsupportedFamily": "dynamic-modules",
                "hint": "dynamic-module-surface",
                "scanPolicy": "scanned-source-files"
            }
        }])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.counters.unresolved_uses, 1);
        assert_eq!(response.counters.unresolved_internal_uses, 1);
        assert!(response.unresolved_internal_by_prefix.is_empty());
        assert!(response.unresolved_internal_specifiers.contains("./*.md"));
        assert_eq!(response.unresolved_internal_specifier_records.len(), 1);
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["reason"],
            "import-meta-glob-target-extension-unsupported"
        );
        assert_eq!(
            response.unresolved_internal_specifier_records[0]["resolverStage"],
            "import-meta-glob"
        );
    }

    #[test]
    fn assembles_namespace_reexport_member_and_escape_uses() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/consumer.ts",
                "C:/repo/src/barrel.ts",
                "C:/repo/src/nested.ts",
                "C:/repo/src/dep.ts"
            ],
            "namespaceReExports": [
                {
                    "barrelFile": "C:/repo/src/nested.ts",
                    "exportedName": "api",
                    "targetFile": "C:/repo/src/dep.ts",
                    "sourceSpec": "./dep"
                }
            ],
            "namedReExports": [
                {
                    "barrelFile": "C:/repo/src/barrel.ts",
                    "exportedName": "api",
                    "targetFile": "C:/repo/src/nested.ts",
                    "sourceSpec": "./nested"
                }
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./barrel",
                    "name": "api",
                    "memberName": "run",
                    "kind": "imported-namespace-member",
                    "line": 4
                },
                {
                    "recordId": "src/consumer.ts#1",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./barrel",
                    "name": "api",
                    "kind": "imported-namespace-escape",
                    "line": 5
                },
                {
                    "recordId": "src/consumer.ts#2",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./barrel",
                    "name": "missing",
                    "memberName": "nope",
                    "kind": "imported-namespace-member"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 3);
        assert_eq!(response.counters.total_uses, 2);
        assert_eq!(response.counters.resolved_internal_uses, 2);
        assert_eq!(response.branch_counts["namespaceReExport"], 3);
        assert_eq!(response.branch_counts["namespaceReExportMember"], 1);
        assert_eq!(response.branch_counts["namespaceReExportEscape"], 1);
        assert_eq!(response.branch_counts["namespaceReExportMiss"], 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
        assert_eq!(
            response.resolved_internal_edges[0].kind,
            "reexport-namespace-member"
        );
        assert_eq!(response.direct_consumers[0].def_file, "src/dep.ts");
        assert_eq!(response.direct_consumers[0].symbol, "run");
        assert_eq!(response.namespace_users[0].def_file, "src/dep.ts");
        assert_eq!(
            response.namespace_re_export_diagnostics[0].reason,
            "namespace-object-escaped"
        );
        assert_eq!(
            response.namespace_re_export_diagnostics[0].chain[0].kind,
            "named-reexport"
        );
        assert_eq!(
            response.namespace_re_export_diagnostics[0].chain[1].kind,
            "namespace-reexport"
        );
    }

    #[test]
    fn side_effect_uses_keep_edges_without_consumers() {
        let response = response(request(json!([
            {
                "recordId": "a",
                "consumerFile": "C:/repo/src/a.ts",
                "resolvedFile": "C:/repo/src/setup.ts",
                "fromSpec": "./setup",
                "kind": "import-side-effect",
                "resolverStage": "relative"
            }
        ])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.branch_counts["sideEffectOnly"], 1);
        assert_eq!(
            response.resolved_internal_edges[0].kind,
            "import-side-effect"
        );
        assert!(response.direct_consumers.is_empty());
        assert!(response.namespace_users.is_empty());
    }

    #[test]
    fn sfc_script_src_uses_keep_reachability_edges_without_consumers() {
        let response = response(request(json!([
            {
                "recordId": "sfc-script-src:0:components/App.vue:../src/setup.ts",
                "consumerFile": "C:/repo/components/App.vue",
                "resolvedFile": "C:/repo/src/setup.ts",
                "fromSpec": "../src/setup.ts",
                "name": "*",
                "kind": "sfc-script-src",
                "sfcLanguage": "vue",
                "line": 2
            }
        ])));

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.branch_counts["sfcScriptSrcReachability"], 1);
        assert_eq!(
            response.resolved_internal_edges[0].from,
            "components/App.vue"
        );
        assert_eq!(response.resolved_internal_edges[0].to, "src/setup.ts");
        assert_eq!(response.resolved_internal_edges[0].kind, "sfc-script-src");
        assert_eq!(
            response.resolved_internal_edges[0].source.as_deref(),
            Some("../src/setup.ts")
        );
        assert!(response.direct_consumers.is_empty());
        assert!(response.namespace_users.is_empty());
    }

    #[test]
    fn records_non_source_asset_uses_without_reachability_edges() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "records": [
                {
                    "recordId": "sfc-script-src:0:components/App.vue:./style.css",
                    "consumerFile": "C:/repo/components/App.vue",
                    "fromSpec": "./style.css",
                    "name": "*",
                    "kind": "sfc-script-src",
                    "resolverStage": "non-source-asset"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.summary.skipped_count, 0);
        assert_eq!(response.counters.non_source_asset_uses, 1);
        assert_eq!(response.branch_counts["asset"], 1);
        assert!(response.resolved_internal_edges.is_empty());
        assert!(response.direct_consumers.is_empty());
        assert!(response.namespace_users.is_empty());
    }

    #[test]
    fn resolves_relative_targets_from_source_files_when_resolved_file_is_absent() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/consumer.ts",
                "C:/repo/src/dep.ts"
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./dep",
                    "name": "value",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
        assert_eq!(response.direct_consumers[0].symbol, "value");
    }

    #[test]
    fn resolves_absolute_consumers_from_root_relative_source_files() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "src/consumer.ts",
                "src/dep.ts"
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./dep",
                    "name": "value",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/dep.ts");
        assert_eq!(
            response.resolved_record_targets[0].resolved_file,
            "C:/repo/src/dep.ts"
        );
    }

    #[test]
    fn jsx_output_import_preserves_jsx_to_tsx_swap_order() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/consumer.ts",
                "C:/repo/src/view.ts",
                "C:/repo/src/view.tsx"
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./view.jsx",
                    "name": "view",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/view.tsx");
    }

    #[test]
    fn jsx_output_import_falls_back_to_ts_when_tsx_source_is_absent() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": [
                "C:/repo/src/consumer.ts",
                "C:/repo/src/view.ts"
            ],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./view.jsx",
                    "name": "view",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 1);
        assert_eq!(response.resolved_internal_edges[0].to, "src/view.ts");
    }

    #[test]
    fn unresolved_relative_targets_are_left_for_js_fallback() {
        let request = must_request(json!({
            "schemaVersion": SOURCE_USE_ASSEMBLY_REQUEST_SCHEMA_VERSION,
            "root": "C:/repo",
            "sourceFiles": ["C:/repo/src/consumer.ts"],
            "records": [
                {
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./missing",
                    "name": "value",
                    "kind": "import"
                }
            ]
        }));
        let response = response(request);

        assert_eq!(response.summary.handled_count, 0);
        assert_eq!(
            response.skipped_records[0].reason,
            "relative-target-missing"
        );
    }
}
