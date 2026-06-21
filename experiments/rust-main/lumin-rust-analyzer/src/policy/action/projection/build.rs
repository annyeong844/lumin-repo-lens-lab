use super::super::{
    ActionPolicy, ACTION_POLICY_SCHEMA_VERSION, JS_TS_PRECEDENT, REVIEW_FIX_GATE_REASON,
    REVIEW_FIX_GATE_STATUS, SAFE_FIX_CURRENTLY_SUPPORTED, SAFE_FIX_GATE_REASON,
    SAFE_FIX_GATE_STATUS, SAFE_FIX_NOT_SAFE_FOR,
};
use super::examples::{SemanticDegradedExamples, SemanticExamples, SemanticReasonExamples};
use super::reasons::{
    ActionPolicyReasons, EvidenceReasons, SemanticFindingConfidence, SyntaxEvidence,
};
use super::{ActionPolicyProjection, ReviewFixGate, SafeFixGate};

impl<'a> ActionPolicy<'a> {
    pub(crate) fn into_projection(self) -> ActionPolicyProjection<'a> {
        let reasons = ActionPolicyReasons::from_counts(self.semantic_actions);
        let evidence_reasons =
            EvidenceReasons::from_counts(self.syntax_evidence, self.semantic_actions);
        let syntax_evidence = SyntaxEvidence::from_counts(self.syntax_evidence);
        let semantic_finding_confidence =
            SemanticFindingConfidence::from_counts(self.semantic_confidence, self.semantic_actions);

        ActionPolicyProjection {
            schema_version: ACTION_POLICY_SCHEMA_VERSION,
            js_ts_precedent: JS_TS_PRECEDENT,
            safe_fix_gate: safe_fix_gate(),
            review_fix_gate: review_fix_gate(),
            summary: self.semantic_actions.action_tier_summary(),
            evidence_tier_summary: self
                .syntax_evidence
                .evidence_tier_summary(self.semantic_actions),
            reasons,
            evidence_reasons,
            syntax_evidence,
            semantic_finding_confidence,
            semantic_safe_actions: SemanticExamples::new(
                self.semantic_actions.safe_actions(),
                self.semantic_safe_action_examples,
            ),
            semantic_action_blockers: SemanticReasonExamples::new(
                self.semantic_actions.action_blockers(),
                self.semantic_action_blockers_by_reason,
                self.semantic_action_blocker_examples,
            ),
            semantic_review: SemanticReasonExamples::new(
                self.semantic_actions.review_findings(),
                self.semantic_review_by_reason,
                self.semantic_review_examples,
            ),
            semantic_degraded: SemanticDegradedExamples::new(
                self.semantic_actions.degraded_findings(),
                self.semantic_actions.degraded_coverage_entries(),
                self.semantic_degraded_by_reason,
                self.semantic_degraded_examples,
            ),
        }
    }
}

fn safe_fix_gate() -> SafeFixGate {
    SafeFixGate {
        status: SAFE_FIX_GATE_STATUS,
        reason: SAFE_FIX_GATE_REASON,
        currently_supported: &SAFE_FIX_CURRENTLY_SUPPORTED,
        not_safe_for: &SAFE_FIX_NOT_SAFE_FOR,
    }
}

fn review_fix_gate() -> ReviewFixGate {
    ReviewFixGate {
        status: REVIEW_FIX_GATE_STATUS,
        reason: REVIEW_FIX_GATE_REASON,
        js_ts_precedent: JS_TS_PRECEDENT,
    }
}
