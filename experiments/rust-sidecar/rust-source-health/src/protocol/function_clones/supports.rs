use serde::Serialize;

use crate::protocol::{
    RUST_FUNCTION_BODY_NORMALIZED_VERSION, RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstFunctionCloneGroupsSupports {
    pub top_level_functions: bool,
    pub impl_methods: bool,
    pub function_fact_visibility: bool,
    pub exact_body_hash: bool,
    pub normalized_exact_hash: bool,
    pub normalized_structure_hash: bool,
    pub normalized_version: &'static str,
    pub normalized_function_signature_hash: bool,
    pub function_signature_groups: bool,
    pub function_signature_normalized_version: &'static str,
    pub near_function_candidates: bool,
    pub generated_file_evidence: bool,
    pub semantic_equivalence: bool,
}

impl Default for AstFunctionCloneGroupsSupports {
    fn default() -> Self {
        Self {
            top_level_functions: true,
            impl_methods: true,
            function_fact_visibility: true,
            exact_body_hash: true,
            normalized_exact_hash: true,
            normalized_structure_hash: true,
            normalized_version: RUST_FUNCTION_BODY_NORMALIZED_VERSION,
            normalized_function_signature_hash: true,
            function_signature_groups: true,
            function_signature_normalized_version: RUST_FUNCTION_SIGNATURE_NORMALIZED_VERSION,
            near_function_candidates: true,
            generated_file_evidence: true,
            semantic_equivalence: false,
        }
    }
}
