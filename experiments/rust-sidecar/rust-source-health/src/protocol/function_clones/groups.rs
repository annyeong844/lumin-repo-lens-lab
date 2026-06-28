use serde::Serialize;

use crate::protocol::AstVisibility;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroup {
    pub kind: AstFunctionCloneGroupKind,
    pub hash: String,
    pub size: usize,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
    pub exact_hash_count: usize,
    pub identities: Vec<String>,
    pub owner_files: Vec<String>,
    pub names: Vec<String>,
    pub visibilities: Vec<AstVisibility>,
    pub lines: Vec<AstFunctionCloneLine>,
    pub body_loc_range: [usize; 2],
    pub shared_call_tokens: Vec<String>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionSignatureGroup {
    pub kind: AstFunctionSignatureGroupKind,
    pub normalized_version: &'static str,
    pub hash: String,
    pub size: usize,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub identities: Vec<String>,
    pub owner_files: Vec<String>,
    pub names: Vec<String>,
    pub visibilities: Vec<AstVisibility>,
    pub lines: Vec<AstFunctionCloneLine>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidate {
    pub kind: AstNearFunctionCandidateKind,
    pub identities: Vec<String>,
    pub owner_files: Vec<String>,
    pub names: Vec<String>,
    pub lines: Vec<AstFunctionCloneLine>,
    pub score: f64,
    pub risk: FunctionCloneRisk,
    pub generated_only: bool,
    pub shared_call_tokens: Vec<String>,
    pub shared_name_tokens: Vec<String>,
    pub call_token_jaccard: f64,
    pub shared_call_token_idf_sum: f64,
    pub call_token_idf_score: f64,
    pub name_token_jaccard: f64,
    pub body_loc_range: [usize; 2],
    pub statement_count_range: [usize; 2],
    pub reasons: Vec<String>,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneLine {
    pub identity: String,
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionCloneGroupKind {
    ExactFunctionBodyGroup,
    FunctionBodyStructureGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstFunctionSignatureGroupKind {
    FunctionSignatureGroup,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstNearFunctionCandidateKind {
    NearFunctionCandidate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionCloneRisk {
    ReviewOnly,
}
