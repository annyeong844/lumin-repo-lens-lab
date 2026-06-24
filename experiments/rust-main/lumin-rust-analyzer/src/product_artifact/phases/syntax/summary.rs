use lumin_rust_source_health::protocol::Summary as SyntaxSummary;
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
    function_body_fingerprints: usize,
    function_clone_exact_body_groups: usize,
    function_clone_structure_groups: usize,
    function_clone_near_candidates: usize,
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
    pub(super) fn from_summary(summary: &SyntaxSummary) -> Self {
        Self {
            files: summary.files,
            skipped_files: summary.skipped_files,
            parse_error_files: summary.parse_error_files,
            parse_errors: summary.parse_errors,
            review_signals: summary.review_signals,
            muted_signals: summary.muted_signals,
            definitions: summary.definitions,
            function_body_fingerprints: summary.function_body_fingerprints,
            function_clone_exact_body_groups: summary.function_clone_exact_body_groups,
            function_clone_structure_groups: summary.function_clone_structure_groups,
            function_clone_near_candidates: summary.function_clone_near_candidates,
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
