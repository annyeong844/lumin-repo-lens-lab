use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::{AstOpaqueMuteReason, HealthResponse, SignalMuteReason};
use serde::Serialize;

use crate::policy::{
    syntax_review_opaque_surface_examples, syntax_review_signal_examples,
    SyntaxReviewOpaqueSurfaceExample, SyntaxReviewSignalExample,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProductSyntaxSummary<'a> {
    syntax_parse_error_files: usize,
    syntax_parse_errors: usize,
    syntax_review_signals: usize,
    syntax_muted_signals: usize,
    syntax_muted_signals_by_reason: &'a BTreeMap<SignalMuteReason, usize>,
    syntax_review_signal_examples: Vec<SyntaxReviewSignalExample<'a>>,
    syntax_definitions: usize,
    syntax_shape_hashes: usize,
    syntax_function_signatures: usize,
    syntax_function_body_fingerprints: usize,
    syntax_function_clone_exact_body_groups: usize,
    syntax_function_clone_structure_groups: usize,
    syntax_function_clone_signature_groups: usize,
    syntax_function_clone_near_candidates: usize,
    syntax_function_clone_near_candidate_projection_limit: usize,
    syntax_inline_patterns: usize,
    syntax_impl_blocks: usize,
    syntax_impl_methods: usize,
    syntax_use_trees: usize,
    syntax_path_refs: usize,
    syntax_method_call_sites: usize,
    syntax_method_calls: usize,
    syntax_macro_calls: usize,
    syntax_cfg_gates: usize,
    syntax_opaque_surfaces: usize,
    syntax_review_opaque_surfaces: usize,
    syntax_muted_opaque_surfaces: usize,
    syntax_muted_opaque_surfaces_by_reason: &'a BTreeMap<AstOpaqueMuteReason, usize>,
    syntax_review_opaque_surface_examples: Vec<SyntaxReviewOpaqueSurfaceExample<'a>>,
}

impl<'a> ProductSyntaxSummary<'a> {
    pub(super) fn from_syntax(response: &'a HealthResponse) -> Self {
        let summary = &response.summary;
        Self {
            syntax_parse_error_files: summary.parse_error_files,
            syntax_parse_errors: summary.parse_errors,
            syntax_review_signals: summary.review_signals,
            syntax_muted_signals: summary.muted_signals,
            syntax_muted_signals_by_reason: &summary.muted_signals_by_reason,
            syntax_review_signal_examples: syntax_review_signal_examples(response),
            syntax_definitions: summary.definitions,
            syntax_shape_hashes: summary.shape_hashes,
            syntax_function_signatures: summary.function_signatures,
            syntax_function_body_fingerprints: summary.function_body_fingerprints,
            syntax_function_clone_exact_body_groups: response
                .function_clone_groups
                .exact_body_group_count,
            syntax_function_clone_structure_groups: response
                .function_clone_groups
                .structure_group_count,
            syntax_function_clone_signature_groups: response
                .function_clone_groups
                .signature_group_count,
            syntax_function_clone_near_candidates: response
                .function_clone_groups
                .near_function_candidate_count,
            syntax_function_clone_near_candidate_projection_limit: response
                .function_clone_groups
                .near_function_candidate_projection_limit,
            syntax_inline_patterns: summary.inline_patterns,
            syntax_impl_blocks: summary.impl_blocks,
            syntax_impl_methods: summary.impl_methods,
            syntax_use_trees: summary.use_trees,
            syntax_path_refs: summary.path_refs,
            syntax_method_call_sites: summary.method_call_sites,
            syntax_method_calls: summary.method_calls,
            syntax_macro_calls: summary.macro_calls,
            syntax_cfg_gates: summary.cfg_gates,
            syntax_opaque_surfaces: summary.opaque_surfaces,
            syntax_review_opaque_surfaces: summary.review_opaque_surfaces,
            syntax_muted_opaque_surfaces: summary.muted_opaque_surfaces,
            syntax_muted_opaque_surfaces_by_reason: &summary.muted_opaque_surfaces_by_reason,
            syntax_review_opaque_surface_examples: syntax_review_opaque_surface_examples(response),
        }
    }
}
