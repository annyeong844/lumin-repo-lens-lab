use crate::scan_scope::{scan_scope_status_for_path, ScanScopeOptions};
use crate::source_use_assembly::{
    build_embedded_source_use_assembly_response, is_namespace_reexport_use, package_root_from_spec,
    SourceUseAssemblyRequest, SourceUseAssemblyResponse,
};
use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub const SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION: &str = "lumin-symbol-graph-producer-request.v1";

const TOOL_NAME: &str = "build-symbol-graph.mjs";
const SYMBOL_META_SCHEMA_VERSION: i64 = 3;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolGraphRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default = "default_true")]
    pub include_tests: bool,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default = "default_generated_artifacts_mode")]
    pub generated_artifacts_mode: String,
    #[serde(default)]
    pub path_table: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub file_ids: Vec<usize>,
    #[serde(default)]
    pub def_index: Vec<DefinitionFile>,
    #[serde(default)]
    pub file_data: Vec<FileDataRecord>,
    #[serde(default)]
    pub parse_errors: usize,
    #[serde(default)]
    pub warnings: Vec<Value>,
    #[serde(default)]
    pub next_cache_entries: BTreeMap<String, Value>,
    #[serde(default)]
    pub unresolved_internal_by_prefix: Vec<CountEntry>,
    #[serde(default)]
    pub prefix_examples: BTreeMap<String, String>,
    #[serde(default)]
    pub unresolved_internal_specifiers: Vec<String>,
    #[serde(default)]
    pub unresolved_internal_specifier_records: Vec<Value>,
    #[serde(default)]
    pub language_support: Value,
    #[serde(default)]
    pub total_uses: usize,
    #[serde(default)]
    pub unresolved_uses: usize,
    #[serde(default)]
    pub resolved_internal_uses: usize,
    #[serde(default)]
    pub resolved_generated_virtual_uses: usize,
    #[serde(default)]
    pub non_source_asset_uses: usize,
    #[serde(default)]
    pub external_uses: usize,
    #[serde(default)]
    pub dependency_import_consumers: Vec<Value>,
    #[serde(default)]
    pub external_dependency_import_inputs: Vec<ExternalDependencyImportInput>,
    #[serde(default)]
    pub resolved_internal_edges: Vec<Value>,
    #[serde(default)]
    pub generated_consumer_blind_zones: Vec<Value>,
    #[serde(default)]
    pub generated_consumer_blind_zone_inputs: Vec<Value>,
    #[serde(default)]
    pub generated_virtual_surfaces: Vec<Value>,
    #[serde(default)]
    pub generated_virtual_import_consumers: Vec<Value>,
    #[serde(default)]
    pub unresolved_internal_uses: usize,
    #[serde(default)]
    pub mdx_consumer_uses: usize,
    #[serde(default)]
    pub sfc_script_consumer_uses: usize,
    #[serde(default)]
    pub sfc_script_src_reachability_uses: usize,
    #[serde(default)]
    pub sfc_style_asset_reference_uses: usize,
    #[serde(default)]
    pub sfc_template_component_ref_uses: usize,
    #[serde(default)]
    pub sfc_global_component_registration_uses: usize,
    #[serde(default)]
    pub sfc_generated_component_manifest_uses: usize,
    #[serde(default)]
    pub sfc_framework_convention_component_uses: usize,
    #[serde(default)]
    pub sfc_style_asset_references: Vec<Value>,
    #[serde(default)]
    pub sfc_style_asset_reference_inputs: Vec<SfcStyleAssetReferenceInput>,
    #[serde(default)]
    pub sfc_template_component_refs: Vec<Value>,
    #[serde(default)]
    pub sfc_template_component_ref_inputs: Vec<SfcTemplateComponentRefInput>,
    #[serde(default)]
    pub sfc_global_component_registrations: Vec<Value>,
    #[serde(default)]
    pub sfc_global_component_registration_inputs: Vec<SfcGlobalComponentRegistrationInput>,
    #[serde(default)]
    pub sfc_generated_component_manifests: Vec<Value>,
    #[serde(default)]
    pub sfc_generated_component_manifest_inputs: Vec<SfcGeneratedComponentManifestInput>,
    #[serde(default)]
    pub sfc_framework_convention_components: Vec<Value>,
    #[serde(default)]
    pub sfc_framework_convention_component_inputs: Vec<SfcFrameworkConventionComponentInput>,
    #[serde(default)]
    pub dead: Vec<Value>,
    #[serde(default)]
    pub truly_dead: Vec<Value>,
    #[serde(default)]
    pub dead_in_prod: Vec<Value>,
    #[serde(default)]
    pub dead_in_test: Vec<Value>,
    #[serde(default)]
    pub symbol_fan_in: Vec<Value>,
    #[serde(default)]
    pub fan_in_by_identity: Value,
    #[serde(default)]
    pub fan_in_by_identity_space: Value,
    #[serde(default)]
    pub fan_in_inputs: Option<FanInInputs>,
    #[serde(default)]
    pub dead_candidate_inputs: Option<DeadCandidateInputs>,
    #[serde(default)]
    pub source_use_assembly: Option<SourceUseAssemblyRequest>,
    #[serde(default)]
    pub namespace_re_export_diagnostics: Vec<Value>,
    #[serde(default)]
    pub any_contamination_facts: Value,
    #[serde(default)]
    pub incremental: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionFile {
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub file_path_id: Option<usize>,
    #[serde(default)]
    pub definitions: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDataRecord {
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub file_path_id: Option<usize>,
    #[serde(default)]
    pub py_dunder_all: Option<Vec<String>>,
    #[serde(default)]
    pub re_exports: Vec<Value>,
    #[serde(default)]
    pub class_methods: Vec<Value>,
    #[serde(default)]
    pub local_operations: Vec<Value>,
    #[serde(default)]
    pub type_escapes: Vec<Value>,
    #[serde(default)]
    pub dynamic_import_opacity: Vec<Value>,
    #[serde(default)]
    pub cjs_export_surface: Option<Value>,
    #[serde(default)]
    pub cjs_require_opacity: Vec<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CountEntry {
    pub key: String,
    pub count: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDependencyImportInput {
    #[serde(default)]
    pub consumer_file: String,
    #[serde(default)]
    pub consumer_file_id: Option<usize>,
    pub from_spec: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub type_only: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanInInputs {
    #[serde(default)]
    pub consumer_entries: Vec<FanInConsumerEntry>,
    #[serde(default)]
    pub namespace_user_entries: Vec<FanInNamespaceUserEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanInConsumerEntry {
    pub def_file: String,
    pub symbol: String,
    pub consumer_file: String,
    #[serde(default)]
    pub space: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FanInNamespaceUserEntry {
    pub def_file: String,
    pub consumer_file: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadCandidateInputs {
    #[serde(default)]
    pub barrel_files: Vec<String>,
    #[serde(default)]
    pub test_like_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcStyleAssetReferenceInput {
    pub consumer_file: String,
    pub from_spec: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub style_kind: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub import_syntax: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
    #[serde(default)]
    pub sfc_block_kind: Option<String>,
    #[serde(default)]
    pub sfc_language: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_generated_artifacts_mode() -> String {
    "default".to_string()
}

#[derive(Debug)]
struct SfcStyleAssetProjection {
    references: Vec<Value>,
    resolved_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcTemplateComponentRefInput {
    pub consumer_file: String,
    #[serde(default)]
    pub tag_name: Option<String>,
    #[serde(default)]
    pub normalized_tag_name: Option<String>,
    #[serde(default)]
    pub binding_name: Option<String>,
    #[serde(default)]
    pub binding_source: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub template_kind: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub source_use_record_id: Option<String>,
    #[serde(default)]
    pub binding_kind: Option<String>,
    #[serde(default)]
    pub imported_name: Option<String>,
    #[serde(default)]
    pub member_name: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
    #[serde(default)]
    pub sfc_block_kind: Option<String>,
}

#[derive(Debug)]
struct SfcTemplateComponentProjection {
    refs: Vec<Value>,
    count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcGlobalComponentRegistrationInput {
    pub registration_file: String,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub component_name: Option<String>,
    #[serde(default)]
    pub normalized_tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub binding_name: Option<String>,
    #[serde(default)]
    pub binding_source: Option<String>,
    #[serde(default)]
    pub from_spec: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub source_use_record_id: Option<String>,
    #[serde(default)]
    pub binding_kind: Option<String>,
    #[serde(default)]
    pub imported_name: Option<String>,
    #[serde(default)]
    pub factory_kind: Option<String>,
    #[serde(default)]
    pub ambiguity_key: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
}

#[derive(Debug)]
struct SfcGlobalComponentRegistrationProjection {
    registrations: Vec<Value>,
    count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcGeneratedComponentManifestInput {
    pub manifest_file: String,
    #[serde(default)]
    pub manifest_kind: Option<String>,
    #[serde(default)]
    pub component_name: Option<String>,
    #[serde(default)]
    pub normalized_tag_names: Vec<String>,
    #[serde(default)]
    pub binding_source: Option<String>,
    #[serde(default)]
    pub from_spec: Option<String>,
    #[serde(default)]
    pub computed_key_source: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub source_use_record_id: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
}

#[derive(Debug)]
struct SfcGeneratedComponentManifestProjection {
    manifests: Vec<Value>,
    count: usize,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcFrameworkConventionComponentInput {
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub convention_kind: Option<String>,
    #[serde(default)]
    pub consumer_file: Option<String>,
    #[serde(default)]
    pub component_name: Option<String>,
    #[serde(default)]
    pub normalized_tag_names: Option<Vec<String>>,
    #[serde(default)]
    pub tag_name: Option<String>,
    #[serde(default)]
    pub normalized_tag_name: Option<String>,
    #[serde(default)]
    pub directive_name: Option<String>,
    #[serde(default)]
    pub action_name: Option<String>,
    #[serde(default)]
    pub subscription_name: Option<String>,
    #[serde(default)]
    pub store_name: Option<String>,
    #[serde(default)]
    pub macro_name: Option<String>,
    #[serde(default)]
    pub option_name: Option<String>,
    #[serde(default)]
    pub hook_name: Option<String>,
    #[serde(default)]
    pub config_shape: Option<String>,
    #[serde(default)]
    pub config_property: Option<String>,
    #[serde(default)]
    pub extends_source: Option<String>,
    #[serde(default)]
    pub extends_source_kind: Option<String>,
    #[serde(default)]
    pub module_source: Option<String>,
    #[serde(default)]
    pub module_source_kind: Option<String>,
    #[serde(default)]
    pub source_file: Option<String>,
    #[serde(default)]
    pub config_file: Option<String>,
    #[serde(default)]
    pub component_dir: Option<String>,
    #[serde(default)]
    pub resolved_dir: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub path_prefix: Option<Value>,
    #[serde(default)]
    pub global: Option<bool>,
    #[serde(default)]
    pub manifest_file: Option<String>,
    #[serde(default)]
    pub manifest_kind: Option<String>,
    #[serde(default)]
    pub resolved_file: Option<String>,
    #[serde(default)]
    pub plugin_name: Option<String>,
    #[serde(default)]
    pub binding_name: Option<String>,
    #[serde(default)]
    pub binding_source: Option<String>,
    #[serde(default)]
    pub from_spec: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub binding_kind: Option<String>,
    #[serde(default)]
    pub imported_name: Option<String>,
    #[serde(default)]
    pub component_path_segments: Option<Vec<String>>,
    #[serde(default)]
    pub sfc_block_kind: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
}

#[derive(Debug)]
struct SfcFrameworkConventionComponentProjection {
    components: Vec<Value>,
    count: usize,
}

fn symbol_path_from_table(path_table: &[String], id: usize, field: &str) -> Result<String> {
    path_table
        .get(id)
        .filter(|path| !path.is_empty())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("symbol-graph-artifact: invalid {field} {id}"))
}

fn normalize_symbol_graph_paths(request: &mut SymbolGraphRequest) -> Result<()> {
    if request.files.is_empty() && !request.file_ids.is_empty() {
        request.files = request
            .file_ids
            .iter()
            .map(|id| symbol_path_from_table(&request.path_table, *id, "fileId"))
            .collect::<Result<Vec<_>>>()?;
    }

    for file in &mut request.def_index {
        if file.file_path.is_empty() {
            let id = file.file_path_id.ok_or_else(|| {
                anyhow::anyhow!("symbol-graph-artifact: defIndex entry missing filePath")
            })?;
            file.file_path = symbol_path_from_table(&request.path_table, id, "filePathId")?;
        }
    }
    for file in &mut request.file_data {
        if file.file_path.is_empty() {
            let id = file.file_path_id.ok_or_else(|| {
                anyhow::anyhow!("symbol-graph-artifact: fileData entry missing filePath")
            })?;
            file.file_path = symbol_path_from_table(&request.path_table, id, "filePathId")?;
        }
    }
    for input in &mut request.external_dependency_import_inputs {
        if input.consumer_file.is_empty() {
            let id = input.consumer_file_id.ok_or_else(|| {
                anyhow::anyhow!(
                    "symbol-graph-artifact: externalDependencyImportInputs entry missing consumerFile"
                )
            })?;
            input.consumer_file =
                symbol_path_from_table(&request.path_table, id, "consumerFileId")?;
        }
    }
    if let Some(source_use_assembly) = &mut request.source_use_assembly {
        if source_use_assembly.path_table.is_empty() && !request.path_table.is_empty() {
            source_use_assembly.path_table = request.path_table.clone();
        }
    }

    Ok(())
}

pub fn build_symbol_graph_artifact(mut request: SymbolGraphRequest) -> Result<Value> {
    if request.schema_version != SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION {
        bail!(
            "symbol-graph-artifact: unsupported schemaVersion '{}'",
            request.schema_version
        );
    }
    normalize_symbol_graph_paths(&mut request)?;

    let mut warnings = request.warnings.clone();
    if request.parse_errors > 0 {
        warnings.push(json!({
            "code": "parse-errors",
            "count": request.parse_errors,
            "message": format!("{} file(s) failed to parse; their defs/uses are missing from the graph", request.parse_errors),
        }));
    }
    let source_use_assembly = request
        .source_use_assembly
        .map(build_embedded_source_use_assembly_response)
        .transpose()?;

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
    meta.insert("generated".to_string(), json!(request.generated));
    meta.insert("root".to_string(), json!(request.root));
    meta.insert(
        "schemaVersion".to_string(),
        json!(SYMBOL_META_SCHEMA_VERSION),
    );
    meta.insert("supports".to_string(), supports);
    meta.insert("languageSupport".to_string(), request.language_support);
    meta.insert("warnings".to_string(), Value::Array(warnings));
    if let Some(incremental) = request.incremental {
        meta.insert("incremental".to_string(), incremental);
    }

    let total_defs = request
        .def_index
        .iter()
        .map(|file| file.definitions.len())
        .sum::<usize>();
    let total_class_methods = request
        .file_data
        .iter()
        .map(|file| file.class_methods.len())
        .sum::<usize>();
    let total_local_operations = request
        .file_data
        .iter()
        .map(|file| file.local_operations.len())
        .sum::<usize>();
    let source_use_counters = source_use_assembly
        .as_ref()
        .map(|assembly| &assembly.counters);
    let external_dependency_projection = project_external_dependency_imports(
        &request.root,
        request.external_dependency_import_inputs,
    );
    let total_uses = request.total_uses
        + source_use_counters
            .map(|counters| counters.total_uses)
            .unwrap_or(0);
    let unresolved_uses = request.unresolved_uses
        + source_use_counters
            .map(|counters| counters.unresolved_uses)
            .unwrap_or(0)
        + external_dependency_projection.count;
    let resolved_internal_uses = request.resolved_internal_uses
        + source_use_counters
            .map(|counters| counters.resolved_internal_uses)
            .unwrap_or(0);
    let external_uses = request.external_uses
        + source_use_counters
            .map(|counters| counters.external_uses)
            .unwrap_or(0)
        + external_dependency_projection.count;
    let resolved_generated_virtual_uses = request.resolved_generated_virtual_uses
        + source_use_counters
            .map(|counters| counters.resolved_generated_virtual_uses)
            .unwrap_or(0);
    let non_source_asset_uses = request.non_source_asset_uses
        + source_use_counters
            .map(|counters| counters.non_source_asset_uses)
            .unwrap_or(0);
    let unresolved_internal_uses = request.unresolved_internal_uses
        + source_use_counters
            .map(|counters| counters.unresolved_internal_uses)
            .unwrap_or(0);
    let mdx_consumer_uses = request.mdx_consumer_uses
        + source_use_counters
            .map(|counters| counters.mdx_consumer_uses)
            .unwrap_or(0);
    let sfc_script_consumer_uses = request.sfc_script_consumer_uses
        + source_use_counters
            .map(|counters| counters.sfc_script_consumer_uses)
            .unwrap_or(0);
    let sfc_script_src_reachability_uses = request.sfc_script_src_reachability_uses
        + source_use_counters
            .map(|counters| counters.sfc_script_src_reachability_uses)
            .unwrap_or(0);
    let unresolved_ratio = if resolved_internal_uses + unresolved_internal_uses > 0 {
        round4(
            unresolved_internal_uses as f64
                / (resolved_internal_uses + unresolved_internal_uses) as f64,
        )
    } else {
        0.0
    };
    let resolved_internal_edges = merge_source_use_edges(
        request.resolved_internal_edges,
        source_use_assembly.as_ref(),
    )?;
    let namespace_re_export_diagnostics = merge_source_use_namespace_diagnostics(
        request.namespace_re_export_diagnostics,
        source_use_assembly.as_ref(),
    )?;
    let dependency_import_consumers = merge_source_use_dependency_consumers(
        request.dependency_import_consumers,
        source_use_assembly.as_ref(),
        external_dependency_projection.consumers,
    )?;
    let generated_virtual_surfaces = merge_source_use_generated_virtual_surfaces(
        request.generated_virtual_surfaces,
        source_use_assembly.as_ref(),
    );
    let generated_virtual_import_consumers = merge_source_use_generated_virtual_import_consumers(
        request.generated_virtual_import_consumers,
        source_use_assembly.as_ref(),
    );
    let unresolved_internal_by_prefix = merge_source_use_unresolved_prefixes(
        request.unresolved_internal_by_prefix,
        source_use_assembly.as_ref(),
    );
    let prefix_examples =
        merge_source_use_prefix_examples(request.prefix_examples, source_use_assembly.as_ref());
    let unresolved_internal_specifiers = merge_source_use_unresolved_specifiers(
        request.unresolved_internal_specifiers,
        source_use_assembly.as_ref(),
    );
    let unresolved_internal_specifier_records = merge_source_use_unresolved_records(
        request.unresolved_internal_specifier_records,
        source_use_assembly.as_ref(),
    );
    let generated_consumer_blind_zones = build_generated_consumer_blind_zones(
        request.generated_consumer_blind_zones,
        &request.root,
        &unresolved_internal_specifier_records,
        &request.generated_consumer_blind_zone_inputs,
        request.include_tests,
        &request.exclude,
        &request.generated_artifacts_mode,
    );
    let fan_in_inputs = merge_source_use_fan_in_inputs(
        &request.root,
        request.fan_in_inputs.as_ref(),
        source_use_assembly.as_ref(),
    );
    let sfc_style_asset_projection = project_sfc_style_asset_references(
        &request.root,
        request.sfc_style_asset_references,
        request.sfc_style_asset_reference_inputs,
    );
    let source_use_resolved_targets = source_use_resolved_target_map(source_use_assembly.as_ref());
    let source_use_external_record_ids =
        source_use_external_record_set(source_use_assembly.as_ref());
    let sfc_template_component_projection = project_sfc_template_component_refs(
        &request.root,
        request.sfc_template_component_refs,
        request.sfc_template_component_ref_inputs,
        &source_use_resolved_targets,
        &source_use_external_record_ids,
    );
    let sfc_global_component_projection = project_sfc_global_component_registrations(
        &request.root,
        request.sfc_global_component_registrations,
        request.sfc_global_component_registration_inputs,
        &source_use_resolved_targets,
        &source_use_external_record_ids,
    );
    let sfc_generated_manifest_projection = project_sfc_generated_component_manifests(
        &request.root,
        request.sfc_generated_component_manifests,
        request.sfc_generated_component_manifest_inputs,
        &source_use_resolved_targets,
        &source_use_external_record_ids,
    );
    let sfc_framework_convention_projection = project_sfc_framework_convention_components(
        &request.root,
        request.sfc_framework_convention_components,
        request.sfc_framework_convention_component_inputs,
    );

    let computed_any_contamination =
        build_any_contamination_facts(&request.root, &request.def_index, &request.file_data);
    let legacy_any_contamination = request
        .any_contamination_facts
        .as_object()
        .filter(|object| {
            object.contains_key("helperOwnersByIdentity")
                || object.contains_key("typeOwnersByIdentity")
        });
    let fan_in = fan_in_inputs
        .as_ref()
        .map(|inputs| build_fan_in(&request.root, &request.def_index, inputs));
    let top_symbol_fan_in = fan_in
        .as_ref()
        .map(|computed| top_symbol_fan_in(computed.symbol_fan_in.clone()))
        .unwrap_or_else(|| top_symbol_fan_in(request.symbol_fan_in));
    let fan_in_by_identity = fan_in
        .as_ref()
        .map(|computed| Value::Object(computed.fan_in_by_identity.clone()))
        .unwrap_or_else(|| object_or_empty(request.fan_in_by_identity));
    let fan_in_by_identity_space = fan_in
        .as_ref()
        .map(|computed| Value::Object(computed.fan_in_by_identity_space.clone()))
        .unwrap_or_else(|| object_or_empty(request.fan_in_by_identity_space));
    let dead_candidates = if let Some(inputs) = request.dead_candidate_inputs.as_ref() {
        let fan_in_inputs = fan_in_inputs.as_ref().ok_or_else(|| {
            anyhow::anyhow!("symbol-graph-artifact: deadCandidateInputs requires fanInInputs")
        })?;
        Some(build_dead_candidates(
            &request.root,
            &request.def_index,
            &request.file_data,
            fan_in_inputs,
            inputs,
        ))
    } else {
        None
    };
    let dead = dead_candidates
        .as_ref()
        .map(|computed| computed.dead.clone())
        .unwrap_or_else(|| request.dead.clone());
    let truly_dead = dead_candidates
        .as_ref()
        .map(|computed| computed.truly_dead.clone())
        .unwrap_or_else(|| request.truly_dead.clone());
    let dead_in_prod = dead_candidates
        .as_ref()
        .map(|computed| computed.dead_in_prod.clone())
        .unwrap_or_else(|| request.dead_in_prod.clone());
    let dead_in_test = dead_candidates
        .as_ref()
        .map(|computed| computed.dead_in_test.clone())
        .unwrap_or_else(|| request.dead_in_test.clone());
    let uses_projection = json!({
        "resolvedInternal": resolved_internal_uses,
        "resolvedGeneratedVirtual": resolved_generated_virtual_uses,
        "nonSourceAsset": non_source_asset_uses,
        "external": external_uses,
        "unresolvedInternal": unresolved_internal_uses,
        "mdxConsumers": mdx_consumer_uses,
        "sfcScriptConsumers": sfc_script_consumer_uses,
        "sfcScriptSrcReachability": sfc_script_src_reachability_uses,
        "sfcStyleAssetReferences": request.sfc_style_asset_reference_uses + sfc_style_asset_projection.resolved_count,
        "sfcTemplateComponentRefs": request.sfc_template_component_ref_uses + sfc_template_component_projection.count,
        "sfcGlobalComponentRegistrations": request.sfc_global_component_registration_uses + sfc_global_component_projection.count,
        "sfcGeneratedComponentManifests": request.sfc_generated_component_manifest_uses + sfc_generated_manifest_projection.count,
        "sfcFrameworkConventionComponents": request.sfc_framework_convention_component_uses + sfc_framework_convention_projection.count,
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
        "files": request.files.len(),
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
        "dynamicImportOpacity": build_dynamic_import_opacity(&request.root, &request.file_data),
        "cjsExportSurfaceByFile": build_cjs_export_surface_by_file(&request.root, &request.file_data),
        "cjsRequireOpacity": build_cjs_require_opacity(&request.root, &request.file_data),
        "unresolvedInternalSpecifiers": sorted_strings(unresolved_internal_specifiers),
        "unresolvedInternalSpecifierRecords": sort_values_by_key(unresolved_internal_specifier_records.clone(), unresolved_record_key),
        "unresolvedInternalSummaryByReason": unresolved_summary_by_reason(&unresolved_internal_specifier_records),
        "filesWithParseErrors": files_with_parse_errors(&request.root, &request.next_cache_entries),
        "deadTotal": dead.len(),
        "trulyDead": truly_dead.len(),
        "deadInProd": dead_in_prod.len(),
        "deadInTest": dead_in_test.len(),
        "topSymbolFanIn": top_symbol_fan_in,
        "fanInByIdentity": fan_in_by_identity,
        "fanInByIdentitySpace": fan_in_by_identity_space,
        "namespaceReExportDiagnostics": sort_values_by_key(namespace_re_export_diagnostics, namespace_re_export_key),
        "helperOwnersByIdentity": legacy_any_contamination
            .and_then(|object| object.get("helperOwnersByIdentity").cloned())
            .unwrap_or_else(|| computed_any_contamination.helper_owners_by_identity.clone()),
        "typeOwnersByIdentity": legacy_any_contamination
            .and_then(|object| object.get("typeOwnersByIdentity").cloned())
            .unwrap_or_else(|| computed_any_contamination.type_owners_by_identity.clone()),
        "defIndex": if legacy_any_contamination.is_some() {
            build_plain_def_index(&request.root, &request.def_index)
        } else {
            computed_any_contamination.def_index
        },
        "classMethodIndex": build_class_method_index(&request.root, &request.file_data),
        "preWriteLocalOperationIndex": build_pre_write_local_operation_index(&request.root, &request.file_data),
        "deadProdList": dead_in_prod,
        "reExportsByFile": build_re_exports_by_file(&request.root, &request.file_data),
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

fn project_sfc_style_asset_references(
    root: &str,
    mut legacy_references: Vec<Value>,
    inputs: Vec<SfcStyleAssetReferenceInput>,
) -> SfcStyleAssetProjection {
    let mut resolved_count = 0;
    for input in inputs {
        let mut object = Map::new();
        let resolved_file = input
            .resolved_file
            .filter(|path| !path.is_empty())
            .or_else(|| resolve_sfc_style_asset_target(&input.consumer_file, &input.from_spec));
        object.insert(
            "consumerFile".to_string(),
            json!(rel_path(root, &input.consumer_file)),
        );
        object.insert("fromSpec".to_string(), json!(input.from_spec));
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "kind", input.kind);
        insert_optional_string(&mut object, "styleKind", input.style_kind);
        insert_optional_string(&mut object, "confidence", input.confidence);
        if let Some(resolved_file) = resolved_file {
            resolved_count += 1;
            object.insert("status".to_string(), json!("resolved"));
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        } else {
            object.insert("status".to_string(), json!("unresolved"));
            object.insert("reason".to_string(), json!("sfc-style-asset-unresolved"));
        }
        insert_optional_string(&mut object, "importSyntax", input.import_syntax);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        insert_optional_string(&mut object, "sfcBlockKind", input.sfc_block_kind);
        insert_optional_string(&mut object, "sfcLanguage", input.sfc_language);
        legacy_references.push(Value::Object(object));
    }

    SfcStyleAssetProjection {
        references: legacy_references,
        resolved_count,
    }
}

fn resolve_sfc_style_asset_target(consumer_file: &str, from_spec: &str) -> Option<String> {
    if !is_relative_spec_text(from_spec) {
        return None;
    }
    let stripped = strip_style_asset_resource_query(from_spec);
    let parent = Path::new(consumer_file).parent()?;
    let target = parent.join(stripped);
    if target.is_file() {
        Some(path_to_string(target))
    } else {
        None
    }
}

fn is_relative_spec_text(spec: &str) -> bool {
    spec.starts_with("./") || spec.starts_with("../")
}

fn strip_style_asset_resource_query(spec: &str) -> &str {
    let query = spec.find('?');
    let hash = spec.find('#').filter(|index| *index > 0);
    match (query, hash) {
        (Some(query), Some(hash)) => &spec[..query.min(hash)],
        (Some(index), None) | (None, Some(index)) => &spec[..index],
        (None, None) => spec,
    }
}

fn path_to_string(path: PathBuf) -> String {
    normalize_path_segments(&path.to_string_lossy())
}

fn source_use_resolved_target_map(
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> BTreeMap<String, String> {
    source_use_assembly
        .map(|assembly| {
            assembly
                .resolved_record_targets
                .iter()
                .map(|target| (target.record_id.clone(), target.resolved_file.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn source_use_external_record_set(
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> BTreeSet<String> {
    source_use_assembly
        .map(|assembly| assembly.external_record_ids.iter().cloned().collect())
        .unwrap_or_default()
}

fn source_use_target_for_record(
    targets: &BTreeMap<String, String>,
    record_id: Option<&str>,
) -> Option<String> {
    record_id
        .filter(|record_id| !record_id.is_empty())
        .and_then(|record_id| targets.get(record_id))
        .filter(|target| !target.is_empty())
        .cloned()
}

fn source_use_record_is_external(targets: &BTreeSet<String>, record_id: Option<&str>) -> bool {
    record_id
        .filter(|record_id| !record_id.is_empty())
        .is_some_and(|record_id| targets.contains(record_id))
}

fn sfc_generated_manifest_status_and_reason(
    status: Option<String>,
    reason: Option<String>,
    source_use_record_id: Option<&str>,
    resolved_file: Option<&str>,
) -> (String, Option<String>) {
    if let Some(status) = status {
        return (status, reason);
    }
    if source_use_record_id.is_some() {
        return match resolved_file {
            Some(target) if is_js_family_target(target) => ("resolved".to_string(), reason),
            Some(_) => (
                "muted".to_string(),
                reason.or_else(|| {
                    Some("sfc-framework-generated-manifest-non-source-binding".to_string())
                }),
            ),
            None => (
                "unresolved".to_string(),
                reason.or_else(|| Some("sfc-framework-generated-manifest-unresolved".to_string())),
            ),
        };
    }
    (
        "unresolved".to_string(),
        reason.or_else(|| Some("sfc-framework-generated-manifest-unresolved".to_string())),
    )
}

fn is_js_family_target(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".d.ts")
        || lower.ends_with(".d.mts")
        || lower.ends_with(".d.cts")
        || matches!(
            Path::new(&lower)
                .extension()
                .and_then(|value| value.to_str()),
            Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts")
        )
}

fn project_sfc_template_component_refs(
    root: &str,
    mut legacy_refs: Vec<Value>,
    inputs: Vec<SfcTemplateComponentRefInput>,
    source_use_resolved_targets: &BTreeMap<String, String>,
    source_use_external_record_ids: &BTreeSet<String>,
) -> SfcTemplateComponentProjection {
    let count = inputs.len();
    for input in inputs {
        let source_use_record_id = input.source_use_record_id;
        let has_source_use_record = source_use_record_id.is_some();
        let has_external_source_use_record = source_use_record_is_external(
            source_use_external_record_ids,
            source_use_record_id.as_deref(),
        );
        let resolved_file = input.resolved_file.or_else(|| {
            source_use_target_for_record(
                source_use_resolved_targets,
                source_use_record_id.as_deref(),
            )
        });
        let status = input.status.unwrap_or_else(|| {
            if has_external_source_use_record {
                "external".to_string()
            } else if has_source_use_record && resolved_file.is_some() {
                "resolved".to_string()
            } else {
                "unresolved".to_string()
            }
        });
        let reason = input.reason.or_else(|| {
            if status == "external" {
                Some("sfc-template-component-external-binding".to_string())
            } else {
                (status == "unresolved").then(|| "sfc-template-component-unresolved".to_string())
            }
        });
        let mut object = Map::new();
        object.insert(
            "consumerFile".to_string(),
            json!(rel_path(root, &input.consumer_file)),
        );
        insert_optional_string(&mut object, "tagName", input.tag_name);
        insert_optional_string(&mut object, "normalizedTagName", input.normalized_tag_name);
        insert_optional_string(&mut object, "bindingName", input.binding_name);
        insert_optional_string(&mut object, "bindingSource", input.binding_source);
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "language", input.language);
        insert_optional_string(&mut object, "templateKind", input.template_kind);
        insert_optional_string(&mut object, "confidence", input.confidence);
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert("status".to_string(), json!(status));
        if let Some(resolved_file) = resolved_file.filter(|path| !path.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "reason", reason);
        insert_optional_string(&mut object, "bindingKind", input.binding_kind);
        insert_optional_string(&mut object, "importedName", input.imported_name);
        insert_optional_string(&mut object, "memberName", input.member_name);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        insert_optional_string(&mut object, "sfcBlockKind", input.sfc_block_kind);
        legacy_refs.push(Value::Object(object));
    }

    SfcTemplateComponentProjection {
        refs: legacy_refs,
        count,
    }
}

fn project_sfc_global_component_registrations(
    root: &str,
    mut legacy_registrations: Vec<Value>,
    inputs: Vec<SfcGlobalComponentRegistrationInput>,
    source_use_resolved_targets: &BTreeMap<String, String>,
    source_use_external_record_ids: &BTreeSet<String>,
) -> SfcGlobalComponentRegistrationProjection {
    let count = inputs.len();
    for input in inputs {
        let source_use_record_id = input.source_use_record_id;
        let has_source_use_record = source_use_record_id.is_some();
        let has_external_source_use_record = source_use_record_is_external(
            source_use_external_record_ids,
            source_use_record_id.as_deref(),
        );
        let resolved_file = input.resolved_file.or_else(|| {
            source_use_target_for_record(
                source_use_resolved_targets,
                source_use_record_id.as_deref(),
            )
        });
        let status = input.status.unwrap_or_else(|| {
            if has_external_source_use_record {
                "external".to_string()
            } else if has_source_use_record && resolved_file.is_some() {
                "resolved".to_string()
            } else {
                "unresolved".to_string()
            }
        });
        let reason = input.reason.or_else(|| {
            if status == "external" {
                Some("sfc-global-component-external-binding".to_string())
            } else {
                (status == "unresolved").then(|| "sfc-global-component-unresolved".to_string())
            }
        });
        let mut object = Map::new();
        object.insert(
            "registrationFile".to_string(),
            json!(rel_path(root, &input.registration_file)),
        );
        insert_optional_string(&mut object, "framework", input.framework);
        insert_optional_string(&mut object, "api", input.api);
        insert_optional_string(&mut object, "componentName", input.component_name);
        if let Some(mut normalized_tag_names) = input.normalized_tag_names {
            normalized_tag_names.sort();
            object.insert(
                "normalizedTagNames".to_string(),
                json!(normalized_tag_names),
            );
        }
        insert_optional_string(&mut object, "bindingName", input.binding_name);
        if let Some(binding_source) = input.binding_source.filter(|value| !value.is_empty()) {
            object.insert("bindingSource".to_string(), json!(binding_source.clone()));
            object.insert("fromSpec".to_string(), json!(binding_source));
        } else {
            insert_optional_string(&mut object, "fromSpec", input.from_spec);
        }
        insert_optional_string(&mut object, "source", input.source);
        object.insert(
            "confidence".to_string(),
            json!(if status == "muted" {
                "muted-review"
            } else {
                "registration-review"
            }),
        );
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert("status".to_string(), json!(status));
        if let Some(resolved_file) = resolved_file.filter(|path| !path.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "reason", reason);
        insert_optional_string(&mut object, "bindingKind", input.binding_kind);
        insert_optional_string(&mut object, "importedName", input.imported_name);
        insert_optional_string(&mut object, "factoryKind", input.factory_kind);
        insert_optional_string(&mut object, "ambiguityKey", input.ambiguity_key);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        legacy_registrations.push(Value::Object(object));
    }

    SfcGlobalComponentRegistrationProjection {
        registrations: legacy_registrations,
        count,
    }
}

fn project_sfc_generated_component_manifests(
    root: &str,
    mut legacy_manifests: Vec<Value>,
    inputs: Vec<SfcGeneratedComponentManifestInput>,
    source_use_resolved_targets: &BTreeMap<String, String>,
    source_use_external_record_ids: &BTreeSet<String>,
) -> SfcGeneratedComponentManifestProjection {
    let count = inputs.len();
    for input in inputs {
        let source_use_record_id = input.source_use_record_id;
        if source_use_record_is_external(
            source_use_external_record_ids,
            source_use_record_id.as_deref(),
        ) {
            continue;
        }
        let source_use_target = source_use_target_for_record(
            source_use_resolved_targets,
            source_use_record_id.as_deref(),
        );
        let resolved_file = input.resolved_file.or(source_use_target);
        let (status, reason) = sfc_generated_manifest_status_and_reason(
            input.status,
            input.reason,
            source_use_record_id.as_deref(),
            resolved_file.as_deref(),
        );
        let mut normalized_tag_names = input.normalized_tag_names;
        normalized_tag_names.sort();
        let mut object = Map::new();
        object.insert(
            "manifestFile".to_string(),
            json!(rel_path(root, &input.manifest_file)),
        );
        insert_optional_string(&mut object, "manifestKind", input.manifest_kind);
        insert_optional_string(&mut object, "componentName", input.component_name);
        object.insert(
            "normalizedTagNames".to_string(),
            json!(normalized_tag_names),
        );
        insert_optional_string(&mut object, "bindingSource", input.binding_source);
        insert_optional_string(&mut object, "fromSpec", input.from_spec);
        insert_optional_string(&mut object, "computedKeySource", input.computed_key_source);
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "confidence", input.confidence);
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert("status".to_string(), json!(status));
        if let Some(resolved_file) = resolved_file.filter(|path| !path.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "reason", reason);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        legacy_manifests.push(Value::Object(object));
    }

    SfcGeneratedComponentManifestProjection {
        manifests: legacy_manifests,
        count,
    }
}

fn project_sfc_framework_convention_components(
    root: &str,
    mut legacy_components: Vec<Value>,
    inputs: Vec<SfcFrameworkConventionComponentInput>,
) -> SfcFrameworkConventionComponentProjection {
    let count = inputs.len();
    for input in inputs {
        let binding_source = input
            .binding_source
            .filter(|value| !value.is_empty())
            .map(|value| rel_path_if_absolute(root, &value));
        let from_spec = input
            .from_spec
            .filter(|value| !value.is_empty())
            .map(|value| rel_path_if_absolute(root, &value));
        let mut object = Map::new();
        insert_optional_string(&mut object, "framework", input.framework);
        insert_optional_string(&mut object, "conventionKind", input.convention_kind);
        if let Some(consumer_file) = input.consumer_file.filter(|value| !value.is_empty()) {
            object.insert(
                "consumerFile".to_string(),
                json!(rel_path(root, &consumer_file)),
            );
        }
        insert_optional_string(&mut object, "componentName", input.component_name);
        if let Some(mut normalized_tag_names) = input.normalized_tag_names {
            normalized_tag_names.sort();
            object.insert(
                "normalizedTagNames".to_string(),
                json!(normalized_tag_names),
            );
        }
        insert_optional_string(&mut object, "tagName", input.tag_name);
        insert_optional_string(&mut object, "normalizedTagName", input.normalized_tag_name);
        insert_optional_string(&mut object, "directiveName", input.directive_name);
        insert_optional_string(&mut object, "actionName", input.action_name);
        insert_optional_string(&mut object, "subscriptionName", input.subscription_name);
        insert_optional_string(&mut object, "storeName", input.store_name);
        insert_optional_string(&mut object, "macroName", input.macro_name);
        insert_optional_string(&mut object, "optionName", input.option_name);
        insert_optional_string(&mut object, "hookName", input.hook_name);
        insert_optional_string(&mut object, "configShape", input.config_shape);
        insert_optional_string(&mut object, "configProperty", input.config_property);
        insert_optional_string(&mut object, "extendsSource", input.extends_source);
        insert_optional_string(&mut object, "extendsSourceKind", input.extends_source_kind);
        insert_optional_string(&mut object, "moduleSource", input.module_source);
        insert_optional_string(&mut object, "moduleSourceKind", input.module_source_kind);
        if let Some(source_file) = input.source_file.filter(|value| !value.is_empty()) {
            object.insert(
                "sourceFile".to_string(),
                json!(rel_path(root, &source_file)),
            );
        }
        if let Some(config_file) = input.config_file.filter(|value| !value.is_empty()) {
            object.insert(
                "configFile".to_string(),
                json!(rel_path(root, &config_file)),
            );
        }
        insert_optional_string(&mut object, "componentDir", input.component_dir);
        if let Some(resolved_dir) = input.resolved_dir.filter(|value| !value.is_empty()) {
            object.insert(
                "resolvedDir".to_string(),
                json!(rel_path(root, &resolved_dir)),
            );
        }
        insert_optional_string(&mut object, "prefix", input.prefix);
        if let Some(path_prefix) = input
            .path_prefix
            .filter(|value| value.is_boolean() || value.is_string())
        {
            object.insert("pathPrefix".to_string(), path_prefix);
        }
        if let Some(global) = input.global {
            object.insert("global".to_string(), json!(global));
        }
        if let Some(manifest_file) = input.manifest_file.filter(|value| !value.is_empty()) {
            object.insert(
                "manifestFile".to_string(),
                json!(rel_path(root, &manifest_file)),
            );
        }
        insert_optional_string(&mut object, "manifestKind", input.manifest_kind);
        if let Some(resolved_file) = input.resolved_file.filter(|value| !value.is_empty()) {
            object.insert(
                "resolvedFile".to_string(),
                json!(rel_path(root, &resolved_file)),
            );
        }
        insert_optional_string(&mut object, "pluginName", input.plugin_name);
        insert_optional_string(&mut object, "bindingName", input.binding_name);
        if let Some(binding_source) = binding_source {
            object.insert("bindingSource".to_string(), json!(binding_source.clone()));
            object.insert("fromSpec".to_string(), json!(binding_source));
        }
        if let Some(from_spec) = from_spec {
            object.insert("fromSpec".to_string(), json!(from_spec));
        }
        insert_optional_string(&mut object, "source", input.source);
        insert_optional_string(&mut object, "confidence", input.confidence);
        object.insert("eligibleForFanIn".to_string(), json!(false));
        object.insert("eligibleForSafeFix".to_string(), json!(false));
        object.insert(
            "status".to_string(),
            json!(input.status.unwrap_or_else(|| "muted".to_string())),
        );
        insert_optional_string(&mut object, "reason", input.reason);
        insert_optional_string(&mut object, "bindingKind", input.binding_kind);
        insert_optional_string(&mut object, "importedName", input.imported_name);
        if let Some(component_path_segments) = input.component_path_segments {
            object.insert(
                "componentPathSegments".to_string(),
                json!(component_path_segments),
            );
        }
        insert_optional_string(&mut object, "sfcBlockKind", input.sfc_block_kind);
        if let Some(line) = input.line {
            object.insert("line".to_string(), json!(line));
        }
        legacy_components.push(Value::Object(object));
    }

    SfcFrameworkConventionComponentProjection {
        components: legacy_components,
        count,
    }
}

fn insert_optional_string(object: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        object.insert(key.to_string(), json!(value));
    }
}

fn rel_path_if_absolute(root: &str, value: &str) -> String {
    let normalized = normalize_slashes(value);
    if is_absolute_like_path(&normalized) {
        rel_path(root, &normalized)
    } else {
        normalized
    }
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
    legacy_zones: Vec<Value>,
    root: &str,
    unresolved_records: &[Value],
    extra_records: &[Value],
    include_tests: bool,
    exclude: &[String],
    mode: &str,
) -> Vec<Value> {
    let root_path = Path::new(root);
    let mut zones = Vec::new();
    let mut seen = BTreeSet::new();

    for zone in legacy_zones {
        let key = generated_consumer_zone_dedupe_key(&zone);
        if seen.insert(key) {
            zones.push(zone);
        }
    }

    let scan_options = ScanScopeOptions {
        include_tests,
        exclude: exclude.to_vec(),
        ..ScanScopeOptions::default()
    };

    for record in unresolved_records.iter().chain(extra_records.iter()) {
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
    counters: &[CountEntry],
    examples: &BTreeMap<String, String>,
) -> Vec<Value> {
    let mut entries = counters.to_vec();
    entries.sort_by_key(|entry| Reverse(entry.count));
    entries
        .into_iter()
        .take(20)
        .map(|entry| {
            let example = examples
                .get(&entry.key)
                .cloned()
                .unwrap_or_else(|| entry.key.clone());
            let mut object = Map::new();
            object.insert("specifierPrefix".to_string(), json!(entry.key));
            object.insert("count".to_string(), json!(entry.count));
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

fn files_with_parse_errors(root: &str, entries: &BTreeMap<String, Value>) -> Vec<String> {
    let mut files = entries
        .iter()
        .filter_map(|(file, entry)| {
            if entry.get("parseError").and_then(Value::as_bool) == Some(true) {
                Some(rel_path(root, file))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn top_symbol_fan_in(mut values: Vec<Value>) -> Vec<Value> {
    values.sort_by(|left, right| {
        let left_count = left.get("count").and_then(Value::as_i64).unwrap_or(0);
        let right_count = right.get("count").and_then(Value::as_i64).unwrap_or(0);
        right_count.cmp(&left_count)
    });
    values.truncate(50);
    values
}

struct ComputedFanIn {
    symbol_fan_in: Vec<Value>,
    fan_in_by_identity: Map<String, Value>,
    fan_in_by_identity_space: Map<String, Value>,
}

#[derive(Default)]
struct DirectFanIn {
    all: BTreeSet<String>,
    value: BTreeSet<String>,
    type_only: BTreeSet<String>,
}

struct ComputedDeadCandidates {
    dead: Vec<Value>,
    truly_dead: Vec<Value>,
    dead_in_prod: Vec<Value>,
    dead_in_test: Vec<Value>,
}

struct ExternalDependencyProjection {
    count: usize,
    consumers: Vec<Value>,
}

fn project_external_dependency_imports(
    root: &str,
    inputs: Vec<ExternalDependencyImportInput>,
) -> ExternalDependencyProjection {
    let mut count = 0usize;
    let mut consumers = Vec::new();
    for input in inputs {
        let kind = input.kind.unwrap_or_else(|| "import".to_string());
        if is_namespace_reexport_use(&kind) {
            continue;
        }
        count += 1;
        let Some(dep_root) = package_root_from_spec(&input.from_spec) else {
            continue;
        };
        let mut object = Map::new();
        object.insert(
            "file".to_string(),
            json!(rel_path(root, &input.consumer_file)),
        );
        object.insert("fromSpec".to_string(), json!(input.from_spec));
        object.insert("depRoot".to_string(), json!(dep_root));
        object.insert("kind".to_string(), json!(kind));
        object.insert(
            "source".to_string(),
            json!(input.source.unwrap_or_else(|| "source-import".to_string())),
        );
        if let Some(type_only) = input.type_only {
            object.insert("typeOnly".to_string(), json!(type_only));
        }
        consumers.push(Value::Object(object));
    }
    ExternalDependencyProjection { count, consumers }
}

fn merge_source_use_edges(
    mut edges: Vec<Value>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Result<Vec<Value>> {
    let Some(assembly) = source_use_assembly else {
        return Ok(edges);
    };
    for edge in &assembly.resolved_internal_edges {
        edges.push(serde_json::to_value(edge)?);
    }
    Ok(edges)
}

fn merge_source_use_namespace_diagnostics(
    mut diagnostics: Vec<Value>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Result<Vec<Value>> {
    let Some(assembly) = source_use_assembly else {
        return Ok(diagnostics);
    };
    for diagnostic in &assembly.namespace_re_export_diagnostics {
        diagnostics.push(serde_json::to_value(diagnostic)?);
    }
    Ok(diagnostics)
}

fn merge_source_use_dependency_consumers(
    mut consumers: Vec<Value>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
    external_consumers: Vec<Value>,
) -> Result<Vec<Value>> {
    consumers.extend(external_consumers);
    let Some(assembly) = source_use_assembly else {
        return Ok(consumers);
    };
    for consumer in &assembly.dependency_import_consumers {
        consumers.push(serde_json::to_value(consumer)?);
    }
    Ok(consumers)
}

fn merge_source_use_generated_virtual_surfaces(
    base: Vec<Value>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Vec<Value> {
    let mut by_id = BTreeMap::<String, Value>::new();
    let mut anonymous = Vec::new();
    for value in base.into_iter().chain(
        source_use_assembly
            .into_iter()
            .flat_map(|assembly| assembly.generated_virtual_surfaces.clone()),
    ) {
        if let Some(id) = value.get("id").and_then(Value::as_str) {
            by_id.entry(id.to_string()).or_insert(value);
        } else {
            anonymous.push(value);
        }
    }
    anonymous.extend(by_id.into_values());
    anonymous
}

fn merge_source_use_generated_virtual_import_consumers(
    mut consumers: Vec<Value>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Vec<Value> {
    if let Some(assembly) = source_use_assembly {
        consumers.extend(assembly.generated_virtual_import_consumers.iter().cloned());
    }
    consumers
}

fn merge_source_use_unresolved_prefixes(
    entries: Vec<CountEntry>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Vec<CountEntry> {
    let mut counts = entries
        .into_iter()
        .map(|entry| (entry.key, entry.count))
        .collect::<BTreeMap<_, _>>();
    if let Some(assembly) = source_use_assembly {
        for (key, count) in &assembly.unresolved_internal_by_prefix {
            *counts.entry(key.clone()).or_insert(0) += count;
        }
    }
    counts
        .into_iter()
        .map(|(key, count)| CountEntry { key, count })
        .collect()
}

fn merge_source_use_prefix_examples(
    mut examples: BTreeMap<String, String>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> BTreeMap<String, String> {
    if let Some(assembly) = source_use_assembly {
        for (key, example) in &assembly.prefix_examples {
            examples
                .entry(key.clone())
                .or_insert_with(|| example.clone());
        }
    }
    examples
}

fn merge_source_use_unresolved_specifiers(
    mut specifiers: Vec<String>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Vec<String> {
    if let Some(assembly) = source_use_assembly {
        specifiers.extend(assembly.unresolved_internal_specifiers.iter().cloned());
    }
    specifiers
}

fn merge_source_use_unresolved_records(
    mut records: Vec<Value>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Vec<Value> {
    if let Some(assembly) = source_use_assembly {
        records.extend(
            assembly
                .unresolved_internal_specifier_records
                .iter()
                .cloned(),
        );
    }
    records
}

fn merge_source_use_fan_in_inputs(
    root: &str,
    base: Option<&FanInInputs>,
    source_use_assembly: Option<&SourceUseAssemblyResponse>,
) -> Option<FanInInputs> {
    let has_source_entries = source_use_assembly.is_some_and(|assembly| {
        !assembly.direct_consumers.is_empty() || !assembly.namespace_users.is_empty()
    });
    if base.is_none() && !has_source_entries {
        return None;
    }

    let mut consumer_entries = base
        .map(|inputs| inputs.consumer_entries.clone())
        .unwrap_or_default();
    let mut namespace_user_entries = base
        .map(|inputs| inputs.namespace_user_entries.clone())
        .unwrap_or_default();

    if let Some(assembly) = source_use_assembly {
        for direct in &assembly.direct_consumers {
            consumer_entries.push(FanInConsumerEntry {
                def_file: request_path_from_response_path(root, &direct.def_file),
                symbol: direct.symbol.clone(),
                consumer_file: request_path_from_response_path(root, &direct.consumer_file),
                space: Some(direct.space.to_string()),
            });
        }
        for namespace_user in &assembly.namespace_users {
            namespace_user_entries.push(FanInNamespaceUserEntry {
                def_file: request_path_from_response_path(root, &namespace_user.def_file),
                consumer_file: request_path_from_response_path(root, &namespace_user.consumer_file),
            });
        }
    }

    Some(FanInInputs {
        consumer_entries,
        namespace_user_entries,
    })
}

fn request_path_from_response_path(root: &str, path: &str) -> String {
    let normalized = normalize_slashes(path);
    if normalized == "." || normalized.is_empty() {
        return normalize_slashes(root).trim_end_matches('/').to_string();
    }
    if is_absolute_like_path(&normalized) {
        return normalized;
    }
    let root = normalize_slashes(root).trim_end_matches('/').to_string();
    normalize_path_segments(&format!("{root}/{normalized}"))
}

fn is_absolute_like_path(path: &str) -> bool {
    path.starts_with('/')
        || (path.len() >= 3 && path.as_bytes()[1] == b':' && path.as_bytes()[2] == b'/')
}

fn build_fan_in(root: &str, def_index: &[DefinitionFile], inputs: &FanInInputs) -> ComputedFanIn {
    let mut def_kind_by_key = BTreeMap::<(String, String), String>::new();
    let mut fan_in_by_identity = Map::new();
    let mut fan_in_by_identity_space = Map::new();

    for file in def_index {
        let rel_file = rel_path(root, &file.file_path);
        for (symbol, definition) in &file.definitions {
            let identity = format!("{rel_file}::{symbol}");
            fan_in_by_identity.insert(identity.clone(), json!(0));
            fan_in_by_identity_space.insert(
                identity,
                json!({
                    "value": 0,
                    "type": 0,
                    "broad": 0,
                }),
            );
            def_kind_by_key.insert(
                (rel_file.clone(), symbol.clone()),
                value_string(definition, "kind"),
            );
        }
    }

    let mut direct = BTreeMap::<(String, String), DirectFanIn>::new();
    let mut direct_order = Vec::<(String, String)>::new();
    let mut direct_seen = BTreeSet::<(String, String)>::new();
    for entry in &inputs.consumer_entries {
        let key = (rel_path(root, &entry.def_file), entry.symbol.clone());
        if direct_seen.insert(key.clone()) {
            direct_order.push(key.clone());
        }
        let fan_in = direct.entry(key).or_default();
        let consumer_file = rel_path(root, &entry.consumer_file);
        fan_in.all.insert(consumer_file.clone());
        if entry.space.as_deref() == Some("type") {
            fan_in.type_only.insert(consumer_file);
        } else {
            fan_in.value.insert(consumer_file);
        }
    }

    let mut namespace_users = BTreeMap::<String, BTreeSet<String>>::new();
    for entry in &inputs.namespace_user_entries {
        namespace_users
            .entry(rel_path(root, &entry.def_file))
            .or_default()
            .insert(rel_path(root, &entry.consumer_file));
    }

    let mut symbol_fan_in = Vec::new();
    for (def_file, symbol) in direct_order {
        let key = (def_file.clone(), symbol.clone());
        let Some(fan_in) = direct.get(&key) else {
            continue;
        };
        let identity = format!("{def_file}::{symbol}");
        let count = fan_in.all.len();
        symbol_fan_in.push(json!({
            "defFile": def_file,
            "symbol": symbol,
            "count": count,
            "kind": def_kind_by_key
                .get(&(def_file.clone(), symbol.clone()))
                .filter(|kind| !kind.is_empty())
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
        }));
        fan_in_by_identity.insert(identity.clone(), json!(count));
        fan_in_by_identity_space.insert(
            identity,
            json!({
                "value": fan_in.value.len(),
                "type": fan_in.type_only.len(),
                "broad": namespace_users.get(&def_file).map(BTreeSet::len).unwrap_or(0),
            }),
        );
    }

    for file in def_index {
        let file_path = rel_path(root, &file.file_path);
        let Some(broad_consumers) = namespace_users.get(&file_path) else {
            continue;
        };
        for symbol in file.definitions.keys() {
            let identity = format!("{file_path}::{symbol}");
            let mut existing = fan_in_by_identity_space
                .get(&identity)
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_else(|| {
                    let mut object = Map::new();
                    object.insert("value".to_string(), json!(0));
                    object.insert("type".to_string(), json!(0));
                    object.insert("broad".to_string(), json!(0));
                    object
                });
            existing.insert("broad".to_string(), json!(broad_consumers.len()));
            fan_in_by_identity_space.insert(identity, Value::Object(existing));
        }
    }

    ComputedFanIn {
        symbol_fan_in,
        fan_in_by_identity,
        fan_in_by_identity_space,
    }
}

fn build_dead_candidates(
    root: &str,
    def_index: &[DefinitionFile],
    file_data: &[FileDataRecord],
    fan_in_inputs: &FanInInputs,
    inputs: &DeadCandidateInputs,
) -> ComputedDeadCandidates {
    let barrel_files = inputs
        .barrel_files
        .iter()
        .map(|file| rel_path(root, file))
        .collect::<BTreeSet<_>>();
    let test_like_files = inputs
        .test_like_files
        .iter()
        .map(|file| rel_path(root, file))
        .collect::<BTreeSet<_>>();
    let direct_consumers = fan_in_inputs
        .consumer_entries
        .iter()
        .map(|entry| (rel_path(root, &entry.def_file), entry.symbol.clone()))
        .collect::<BTreeSet<_>>();
    let namespace_files = fan_in_inputs
        .namespace_user_entries
        .iter()
        .map(|entry| rel_path(root, &entry.def_file))
        .collect::<BTreeSet<_>>();
    let file_data_by_path = file_data
        .iter()
        .map(|file| (rel_path(root, &file.file_path), file))
        .collect::<BTreeMap<_, _>>();

    let mut dead = Vec::new();
    for file in def_index {
        let file_path = rel_path(root, &file.file_path);
        if barrel_files.contains(&file_path) {
            continue;
        }
        let file_namespace_used = namespace_files.contains(&file_path);
        let file_info = file_data_by_path.get(&file_path).copied();
        let public_set = file_info
            .and_then(|info| info.py_dunder_all.as_ref())
            .map(|items| items.iter().cloned().collect::<BTreeSet<_>>());
        let rel_file = file_path.clone();

        for (symbol, definition) in &file.definitions {
            if direct_consumers.contains(&(file_path.clone(), symbol.clone())) {
                continue;
            }
            if public_set
                .as_ref()
                .is_some_and(|symbols| !symbols.contains(symbol))
            {
                continue;
            }
            if definition
                .get("frameworkRegistered")
                .and_then(Value::as_bool)
                == Some(true)
            {
                continue;
            }

            let mut candidate = Map::new();
            candidate.insert("file".to_string(), json!(rel_file));
            candidate.insert("symbol".to_string(), json!(symbol));
            candidate.insert(
                "kind".to_string(),
                definition
                    .get("kind")
                    .cloned()
                    .unwrap_or_else(|| json!("unknown")),
            );
            candidate.insert(
                "line".to_string(),
                definition.get("line").cloned().unwrap_or(Value::Null),
            );
            if let Some(local_name) = definition.get("localName") {
                candidate.insert("localName".to_string(), local_name.clone());
            }
            candidate.insert("namespaceShadowed".to_string(), json!(file_namespace_used));
            dead.push(Value::Object(candidate));
        }
    }

    let mut truly_dead = Vec::new();
    let mut dead_in_prod = Vec::new();
    let mut dead_in_test = Vec::new();
    for candidate in &dead {
        if candidate
            .get("namespaceShadowed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }
        truly_dead.push(candidate.clone());
        let file = value_string(candidate, "file");
        if test_like_files.contains(&file) {
            dead_in_test.push(candidate.clone());
        } else {
            dead_in_prod.push(candidate.clone());
        }
    }

    ComputedDeadCandidates {
        dead,
        truly_dead,
        dead_in_prod,
        dead_in_test,
    }
}

fn object_or_empty(value: Value) -> Value {
    if value.is_object() {
        value
    } else {
        json!({})
    }
}

#[derive(Debug)]
struct AnyOwnerRow {
    identity: String,
    name: String,
    file: String,
    kind: String,
    line: Option<i64>,
}

#[derive(Debug)]
struct ComputedAnyContamination {
    helper_owners_by_identity: Value,
    type_owners_by_identity: Value,
    def_index: Value,
}

fn build_any_contamination_facts(
    root: &str,
    def_index: &[DefinitionFile],
    file_data: &[FileDataRecord],
) -> ComputedAnyContamination {
    let (identity_to_row, defs_by_file) = build_any_owner_lookups(root, def_index);
    let mut facts_by_identity = BTreeMap::<String, Vec<Value>>::new();

    for file in file_data {
        for fact in &file.type_escapes {
            if let Some(identity) = identity_for_escape(fact, &identity_to_row, &defs_by_file) {
                facts_by_identity
                    .entry(identity)
                    .or_default()
                    .push(fact.clone());
            }
        }
    }

    let mut helper_owners = Map::new();
    let mut type_owners = Map::new();
    let mut annotations = BTreeMap::<String, Value>::new();

    for (identity, row) in &identity_to_row {
        let annotation = build_any_annotation(
            facts_by_identity
                .get(identity)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            &row.kind,
        );
        if let Some(annotation) = annotation.clone() {
            annotations.insert(identity.clone(), annotation);
        }

        let mut owner = Map::new();
        owner.insert("ownerFile".to_string(), json!(row.file));
        owner.insert("exportedName".to_string(), json!(row.name));
        owner.insert("kind".to_string(), json!(row.kind));
        owner.insert(
            "line".to_string(),
            row.line.map_or(Value::Null, |line| json!(line)),
        );
        owner.insert(
            "anyContamination".to_string(),
            annotation.unwrap_or(Value::Null),
        );

        if is_type_owner_kind(&row.kind) {
            type_owners.insert(identity.clone(), Value::Object(owner));
        } else if is_helper_owner_kind(&row.kind) {
            helper_owners.insert(identity.clone(), Value::Object(owner));
        }
    }

    ComputedAnyContamination {
        helper_owners_by_identity: Value::Object(helper_owners),
        type_owners_by_identity: Value::Object(type_owners),
        def_index: build_annotated_def_index(root, def_index, &annotations),
    }
}

fn build_any_owner_lookups(
    root: &str,
    def_index: &[DefinitionFile],
) -> (
    BTreeMap<String, AnyOwnerRow>,
    BTreeMap<String, Vec<AnyOwnerRow>>,
) {
    let mut identity_to_row = BTreeMap::new();
    let mut defs_by_file = BTreeMap::<String, Vec<AnyOwnerRow>>::new();

    for file in def_index {
        let rel_file = rel_path(root, &file.file_path);
        for (name, def) in &file.definitions {
            let kind = value_string(def, "kind");
            if !is_any_owner_kind(&kind) {
                continue;
            }
            let identity = format!("{rel_file}::{name}");
            let row = AnyOwnerRow {
                identity: identity.clone(),
                name: name.clone(),
                file: rel_file.clone(),
                kind,
                line: value_line(def, "line"),
            };
            identity_to_row.insert(
                identity,
                AnyOwnerRow {
                    identity: row.identity.clone(),
                    name: row.name.clone(),
                    file: row.file.clone(),
                    kind: row.kind.clone(),
                    line: row.line,
                },
            );
            defs_by_file.entry(rel_file.clone()).or_default().push(row);
        }
    }

    for rows in defs_by_file.values_mut() {
        rows.sort_by(|left, right| {
            left.line
                .unwrap_or(0)
                .cmp(&right.line.unwrap_or(0))
                .then_with(|| left.name.cmp(&right.name))
        });
    }

    (identity_to_row, defs_by_file)
}

fn identity_for_escape(
    fact: &Value,
    identity_to_row: &BTreeMap<String, AnyOwnerRow>,
    defs_by_file: &BTreeMap<String, Vec<AnyOwnerRow>>,
) -> Option<String> {
    if let Some(identity) = fact
        .get("insideExportedIdentity")
        .and_then(Value::as_str)
        .filter(|identity| identity_to_row.contains_key(*identity))
    {
        return Some(identity.to_string());
    }

    if value_string(fact, "escapeKind") != "jsdoc-any" {
        return None;
    }
    let file = value_string(fact, "file");
    let line = value_line(fact, "line")?;
    defs_by_file.get(&file).and_then(|rows| {
        rows.iter()
            .find(|row| {
                let def_line = row.line.unwrap_or(0);
                def_line >= line && def_line - line <= 3
            })
            .map(|row| row.identity.clone())
    })
}

fn build_any_annotation(facts: &[Value], owner_kind: &str) -> Option<Value> {
    if facts.is_empty() {
        return None;
    }

    let mut counts = BTreeMap::<String, usize>::new();
    for fact in facts {
        let escape_kind = value_string(fact, "escapeKind");
        if !escape_kind.is_empty() {
            *counts.entry(escape_kind).or_insert(0) += 1;
        }
    }

    let any_escape_count = counts
        .iter()
        .filter(|(kind, _)| is_any_escape_kind(kind))
        .map(|(_, count)| *count)
        .sum::<usize>();
    if any_escape_count == 0 {
        return None;
    }

    let explicit_any_count = count_escape(&counts, "explicit-any");
    let as_any_count = count_escape(&counts, "as-any") + count_escape(&counts, "angle-any");
    let laundering_count = count_escape(&counts, "as-unknown-as-T");
    let rest_any_args_count = count_escape(&counts, "rest-any-args");
    let index_signature_any_count = count_escape(&counts, "index-sig-any");
    let generic_default_any_count = count_escape(&counts, "generic-default-any");
    let jsdoc_any_count = count_escape(&counts, "jsdoc-any");
    let no_explicit_any_disable_count = count_escape(&counts, "no-explicit-any-disable");
    let is_type = is_type_owner_kind(owner_kind);
    let is_helper = is_helper_owner_kind(owner_kind);
    let mut labels = BTreeSet::<String>::from(["has-any".to_string()]);

    if is_type
        || as_any_count > 0
        || explicit_any_count > 0
        || rest_any_args_count > 0
        || laundering_count > 0
        || jsdoc_any_count > 0
        || no_explicit_any_disable_count > 0
    {
        labels.insert("any-contaminated".to_string());
    }

    if laundering_count > 0
        || rest_any_args_count > 0
        || as_any_count >= 2
        || explicit_any_count >= 3
        || index_signature_any_count > 0
        || (is_type && any_escape_count >= 3)
        || (is_helper && jsdoc_any_count >= 2)
    {
        labels.insert("severely-any-contaminated".to_string());
    }

    let mut sorted_labels = labels.into_iter().collect::<Vec<_>>();
    sorted_labels.sort_by_key(|label| severity_rank(label));
    let label = sorted_labels
        .iter()
        .max_by_key(|label| severity_rank(label))
        .cloned()
        .unwrap_or_else(|| "has-any".to_string());
    let mut lines = BTreeSet::<i64>::new();
    for fact in facts {
        if let Some(line) = value_line(fact, "line") {
            lines.insert(line);
        }
    }

    Some(json!({
        "label": label,
        "labels": sorted_labels,
        "measurements": {
            "escapeCount": facts.len(),
            "anyEscapeCount": any_escape_count,
            "escapeKindCounts": counts,
            "explicitAnyCount": explicit_any_count,
            "asAnyCount": as_any_count,
            "launderingCount": laundering_count,
            "restAnyArgsCount": rest_any_args_count,
            "indexSignatureAnyCount": index_signature_any_count,
            "genericDefaultAnyCount": generic_default_any_count,
            "jsdocAnyCount": jsdoc_any_count,
            "noExplicitAnyDisableCount": no_explicit_any_disable_count,
            "lines": lines.into_iter().collect::<Vec<_>>(),
        },
    }))
}

fn build_annotated_def_index(
    root: &str,
    def_index: &[DefinitionFile],
    annotations: &BTreeMap<String, Value>,
) -> Value {
    let mut out = Map::new();
    for file in def_index {
        let rel_file = rel_path(root, &file.file_path);
        let mut definitions = Map::new();
        for (name, definition) in &file.definitions {
            let mut definition = definition.clone();
            let kind = value_string(&definition, "kind");
            if is_any_owner_kind(&kind) {
                let identity = format!("{rel_file}::{name}");
                if let Some(annotation) = annotations.get(&identity) {
                    if let Some(object) = definition.as_object_mut() {
                        object.insert("anyContamination".to_string(), annotation.clone());
                    }
                } else if let Some(object) = definition.as_object_mut() {
                    object.remove("anyContamination");
                }
            }
            definitions.insert(name.clone(), definition);
        }
        out.insert(rel_file, Value::Object(definitions));
    }
    Value::Object(out)
}

fn is_any_owner_kind(kind: &str) -> bool {
    is_type_owner_kind(kind) || is_helper_owner_kind(kind)
}

fn is_type_owner_kind(kind: &str) -> bool {
    matches!(
        kind,
        "TSInterfaceDeclaration"
            | "TSTypeAliasDeclaration"
            | "TSEnumDeclaration"
            | "TSModuleDeclaration"
    )
}

fn is_helper_owner_kind(kind: &str) -> bool {
    matches!(
        kind,
        "FunctionDeclaration" | "const-var" | "let-var" | "var-var"
    )
}

fn is_any_escape_kind(kind: &str) -> bool {
    matches!(
        kind,
        "explicit-any"
            | "as-any"
            | "angle-any"
            | "as-unknown-as-T"
            | "rest-any-args"
            | "index-sig-any"
            | "generic-default-any"
            | "no-explicit-any-disable"
            | "jsdoc-any"
    )
}

fn count_escape(counts: &BTreeMap<String, usize>, kind: &str) -> usize {
    counts.get(kind).copied().unwrap_or(0)
}

fn severity_rank(label: &str) -> i32 {
    match label {
        "severely-any-contaminated" => 3,
        "any-contaminated" => 2,
        "has-any" => 1,
        "unknown-surface" => 0,
        _ => -1,
    }
}

fn value_line(value: &Value, field: &str) -> Option<i64> {
    let value = value.get(field)?;
    if let Some(line) = value.as_i64() {
        return Some(line);
    }
    value
        .as_u64()
        .and_then(|line| i64::try_from(line).ok())
        .or_else(|| {
            value
                .as_f64()
                .filter(|line| line.is_finite())
                .map(|line| line as i64)
        })
}

fn build_plain_def_index(root: &str, def_index: &[DefinitionFile]) -> Value {
    let mut out = Map::new();
    for file in def_index {
        out.insert(rel_path(root, &file.file_path), json!(file.definitions));
    }
    Value::Object(out)
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
mod tests {
    use super::*;
    use anyhow::Context;
    use std::fs;

    #[test]
    fn builds_symbols_artifact_from_js_facts() -> Result<()> {
        let artifact = build_symbol_graph_artifact(SymbolGraphRequest {
            schema_version: SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            generated_artifacts_mode: "default".to_string(),
            path_table: vec![],
            files: vec![
                "C:/repo/src/a.ts".to_string(),
                "C:/repo/src/b.ts".to_string(),
            ],
            file_ids: vec![],
            def_index: vec![DefinitionFile {
                file_path: "C:/repo/src/a.ts".to_string(),
                file_path_id: None,
                definitions: BTreeMap::from([(
                    "alpha".to_string(),
                    json!({"name": "alpha", "kind": "FunctionDeclaration", "line": 1}),
                )]),
            }],
            file_data: vec![FileDataRecord {
                file_path: "C:/repo/src/a.ts".to_string(),
                file_path_id: None,
                py_dunder_all: None,
                re_exports: vec![json!({"source": "./b", "line": 2})],
                class_methods: vec![json!({
                    "className": "Thing",
                    "name": "run",
                    "line": 3,
                })],
                local_operations: vec![json!({
                    "identity": "src/a.ts::outer#inner",
                    "name": "inner",
                    "containerName": "outer",
                    "line": 4,
                    "operationFamily": "format",
                    "domainTokens": ["z", "a"],
                })],
                type_escapes: vec![json!({
                    "file": "src/a.ts",
                    "line": 1,
                    "escapeKind": "explicit-any",
                    "insideExportedIdentity": "src/a.ts::alpha",
                })],
                dynamic_import_opacity: vec![
                    json!({"line": 5, "kind": "template", "prefix": "../routes"}),
                ],
                cjs_export_surface: Some(
                    json!({"exact": [{"name": "foo", "kind": "property", "line": 6}], "opaque": []}),
                ),
                cjs_require_opacity: vec![json!({"line": 7, "kind": "dynamic-require"})],
            }],
            parse_errors: 0,
            warnings: vec![],
            next_cache_entries: BTreeMap::new(),
            unresolved_internal_by_prefix: vec![CountEntry {
                key: "@/missing".to_string(),
                count: 2,
            }],
            prefix_examples: BTreeMap::from([(
                "@/missing".to_string(),
                "@/missing/foo".to_string(),
            )]),
            unresolved_internal_specifiers: vec!["@/missing/foo".to_string()],
            unresolved_internal_specifier_records: vec![json!({
                "specifier": "@/missing/foo",
                "consumerFile": "src/b.ts",
                "kind": "import",
                "typeOnly": false,
                "reason": "alias-miss",
            })],
            language_support: json!({"ts": {"enabled": true}}),
            total_uses: 1,
            unresolved_uses: 1,
            resolved_internal_uses: 1,
            resolved_generated_virtual_uses: 0,
            non_source_asset_uses: 0,
            external_uses: 0,
            dependency_import_consumers: vec![
                json!({"depRoot": "react", "fromSpec": "react", "file": "src/a.ts", "kind": "import"}),
            ],
            external_dependency_import_inputs: vec![],
            resolved_internal_edges: vec![
                json!({"from": "src/b.ts", "to": "src/a.ts", "kind": "import", "source": "./a", "typeOnly": false}),
            ],
            generated_consumer_blind_zones: vec![],
            generated_consumer_blind_zone_inputs: vec![],
            generated_virtual_surfaces: vec![],
            generated_virtual_import_consumers: vec![],
            unresolved_internal_uses: 1,
            mdx_consumer_uses: 0,
            sfc_script_consumer_uses: 0,
            sfc_script_src_reachability_uses: 0,
            sfc_style_asset_reference_uses: 0,
            sfc_template_component_ref_uses: 0,
            sfc_global_component_registration_uses: 0,
            sfc_generated_component_manifest_uses: 0,
            sfc_framework_convention_component_uses: 0,
            sfc_style_asset_references: vec![],
            sfc_style_asset_reference_inputs: vec![
                SfcStyleAssetReferenceInput {
                    consumer_file: "C:/repo/src/App.vue".to_string(),
                    from_spec: "./App.css?inline".to_string(),
                    source: Some("sfc-style".to_string()),
                    kind: Some("sfc-style-asset-reference".to_string()),
                    style_kind: Some("css".to_string()),
                    confidence: Some("path-evidence".to_string()),
                    resolved_file: Some("C:/repo/src/App.css".to_string()),
                    import_syntax: Some("style-src".to_string()),
                    line: Some(7),
                    sfc_block_kind: Some("style".to_string()),
                    sfc_language: Some("vue".to_string()),
                },
                SfcStyleAssetReferenceInput {
                    consumer_file: "C:/repo/src/App.vue".to_string(),
                    from_spec: "./missing.css".to_string(),
                    source: Some("sfc-style".to_string()),
                    kind: Some("sfc-style-asset-reference".to_string()),
                    style_kind: Some("css".to_string()),
                    confidence: Some("path-evidence".to_string()),
                    resolved_file: None,
                    import_syntax: None,
                    line: Some(9),
                    sfc_block_kind: Some("style".to_string()),
                    sfc_language: Some("vue".to_string()),
                },
            ],
            sfc_template_component_refs: vec![],
            sfc_template_component_ref_inputs: vec![
                SfcTemplateComponentRefInput {
                    consumer_file: "C:/repo/src/App.vue".to_string(),
                    tag_name: Some("UiButton".to_string()),
                    normalized_tag_name: Some("ui-button".to_string()),
                    binding_name: Some("UiButton".to_string()),
                    binding_source: Some("./UiButton.vue".to_string()),
                    source: Some("sfc-template".to_string()),
                    language: Some("vue".to_string()),
                    template_kind: Some("template".to_string()),
                    confidence: Some("component-binding".to_string()),
                    status: Some("resolved".to_string()),
                    resolved_file: Some("C:/repo/src/UiButton.vue".to_string()),
                    reason: None,
                    source_use_record_id: None,
                    binding_kind: Some("import".to_string()),
                    imported_name: Some("default".to_string()),
                    member_name: None,
                    line: Some(12),
                    sfc_block_kind: Some("template".to_string()),
                },
                SfcTemplateComponentRefInput {
                    consumer_file: "C:/repo/src/App.vue".to_string(),
                    tag_name: Some("ExternalWidget".to_string()),
                    normalized_tag_name: Some("external-widget".to_string()),
                    binding_name: Some("ExternalWidget".to_string()),
                    binding_source: Some("@pkg/widgets".to_string()),
                    source: Some("sfc-template".to_string()),
                    language: Some("vue".to_string()),
                    template_kind: Some("template".to_string()),
                    confidence: Some("component-binding".to_string()),
                    status: Some("external".to_string()),
                    resolved_file: None,
                    reason: Some("sfc-template-component-external-binding".to_string()),
                    source_use_record_id: None,
                    binding_kind: Some("import".to_string()),
                    imported_name: Some("ExternalWidget".to_string()),
                    member_name: None,
                    line: Some(16),
                    sfc_block_kind: Some("template".to_string()),
                },
            ],
            sfc_global_component_registrations: vec![],
            sfc_global_component_registration_inputs: vec![
                SfcGlobalComponentRegistrationInput {
                    registration_file: "C:/repo/src/main.ts".to_string(),
                    framework: Some("vue".to_string()),
                    api: Some("app.component".to_string()),
                    component_name: Some("RegisteredSource".to_string()),
                    normalized_tag_names: Some(vec!["registered-source".to_string()]),
                    binding_name: Some("RegisteredSource".to_string()),
                    binding_source: Some("./registered-source".to_string()),
                    from_spec: None,
                    source: Some("sfc-global-component-registration".to_string()),
                    status: Some("resolved".to_string()),
                    resolved_file: Some("C:/repo/src/registered-source.ts".to_string()),
                    reason: None,
                    source_use_record_id: None,
                    binding_kind: Some("import".to_string()),
                    imported_name: Some("default".to_string()),
                    factory_kind: None,
                    ambiguity_key: None,
                    line: Some(20),
                },
                SfcGlobalComponentRegistrationInput {
                    registration_file: "C:/repo/src/main.ts".to_string(),
                    framework: Some("vue".to_string()),
                    api: Some("app.component".to_string()),
                    component_name: Some("AsyncGlobal".to_string()),
                    normalized_tag_names: Some(vec!["async-global".to_string()]),
                    binding_name: None,
                    binding_source: None,
                    from_spec: Some("./AsyncGlobal.vue".to_string()),
                    source: Some("sfc-global-component-registration".to_string()),
                    status: Some("muted".to_string()),
                    resolved_file: Some("C:/repo/src/AsyncGlobal.vue".to_string()),
                    reason: Some("sfc-global-component-async-factory".to_string()),
                    source_use_record_id: None,
                    binding_kind: None,
                    imported_name: None,
                    factory_kind: Some("defineAsyncComponent".to_string()),
                    ambiguity_key: None,
                    line: Some(24),
                },
            ],
            sfc_generated_component_manifests: vec![],
            sfc_generated_component_manifest_inputs: vec![
                SfcGeneratedComponentManifestInput {
                    manifest_file: "C:/repo/components.d.ts".to_string(),
                    manifest_kind: Some("unplugin-vue-components-dts".to_string()),
                    component_name: Some("ManifestSource".to_string()),
                    normalized_tag_names: vec!["manifest-source".to_string()],
                    binding_source: Some("./src/ManifestSource.ts".to_string()),
                    from_spec: Some("./src/ManifestSource.ts".to_string()),
                    computed_key_source: None,
                    source: Some("sfc-framework-generated-manifest".to_string()),
                    confidence: Some("generated-manifest-availability".to_string()),
                    status: Some("resolved".to_string()),
                    resolved_file: Some("C:/repo/src/ManifestSource.ts".to_string()),
                    reason: None,
                    source_use_record_id: None,
                    line: Some(30),
                },
                SfcGeneratedComponentManifestInput {
                    manifest_file: "C:/repo/components.d.ts".to_string(),
                    manifest_kind: Some("unplugin-vue-components-dts".to_string()),
                    component_name: Some("ManifestButton".to_string()),
                    normalized_tag_names: vec!["manifest-button".to_string()],
                    binding_source: Some("./components/ManifestButton.vue".to_string()),
                    from_spec: Some("./components/ManifestButton.vue".to_string()),
                    computed_key_source: None,
                    source: Some("sfc-framework-generated-manifest".to_string()),
                    confidence: Some("generated-manifest-availability".to_string()),
                    status: Some("muted".to_string()),
                    resolved_file: Some("C:/repo/components/ManifestButton.vue".to_string()),
                    reason: Some("sfc-framework-generated-manifest-non-source-binding".to_string()),
                    source_use_record_id: None,
                    line: Some(31),
                },
                SfcGeneratedComponentManifestInput {
                    manifest_file: "C:/repo/components.d.ts".to_string(),
                    manifest_kind: Some("unplugin-vue-components-dts".to_string()),
                    component_name: Some("DynamicManifest".to_string()),
                    normalized_tag_names: vec!["dynamic-manifest".to_string()],
                    binding_source: Some("./components/DynamicManifest.vue".to_string()),
                    from_spec: Some("./components/DynamicManifest.vue".to_string()),
                    computed_key_source: Some("prefix + 'Manifest'".to_string()),
                    source: Some("sfc-framework-generated-manifest".to_string()),
                    confidence: Some("generated-manifest-availability".to_string()),
                    status: Some("skipped".to_string()),
                    resolved_file: None,
                    reason: Some("sfc-framework-generated-manifest-nonliteral".to_string()),
                    source_use_record_id: None,
                    line: Some(32),
                },
            ],
            sfc_framework_convention_components: vec![],
            sfc_framework_convention_component_inputs: vec![
                SfcFrameworkConventionComponentInput {
                    framework: Some("nuxt".to_string()),
                    convention_kind: Some("components-dir".to_string()),
                    consumer_file: Some("C:/repo/pages/index.vue".to_string()),
                    component_name: Some("ConventionCard".to_string()),
                    normalized_tag_names: Some(vec![
                        "convention-card".to_string(),
                        "ConventionCard".to_string(),
                    ]),
                    source_file: Some("C:/repo/components/ConventionCard.vue".to_string()),
                    component_dir: Some("components".to_string()),
                    resolved_dir: Some("C:/repo/components".to_string()),
                    path_prefix: Some(json!(true)),
                    global: Some(true),
                    resolved_file: Some("C:/repo/components/ConventionCard.vue".to_string()),
                    binding_source: Some("C:/repo/components/ConventionCard.vue".to_string()),
                    from_spec: Some("./ignored-by-binding-source.vue".to_string()),
                    source: Some("sfc-framework-convention-component".to_string()),
                    confidence: Some("framework-convention".to_string()),
                    status: Some("resolved".to_string()),
                    binding_kind: Some("filesystem".to_string()),
                    component_path_segments: Some(vec![
                        "components".to_string(),
                        "ConventionCard.vue".to_string(),
                    ]),
                    line: Some(40),
                    ..Default::default()
                },
                SfcFrameworkConventionComponentInput {
                    framework: Some("svelte".to_string()),
                    convention_kind: Some("store-subscription".to_string()),
                    consumer_file: Some("C:/repo/src/Counter.svelte".to_string()),
                    subscription_name: Some("$count".to_string()),
                    store_name: Some("count".to_string()),
                    binding_source: Some("./stores/count".to_string()),
                    source: Some("sfc-framework-convention-component".to_string()),
                    confidence: Some("framework-convention".to_string()),
                    reason: Some("sfc-framework-svelte-store-subscription".to_string()),
                    sfc_block_kind: Some("template".to_string()),
                    line: Some(42),
                    ..Default::default()
                },
            ],
            dead: vec![json!({"file": "src/a.ts", "symbol": "alpha", "line": 1})],
            truly_dead: vec![json!({"file": "src/a.ts", "symbol": "alpha", "line": 1})],
            dead_in_prod: vec![json!({"file": "src/a.ts", "symbol": "alpha", "line": 1})],
            dead_in_test: vec![],
            symbol_fan_in: vec![
                json!({"defFile": "src/a.ts", "symbol": "alpha", "count": 0, "kind": "FunctionDeclaration"}),
            ],
            fan_in_by_identity: json!({"src/a.ts::alpha": 0}),
            fan_in_by_identity_space: json!({"src/a.ts::alpha": {"value": 0, "type": 0, "broad": 0}}),
            fan_in_inputs: None,
            dead_candidate_inputs: None,
            source_use_assembly: None,
            namespace_re_export_diagnostics: vec![],
            any_contamination_facts: json!({}),
            incremental: None,
        })?;

        assert_eq!(artifact["meta"]["tool"], TOOL_NAME);
        assert_eq!(artifact["meta"]["schemaVersion"], 3);
        assert_eq!(artifact["files"], 2);
        assert_eq!(artifact["totalDefs"], 1);
        assert_eq!(
            artifact["defIndex"]["src/a.ts"]["alpha"]["anyContamination"]["label"],
            "any-contaminated"
        );
        assert_eq!(
            artifact["helperOwnersByIdentity"]["src/a.ts::alpha"]["anyContamination"]
                ["measurements"]["explicitAnyCount"],
            1
        );
        assert_eq!(artifact["uses"]["unresolvedInternalRatio"], 0.5);
        assert_eq!(artifact["uses"]["sfcStyleAssetReferences"], 1);
        assert_eq!(
            artifact["sfcStyleAssetReferences"][0]["resolvedFile"],
            "src/App.css"
        );
        assert_eq!(
            artifact["sfcStyleAssetReferences"][1]["reason"],
            "sfc-style-asset-unresolved"
        );
        assert_eq!(artifact["uses"]["sfcTemplateComponentRefs"], 2);
        let template_refs = artifact["sfcTemplateComponentRefs"]
            .as_array()
            .context("template refs array")?;
        assert!(template_refs.iter().any(|item| {
            item["bindingName"] == "UiButton" && item["resolvedFile"] == "src/UiButton.vue"
        }));
        assert!(template_refs.iter().any(|item| {
            item["bindingName"] == "ExternalWidget"
                && item["reason"] == "sfc-template-component-external-binding"
        }));
        assert_eq!(artifact["uses"]["sfcGlobalComponentRegistrations"], 2);
        let global_registrations = artifact["sfcGlobalComponentRegistrations"]
            .as_array()
            .context("global component registrations array")?;
        assert!(global_registrations.iter().any(|item| {
            item["componentName"] == "RegisteredSource"
                && item["resolvedFile"] == "src/registered-source.ts"
                && item["fromSpec"] == "./registered-source"
                && item["confidence"] == "registration-review"
        }));
        assert!(global_registrations.iter().any(|item| {
            item["componentName"] == "AsyncGlobal"
                && item["resolvedFile"] == "src/AsyncGlobal.vue"
                && item["fromSpec"] == "./AsyncGlobal.vue"
                && item["confidence"] == "muted-review"
                && item["reason"] == "sfc-global-component-async-factory"
        }));
        assert_eq!(artifact["uses"]["sfcGeneratedComponentManifests"], 3);
        let generated_manifests = artifact["sfcGeneratedComponentManifests"]
            .as_array()
            .context("generated component manifests array")?;
        assert!(generated_manifests.iter().any(|item| {
            item["componentName"] == "ManifestSource"
                && item["resolvedFile"] == "src/ManifestSource.ts"
                && item["status"] == "resolved"
        }));
        assert!(generated_manifests.iter().any(|item| {
            item["componentName"] == "ManifestButton"
                && item["resolvedFile"] == "components/ManifestButton.vue"
                && item["reason"] == "sfc-framework-generated-manifest-non-source-binding"
        }));
        assert!(generated_manifests.iter().any(|item| {
            item["componentName"] == "DynamicManifest"
                && item["computedKeySource"] == "prefix + 'Manifest'"
                && item["status"] == "skipped"
        }));
        assert_eq!(artifact["uses"]["sfcFrameworkConventionComponents"], 2);
        let convention_components = artifact["sfcFrameworkConventionComponents"]
            .as_array()
            .context("framework convention components array")?;
        assert!(convention_components.iter().any(|item| {
            item["componentName"] == "ConventionCard"
                && item["consumerFile"] == "pages/index.vue"
                && item["sourceFile"] == "components/ConventionCard.vue"
                && item["resolvedDir"] == "components"
                && item["bindingSource"] == "components/ConventionCard.vue"
                && item["fromSpec"] == "./ignored-by-binding-source.vue"
                && item["pathPrefix"] == true
                && item["global"] == true
                && item["eligibleForFanIn"] == false
                && item["eligibleForSafeFix"] == false
        }));
        assert!(convention_components.iter().any(|item| {
            item["subscriptionName"] == "$count"
                && item["storeName"] == "count"
                && item["bindingSource"] == "./stores/count"
                && item["fromSpec"] == "./stores/count"
                && item["status"] == "muted"
                && item["reason"] == "sfc-framework-svelte-store-subscription"
        }));
        assert_eq!(artifact["defIndex"]["src/a.ts"]["alpha"]["name"], "alpha");
        assert_eq!(artifact["deadProdList"][0]["symbol"], "alpha");
        assert_eq!(
            artifact["preWriteLocalOperationIndex"]["byOwnerFile"]["src/a.ts"][0]["domainTokens"]
                [0],
            "a"
        );
        Ok(())
    }

    #[test]
    fn sfc_style_asset_resolution_requires_file_targets() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir)?;
        let consumer = src_dir.join("App.vue");
        fs::write(&consumer, "")?;
        fs::create_dir(src_dir.join("App.css"))?;

        let consumer_text = path_to_string(consumer);
        assert_eq!(
            resolve_sfc_style_asset_target(&consumer_text, "./App.css"),
            None
        );

        fs::remove_dir(src_dir.join("App.css"))?;
        fs::write(src_dir.join("App.css"), "")?;
        assert_eq!(
            resolve_sfc_style_asset_target(&consumer_text, "./App.css")
                .as_deref()
                .and_then(|path| path.rsplit('/').next()),
            Some("App.css")
        );
        Ok(())
    }

    #[test]
    fn preserves_external_generated_manifest_evidence_count() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": ["C:/repo/components.d.ts"],
            "sfcGeneratedComponentManifestUses": 1,
            "sfcGeneratedComponentManifestInputs": [{
                "manifestFile": "C:/repo/components.d.ts",
                "manifestKind": "unplugin-vue-components-dts",
                "componentName": "LocalComponent",
                "normalizedTagNames": ["local-component"],
                "bindingSource": "./src/LocalComponent.ts",
                "fromSpec": "./src/LocalComponent.ts",
                "source": "sfc-framework-generated-manifest",
                "confidence": "generated-manifest-availability",
                "status": "resolved",
                "resolvedFile": "C:/repo/src/LocalComponent.ts"
            }]
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["uses"]["sfcGeneratedComponentManifests"], 2);
        assert_eq!(
            artifact["sfcGeneratedComponentManifests"]
                .as_array()
                .context("generated manifest array")?
                .len(),
            1
        );
        Ok(())
    }

    #[test]
    fn preserves_external_generated_manifest_source_use_count_without_rows() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": ["C:/repo/components.d.ts"],
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "records": [{
                    "recordId": "generated-manifest-external#0",
                    "consumerFile": "C:/repo/components.d.ts",
                    "fromSpec": "@scope/external-components",
                    "kind": "sfc-generated-component-manifest",
                    "name": "*",
                    "consumerSource": "sfc-generated-component-manifest",
                    "resolverStage": "external"
                }]
            },
            "sfcGeneratedComponentManifestInputs": [{
                "manifestFile": "C:/repo/components.d.ts",
                "manifestKind": "unplugin-vue-components-dts",
                "componentName": "ExternalComponent",
                "normalizedTagNames": ["external-component"],
                "bindingSource": "@scope/external-components",
                "fromSpec": "@scope/external-components",
                "source": "sfc-framework-generated-manifest",
                "confidence": "generated-manifest-availability",
                "sourceUseRecordId": "generated-manifest-external#0"
            }]
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["uses"]["sfcGeneratedComponentManifests"], 1);
        assert_eq!(
            artifact["sfcGeneratedComponentManifests"]
                .as_array()
                .context("generated manifest array")?
                .len(),
            0
        );
        Ok(())
    }

    #[test]
    fn applies_embedded_source_use_targets_to_sfc_component_projection_only() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": [
                "C:/repo/src/App.vue",
                "C:/repo/src/Button.ts",
                "C:/repo/src/Registered.ts",
                "C:/repo/src/ManifestSource.ts",
                "C:/repo/components/ManifestButton.vue"
            ],
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "sourceFiles": [
                    "src/Button.ts",
                    "src/Registered.ts",
                    "src/ManifestSource.ts"
                ],
                "records": [
                    {
                        "consumerFile": "C:/repo/src/App.vue",
                        "fromSpec": "./Button",
                        "kind": "sfc-template-component-ref",
                        "name": "*",
                        "consumerSource": "sfc-template-component-ref",
                        "resolverStage": "relative"
                    },
                    {
                        "consumerFile": "C:/repo/src/App.vue",
                        "fromSpec": "./Registered",
                        "kind": "sfc-global-component-registration",
                        "name": "*",
                        "consumerSource": "sfc-global-component-registration",
                        "resolverStage": "relative"
                    },
                    {
                        "consumerFile": "C:/repo/components.d.ts",
                        "fromSpec": "./src/ManifestSource",
                        "kind": "sfc-generated-component-manifest",
                        "name": "*",
                        "consumerSource": "sfc-generated-component-manifest",
                        "resolverStage": "relative"
                    },
                    {
                        "consumerFile": "C:/repo/components.d.ts",
                        "fromSpec": "./components/ManifestButton.vue",
                        "resolvedFile": "C:/repo/components/ManifestButton.vue",
                        "kind": "sfc-generated-component-manifest",
                        "name": "*",
                        "consumerSource": "sfc-generated-component-manifest",
                        "resolverStage": "resolved-internal"
                    }
                ]
            },
            "sfcTemplateComponentRefInputs": [{
                "consumerFile": "C:/repo/src/App.vue",
                "tagName": "UiButton",
                "bindingName": "UiButton",
                "bindingSource": "./Button",
                "source": "sfc-template",
                "sourceUseRecordId": "r0"
            }],
            "sfcGlobalComponentRegistrationInputs": [{
                "registrationFile": "C:/repo/src/App.vue",
                "componentName": "RegisteredSource",
                "bindingName": "RegisteredSource",
                "bindingSource": "./Registered",
                "source": "sfc-global-component-registration",
                "sourceUseRecordId": "r1"
            }],
            "sfcGeneratedComponentManifestInputs": [
                {
                    "manifestFile": "C:/repo/components.d.ts",
                    "componentName": "ManifestSource",
                    "normalizedTagNames": ["manifest-source"],
                    "bindingSource": "./src/ManifestSource",
                    "fromSpec": "./src/ManifestSource",
                    "source": "sfc-framework-generated-manifest",
                    "sourceUseRecordId": "r2"
                },
                {
                    "manifestFile": "C:/repo/components.d.ts",
                    "componentName": "ManifestButton",
                    "normalizedTagNames": ["manifest-button"],
                    "bindingSource": "./components/ManifestButton.vue",
                    "fromSpec": "./components/ManifestButton.vue",
                    "source": "sfc-framework-generated-manifest",
                    "sourceUseRecordId": "r3"
                }
            ]
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["uses"]["resolvedInternal"], 0);
        assert_eq!(
            artifact["resolvedInternalEdges"]
                .as_array()
                .context("resolved internal edges")?
                .len(),
            0
        );
        assert_eq!(artifact["uses"]["sfcTemplateComponentRefs"], 1);
        assert_eq!(
            artifact["sfcTemplateComponentRefs"][0]["resolvedFile"],
            "src/Button.ts"
        );
        assert_eq!(
            artifact["sfcTemplateComponentRefs"][0]["status"],
            "resolved"
        );
        assert_eq!(artifact["uses"]["sfcGlobalComponentRegistrations"], 1);
        assert_eq!(
            artifact["sfcGlobalComponentRegistrations"][0]["resolvedFile"],
            "src/Registered.ts"
        );
        let manifests = artifact["sfcGeneratedComponentManifests"]
            .as_array()
            .context("generated manifests")?;
        assert!(manifests.iter().any(|item| {
            item["componentName"] == "ManifestSource"
                && item["resolvedFile"] == "src/ManifestSource.ts"
                && item["status"] == "resolved"
        }));
        assert!(manifests.iter().any(|item| {
            item["componentName"] == "ManifestButton"
                && item["resolvedFile"] == "components/ManifestButton.vue"
                && item["status"] == "muted"
                && item["reason"] == "sfc-framework-generated-manifest-non-source-binding"
        }));
        Ok(())
    }

    #[test]
    fn builds_generated_consumer_blind_zones_from_embedded_unresolved_records() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "generatedArtifactsMode": "prepared",
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "records": [{
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "@scope/generated-client",
                    "kind": "import",
                    "resolverStage": "unresolved-internal",
                    "unresolvedEvidence": {
                        "reason": "workspace-generated-artifact-missing",
                        "resolverStage": "tsconfig-paths",
                        "targetCandidates": ["packages/api/generated/client.ts"],
                        "generatedArtifact": {
                            "matchedPackage": "@scope/api",
                            "targetSubpath": "packages/api/generated/client.ts",
                            "generatorFamily": "path-segment",
                            "confidence": "supporting",
                            "packageRoot": "packages/api"
                        }
                    }
                }]
            }
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;
        let zones = artifact["generatedConsumerBlindZones"]
            .as_array()
            .context("generated consumer blind zones")?;

        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0]["reason"], "generated-consumer-blind-zone");
        assert_eq!(
            zones[0]["sourceReason"],
            "workspace-generated-artifact-missing"
        );
        assert_eq!(zones[0]["specifier"], "@scope/generated-client");
        assert_eq!(zones[0]["consumerFile"], "src/consumer.ts");
        assert_eq!(
            zones[0]["candidatePath"],
            "packages/api/generated/client.ts"
        );
        assert_eq!(zones[0]["scopePackageRoot"], "packages/api");
        assert_eq!(zones[0]["mode"], "prepared");
        assert_eq!(zones[0]["staleStatus"], "unknown");
        assert_eq!(
            artifact["artifactSummary"]["generatedConsumerBlindZoneCount"],
            1
        );
        Ok(())
    }

    #[test]
    fn builds_generated_consumer_blind_zones_from_extra_inputs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = path_to_string(temp.path().to_path_buf());
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": root,
            "generatedConsumerBlindZoneInputs": [{
                "specifier": "@scope/generated-client",
                "consumerFile": "src/component.ts",
                "reason": "workspace-generated-artifact-missing",
                "targetCandidates": ["packages/api/generated/client.ts"],
                "generatedArtifact": {
                    "matchedPackage": "@scope/api",
                    "targetSubpath": "packages/api/generated/client.ts",
                    "generatorFamily": "path-segment",
                    "confidence": "supporting",
                    "packageRoot": "packages/api"
                }
            }]
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(
            artifact["generatedConsumerBlindZones"][0]["consumerFile"],
            "src/component.ts"
        );
        assert_eq!(
            artifact["artifactSummary"]["generatedConsumerBlindZoneCount"],
            1
        );
        Ok(())
    }

    #[test]
    fn projects_external_dependency_import_inputs() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": ["C:/repo/src/a.ts"],
            "externalDependencyImportInputs": [{
                "consumerFile": "C:/repo/src/a.ts",
                "fromSpec": "@scope/pkg/subpath",
                "kind": "import",
                "source": "source-import",
                "typeOnly": true
            }, {
                "consumerFile": "C:/repo/src/a.ts",
                "fromSpec": "ignored",
                "kind": "imported-namespace-escape",
                "source": "source-import"
            }]
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["uses"]["external"], 1);
        assert_eq!(artifact["unresolvedUses"], 1);
        let consumers = artifact["dependencyImportConsumers"]
            .as_array()
            .context("dependency import consumers")?;
        assert_eq!(consumers.len(), 1);
        assert_eq!(consumers[0]["file"], "src/a.ts");
        assert_eq!(consumers[0]["fromSpec"], "@scope/pkg/subpath");
        assert_eq!(consumers[0]["depRoot"], "@scope/pkg");
        assert_eq!(consumers[0]["typeOnly"], true);
        Ok(())
    }

    #[test]
    fn parse_errors_are_visible() -> Result<()> {
        let artifact = build_symbol_graph_artifact(SymbolGraphRequest {
            schema_version: SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION.to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            generated_artifacts_mode: "default".to_string(),
            path_table: vec![],
            files: vec!["C:/repo/src/bad.ts".to_string()],
            file_ids: vec![],
            def_index: vec![],
            file_data: vec![],
            parse_errors: 1,
            warnings: vec![],
            next_cache_entries: BTreeMap::from([(
                "C:/repo/src/bad.ts".to_string(),
                json!({"parseError": true}),
            )]),
            unresolved_internal_by_prefix: vec![],
            prefix_examples: BTreeMap::new(),
            unresolved_internal_specifiers: vec![],
            unresolved_internal_specifier_records: vec![],
            language_support: json!({}),
            total_uses: 0,
            unresolved_uses: 0,
            resolved_internal_uses: 0,
            resolved_generated_virtual_uses: 0,
            non_source_asset_uses: 0,
            external_uses: 0,
            dependency_import_consumers: vec![],
            external_dependency_import_inputs: vec![],
            resolved_internal_edges: vec![],
            generated_consumer_blind_zones: vec![],
            generated_consumer_blind_zone_inputs: vec![],
            generated_virtual_surfaces: vec![],
            generated_virtual_import_consumers: vec![],
            unresolved_internal_uses: 0,
            mdx_consumer_uses: 0,
            sfc_script_consumer_uses: 0,
            sfc_script_src_reachability_uses: 0,
            sfc_style_asset_reference_uses: 0,
            sfc_template_component_ref_uses: 0,
            sfc_global_component_registration_uses: 0,
            sfc_generated_component_manifest_uses: 0,
            sfc_framework_convention_component_uses: 0,
            sfc_style_asset_references: vec![],
            sfc_style_asset_reference_inputs: vec![],
            sfc_template_component_refs: vec![],
            sfc_template_component_ref_inputs: vec![],
            sfc_global_component_registrations: vec![],
            sfc_global_component_registration_inputs: vec![],
            sfc_generated_component_manifests: vec![],
            sfc_generated_component_manifest_inputs: vec![],
            sfc_framework_convention_components: vec![],
            sfc_framework_convention_component_inputs: vec![],
            dead: vec![],
            truly_dead: vec![],
            dead_in_prod: vec![],
            dead_in_test: vec![],
            symbol_fan_in: vec![],
            fan_in_by_identity: json!({}),
            fan_in_by_identity_space: json!({}),
            fan_in_inputs: None,
            dead_candidate_inputs: None,
            source_use_assembly: None,
            namespace_re_export_diagnostics: vec![],
            any_contamination_facts: json!({}),
            incremental: None,
        })?;

        assert_eq!(artifact["meta"]["warnings"][0]["code"], "parse-errors");
        assert_eq!(artifact["filesWithParseErrors"][0], "src/bad.ts");
        Ok(())
    }

    #[test]
    fn computes_fan_in_from_typed_inputs() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": ["C:/repo/src/a.ts", "C:/repo/src/b.ts", "C:/repo/src/c.ts"],
            "defIndex": [{
                "filePath": "C:/repo/src/a.ts",
                "definitions": {
                    "alpha": {"name": "alpha", "kind": "FunctionDeclaration", "line": 1},
                    "Beta": {"name": "Beta", "kind": "ClassDeclaration", "line": 2}
                }
            }],
            "fanInInputs": {
                "consumerEntries": [
                    {
                        "defFile": "C:/repo/src/a.ts",
                        "symbol": "alpha",
                        "consumerFile": "C:/repo/src/b.ts",
                        "space": "value"
                    },
                    {
                        "defFile": "C:/repo/src/a.ts",
                        "symbol": "alpha",
                        "consumerFile": "C:/repo/src/c.ts",
                        "space": "type"
                    },
                    {
                        "defFile": "C:/repo/src/a.ts",
                        "symbol": "alpha",
                        "consumerFile": "C:/repo/src/c.ts",
                        "space": "value"
                    }
                ],
                "namespaceUserEntries": [{
                    "defFile": "C:/repo/src/a.ts",
                    "consumerFile": "C:/repo/src/b.ts"
                }]
            }
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["topSymbolFanIn"][0]["defFile"], "src/a.ts");
        assert_eq!(artifact["topSymbolFanIn"][0]["symbol"], "alpha");
        assert_eq!(artifact["topSymbolFanIn"][0]["count"], 2);
        assert_eq!(artifact["fanInByIdentity"]["src/a.ts::alpha"], 2);
        assert_eq!(artifact["fanInByIdentity"]["src/a.ts::Beta"], 0);
        assert_eq!(
            artifact["fanInByIdentitySpace"]["src/a.ts::alpha"],
            json!({"value": 2, "type": 1, "broad": 1})
        );
        assert_eq!(
            artifact["fanInByIdentitySpace"]["src/a.ts::Beta"],
            json!({"value": 0, "type": 0, "broad": 1})
        );
        Ok(())
    }

    #[test]
    fn computes_dead_candidates_from_typed_inputs() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": [
                "C:/repo/src/a.ts",
                "C:/repo/src/barrel.ts",
                "C:/repo/tests/helper.ts",
                "C:/repo/pkg/mod.py"
            ],
            "defIndex": [
                {
                    "filePath": "C:/repo/src/a.ts",
                    "definitions": {
                        "used": {"name": "used", "kind": "FunctionDeclaration", "line": 1},
                        "unused": {"name": "unused", "kind": "FunctionDeclaration", "line": 2},
                        "framework": {"name": "framework", "kind": "FunctionDeclaration", "line": 3, "frameworkRegistered": true},
                        "shadowOnly": {"name": "shadowOnly", "kind": "FunctionDeclaration", "line": 4}
                    }
                },
                {
                    "filePath": "C:/repo/src/barrel.ts",
                    "definitions": {
                        "barrelExport": {"name": "barrelExport", "kind": "FunctionDeclaration", "line": 1}
                    }
                },
                {
                    "filePath": "C:/repo/tests/helper.ts",
                    "definitions": {
                        "testOnly": {"name": "testOnly", "kind": "FunctionDeclaration", "line": 1}
                    }
                },
                {
                    "filePath": "C:/repo/pkg/mod.py",
                    "definitions": {
                        "public_py": {"name": "public_py", "kind": "FunctionDef", "line": 1},
                        "private_py": {"name": "private_py", "kind": "FunctionDef", "line": 2}
                    }
                }
            ],
            "fileData": [{
                "filePath": "C:/repo/pkg/mod.py",
                "pyDunderAll": ["public_py"]
            }],
            "fanInInputs": {
                "consumerEntries": [{
                    "defFile": "C:/repo/src/a.ts",
                    "symbol": "used",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "space": "value"
                }],
                "namespaceUserEntries": [{
                    "defFile": "C:/repo/src/a.ts",
                    "consumerFile": "C:/repo/src/ns.ts"
                }]
            },
            "deadCandidateInputs": {
                "barrelFiles": ["C:/repo/src/barrel.ts"],
                "testLikeFiles": ["tests/helper.ts"]
            }
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["deadTotal"], 4);
        assert_eq!(artifact["trulyDead"], 2);
        assert_eq!(artifact["deadInProd"], 1);
        assert_eq!(artifact["deadInTest"], 1);
        assert_eq!(artifact["deadProdList"][0]["symbol"], "public_py");
        assert_eq!(artifact["deadProdList"][0]["file"], "pkg/mod.py");
        Ok(())
    }

    #[test]
    fn applies_embedded_source_use_assembly_before_fan_in_and_dead_candidates() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "files": ["C:/repo/src/a.ts", "C:/repo/src/consumer.ts"],
            "defIndex": [{
                "filePath": "C:/repo/src/a.ts",
                "definitions": {
                    "used": {"name": "used", "kind": "FunctionDeclaration", "line": 1},
                    "unused": {"name": "unused", "kind": "FunctionDeclaration", "line": 2}
                }
            }],
            "totalUses": 0,
            "resolvedInternalUses": 0,
            "fanInInputs": {
                "consumerEntries": [],
                "namespaceUserEntries": []
            },
            "deadCandidateInputs": {
                "barrelFiles": [],
                "testLikeFiles": []
            },
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "sourceFiles": [],
                "namespaceReExports": [],
                "namedReExports": [],
                "records": [{
                    "recordId": "src/consumer.ts#0",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "resolvedFile": "C:/repo/src/a.ts",
                    "fromSpec": "@/a",
                    "name": "used",
                    "kind": "import",
                    "typeOnly": false,
                    "resolverStage": "resolved-internal",
                    "consumerSource": "mdx-import"
                }, {
                    "recordId": "src/consumer.ts#1",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "react/jsx-runtime",
                    "kind": "import",
                    "typeOnly": false,
                    "typeOnlyPresent": true,
                    "resolverStage": "external",
                    "consumerSource": "source-import"
                }, {
                    "recordId": "src/consumer.ts#2",
                    "consumerFile": "C:/repo/src/consumer.ts",
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
                }, {
                    "recordId": "src/consumer.ts#3",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "@pkg/db/enums",
                    "name": "Role",
                    "kind": "import",
                    "typeOnly": false,
                    "typeOnlyPresent": true,
                    "resolverStage": "generated-virtual",
                    "generatedVirtualSurface": {
                        "id": "generated-virtual:prisma-enums:@pkg/db:enums",
                        "source": "generated-virtual",
                        "mode": "virtual",
                        "virtual": true,
                        "exports": [{
                            "name": "Role",
                            "kind": "prisma-enum",
                            "spaces": ["value", "type"]
                        }]
                    }
                }, {
                    "recordId": "src/consumer.ts#4",
                    "consumerFile": "C:/repo/src/consumer.ts",
                    "fromSpec": "./style.css",
                    "kind": "import-side-effect",
                    "resolverStage": "non-source-asset"
                }]
            }
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["totalUsesResolved"], 2);
        assert_eq!(artifact["unresolvedUses"], 2);
        assert_eq!(artifact["uses"]["resolvedInternal"], 2);
        assert_eq!(artifact["uses"]["resolvedGeneratedVirtual"], 1);
        assert_eq!(artifact["uses"]["external"], 1);
        assert_eq!(artifact["uses"]["nonSourceAsset"], 1);
        assert_eq!(artifact["uses"]["mdxConsumers"], 1);
        assert_eq!(artifact["uses"]["unresolvedInternal"], 1);
        assert_eq!(artifact["artifactSummary"]["totalUsesResolved"], 2);
        assert_eq!(artifact["artifactSummary"]["unresolvedUses"], 2);
        assert_eq!(artifact["artifactSummary"]["uses"], artifact["uses"]);
        assert_eq!(artifact["artifactSummary"]["resolvedInternalEdgeCount"], 1);
        assert_eq!(artifact["fanInByIdentity"]["src/a.ts::used"], 1);
        assert_eq!(artifact["fanInByIdentity"]["src/a.ts::unused"], 0);
        assert_eq!(artifact["deadTotal"], 1);
        assert_eq!(artifact["artifactSummary"]["deadTotal"], 1);
        assert_eq!(artifact["artifactSummary"]["deadInProd"], 1);
        assert_eq!(artifact["deadProdList"][0]["symbol"], "unused");
        assert_eq!(artifact["resolvedInternalEdges"][0]["to"], "src/a.ts");
        assert_eq!(artifact["dependencyImportConsumers"][0]["depRoot"], "react");
        assert_eq!(
            artifact["dependencyImportConsumers"][0]["fromSpec"],
            "react/jsx-runtime"
        );
        assert_eq!(
            artifact["topUnresolvedSpecifiers"][0]["specifierPrefix"],
            "@/"
        );
        assert_eq!(artifact["unresolvedInternalSpecifiers"][0], "@/missing");
        assert_eq!(
            artifact["unresolvedInternalSpecifierRecords"][0]["reason"],
            "tsconfig-path-target-missing"
        );
        assert_eq!(
            artifact["unresolvedInternalSummaryByReason"]["tsconfig-path-target-missing"]["count"],
            1
        );
        assert_eq!(
            artifact["generatedVirtualSurfaces"][0]["id"],
            "generated-virtual:prisma-enums:@pkg/db:enums"
        );
        assert_eq!(
            artifact["generatedVirtualImportConsumers"][0]["surfaceId"],
            "generated-virtual:prisma-enums:@pkg/db:enums"
        );
        assert_eq!(
            artifact["generatedVirtualImportConsumers"][0]["name"],
            "Role"
        );
        Ok(())
    }

    #[test]
    fn embedded_namespace_reexport_fan_in_matches_compacted_relative_def_paths() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "pathTable": [
                "src/source.ts",
                "src/barrel.ts",
                "src/consumer.ts"
            ],
            "files": [
                "src/source.ts",
                "src/barrel.ts",
                "src/consumer.ts"
            ],
            "defIndex": [{
                "filePathId": 0,
                "definitions": {
                    "escapeFunc": {"name": "escapeFunc", "kind": "FunctionDeclaration", "line": 1},
                    "escapeConst": {"name": "escapeConst", "kind": "VariableDeclaration", "line": 2}
                }
            }],
            "fanInInputs": {
                "consumerEntries": [],
                "namespaceUserEntries": []
            },
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "sourceFileIds": [0, 1, 2],
                "namespaceReExports": [{
                    "barrelFile": "src/barrel.ts",
                    "exportedName": "ns",
                    "targetFile": "src/source.ts",
                    "sourceSpec": "./source"
                }],
                "records": [{
                    "recordId": "src/consumer.ts#0",
                    "consumerFileId": 2,
                    "fromSpec": "./barrel",
                    "name": "ns",
                    "kind": "imported-namespace-escape",
                    "resolverStage": "relative"
                }]
            }
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(
            artifact["fanInByIdentitySpace"]["src/source.ts::escapeFunc"]["broad"],
            1
        );
        assert_eq!(
            artifact["fanInByIdentitySpace"]["src/source.ts::escapeConst"]["broad"],
            1
        );
        assert_eq!(artifact["fanInByIdentity"]["src/source.ts::escapeFunc"], 0);
        assert_eq!(
            artifact["namespaceReExportDiagnostics"][0]["targetFile"],
            "src/source.ts"
        );
        Ok(())
    }

    #[test]
    fn accepts_path_table_compacted_core_file_identities() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "pathTable": ["C:/repo/src/a.ts", "C:/repo/src/b.ts"],
            "fileIds": [0, 1],
            "defIndex": [{
                "filePathId": 0,
                "definitions": {
                    "alpha": {"name": "alpha", "kind": "FunctionDeclaration", "line": 1}
                }
            }],
            "fileData": [{
                "filePathId": 0,
                "reExports": [{"source": "./b", "line": 2}]
            }]
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(artifact["files"], 2);
        assert_eq!(artifact["defIndex"]["src/a.ts"]["alpha"]["name"], "alpha");
        assert_eq!(artifact["reExportsByFile"]["src/a.ts"][0]["source"], "./b");
        Ok(())
    }

    #[test]
    fn embedded_source_use_can_share_symbol_graph_path_table() -> Result<()> {
        let request = serde_json::from_value::<SymbolGraphRequest>(json!({
            "schemaVersion": SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION,
            "generated": "2026-07-08T00:00:00.000Z",
            "root": "C:/repo",
            "pathTable": ["src/consumer.ts", "src/dep.ts"],
            "fileIds": [0, 1],
            "defIndex": [{
                "filePathId": 1,
                "definitions": {
                    "alpha": {"name": "alpha", "kind": "FunctionDeclaration", "line": 1}
                }
            }],
            "sourceUseAssembly": {
                "schemaVersion": "lumin-source-use-assembly-request.v1",
                "root": "C:/repo",
                "records": [{
                    "recordId": "src/consumer.ts#0",
                    "consumerFileId": 0,
                    "resolvedFileId": 1,
                    "fromSpec": "./dep",
                    "name": "alpha",
                    "kind": "import",
                    "resolverStage": "resolved-internal"
                }]
            }
        }))?;

        let artifact = build_symbol_graph_artifact(request)?;

        assert_eq!(
            artifact["resolvedInternalEdges"][0]["from"],
            "src/consumer.ts"
        );
        assert_eq!(artifact["resolvedInternalEdges"][0]["to"], "src/dep.ts");
        assert_eq!(artifact["fanInByIdentity"]["src/dep.ts::alpha"], 1);
        Ok(())
    }

    #[test]
    fn rejects_unknown_schema() {
        let error = match build_symbol_graph_artifact(SymbolGraphRequest {
            schema_version: "future".to_string(),
            generated: "2026-07-05T00:00:00.000Z".to_string(),
            root: "C:/repo".to_string(),
            include_tests: true,
            exclude: vec![],
            generated_artifacts_mode: "default".to_string(),
            path_table: vec![],
            files: vec![],
            file_ids: vec![],
            def_index: vec![],
            file_data: vec![],
            parse_errors: 0,
            warnings: vec![],
            next_cache_entries: BTreeMap::new(),
            unresolved_internal_by_prefix: vec![],
            prefix_examples: BTreeMap::new(),
            unresolved_internal_specifiers: vec![],
            unresolved_internal_specifier_records: vec![],
            language_support: json!({}),
            total_uses: 0,
            unresolved_uses: 0,
            resolved_internal_uses: 0,
            resolved_generated_virtual_uses: 0,
            non_source_asset_uses: 0,
            external_uses: 0,
            dependency_import_consumers: vec![],
            external_dependency_import_inputs: vec![],
            resolved_internal_edges: vec![],
            generated_consumer_blind_zones: vec![],
            generated_consumer_blind_zone_inputs: vec![],
            generated_virtual_surfaces: vec![],
            generated_virtual_import_consumers: vec![],
            unresolved_internal_uses: 0,
            mdx_consumer_uses: 0,
            sfc_script_consumer_uses: 0,
            sfc_script_src_reachability_uses: 0,
            sfc_style_asset_reference_uses: 0,
            sfc_template_component_ref_uses: 0,
            sfc_global_component_registration_uses: 0,
            sfc_generated_component_manifest_uses: 0,
            sfc_framework_convention_component_uses: 0,
            sfc_style_asset_references: vec![],
            sfc_style_asset_reference_inputs: vec![],
            sfc_template_component_refs: vec![],
            sfc_template_component_ref_inputs: vec![],
            sfc_global_component_registrations: vec![],
            sfc_global_component_registration_inputs: vec![],
            sfc_generated_component_manifests: vec![],
            sfc_generated_component_manifest_inputs: vec![],
            sfc_framework_convention_components: vec![],
            sfc_framework_convention_component_inputs: vec![],
            dead: vec![],
            truly_dead: vec![],
            dead_in_prod: vec![],
            dead_in_test: vec![],
            symbol_fan_in: vec![],
            fan_in_by_identity: json!({}),
            fan_in_by_identity_space: json!({}),
            fan_in_inputs: None,
            dead_candidate_inputs: None,
            source_use_assembly: None,
            namespace_re_export_diagnostics: vec![],
            any_contamination_facts: json!({}),
            incremental: None,
        }) {
            Ok(_) => panic!("schema should reject"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("unsupported schemaVersion"));
    }
}
