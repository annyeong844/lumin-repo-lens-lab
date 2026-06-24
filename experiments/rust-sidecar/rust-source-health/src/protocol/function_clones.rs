use serde::Serialize;

use super::AstVisibility;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroups {
    pub policy: AstFunctionCloneGroupsPolicy,
    pub exact_body_groups: Vec<AstFunctionCloneGroup>,
    pub structure_groups: Vec<AstFunctionCloneGroup>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroupsPolicy {
    pub policy_id: &'static str,
    pub policy_version: &'static str,
    pub normalized_version: &'static str,
    pub min_group_size: usize,
    pub exact_min_body_loc: usize,
    pub exact_min_statements: usize,
    pub structure_min_body_loc: usize,
    pub structure_min_statements: usize,
    pub caveat: &'static str,
}

impl Default for AstFunctionCloneGroupsPolicy {
    fn default() -> Self {
        Self {
            policy_id: super::RUST_FUNCTION_CLONE_GROUP_POLICY_ID,
            policy_version: super::RUST_FUNCTION_CLONE_GROUP_POLICY_VERSION,
            normalized_version: super::RUST_FUNCTION_BODY_NORMALIZED_VERSION,
            min_group_size: super::RUST_FUNCTION_CLONE_MIN_GROUP_SIZE,
            exact_min_body_loc: super::RUST_FUNCTION_CLONE_EXACT_MIN_BODY_LOC,
            exact_min_statements: super::RUST_FUNCTION_CLONE_EXACT_MIN_STATEMENTS,
            structure_min_body_loc: super::RUST_FUNCTION_CLONE_STRUCTURE_MIN_BODY_LOC,
            structure_min_statements: super::RUST_FUNCTION_CLONE_STRUCTURE_MIN_STATEMENTS,
            caveat: "Function clone groups are deterministic review evidence. They do not prove semantic equivalence, auto-reuse, or auto-fix safety.",
        }
    }
}

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
pub enum FunctionCloneRisk {
    ReviewOnly,
}
