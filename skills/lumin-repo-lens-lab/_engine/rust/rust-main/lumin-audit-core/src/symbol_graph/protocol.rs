use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::source_use_assembly::SourceUseAssemblyRequest;

pub const SYMBOL_GRAPH_REQUEST_SCHEMA_VERSION: &str = "lumin-symbol-graph-producer-request.v2";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SymbolGraphRequest {
    pub(super) schema_version: String,
    pub(super) context: SymbolGraphContext,
    pub(super) extraction: SymbolGraphExtraction,
    pub(super) source_use_assembly: SourceUseAssemblyRequest,
    pub(super) graph: SymbolGraphInputs,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SymbolGraphContext {
    pub(super) generated: String,
    pub(super) root: String,
    pub(super) include_tests: bool,
    pub(super) exclude: Vec<String>,
    pub(super) generated_artifacts_mode: String,
    pub(super) language_support: Value,
    pub(super) warnings: Vec<Value>,
    pub(super) incremental: Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SymbolGraphExtraction {
    pub(super) path_table: Vec<String>,
    pub(super) file_ids: Vec<usize>,
    pub(super) def_index: Vec<DefinitionFileInput>,
    pub(super) file_data: Vec<FileDataInput>,
    pub(super) parse_error_file_ids: Vec<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DefinitionFileInput {
    pub(super) file_path_id: usize,
    pub(super) definitions: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FileDataInput {
    pub(super) file_path_id: usize,
    pub(super) py_dunder_all: Option<Vec<String>>,
    pub(super) re_exports: Vec<Value>,
    pub(super) class_methods: Vec<Value>,
    pub(super) local_operations: Vec<Value>,
    pub(super) type_escapes: Vec<Value>,
    pub(super) dynamic_import_opacity: Vec<Value>,
    pub(super) cjs_export_surface: Option<Value>,
    pub(super) cjs_require_opacity: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SymbolGraphInputs {
    pub(super) fan_in: FanInInputs,
    pub(super) dead_candidates: DeadCandidateInputs,
    pub(super) sfc: SymbolGraphSfcInputs,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SymbolGraphSfcInputs {
    pub(super) style_asset_references: Vec<SfcStyleAssetReferenceInput>,
    pub(super) template_component_refs: Vec<SfcTemplateComponentRefInput>,
    pub(super) global_component_registrations: Vec<SfcGlobalComponentRegistrationInput>,
    pub(super) generated_component_manifests: Vec<SfcGeneratedComponentManifestInput>,
    pub(super) generated_manifest_external_uses: usize,
    pub(super) framework_convention_components: Vec<SfcFrameworkConventionComponentInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FanInInputs {
    pub(super) consumer_entries: Vec<FanInConsumerEntry>,
    pub(super) namespace_user_entries: Vec<FanInNamespaceUserEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FanInConsumerEntry {
    pub(super) def_file: String,
    pub(super) symbol: String,
    pub(super) consumer_file: String,
    pub(super) space: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct FanInNamespaceUserEntry {
    pub(super) def_file: String,
    pub(super) consumer_file: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct DeadCandidateInputs {
    pub(super) barrel_files: Vec<String>,
    pub(super) test_like_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SfcStyleAssetReferenceInput {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SfcTemplateComponentRefInput {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SfcGlobalComponentRegistrationInput {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SfcGeneratedComponentManifestInput {
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct SfcFrameworkConventionComponentInput {
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
