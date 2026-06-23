use serde::Serialize;

use lumin_rust_cargo_oracle::protocol::{ConfidenceTier, CoverageStatus};

use super::lane::RawLaneOwner;
use crate::policy::{ORACLE_SCOPE_SAMPLE_LIMIT, SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT};

pub(super) fn semantic_policy() -> SemanticPolicy {
    SemanticPolicy {
        confidence_tiers: [
            ConfidenceTier::Verified,
            ConfidenceTier::RuleBacked,
            ConfidenceTier::Candidate,
        ],
        coverage_unavailable_status: CoverageStatus::Unavailable,
        raw_evidence_preserved: true,
        raw_evidence_embedded_in_product: false,
        product_projection: SemanticProductProjectionPolicy {
            coverage: SemanticCoverageProjectionPolicy::SummaryAndCappedScopeExamples,
            raw_lane_owner: RawLaneOwner::RustCargoOracle,
            sample_limits: SemanticProductSampleLimits {
                oracle_scope: ORACLE_SCOPE_SAMPLE_LIMIT,
                finding_spans: SEMANTIC_FINDING_SPAN_SAMPLE_LIMIT,
            },
        },
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticPolicy {
    confidence_tiers: [ConfidenceTier; 3],
    coverage_unavailable_status: CoverageStatus,
    raw_evidence_preserved: bool,
    raw_evidence_embedded_in_product: bool,
    product_projection: SemanticProductProjectionPolicy,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticProductProjectionPolicy {
    coverage: SemanticCoverageProjectionPolicy,
    raw_lane_owner: RawLaneOwner,
    sample_limits: SemanticProductSampleLimits,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum SemanticCoverageProjectionPolicy {
    SummaryAndCappedScopeExamples,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticProductSampleLimits {
    oracle_scope: usize,
    finding_spans: usize,
}
