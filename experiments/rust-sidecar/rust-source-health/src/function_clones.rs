mod body;
mod common;
mod input;
mod near;
mod signatures;

use std::collections::BTreeMap;

use crate::protocol::{
    AstFunctionCloneGroups, FileHealth, PathClassification, SkippedFile,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
};

pub(crate) fn group_function_body_fingerprints(
    files: &BTreeMap<String, FileHealth>,
    skipped_files: &[SkippedFile],
) -> AstFunctionCloneGroups {
    let exact_body_groups = body::group_exact_body_groups(files);
    let structure_groups = body::group_structure_groups(files);
    let signature_groups = signatures::group_signature_facts(files);
    let near_function_candidates =
        near::build_near_function_candidates(files, &exact_body_groups, &structure_groups);
    let files_with_parse_errors = input::files_with_parse_errors(files);
    let files_with_read_errors = input::files_with_read_errors(skipped_files);
    let complete = files_with_parse_errors.is_empty() && files_with_read_errors.is_empty();

    AstFunctionCloneGroups {
        complete,
        files_with_parse_errors,
        files_with_read_errors,
        exact_body_group_count: body::review_visible_group_count(&exact_body_groups),
        structure_group_count: body::review_visible_group_count(&structure_groups),
        signature_group_count: signatures::review_visible_signature_group_count(&signature_groups),
        near_function_candidate_count: near_function_candidates.review_visible_count,
        near_function_candidate_projection_limit: RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES,
        generated_file_fact_count: generated_file_fact_count(files),
        exact_body_groups,
        structure_groups,
        signature_groups,
        near_function_candidates: near_function_candidates.candidates,
        ..AstFunctionCloneGroups::default()
    }
}

fn generated_file_fact_count(files: &BTreeMap<String, FileHealth>) -> usize {
    files
        .values()
        .filter(|health| {
            health
                .path
                .classifications
                .contains(&PathClassification::Generated)
        })
        .map(|health| health.ast.function_body_fingerprints.len())
        .sum()
}
