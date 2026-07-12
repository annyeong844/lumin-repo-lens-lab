use serde::{Deserialize, Serialize};
use serde_json::Value;
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
    pub source_file_ids: Vec<usize>,
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
    pub name_table: Vec<String>,
    #[serde(default)]
    pub record_row_fields: Vec<String>,
    #[serde(default)]
    pub record_rows: Vec<Vec<Value>>,
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
    #[serde(default)]
    pub record_id: Option<String>,
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
    pub name_id: Option<usize>,
    #[serde(default)]
    pub member_name: Option<String>,
    #[serde(default)]
    pub member_name_id: Option<usize>,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceUseAssemblyResponse {
    pub schema_version: &'static str,
    pub root: String,
    pub summary: SourceUseAssemblySummary,
    pub handled_record_ids: Vec<String>,
    pub resolved_record_targets: Vec<ResolvedRecordTarget>,
    pub external_record_ids: Vec<String>,
    pub non_source_asset_record_ids: Vec<String>,
    pub non_source_asset_record_targets: Vec<ResolvedRecordTarget>,
    pub generated_virtual_record_ids: Vec<String>,
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
