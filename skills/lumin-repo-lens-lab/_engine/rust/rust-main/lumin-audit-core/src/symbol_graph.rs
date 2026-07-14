pub(crate) mod any_contamination;
mod evidence;
mod prepare;
mod protocol;
mod reachability;
mod sfc;

use crate::source_use_assembly::{
    build_embedded_source_use_assembly_response_with_path_table, SourceUseAssemblyResponse,
};
use anyhow::{bail, Result};
use serde_json::{json, Map, Value};

use any_contamination::{build_any_contamination_facts, ComputedAnyContamination};
use evidence::{
    build_cjs_export_surface_by_file, build_cjs_require_opacity, build_class_method_index,
    build_dynamic_import_opacity, build_generated_consumer_blind_zones,
    build_pre_write_local_operation_index, build_re_exports_by_file, dependency_consumer_key,
    files_with_parse_errors, generated_blind_zone_key, generated_import_consumer_key,
    namespace_re_export_key, resolved_internal_edge_key, sfc_framework_convention_key,
    sfc_generated_manifest_key, sfc_global_registration_key, sfc_style_asset_key,
    sfc_template_ref_key, sort_generated_virtual_surfaces, sort_values_by_key, sorted_strings,
    top_unresolved_specifiers, unresolved_record_key, unresolved_summary_by_reason,
};
use prepare::{prepare_symbol_graph_request, PreparedSymbolGraphRequest};
use protocol::SymbolGraphSfcInputs;
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
        object.insert("deadTestList".to_string(), Value::Array(dead_in_test));
        object.insert("artifactSummary".to_string(), artifact_summary);
    }
    Ok(artifact)
}

fn round4(value: f64) -> f64 {
    (value * 10000.0).round() / 10000.0
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

#[cfg(test)]
mod tests;
