use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistFactsRequest {
    pub schema_version: String,
    pub generated: String,
    pub root: String,
    #[serde(default)]
    pub files_scanned: usize,
    #[serde(default)]
    pub inputs: ChecklistInputArtifacts,
    pub ast_facts: ChecklistAstFacts,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistInputArtifacts {
    #[serde(default)]
    pub topology: Option<Value>,
    #[serde(default)]
    pub dead_classify: Option<Value>,
    #[serde(default)]
    pub fix_plan: Option<Value>,
    #[serde(default)]
    pub barrels: Option<Value>,
    #[serde(default)]
    pub triage: Option<Value>,
    #[serde(default)]
    pub shape_index: Option<Value>,
    #[serde(default)]
    pub function_clones: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistAstFacts {
    pub function_size: FunctionSizeFacts,
    pub silent_catch: SilentCatchFacts,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSizeFacts {
    #[serde(default)]
    pub entries: Vec<FunctionSizeEntry>,
    #[serde(default)]
    pub parse_errors: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionSizeEntry {
    pub file: String,
    #[serde(default)]
    pub line: usize,
    pub name: String,
    #[serde(default)]
    pub loc: usize,
    #[serde(default)]
    pub file_role: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SilentCatchFacts {
    #[serde(default = "default_silent_catch_analysis")]
    pub analysis: String,
    #[serde(default)]
    pub parse_errors: usize,
    #[serde(default)]
    pub sites: Vec<Value>,
    #[serde(default)]
    pub documented_sites: Vec<Value>,
    #[serde(default)]
    pub anonymous_sites: Vec<Value>,
    #[serde(default)]
    pub non_empty_anonymous_sites: Vec<Value>,
    #[serde(default)]
    pub unused_param_sites: Vec<Value>,
}

fn default_silent_catch_analysis() -> String {
    "oxc-ast-catch-clause".to_string()
}
