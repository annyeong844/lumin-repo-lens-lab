use super::{FunctionSignatureFact, InlinePatternOccurrence};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractRequest {
    pub schema_version: String,
    #[serde(default)]
    pub files: Vec<JsTsExtractInputFile>,
    #[serde(default)]
    pub source_files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractInputFile {
    pub file_path: String,
    pub artifact_file_path: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractResponse {
    pub schema_version: &'static str,
    pub files: Vec<JsTsExtractFileResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsTsExtractFileResult {
    pub file_path: String,
    pub defs: Vec<DefinitionRecord>,
    pub uses: Vec<UseRecord>,
    pub re_exports: Vec<ReExportRecord>,
    pub class_methods: Vec<ClassMethodRecord>,
    pub local_operations: Vec<serde_json::Value>,
    pub type_escapes: Vec<TypeEscapeRecord>,
    #[serde(default)]
    pub global_component_registrations: Vec<VueGlobalComponentRegistration>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub function_signature_facts: Vec<FunctionSignatureFact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inline_pattern_occurrences: Vec<InlinePatternOccurrence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inline_pattern_diagnostics: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shape_facts: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shape_diagnostics: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dynamic_import_opacity: Vec<DynamicImportOpacityRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cjs_require_opacity: Vec<CjsRequireOpacityRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cjs_export_surface: Option<CjsExportSurface>,
    pub loc: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DefinitionRecord {
    pub name: String,
    pub kind: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UseRecord {
    pub from_spec: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_name: Option<String>,
    pub kind: String,
    pub type_only: bool,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_name: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub degraded: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver_stage: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicImportOpacityRecord {
    pub line: usize,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CjsRequireOpacityRecord {
    pub line: usize,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CjsExportSurface {
    pub exact: Vec<CjsExportExactRecord>,
    pub opaque: Vec<CjsExportOpaqueRecord>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CjsExportExactRecord {
    pub name: String,
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CjsExportOpaqueRecord {
    pub kind: String,
    pub line: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReExportRecord {
    pub source: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassMethodRecord {
    pub identity: String,
    pub owner_file: String,
    pub class_name: String,
    pub name: String,
    pub method_name: String,
    pub kind: String,
    pub member_kind: String,
    pub visibility: String,
    pub r#static: bool,
    pub computed: bool,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeEscapeRecord {
    pub file: String,
    pub line: usize,
    pub escape_kind: String,
    pub code_shape: String,
    pub normalized_code_shape: String,
    pub inside_exported_identity: Option<String>,
    pub occurrence_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VueGlobalComponentRegistration {
    pub registration_file: String,
    pub framework: String,
    pub api: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub normalized_tag_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_spec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub imported_name: Option<String>,
    pub source: String,
    pub status: String,
    pub confidence: String,
    pub eligible_for_fan_in: bool,
    pub eligible_for_safe_fix: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiguity_key: Option<String>,
    pub line: usize,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(value: &bool) -> bool {
    !*value
}
