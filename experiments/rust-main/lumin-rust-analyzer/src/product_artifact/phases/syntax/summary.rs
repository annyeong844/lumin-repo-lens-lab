use lumin_rust_source_health::protocol::HealthResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyntaxPhaseSummaryBrief {
    files: usize,
    skipped_files: usize,
    parse_error_files: usize,
    parse_errors: usize,
    review_signals: usize,
    muted_signals: usize,
    definitions: usize,
    shape_hashes: usize,
    function_signatures: usize,
    function_body_fingerprints: usize,
    function_clone_exact_body_groups: usize,
    function_clone_structure_groups: usize,
    function_clone_signature_groups: usize,
    function_clone_near_candidates: usize,
    function_clone_near_candidate_projection_limit: usize,
    inline_patterns: usize,
    impl_blocks: usize,
    impl_methods: usize,
    use_trees: usize,
    path_refs: usize,
    method_call_sites: usize,
    method_calls: usize,
    macro_calls: usize,
    cfg_gates: usize,
    opaque_surfaces: usize,
    review_opaque_surfaces: usize,
    muted_opaque_surfaces: usize,
}

impl SyntaxPhaseSummaryBrief {
    pub(super) fn from_syntax(syntax: &HealthResponse) -> Self {
        let summary = &syntax.summary;
        Self {
            files: summary.files,
            skipped_files: summary.skipped_files,
            parse_error_files: summary.parse_error_files,
            parse_errors: summary.parse_errors,
            review_signals: summary.review_signals,
            muted_signals: summary.muted_signals,
            definitions: summary.definitions,
            shape_hashes: summary.shape_hashes,
            function_signatures: summary.function_signatures,
            function_body_fingerprints: summary.function_body_fingerprints,
            function_clone_exact_body_groups: syntax.function_clone_groups.exact_body_group_count,
            function_clone_structure_groups: syntax.function_clone_groups.structure_group_count,
            function_clone_signature_groups: syntax.function_clone_groups.signature_group_count,
            function_clone_near_candidates: syntax
                .function_clone_groups
                .near_function_candidate_count,
            function_clone_near_candidate_projection_limit: syntax
                .function_clone_groups
                .near_function_candidate_projection_limit,
            inline_patterns: summary.inline_patterns,
            impl_blocks: summary.impl_blocks,
            impl_methods: summary.impl_methods,
            use_trees: summary.use_trees,
            path_refs: summary.path_refs,
            method_call_sites: summary.method_call_sites,
            method_calls: summary.method_calls,
            macro_calls: summary.macro_calls,
            cfg_gates: summary.cfg_gates,
            opaque_surfaces: summary.opaque_surfaces,
            review_opaque_surfaces: summary.review_opaque_surfaces,
            muted_opaque_surfaces: summary.muted_opaque_surfaces,
        }
    }
}
