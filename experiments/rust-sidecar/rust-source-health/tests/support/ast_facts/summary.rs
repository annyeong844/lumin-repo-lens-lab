use serde_json::Value;

use crate::artifact::summary_count;

pub struct AstSummaryCounts {
    pub definitions: u64,
    pub shape_hashes: u64,
    pub function_signatures: u64,
    pub inline_patterns: u64,
    pub impl_blocks: u64,
    pub impl_methods: u64,
    pub use_trees: u64,
    pub path_refs: u64,
    pub method_call_sites: u64,
    pub method_calls: u64,
    pub macro_calls: u64,
}

pub fn assert_ast_summary_counts(artifact: &Value, expected: AstSummaryCounts) {
    assert_eq!(summary_count(artifact, "definitions"), expected.definitions);
    assert_eq!(
        summary_count(artifact, "shapeHashes"),
        expected.shape_hashes
    );
    assert_eq!(
        summary_count(artifact, "functionSignatures"),
        expected.function_signatures
    );
    assert_eq!(
        summary_count(artifact, "inlinePatterns"),
        expected.inline_patterns
    );
    assert_eq!(summary_count(artifact, "implBlocks"), expected.impl_blocks);
    assert_eq!(
        summary_count(artifact, "implMethods"),
        expected.impl_methods
    );
    assert_eq!(summary_count(artifact, "useTrees"), expected.use_trees);
    assert_eq!(summary_count(artifact, "pathRefs"), expected.path_refs);
    assert_eq!(
        summary_count(artifact, "methodCallSites"),
        expected.method_call_sites
    );
    assert_eq!(
        summary_count(artifact, "methodCalls"),
        expected.method_calls
    );
    assert_eq!(summary_count(artifact, "macroCalls"), expected.macro_calls);
}
