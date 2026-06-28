use serde::Serialize;

use crate::protocol::{
    AstFunctionCloneGroup, AstFunctionCloneGroups, AstFunctionCloneGroupsPolicy,
    AstFunctionCloneGroupsSupports, AstFunctionCloneInputError, AstFunctionSignatureGroup,
    AstNearFunctionCandidate,
};

const FUNCTION_CLONE_GROUP_EXAMPLE_LIMIT: usize = 10;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CompactFunctionCloneGroups<'a> {
    policy: &'a AstFunctionCloneGroupsPolicy,
    supports: &'a AstFunctionCloneGroupsSupports,
    complete: bool,
    files_with_parse_errors: &'a [AstFunctionCloneInputError],
    files_with_read_errors: &'a [AstFunctionCloneInputError],
    exact_body_group_count: usize,
    structure_group_count: usize,
    signature_group_count: usize,
    near_function_candidate_count: usize,
    near_function_candidate_projection_limit: usize,
    generated_file_fact_count: usize,
    example_limit: usize,
    exact_body_group_examples: &'a [AstFunctionCloneGroup],
    structure_group_examples: &'a [AstFunctionCloneGroup],
    signature_group_examples: &'a [AstFunctionSignatureGroup],
    near_function_candidate_examples: &'a [AstNearFunctionCandidate],
}

impl<'a> CompactFunctionCloneGroups<'a> {
    pub(super) fn from_groups(groups: &'a AstFunctionCloneGroups) -> Self {
        Self {
            policy: &groups.policy,
            supports: &groups.supports,
            complete: groups.complete,
            files_with_parse_errors: &groups.files_with_parse_errors,
            files_with_read_errors: &groups.files_with_read_errors,
            exact_body_group_count: groups.exact_body_group_count,
            structure_group_count: groups.structure_group_count,
            signature_group_count: groups.signature_group_count,
            near_function_candidate_count: groups.near_function_candidate_count,
            near_function_candidate_projection_limit: groups
                .near_function_candidate_projection_limit,
            generated_file_fact_count: groups.generated_file_fact_count,
            example_limit: FUNCTION_CLONE_GROUP_EXAMPLE_LIMIT,
            exact_body_group_examples: &groups.exact_body_groups[..groups
                .exact_body_groups
                .len()
                .min(FUNCTION_CLONE_GROUP_EXAMPLE_LIMIT)],
            structure_group_examples: &groups.structure_groups[..groups
                .structure_groups
                .len()
                .min(FUNCTION_CLONE_GROUP_EXAMPLE_LIMIT)],
            signature_group_examples: &groups.signature_groups[..groups
                .signature_groups
                .len()
                .min(FUNCTION_CLONE_GROUP_EXAMPLE_LIMIT)],
            near_function_candidate_examples: &groups.near_function_candidates[..groups
                .near_function_candidates
                .len()
                .min(FUNCTION_CLONE_GROUP_EXAMPLE_LIMIT)],
        }
    }
}
