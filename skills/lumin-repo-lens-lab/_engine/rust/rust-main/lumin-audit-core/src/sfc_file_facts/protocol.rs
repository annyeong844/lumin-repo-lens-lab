use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SfcFileFactsRequest {
    pub schema_version: String,
    pub files: Vec<SfcFileInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SfcFileInput {
    pub file_path: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcFileFactsResponse {
    pub schema_version: &'static str,
    pub files: Vec<SfcFileFacts>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcFileFacts {
    pub file_path: String,
    pub script_import_consumers: Vec<SfcScriptImportConsumer>,
    pub script_sources: Vec<SfcScriptSource>,
    pub style_asset_references: Vec<SfcStyleAssetReference>,
    pub template_component_refs: Vec<SfcTemplateComponentRef>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcScriptImportConsumer {
    pub consumer_file: String,
    pub from_spec: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    pub kind: String,
    pub type_only: bool,
    pub line: usize,
    pub sfc_block_kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcScriptSource {
    pub consumer_file: String,
    pub from_spec: String,
    pub name: &'static str,
    pub kind: &'static str,
    pub type_only: bool,
    pub line: usize,
    pub sfc_block_kind: String,
    pub sfc_language: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcStyleAssetReference {
    pub consumer_file: String,
    pub from_spec: String,
    pub kind: &'static str,
    pub source: &'static str,
    pub style_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_syntax: Option<&'static str>,
    pub confidence: &'static str,
    pub line: usize,
    pub sfc_block_kind: String,
    pub sfc_language: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcTemplateComponentRef {
    pub consumer_file: String,
    pub tag_name: String,
    pub normalized_tag_name: String,
    pub binding_name: String,
    pub binding_source: String,
    pub from_spec: String,
    pub binding_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_name: Option<String>,
    pub source: &'static str,
    pub language: &'static str,
    pub template_kind: &'static str,
    pub confidence: &'static str,
    pub eligible_for_fan_in: bool,
    pub eligible_for_safe_fix: bool,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
    pub line: usize,
    pub sfc_block_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_name: Option<String>,
}
