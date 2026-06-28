use lumin_rust_cargo_oracle::protocol::{
    ActionBlockerReason, ClaimKind, ConfidenceTier, Finding, OracleId,
};
use serde::Serialize;

use crate::policy::{semantic::FindingActionRecord, ProductPrimarySpanProjection};

pub(in crate::policy) fn finding_action_blocker_examples<'a>(
    records: &[FindingActionRecord<'a>],
    limit: usize,
) -> Vec<ActionBlockerExample<'a>> {
    records
        .iter()
        .filter(|record| !record.action.action_blockers.is_empty())
        .take(limit)
        .map(finding_action_blocker_example)
        .collect()
}

pub(in crate::policy) fn finding_review_examples<'a>(
    records: &[FindingActionRecord<'a>],
    limit: usize,
) -> Vec<ReviewExample<'a>> {
    records
        .iter()
        .filter(|record| record.action.is_review)
        .take(limit)
        .map(|record| finding_review_example(record.finding))
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct ActionBlockerExample<'a> {
    oracle_id: OracleId,
    claim_kind: ClaimKind,
    confidence_tier: ConfidenceTier,
    message: Option<&'a str>,
    span: Option<ProductPrimarySpanProjection<'a>>,
    action_blockers: Vec<ActionBlockerReason>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct ReviewExample<'a> {
    reason: ClaimKind,
    oracle_id: OracleId,
    claim_kind: ClaimKind,
    confidence_tier: ConfidenceTier,
    message: Option<&'a str>,
    span: Option<ProductPrimarySpanProjection<'a>>,
}

fn finding_action_blocker_example<'a>(
    record: &FindingActionRecord<'a>,
) -> ActionBlockerExample<'a> {
    let finding = record.finding;
    ActionBlockerExample {
        oracle_id: finding.oracle_id,
        claim_kind: finding.claim_kind,
        confidence_tier: finding.confidence_tier,
        message: finding.message.as_deref(),
        span: finding
            .span
            .as_ref()
            .map(ProductPrimarySpanProjection::from_span),
        action_blockers: record.action.action_blockers.to_vec(),
    }
}

fn finding_review_example(finding: &Finding) -> ReviewExample<'_> {
    ReviewExample {
        reason: finding.claim_kind,
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
