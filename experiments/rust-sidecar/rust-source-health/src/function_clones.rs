mod body;
mod common;
mod input;
mod near;
mod signatures;

use std::collections::BTreeMap;

use crate::protocol::{
    AstFunctionCloneGroups, AstNearFunctionCandidateGenerationSummary,
    AstNearFunctionCompatibilitySkippedPairEstimates, FileHealth, PathClassification, SkippedFile,
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES, RUST_FUNCTION_CLONE_NEAR_SKIPPED_PAIR_ESTIMATE_KIND,
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
    let near_diagnostics = near_function_candidates.diagnostics;
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
        candidate_generation_summary: AstNearFunctionCandidateGenerationSummary {
            eligible_function_count: near_diagnostics.eligible_function_count,
            retained_call_token_bucket_count: near_diagnostics.retained_call_token_bucket_count,
            retained_raw_pair_estimate: near_diagnostics.retained_raw_pair_estimate,
            generated_unique_pair_count: near_diagnostics.generated_unique_pair_count,
            scored_pair_count: near_diagnostics.scored_pair_count,
            compatibility_skipped_raw_pair_estimate_by_reason:
                AstNearFunctionCompatibilitySkippedPairEstimates {
                    qualifier_mismatch: near_diagnostics
                        .compatibility_skipped_raw_pair_estimate_by_reason
                        .qualifier_mismatch,
                    parameter_count_delta: near_diagnostics
                        .compatibility_skipped_raw_pair_estimate_by_reason
                        .parameter_count_delta,
                    body_loc_band_mismatch: near_diagnostics
                        .compatibility_skipped_raw_pair_estimate_by_reason
                        .body_loc_band_mismatch,
                    statement_count_band_mismatch: near_diagnostics
                        .compatibility_skipped_raw_pair_estimate_by_reason
                        .statement_count_band_mismatch,
                },
            debug_formatter_boilerplate_skipped_pair_count: near_diagnostics
                .debug_formatter_boilerplate_skipped_pair_count,
            display_formatter_boilerplate_skipped_pair_count: near_diagnostics
                .display_formatter_boilerplate_skipped_pair_count,
            compatibility_skipped_pair_estimate_kind: near_diagnostics
                .compatibility_skipped_pair_estimate_kind(),
            near_function_candidate_count_scope: near_diagnostics
                .near_function_candidate_count_scope(),
        },
        skipped_low_discrimination_buckets: near_diagnostics.skipped_low_discrimination_buckets,
        skipped_low_discrimination_bucket_count: near_diagnostics
            .skipped_low_discrimination_bucket_count,
        skipped_low_discrimination_raw_pair_estimate: near_diagnostics
            .skipped_low_discrimination_raw_pair_estimate,
        skipped_low_discrimination_pair_estimate_kind:
            RUST_FUNCTION_CLONE_NEAR_SKIPPED_PAIR_ESTIMATE_KIND,
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
