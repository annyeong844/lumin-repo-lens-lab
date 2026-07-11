pub(crate) mod any_contamination;
mod protocol;
mod reachability;
mod sfc;

use crate::scan_scope::{scan_scope_status_for_path, ScanScopeOptions};
use crate::source_use_assembly::{
    build_embedded_source_use_assembly_response_with_path_table, SourceUseAssemblyRequest,
    SourceUseAssemblyResponse,
};
use anyhow::{bail, Result};
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use any_contamination::{build_any_contamination_facts, ComputedAnyContamination};
use protocol::{
    DeadCandidateInputs, DefinitionFileInput, FanInInputs, FileDataInput, SymbolGraphContext,
    SymbolGraphExtraction, SymbolGraphInputs, SymbolGraphSfcInputs,
};
pub use protocol::{SymbolGraphRequest, SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION};
use reachability::{
    build_dead_candidates, build_fan_in, merge_source_use_fan_in_inputs, top_symbol_fan_in,
    ComputedDeadCandidates, ComputedFanIn,
};
use sfc::{
    project_sfc_framework_convention_components, project_sfc_generated_component_manifests,
    project_sfc_global_component_registrations, project_sfc_style_asset_references,
    project_sfc_template_component_refs, source_use_external_record_set,
    source_use_generated_virtual_record_set, source_use_non_source_asset_record_set,
    source_use_non_source_asset_target_map, source_use_resolved_target_map,
};

const TOOL_NAME: &str = "build-symbol-graph.mjs";
const SYMBOL_META_SCHEMA_VERSION: i64 = 3;

#[derive(Debug)]
struct DefinitionFile {
    file_path: String,
    definitions: BTreeMap<String, Value>,
}

#[derive(Debug)]
struct FileDataRecord {
    file_path: String,
    py_dunder_all: Option<Vec<String>>,
    re_exports: Vec<Value>,
    class_methods: Vec<Value>,
    local_operations: Vec<Value>,
    type_escapes: Vec<Value>,
    dynamic_import_opacity: Vec<Value>,
    cjs_export_surface: Option<Value>,
    cjs_require_opacity: Vec<Value>,
}

#[derive(Debug)]
struct PreparedSymbolGraphRequest {
    generated: String,
    root: String,
    include_tests: bool,
    exclude: Vec<String>,
    generated_artifacts_mode: String,
    language_support: Value,
    warnings: Vec<Value>,
    incremental: Value,
    path_table: Vec<String>,
    files: Vec<String>,
    def_index: Vec<DefinitionFile>,
    file_data: Vec<FileDataRecord>,
    parse_error_files: Vec<String>,
    source_use_assembly: SourceUseAssemblyRequest,
    fan_in_inputs: FanInInputs,
    dead_candidate_inputs: DeadCandidateInputs,
    sfc: SymbolGraphSfcInputs,
}

fn symbol_path_from_table(path_table: &[String], id: usize, field: &str) -> Result<String> {
    path_table
        .get(id)
        .filter(|path| !path.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("symbol-graph-artifact: invalid {field} {id}"))
}

fn require_object_values(label: &str, values: &[Value]) -> Result<()> {
    if let Some((index, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_object())
    {
        bail!("symbol-graph-artifact: {label}[{index}] must be an object");
    }
    Ok(())
}

fn validate_extraction_facts(extraction: &SymbolGraphExtraction) -> Result<()> {
    for (file_index, file) in extraction.def_index.iter().enumerate() {
        for (name, definition) in &file.definitions {
            if !definition.is_object() {
                bail!(
                    "symbol-graph-artifact: extraction.defIndex[{file_index}].definitions.{name} must be an object"
                );
            }
        }
    }
    for (file_index, file) in extraction.file_data.iter().enumerate() {
        for (field, values) in [
            ("reExports", file.re_exports.as_slice()),
            ("classMethods", file.class_methods.as_slice()),
            ("localOperations", file.local_operations.as_slice()),
            ("typeEscapes", file.type_escapes.as_slice()),
            (
                "dynamicImportOpacity",
                file.dynamic_import_opacity.as_slice(),
            ),
            ("cjsRequireOpacity", file.cjs_require_opacity.as_slice()),
        ] {
            require_object_values(
                &format!("extraction.fileData[{file_index}].{field}"),
                values,
            )?;
        }
        if let Some(surface) = &file.cjs_export_surface {
            let object = surface.as_object().ok_or_else(|| {
                anyhow::anyhow!(
                    "symbol-graph-artifact: extraction.fileData[{file_index}].cjsExportSurface must be an object"
                )
            })?;
            for field in ["exact", "opaque"] {
                if let Some(values) = object.get(field) {
                    let values = values.as_array().ok_or_else(|| {
                        anyhow::anyhow!(
                            "symbol-graph-artifact: extraction.fileData[{file_index}].cjsExportSurface.{field} must be an array"
                        )
                    })?;
                    require_object_values(
                        &format!("extraction.fileData[{file_index}].cjsExportSurface.{field}"),
                        values,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn validate_source_use_facts(request: &SourceUseAssemblyRequest) -> Result<()> {
    for (record_index, record) in request.records.iter().enumerate() {
        if record
            .unresolved_evidence
            .as_ref()
            .is_some_and(|value| !value.is_object())
        {
            bail!(
                "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].unresolvedEvidence must be an object"
            );
        }
        let Some(surface) = record.generated_virtual_surface.as_ref() else {
            continue;
        };
        let object = surface.as_object().ok_or_else(|| {
            anyhow::anyhow!(
                "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].generatedVirtualSurface must be an object"
            )
        })?;
        let id = object.get("id").and_then(Value::as_str).unwrap_or_default();
        if id.is_empty() {
            bail!(
                "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].generatedVirtualSurface.id must be non-empty"
            );
        }
        if let Some(exports) = object.get("exports") {
            let exports = exports.as_array().ok_or_else(|| {
                anyhow::anyhow!(
                    "symbol-graph-artifact: sourceUseAssembly.records[{record_index}].generatedVirtualSurface.exports must be an array"
                )
            })?;
            require_object_values(
                &format!(
                    "sourceUseAssembly.records[{record_index}].generatedVirtualSurface.exports"
                ),
                exports,
            )?;
        }
    }
    Ok(())
}

fn validate_sfc_facts(graph: &SymbolGraphInputs) -> Result<()> {
    for (index, component) in graph.sfc.framework_convention_components.iter().enumerate() {
        if component
            .path_prefix
            .as_ref()
            .is_some_and(|value| !value.is_boolean() && !value.is_string())
        {
            bail!(
                "symbol-graph-artifact: graph.sfc.frameworkConventionComponents[{index}].pathPrefix must be a boolean or string"
            );
        }
    }
    Ok(())
}

fn prepare_symbol_graph_request(request: SymbolGraphRequest) -> Result<PreparedSymbolGraphRequest> {
    let SymbolGraphRequest {
        schema_version,
        context,
        extraction,
        source_use_assembly,
        graph,
    } = request;
    if schema_version != SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION {
        bail!("symbol-graph-artifact: unsupported schemaVersion '{schema_version}'");
    }

    let SymbolGraphContext {
        generated,
        root,
        include_tests,
        exclude,
        generated_artifacts_mode,
        language_support,
        warnings,
        incremental,
    } = context;
    if generated.is_empty() {
        bail!("symbol-graph-artifact: context.generated must be non-empty");
    }
    if root.is_empty() {
        bail!("symbol-graph-artifact: context.root must be non-empty");
    }
    if !matches!(
        generated_artifacts_mode.as_str(),
        "default" | "present" | "prepared"
    ) {
        bail!(
            "symbol-graph-artifact: unsupported generatedArtifactsMode '{generated_artifacts_mode}'"
        );
    }
    if !language_support.is_object() {
        bail!("symbol-graph-artifact: context.languageSupport must be an object");
    }
    if !incremental.is_null() && !incremental.is_object() {
        bail!("symbol-graph-artifact: context.incremental must be an object or null");
    }
    require_object_values("context.warnings", &warnings)?;
    validate_extraction_facts(&extraction)?;
    validate_source_use_facts(&source_use_assembly)?;
    validate_sfc_facts(&graph)?;
    if normalize_slashes(&source_use_assembly.root).trim_end_matches('/')
        != normalize_slashes(&root).trim_end_matches('/')
    {
        bail!("symbol-graph-artifact: sourceUseAssembly.root must match context.root");
    }

    let SymbolGraphExtraction {
        path_table,
        file_ids,
        def_index,
        file_data,
        parse_error_file_ids,
    } = extraction;
    let files = file_ids
        .into_iter()
        .map(|id| symbol_path_from_table(&path_table, id, "fileIds"))
        .collect::<Result<Vec<_>>>()?;
    let def_index = def_index
        .into_iter()
        .map(|input: DefinitionFileInput| {
            Ok(DefinitionFile {
                file_path: symbol_path_from_table(
                    &path_table,
                    input.file_path_id,
                    "defIndex.filePathId",
                )?,
                definitions: input.definitions,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let file_data = file_data
        .into_iter()
        .map(|input: FileDataInput| {
            Ok(FileDataRecord {
                file_path: symbol_path_from_table(
                    &path_table,
                    input.file_path_id,
                    "fileData.filePathId",
                )?,
                py_dunder_all: input.py_dunder_all,
                re_exports: input.re_exports,
                class_methods: input.class_methods,
                local_operations: input.local_operations,
                type_escapes: input.type_escapes,
                dynamic_import_opacity: input.dynamic_import_opacity,
                cjs_export_surface: input.cjs_export_surface,
                cjs_require_opacity: input.cjs_require_opacity,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let parse_error_files = parse_error_file_ids
        .into_iter()
        .map(|id| symbol_path_from_table(&path_table, id, "parseErrorFileIds"))
        .collect::<Result<Vec<_>>>()?;
    let SymbolGraphInputs {
        fan_in,
        dead_candidates,
        sfc,
    } = graph;

    Ok(PreparedSymbolGraphRequest {
        generated,
        root,
        include_tests,
        exclude,
        generated_artifacts_mode,
        language_support,
        warnings,
        incremental,
        path_table,
        files,
        def_index,
        file_data,
        parse_error_files,
        source_use_assembly,
        fan_in_inputs: fan_in,
        dead_candidate_inputs: dead_candidates,
        sfc,
    })
}

pub fn build_symbol_graph_artifact(request: SymbolGraphRequest) -> Result<Value> {
    let PreparedSymbolGraphRequest {
        generated,
        root,
        include_tests,
        exclude,
        generated_artifacts_mode,
        language_support,
        mut warnings,
        incremental,
        path_table,
        files,
        def_index,
        file_data,
        parse_error_files,
        source_use_assembly,
        fan_in_inputs,
        dead_candidate_inputs,
        sfc,
    } = prepare_symbol_graph_request(request)?;

    if !parse_error_files.is_empty() {
        let parse_error_count = parse_error_files.len();
        warnings.push(json!({
            "code": "parse-errors",
            "count": parse_error_count,
            "message": format!("{parse_error_count} file(s) failed to parse; their defs/uses are missing from the graph"),
        }));
    }
    let source_use_assembly = build_embedded_source_use_assembly_response_with_path_table(
        source_use_assembly,
        &path_table,
    )?;
    if source_use_assembly.summary.skipped_count != 0 {
        bail!(
            "symbol-graph-artifact: sourceUseAssembly skipped {} record(s)",
            source_use_assembly.summary.skipped_count
        );
    }

    let supports = json!({
        "anyContamination": true,
        "identityFanIn": true,
        "identityFanInSpace": true,
        "reExportRecords": "file-level",
        "mdxImportConsumers": true,
        "sfcScriptImportConsumers": true,
        "sfcScriptSrcReachability": true,
        "sfcStyleAssetReferences": true,
        "sfcTemplateComponentRefs": true,
        "sfcGlobalComponentRegistrations": true,
        "sfcGeneratedComponentManifests": true,
        "sfcFrameworkConventionComponents": true,
        "dependencyImportConsumers": true,
        "resolvedInternalEdges": true,
        "definitionIds": true,
        "unresolvedInternalSummaryByReason": true,
        "cjsExportSurface": true,
        "cjsRequireOpacity": true,
        "generatedConsumerBlindZones": true,
        "generatedVirtualSurfaces": true,
        "nonSourceAssetImports": true,
        "namespaceReExportDiagnostics": true,
        "classMethodIndex": true,
        "nestedLocalOperationIndex": true,
    });

    let mut meta = Map::new();
    meta.insert("tool".to_string(), json!(TOOL_NAME));
    meta.insert("generated".to_string(), json!(generated));
    meta.insert("root".to_string(), json!(root));
    meta.insert(
        "schemaVersion".to_string(),
        json!(SYMBOL_META_SCHEMA_VERSION),
    );
    meta.insert("supports".to_string(), supports);
    meta.insert("languageSupport".to_string(), language_support);
    meta.insert("warnings".to_string(), Value::Array(warnings));
    if !incremental.is_null() {
        meta.insert("incremental".to_string(), incremental);
    }

    let total_defs = def_index
        .iter()
        .map(|file| file.definitions.len())
        .sum::<usize>();
    let total_class_methods = file_data
        .iter()
        .map(|file| file.class_methods.len())
        .sum::<usize>();
    let total_local_operations = file_data
        .iter()
        .map(|file| file.local_operations.len())
        .sum::<usize>();
    let source_use_counters = &source_use_assembly.counters;
    let total_uses = source_use_counters.total_uses;
    let unresolved_uses = source_use_counters.unresolved_uses;
    let resolved_internal_uses = source_use_counters.resolved_internal_uses;
    let external_uses = source_use_counters.external_uses;
    let resolved_generated_virtual_uses = source_use_counters.resolved_generated_virtual_uses;
    let non_source_asset_uses = source_use_counters.non_source_asset_uses;
    let unresolved_internal_uses = source_use_counters.unresolved_internal_uses;
    let mdx_consumer_uses = source_use_counters.mdx_consumer_uses;
    let sfc_script_consumer_uses = source_use_counters.sfc_script_consumer_uses;
    let sfc_script_src_reachability_uses = source_use_counters.sfc_script_src_reachability_uses;
    let unresolved_ratio = if resolved_internal_uses + unresolved_internal_uses > 0 {
        round4(
            unresolved_internal_uses as f64
                / (resolved_internal_uses + unresolved_internal_uses) as f64,
        )
    } else {
        0.0
    };
    let source_use_resolved_targets = source_use_resolved_target_map(&source_use_assembly);
    let source_use_external_record_ids = source_use_external_record_set(&source_use_assembly);
    let source_use_non_source_asset_record_ids =
        source_use_non_source_asset_record_set(&source_use_assembly);
    let source_use_non_source_asset_targets =
        source_use_non_source_asset_target_map(&source_use_assembly);
    let source_use_generated_virtual_record_ids =
        source_use_generated_virtual_record_set(&source_use_assembly);
    let fan_in_inputs = merge_source_use_fan_in_inputs(&root, fan_in_inputs, &source_use_assembly);

    let SymbolGraphSfcInputs {
        style_asset_references,
        template_component_refs,
        global_component_registrations,
        generated_component_manifests,
        generated_manifest_external_uses,
        framework_convention_components,
    } = sfc;
    let sfc_style_asset_projection =
        project_sfc_style_asset_references(&root, style_asset_references);
    let sfc_template_component_projection = project_sfc_template_component_refs(
        &root,
        template_component_refs,
        &source_use_resolved_targets,
        &source_use_external_record_ids,
        &source_use_non_source_asset_record_ids,
        &source_use_non_source_asset_targets,
        &source_use_generated_virtual_record_ids,
    );
    let sfc_global_component_projection = project_sfc_global_component_registrations(
        &root,
        global_component_registrations,
        &source_use_resolved_targets,
        &source_use_external_record_ids,
        &source_use_non_source_asset_record_ids,
        &source_use_non_source_asset_targets,
        &source_use_generated_virtual_record_ids,
    );
    let sfc_generated_manifest_projection = project_sfc_generated_component_manifests(
        &root,
        generated_component_manifests,
        &source_use_resolved_targets,
        &source_use_external_record_ids,
        &source_use_non_source_asset_record_ids,
        &source_use_non_source_asset_targets,
        &source_use_generated_virtual_record_ids,
    );
    let sfc_framework_convention_projection =
        project_sfc_framework_convention_components(&root, framework_convention_components);

    let ComputedAnyContamination {
        helper_owners_by_identity,
        type_owners_by_identity,
        def_index: projected_def_index,
    } = build_any_contamination_facts(&root, &def_index, &file_data);
    let ComputedFanIn {
        symbol_fan_in,
        fan_in_by_identity,
        fan_in_by_identity_space,
    } = build_fan_in(&root, &def_index, &fan_in_inputs);
    let top_symbol_fan_in = top_symbol_fan_in(symbol_fan_in);
    let ComputedDeadCandidates {
        dead,
        truly_dead,
        dead_in_prod,
        dead_in_test,
    } = build_dead_candidates(
        &root,
        &def_index,
        &file_data,
        &fan_in_inputs,
        &dead_candidate_inputs,
    );

    let SourceUseAssemblyResponse {
        resolved_internal_edges,
        dependency_import_consumers,
        unresolved_internal_by_prefix,
        prefix_examples,
        unresolved_internal_specifiers,
        unresolved_internal_specifier_records,
        namespace_re_export_diagnostics,
        generated_virtual_surfaces,
        generated_virtual_import_consumers,
        ..
    } = source_use_assembly;
    let resolved_internal_edges = resolved_internal_edges
        .into_iter()
        .map(serde_json::to_value)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let dependency_import_consumers = dependency_import_consumers
        .into_iter()
        .map(serde_json::to_value)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let namespace_re_export_diagnostics = namespace_re_export_diagnostics
        .into_iter()
        .map(serde_json::to_value)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let unresolved_internal_specifiers = unresolved_internal_specifiers.into_iter().collect();
    let generated_consumer_blind_zones = build_generated_consumer_blind_zones(
        &root,
        &unresolved_internal_specifier_records,
        include_tests,
        &exclude,
        &generated_artifacts_mode,
    );
    let uses_projection = json!({
        "resolvedInternal": resolved_internal_uses,
        "resolvedGeneratedVirtual": resolved_generated_virtual_uses,
        "nonSourceAsset": non_source_asset_uses,
        "external": external_uses,
        "unresolvedInternal": unresolved_internal_uses,
        "mdxConsumers": mdx_consumer_uses,
        "sfcScriptConsumers": sfc_script_consumer_uses,
        "sfcScriptSrcReachability": sfc_script_src_reachability_uses,
        "sfcStyleAssetReferences": sfc_style_asset_projection.resolved_count,
        "sfcTemplateComponentRefs": sfc_template_component_projection.count,
        "sfcGlobalComponentRegistrations": sfc_global_component_projection.count,
        "sfcGeneratedComponentManifests": generated_manifest_external_uses + sfc_generated_manifest_projection.count,
        "sfcFrameworkConventionComponents": sfc_framework_convention_projection.count,
        "unresolvedInternalRatio": unresolved_ratio,
    });
    let artifact_summary = json!({
        "totalUsesResolved": total_uses,
        "unresolvedUses": unresolved_uses,
        "uses": uses_projection,
        "resolvedInternalEdgeCount": resolved_internal_edges.len(),
        "deadTotal": dead.len(),
        "trulyDead": truly_dead.len(),
        "deadInProd": dead_in_prod.len(),
        "deadInTest": dead_in_test.len(),
        "generatedConsumerBlindZoneCount": generated_consumer_blind_zones.len(),
    });

    let mut artifact = json!({
        "meta": Value::Object(meta),
        "files": files.len(),
        "totalDefs": total_defs,
        "totalClassMethods": total_class_methods,
        "totalPreWriteLocalOperations": total_local_operations,
        "totalUsesResolved": total_uses,
        "unresolvedUses": unresolved_uses,
        "uses": uses_projection,
        "dependencyImportConsumers": sort_values_by_key(dependency_import_consumers, dependency_consumer_key),
        "resolvedInternalEdges": sort_values_by_key(resolved_internal_edges, resolved_internal_edge_key),
        "sfcStyleAssetReferences": sort_values_by_key(sfc_style_asset_projection.references, sfc_style_asset_key),
        "sfcTemplateComponentRefs": sort_values_by_key(sfc_template_component_projection.refs, sfc_template_ref_key),
        "sfcGlobalComponentRegistrations": sort_values_by_key(sfc_global_component_projection.registrations, sfc_global_registration_key),
        "sfcGeneratedComponentManifests": sort_values_by_key(sfc_generated_manifest_projection.manifests, sfc_generated_manifest_key),
        "sfcFrameworkConventionComponents": sort_values_by_key(sfc_framework_convention_projection.components, sfc_framework_convention_key),
        "generatedConsumerBlindZones": sort_values_by_key(generated_consumer_blind_zones, generated_blind_zone_key),
        "generatedVirtualSurfaces": sort_generated_virtual_surfaces(generated_virtual_surfaces),
        "generatedVirtualImportConsumers": sort_values_by_key(generated_virtual_import_consumers, generated_import_consumer_key),
        "topUnresolvedSpecifiers": top_unresolved_specifiers(&unresolved_internal_by_prefix, &prefix_examples),
        "dynamicImportOpacity": build_dynamic_import_opacity(&root, &file_data),
        "cjsExportSurfaceByFile": build_cjs_export_surface_by_file(&root, &file_data),
        "cjsRequireOpacity": build_cjs_require_opacity(&root, &file_data),
        "unresolvedInternalSpecifiers": sorted_strings(unresolved_internal_specifiers),
        "unresolvedInternalSpecifierRecords": sort_values_by_key(unresolved_internal_specifier_records.clone(), unresolved_record_key),
        "unresolvedInternalSummaryByReason": unresolved_summary_by_reason(&unresolved_internal_specifier_records),
        "filesWithParseErrors": files_with_parse_errors(&root, &parse_error_files),
        "deadTotal": dead.len(),
        "trulyDead": truly_dead.len(),
        "deadInProd": dead_in_prod.len(),
        "deadInTest": dead_in_test.len(),
        "topSymbolFanIn": top_symbol_fan_in,
        "fanInByIdentity": Value::Object(fan_in_by_identity),
        "fanInByIdentitySpace": Value::Object(fan_in_by_identity_space),
        "namespaceReExportDiagnostics": sort_values_by_key(namespace_re_export_diagnostics, namespace_re_export_key),
        "helperOwnersByIdentity": helper_owners_by_identity,
        "typeOwnersByIdentity": type_owners_by_identity,
        "defIndex": projected_def_index,
        "classMethodIndex": build_class_method_index(&root, &file_data),
        "preWriteLocalOperationIndex": build_pre_write_local_operation_index(&root, &file_data),
        "deadProdList": dead_in_prod,
        "reExportsByFile": build_re_exports_by_file(&root, &file_data),
    });
    if let Some(object) = artifact.as_object_mut() {
        object.insert("artifactSummary".to_string(), artifact_summary);
    }
    Ok(artifact)
}

fn sort_values_by_key(mut values: Vec<Value>, key_fn: fn(&Value) -> String) -> Vec<Value> {
    values.sort_by_key(key_fn);
    values
}

fn sorted_strings(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn value_string(value: &Value, field: &str) -> String {
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

fn padded_line(value: &Value) -> String {
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

fn dependency_consumer_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "depRoot"),
        value_string(value, "fromSpec"),
        value_string(value, "file"),
        value_string(value, "kind")
    )
}

fn resolved_internal_edge_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "from"),
        value_string(value, "to"),
        value_string(value, "kind"),
        value_string(value, "source"),
        value_bool_key(value, "typeOnly")
    )
}

fn sfc_style_asset_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "fromSpec"),
        value_string(value, "source"),
        value_string(value, "status")
    )
}

fn sfc_template_ref_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "tagName"),
        value_string(value, "bindingName"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

fn sfc_global_registration_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "registrationFile"),
        value_string(value, "componentName"),
        value_string(value, "bindingName"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

fn sfc_generated_manifest_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "manifestFile"),
        value_string(value, "componentName"),
        value_string(value, "fromSpec"),
        value_string(value, "status"),
        value_string(value, "reason")
    )
}

fn sfc_framework_convention_key(value: &Value) -> String {
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

fn generated_blind_zone_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}",
        value_string(value, "scopePackageRoot"),
        value_string(value, "candidatePath"),
        value_string(value, "specifier"),
        value_string(value, "consumerFile")
    )
}

fn build_generated_consumer_blind_zones(
    root: &str,
    unresolved_records: &[Value],
    include_tests: bool,
    exclude: &[String],
    mode: &str,
) -> Vec<Value> {
    let root_path = Path::new(root);
    let mut zones = Vec::new();
    let mut seen = BTreeSet::new();

    let scan_options = ScanScopeOptions {
        include_tests,
        exclude: exclude.to_vec(),
        ..ScanScopeOptions::default()
    };

    for record in unresolved_records {
        if !is_generated_artifact_missing_record(record) {
            continue;
        }
        let Some(artifact) = record.get("generatedArtifact").and_then(Value::as_object) else {
            continue;
        };
        for candidate in target_candidates(record) {
            let Some(candidate_path) = generated_candidate_repo_relative(root_path, &candidate)
            else {
                continue;
            };
            let Some(scope_package_root) =
                consumer_zone_scope_root(record, artifact, &candidate_path)
            else {
                continue;
            };

            let abs_candidate = root_path.join(&candidate_path);
            let mut status = "missing";
            let mut scan_scope_reason = None;
            if abs_candidate.exists() {
                let scope = scan_scope_status_for_path(root_path, &abs_candidate, &scan_options);
                if scope.included {
                    continue;
                }
                status = "present-but-out-of-scope";
                scan_scope_reason = scope.reason.or(Some("excluded"));
            }

            let mut object = Map::new();
            object.insert("reason".to_string(), json!("generated-consumer-blind-zone"));
            object.insert(
                "sourceReason".to_string(),
                json!(value_string(record, "reason")),
            );
            object.insert(
                "specifier".to_string(),
                json!(nullable_string(record, "specifier")),
            );
            object.insert(
                "consumerFile".to_string(),
                json!(nullable_string(record, "consumerFile")
                    .or_else(|| nullable_string(record, "fromHint"))),
            );
            object.insert(
                "matchedPackage".to_string(),
                json!(nullable_string_from_map(artifact, "matchedPackage")),
            );
            object.insert(
                "targetSubpath".to_string(),
                json!(nullable_string_from_map(artifact, "targetSubpath")),
            );
            object.insert(
                "generatorFamily".to_string(),
                json!(nullable_string_from_map(artifact, "generatorFamily")),
            );
            object.insert(
                "confidence".to_string(),
                json!(nullable_string_from_map(artifact, "confidence")),
            );
            object.insert("candidatePath".to_string(), json!(candidate_path));
            object.insert("status".to_string(), json!(status));
            object.insert("scopePackageRoot".to_string(), json!(scope_package_root));
            object.insert("mode".to_string(), json!(mode));
            if let Some(reason) = scan_scope_reason {
                object.insert("scanScopeReason".to_string(), json!(reason));
            }
            if mode == "prepared" {
                object.insert("staleStatus".to_string(), json!("unknown"));
                object.insert(
                    "staleReason".to_string(),
                    json!("generator-input-hash-not-recorded"),
                );
            }
            let zone = Value::Object(object);
            let key = generated_consumer_zone_dedupe_key(&zone);
            if seen.insert(key) {
                zones.push(zone);
            }
        }
    }

    zones
}

fn is_generated_artifact_missing_record(record: &Value) -> bool {
    value_string(record, "reason") == "workspace-generated-artifact-missing"
        && record
            .get("generatedArtifact")
            .is_some_and(Value::is_object)
}

fn target_candidates(record: &Value) -> Vec<String> {
    record
        .get("targetCandidates")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn generated_candidate_repo_relative(root: &Path, candidate: &str) -> Option<String> {
    let root_text = normalize_path_segments(&root.to_string_lossy());
    let root_text = root_text.trim_end_matches('/');
    let candidate_path = Path::new(candidate);
    let candidate_text = if candidate_path.is_absolute() {
        normalize_path_segments(&candidate_path.to_string_lossy())
    } else {
        normalize_path_segments(&format!("{root_text}/{candidate}"))
    };
    let prefix = format!("{root_text}/");
    candidate_text
        .strip_prefix(&prefix)
        .filter(|relative| !relative.is_empty() && *relative != "..")
        .filter(|relative| !relative.starts_with("../"))
        .map(ToString::to_string)
}

fn generated_package_root(artifact: &Map<String, Value>) -> Option<String> {
    nullable_string_from_map(artifact, "packageRoot")
        .or_else(|| nullable_string_from_map(artifact, "packageDir"))
        .or_else(|| nullable_string_from_map(artifact, "workspaceRoot"))
}

fn package_root_from_candidate(candidate_path: &str) -> Option<String> {
    let parts = candidate_path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if matches!(parts.first(), Some(&"apps" | &"packages")) && parts.len() >= 2 {
        return Some(format!("{}/{}", parts[0], parts[1]));
    }
    None
}

fn consumer_zone_scope_root(
    _record: &Value,
    artifact: &Map<String, Value>,
    candidate_path: &str,
) -> Option<String> {
    generated_package_root(artifact).or_else(|| package_root_from_candidate(candidate_path))
}

fn nullable_string(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn nullable_string_from_map(object: &Map<String, Value>, field: &str) -> Option<String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn generated_consumer_zone_dedupe_key(zone: &Value) -> String {
    [
        value_string(zone, "specifier"),
        value_string(zone, "consumerFile"),
        value_string(zone, "candidatePath"),
        value_string(zone, "mode"),
    ]
    .join("|")
}

fn generated_import_consumer_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "specifier"),
        value_string(value, "name"),
        value_string(value, "kind"),
        value_string(value, "surfaceId")
    )
}

fn unresolved_record_key(value: &Value) -> String {
    format!(
        "{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "specifier"),
        value_string(value, "kind")
    )
}

fn namespace_re_export_key(value: &Value) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        value_string(value, "consumerFile"),
        value_string(value, "exportedName"),
        value_string(value, "targetFile"),
        value_string(value, "kind"),
        value.get("line").map(Value::to_string).unwrap_or_default()
    )
}

fn top_unresolved_specifiers(
    counters: &BTreeMap<String, usize>,
    examples: &BTreeMap<String, String>,
) -> Vec<Value> {
    let mut entries = counters.iter().collect::<Vec<_>>();
    entries.sort_by_key(|(key, count)| (Reverse(**count), (*key).clone()));
    entries
        .into_iter()
        .take(20)
        .map(|(key, count)| {
            let example = examples
                .get(key)
                .cloned()
                .unwrap_or_else(|| key.clone());
            let mut object = Map::new();
            object.insert("specifierPrefix".to_string(), json!(key));
            object.insert("count".to_string(), json!(count));
            object.insert("example".to_string(), json!(example));
            if likely_alias_prefix(object.get("specifierPrefix").and_then(Value::as_str).unwrap_or_default()) {
                object.insert(
                    "likelyCause".to_string(),
                    json!("possible unresolved tsconfig paths alias. Check per-app tsconfig.json for a compilerOptions.paths entry matching this prefix. See FP-36 in references/false-positive-index.md."),
                );
            }
            Value::Object(object)
        })
        .collect()
}

fn likely_alias_prefix(prefix: &str) -> bool {
    prefix.starts_with("@/")
        || prefix.starts_with("~/")
        || prefix.starts_with("#/")
        || (prefix.starts_with('@') && prefix.get(1..).is_some_and(|rest| rest.contains('/')))
}

fn compact_unresolved_example(record: &Value) -> Value {
    let mut object = Map::new();
    for field in ["specifier", "consumerFile", "kind"] {
        if let Some(value) = record.get(field) {
            object.insert(field.to_string(), value.clone());
        }
    }
    if let Some(value) = record.get("typeOnly").filter(|value| value.is_boolean()) {
        object.insert("typeOnly".to_string(), value.clone());
    }
    for field in ["resolverStage", "matchedPattern", "hint"] {
        if let Some(value) = record.get(field).filter(|value| value.is_string()) {
            object.insert(field.to_string(), value.clone());
        }
    }
    if let Some(candidates) = record.get("targetCandidates").and_then(Value::as_array) {
        if !candidates.is_empty() {
            object.insert(
                "targetCandidates".to_string(),
                Value::Array(candidates.iter().take(3).cloned().collect()),
            );
        }
    }
    Value::Object(object)
}

fn unresolved_space(record: &Value) -> &'static str {
    match record.get("typeOnly").and_then(Value::as_bool) {
        Some(true) => "type",
        Some(false) => "value",
        None => "unknown",
    }
}

#[derive(Default)]
struct UnresolvedGroup {
    count: usize,
    spaces_type: usize,
    spaces_value: usize,
    spaces_unknown: usize,
    resolver_stages: BTreeMap<String, usize>,
    hints: BTreeMap<String, usize>,
    examples: Vec<Value>,
}

fn unresolved_summary_by_reason(records: &[Value]) -> Value {
    let mut groups = BTreeMap::<String, UnresolvedGroup>::new();
    for record in records {
        let reason = value_string(record, "reason");
        let reason = if reason.is_empty() {
            "unknown-internal-resolution".to_string()
        } else {
            reason
        };
        let group = groups.entry(reason).or_default();
        group.count += 1;
        match unresolved_space(record) {
            "type" => group.spaces_type += 1,
            "value" => group.spaces_value += 1,
            _ => group.spaces_unknown += 1,
        }
        let resolver_stage = value_string(record, "resolverStage");
        if !resolver_stage.is_empty() {
            *group.resolver_stages.entry(resolver_stage).or_insert(0) += 1;
        }
        let hint = value_string(record, "hint");
        if !hint.is_empty() {
            *group.hints.entry(hint).or_insert(0) += 1;
        }
        group.examples.push(compact_unresolved_example(record));
    }

    let mut entries = groups.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .count
            .cmp(&left.1.count)
            .then_with(|| left.0.cmp(&right.0))
    });
    let mut out = Map::new();
    for (reason, mut group) in entries {
        group.examples.sort_by_key(unresolved_record_key);
        let mut object = Map::new();
        object.insert("count".to_string(), json!(group.count));
        object.insert(
            "spaces".to_string(),
            json!({
                "type": group.spaces_type,
                "value": group.spaces_value,
                "unknown": group.spaces_unknown,
            }),
        );
        if !group.resolver_stages.is_empty() {
            object.insert("resolverStages".to_string(), json!(group.resolver_stages));
        }
        if !group.hints.is_empty() {
            object.insert("hints".to_string(), json!(group.hints));
        }
        object.insert(
            "examples".to_string(),
            Value::Array(group.examples.into_iter().take(5).collect()),
        );
        out.insert(reason, Value::Object(object));
    }
    Value::Object(out)
}

fn build_dynamic_import_opacity(root: &str, file_data: &[FileDataRecord]) -> Vec<Value> {
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

fn build_cjs_export_surface_by_file(root: &str, file_data: &[FileDataRecord]) -> Value {
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

fn build_cjs_require_opacity(root: &str, file_data: &[FileDataRecord]) -> Vec<Value> {
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

fn files_with_parse_errors(root: &str, entries: &[String]) -> Vec<String> {
    let mut files = entries
        .iter()
        .map(|file| rel_path(root, file))
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    files
}

fn is_absolute_like_path(path: &str) -> bool {
    path.starts_with('/')
        || (path.len() >= 3 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'/')
}

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

fn build_class_method_index(root: &str, file_data: &[FileDataRecord]) -> Value {
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

fn build_pre_write_local_operation_index(root: &str, file_data: &[FileDataRecord]) -> Value {
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

fn build_re_exports_by_file(root: &str, file_data: &[FileDataRecord]) -> Value {
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

fn sort_generated_virtual_surfaces(values: Vec<Value>) -> Vec<Value> {
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

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

fn rel_path(root: &str, file: &str) -> String {
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    let file = normalize_slashes(file);
    let prefix = format!("{root}/");
    if let Some(stripped) = file.strip_prefix(&prefix) {
        stripped.to_string()
    } else {
        file
    }
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

fn resolve_prefix_target(file: &str, prefix: &str) -> String {
    let normalized_file = normalize_slashes(file);
    let base = normalized_file
        .rsplit_once('/')
        .map(|(base, _)| base)
        .unwrap_or("");
    normalize_path_segments(&format!("{base}/{prefix}"))
}

fn normalize_path_segments(path: &str) -> String {
    let mut prefix = String::new();
    let mut rest = normalize_slashes(path);
    if rest.len() >= 3 && rest.as_bytes()[1] == b':' && rest.as_bytes()[2] == b'/' {
        prefix = rest[..3].to_string();
        rest = rest[3..].to_string();
    } else if rest.starts_with('/') {
        prefix = "/".to_string();
        rest = rest.trim_start_matches('/').to_string();
    }

    let mut parts = Vec::new();
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    format!("{prefix}{}", parts.join("/"))
}

#[cfg(test)]
mod tests;
