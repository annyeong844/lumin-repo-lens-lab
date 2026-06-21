use lumin_rust_cargo_oracle::protocol::{
    ClaimKind, ConfidenceTier, CoverageEntry, CoverageId, CoverageKind, CoverageStatus, Finding,
    OracleId,
};
use serde::Serialize;

use crate::policy::{
    semantic::FindingActionRecord, DegradedReason, ProductCoverageUnavailableReason,
    ProductPrimarySpanProjection,
};

pub(in crate::policy) fn finding_degraded_examples<'a>(
    records: &[FindingActionRecord<'a>],
    coverage: &'a [CoverageEntry],
    limit: usize,
) -> Vec<DegradedExample<'a>> {
    let mut examples = Vec::new();
    for record in records.iter().filter(|record| record.action.is_candidate) {
        examples.push(DegradedExample::Finding(Box::new(
            finding_degraded_example(record.finding),
        )));
        if examples.len() >= limit {
            return examples;
        }
    }
    for entry in coverage
        .iter()
        .filter(|entry| entry.status == CoverageStatus::Unavailable)
    {
        examples.push(DegradedExample::Coverage(degraded_coverage_example(entry)));
        if examples.len() >= limit {
            break;
        }
    }
    examples
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(in crate::policy) enum DegradedExample<'a> {
    Finding(Box<DegradedFindingExample<'a>>),
    Coverage(DegradedCoverageExample),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct DegradedFindingExample<'a> {
    reason: DegradedReason,
    oracle_id: OracleId,
    claim_kind: ClaimKind,
    confidence_tier: ConfidenceTier,
    message: Option<&'a str>,
    span: Option<ProductPrimarySpanProjection<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct DegradedCoverageExample {
    reason: DegradedReason,
    coverage_id: CoverageId,
    coverage_kind: CoverageKind,
    status: CoverageStatus,
    detail: Option<ProductCoverageUnavailableReason>,
}

fn finding_degraded_example(finding: &Finding) -> DegradedFindingExample<'_> {
    DegradedFindingExample {
        reason: DegradedReason::SemanticCandidateFinding,
        oracle_id: finding.oracle_id,
        claim_kind: finding.claim_kind,
        confidence_tier: finding.confidence_tier,
        message: finding.message.as_deref(),
        span: finding
            .span
            .as_ref()
            .map(ProductPrimarySpanProjection::from_span),
    }
}

fn degraded_coverage_example(entry: &CoverageEntry) -> DegradedCoverageExample {
    DegradedCoverageExample {
        reason: DegradedReason::CoverageUnavailableEntry,
        coverage_id: entry.id,
        coverage_kind: entry.coverage_kind,
        status: entry.status,
        detail: entry
            .reason
            .as_ref()
            .map(ProductCoverageUnavailableReason::from_reason),
    }
}
