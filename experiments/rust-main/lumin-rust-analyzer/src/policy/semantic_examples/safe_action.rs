use lumin_rust_cargo_oracle::protocol::{
    ClaimKind, ConfidenceTier, OracleId, SafeActionEdit, SafeActionKind, SafeActionProof,
};
use serde::Serialize;

use crate::policy::{semantic::FindingActionRecord, ProductPrimarySpanProjection};

pub(in crate::policy) fn finding_safe_action_examples<'a>(
    records: &[FindingActionRecord<'a>],
    limit: usize,
) -> Vec<SafeActionExample<'a>> {
    records
        .iter()
        .filter_map(finding_safe_action_example)
        .take(limit)
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy) struct SafeActionExample<'a> {
    oracle_id: OracleId,
    claim_kind: ClaimKind,
    confidence_tier: ConfidenceTier,
    message: Option<&'a str>,
    span: Option<ProductPrimarySpanProjection<'a>>,
    safe_action: SafeActionProjection<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SafeActionProjection<'a> {
    kind: SafeActionKind,
    proof_complete: bool,
    edit_count: usize,
    first_edit: Option<&'a SafeActionEdit>,
    proof: &'a SafeActionProof,
}

fn finding_safe_action_example<'a>(
    record: &FindingActionRecord<'a>,
) -> Option<SafeActionExample<'a>> {
    let finding = record.finding;
    let action = finding.safe_action.as_ref()?;
    if !record.action.has_safe_action {
        return None;
    }

    Some(SafeActionExample {
        oracle_id: finding.oracle_id,
        claim_kind: finding.claim_kind,
        confidence_tier: finding.confidence_tier,
        message: finding.message.as_deref(),
        span: finding
            .span
            .as_ref()
            .map(ProductPrimarySpanProjection::from_span),
        safe_action: SafeActionProjection {
            kind: action.kind,
            proof_complete: action.proof_complete,
            edit_count: action.edits.len(),
            first_edit: action.edits.first(),
            proof: &action.proof,
        },
    })
}
