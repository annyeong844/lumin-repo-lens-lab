use serde::Serialize;

mod groups;
mod policy;
mod supports;

pub use groups::{
    AstFunctionCloneGroup, AstFunctionCloneGroupKind, AstFunctionCloneLine,
    AstFunctionSignatureGroup, AstFunctionSignatureGroupKind, AstNearFunctionCandidate,
    AstNearFunctionCandidateKind, FunctionCloneRisk,
};
pub use policy::{
    AstFunctionCloneGroupsPolicy, AstNearFunctionCandidatePolicy, AstNearFunctionCandidateWeights,
};
pub use supports::AstFunctionCloneGroupsSupports;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroups {
    pub policy: AstFunctionCloneGroupsPolicy,
    pub supports: AstFunctionCloneGroupsSupports,
    pub complete: bool,
    pub files_with_parse_errors: Vec<AstFunctionCloneInputError>,
    pub files_with_read_errors: Vec<AstFunctionCloneInputError>,
    pub exact_body_group_count: usize,
    pub structure_group_count: usize,
    pub signature_group_count: usize,
    pub near_function_candidate_count: usize,
    pub near_function_candidate_projection_limit: usize,
    pub generated_file_fact_count: usize,
    pub exact_body_groups: Vec<AstFunctionCloneGroup>,
    pub structure_groups: Vec<AstFunctionCloneGroup>,
    pub signature_groups: Vec<AstFunctionSignatureGroup>,
    pub near_function_candidates: Vec<AstNearFunctionCandidate>,
}

impl Default for AstFunctionCloneGroups {
    fn default() -> Self {
        Self {
            policy: AstFunctionCloneGroupsPolicy::default(),
            supports: AstFunctionCloneGroupsSupports::default(),
            complete: true,
            files_with_parse_errors: Vec::new(),
            files_with_read_errors: Vec::new(),
            exact_body_group_count: 0,
            structure_group_count: 0,
            signature_group_count: 0,
            near_function_candidate_count: 0,
            near_function_candidate_projection_limit:
                super::RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
            generated_file_fact_count: 0,
            exact_body_groups: Vec::new(),
            structure_groups: Vec::new(),
            signature_groups: Vec::new(),
            near_function_candidates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneInputError {
    pub file: String,
    pub message: String,
}
