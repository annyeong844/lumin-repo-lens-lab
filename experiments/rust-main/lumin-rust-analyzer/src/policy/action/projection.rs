mod build;
mod examples;
mod reasons;

use lumin_rust_cargo_oracle::protocol::{ActionBlockerReason, ClaimKind};
use serde::Serialize;

use super::{
    ActionTierSummary, EvidenceTierSummary, ReviewFixGateReason, ReviewFixGateStatus,
    SafeFixGateReason, SafeFixGateStatus, SafeFixSupportedProof, SafeFixUnsupportedSurface,
};
use crate::policy::semantic_examples;
use examples::{SemanticDegradedExamples, SemanticExamples, SemanticReasonExamples};
use reasons::{ActionPolicyReasons, EvidenceReasons, SemanticFindingConfidence, SyntaxEvidence};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SafeFixGate {
    status: SafeFixGateStatus,
    reason: SafeFixGateReason,
    currently_supported: &'static [SafeFixSupportedProof],
    not_safe_for: &'static [SafeFixUnsupportedSurface],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReviewFixGate {
    status: ReviewFixGateStatus,
    reason: ReviewFixGateReason,
    js_ts_precedent: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionPolicyProjection<'a> {
    schema_version: &'static str,
    js_ts_precedent: &'static str,
    safe_fix_gate: SafeFixGate,
    review_fix_gate: ReviewFixGate,
    summary: ActionTierSummary,
    evidence_tier_summary: EvidenceTierSummary,
    reasons: ActionPolicyReasons,
    evidence_reasons: EvidenceReasons,
    syntax_evidence: SyntaxEvidence,
    semantic_finding_confidence: SemanticFindingConfidence,
    semantic_safe_actions: SemanticExamples<semantic_examples::SafeActionExample<'a>>,
    semantic_action_blockers:
        SemanticReasonExamples<ActionBlockerReason, semantic_examples::ActionBlockerExample<'a>>,
    semantic_review: SemanticReasonExamples<ClaimKind, semantic_examples::ReviewExample<'a>>,
    semantic_degraded: SemanticDegradedExamples<'a>,
}

impl ActionPolicyProjection<'_> {
    pub(crate) fn action_tier_summary(&self) -> ActionTierSummary {
        self.summary
    }

    pub(crate) fn evidence_tier_summary(&self) -> EvidenceTierSummary {
        self.evidence_tier_summary
    }

    pub(crate) fn semantic_safe_actions(&self) -> usize {
        self.semantic_safe_actions.findings()
    }

    pub(crate) fn semantic_action_blocked_findings(&self) -> usize {
        self.semantic_action_blockers.findings()
    }

    pub(crate) fn semantic_review_findings(&self) -> usize {
        self.semantic_review.findings()
    }

    pub(crate) fn semantic_degraded_findings(&self) -> usize {
        self.semantic_degraded.findings()
    }

    pub(crate) fn semantic_degraded_coverage_entries(&self) -> usize {
        self.semantic_degraded.coverage_entries()
    }
}
