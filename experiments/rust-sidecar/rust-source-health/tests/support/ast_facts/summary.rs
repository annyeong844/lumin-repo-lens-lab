use serde_json::Value;

use crate::artifact::summary_count;

pub fn assert_ast_summary_counts(
    artifact: &Value,
    definitions: u64,
    use_trees: u64,
    path_refs: u64,
    method_call_sites: u64,
    method_calls: u64,
    macro_calls: u64,
) {
    assert_eq!(summary_count(artifact, "definitions"), definitions);
    assert_eq!(summary_count(artifact, "useTrees"), use_trees);
    assert_eq!(summary_count(artifact, "pathRefs"), path_refs);
    assert_eq!(
        summary_count(artifact, "methodCallSites"),
        method_call_sites
    );
    assert_eq!(summary_count(artifact, "methodCalls"), method_calls);
    assert_eq!(summary_count(artifact, "macroCalls"), macro_calls);
}
