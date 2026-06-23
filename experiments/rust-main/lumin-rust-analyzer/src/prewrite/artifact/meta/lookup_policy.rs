use serde::Serialize;

use crate::prewrite::lookup;

const LOOKUP_POLICY_JS_TS_PRECEDENT: &[&str] = &[
    "_lib/pre-write-intent.mjs",
    "_lib/pre-write-cue-tiers.mjs",
    "_lib/pre-write-lookup-name.mjs",
    "_lib/pre-write-lookup-file.mjs",
    "_lib/pre-write-lookup-shape.mjs",
    "_lib/pre-write-lookup-dep.mjs",
    "_lib/pre-write-lookup-inline-patterns.mjs",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct LookupPolicyMeta {
    js_ts_precedent: &'static [&'static str],
    near_name: NearNameLookupPolicyMeta,
    semantic_hint: SemanticHintLookupPolicyMeta,
    service_operation_sibling: OperationSiblingPolicyMeta,
    local_operation_sibling: OperationSiblingPolicyMeta,
    file_domain_cluster: FileDomainClusterPolicyMeta,
    dependency_hub: DependencyHubPolicyMeta,
}

impl LookupPolicyMeta {
    pub(super) fn from_constants() -> Self {
        Self {
            js_ts_precedent: LOOKUP_POLICY_JS_TS_PRECEDENT,
            near_name: NearNameLookupPolicyMeta {
                max_length_delta: lookup::NEAR_NAME_MAX_LENGTH_DELTA,
                shared_prefix_min: lookup::NEAR_NAME_SHARED_PREFIX_MIN,
                max_distance: lookup::NEAR_NAME_MAX_DISTANCE,
                max_results: lookup::NEAR_NAME_MAX_RESULTS,
            },
            semantic_hint: SemanticHintLookupPolicyMeta {
                min_score: lookup::SEMANTIC_HINT_MIN_SCORE,
                max_results: lookup::SEMANTIC_HINT_MAX_RESULTS,
            },
            service_operation_sibling: OperationSiblingPolicyMeta {
                policy_id: lookup::SERVICE_OPERATION_POLICY_ID,
                policy_version: lookup::SERVICE_OPERATION_POLICY_VERSION,
                max_results: lookup::SERVICE_OPERATION_POLICY_MAX_RESULTS,
            },
            local_operation_sibling: OperationSiblingPolicyMeta {
                policy_id: lookup::LOCAL_OPERATION_POLICY_ID,
                policy_version: lookup::LOCAL_OPERATION_POLICY_VERSION,
                max_results: lookup::LOCAL_OPERATION_POLICY_MAX_RESULTS,
            },
            file_domain_cluster: FileDomainClusterPolicyMeta {
                min_matches: lookup::DOMAIN_CLUSTER_MIN_MATCHES,
                max_examples: lookup::DOMAIN_CLUSTER_MAX_EXAMPLES,
                min_prefix_len: lookup::DOMAIN_CLUSTER_MIN_PREFIX_LEN,
            },
            dependency_hub: DependencyHubPolicyMeta {
                example_limit: lookup::DEPENDENCY_EXAMPLE_LIMIT,
                watch_for_threshold: lookup::DEPENDENCY_WATCH_FOR_THRESHOLD,
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NearNameLookupPolicyMeta {
    max_length_delta: usize,
    shared_prefix_min: usize,
    max_distance: usize,
    max_results: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticHintLookupPolicyMeta {
    min_score: usize,
    max_results: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationSiblingPolicyMeta {
    policy_id: &'static str,
    policy_version: &'static str,
    max_results: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDomainClusterPolicyMeta {
    min_matches: usize,
    max_examples: usize,
    min_prefix_len: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyHubPolicyMeta {
    example_limit: usize,
    watch_for_threshold: usize,
}
