use std::collections::BTreeMap;

use lumin_rust_cargo_oracle::protocol::{ActionBlockerReason, ClaimKind};

use super::{ActionTierSummary, EvidenceTierSummary};
use crate::policy::{semantic_examples, CleanupCandidate, DegradedReason};

pub(crate) struct ActionPolicy<'a> {
    pub(super) syntax_evidence: SyntaxEvidenceCounts,
    pub(super) semantic_confidence: SemanticFindingCounts,
    pub(super) semantic_actions: SemanticActionCounts,
    pub(super) semantic_action_blockers_by_reason: BTreeMap<ActionBlockerReason, usize>,
    pub(super) semantic_action_blocker_examples: Vec<semantic_examples::ActionBlockerExample<'a>>,
    pub(super) semantic_review_by_reason: BTreeMap<ClaimKind, usize>,
    pub(super) semantic_review_examples: Vec<semantic_examples::ReviewExample<'a>>,
    pub(super) semantic_degraded_by_reason: BTreeMap<DegradedReason, usize>,
    pub(super) semantic_degraded_examples: Vec<semantic_examples::DegradedExample<'a>>,
    pub(super) semantic_safe_action_examples: Vec<semantic_examples::SafeActionExample<'a>>,
    pub(super) semantic_cleanup_candidates: Vec<CleanupCandidate<'a>>,
}

#[derive(Debug, Copy, Clone)]
pub(super) struct SyntaxEvidenceCounts {
    review_signals: usize,
    review_opaque_surfaces: usize,
    muted_signals: usize,
    muted_opaque_surfaces: usize,
}

impl SyntaxEvidenceCounts {
    pub(super) fn new(
        review_signals: usize,
        review_opaque_surfaces: usize,
        muted_signals: usize,
        muted_opaque_surfaces: usize,
    ) -> Self {
        Self {
            review_signals,
            review_opaque_surfaces,
            muted_signals,
            muted_opaque_surfaces,
        }
    }

    pub(super) fn review(self) -> usize {
        self.review_signals + self.review_opaque_surfaces
    }

    pub(super) fn review_signals(self) -> usize {
        self.review_signals
    }

    pub(super) fn review_opaque_surfaces(self) -> usize {
        self.review_opaque_surfaces
    }

    pub(super) fn muted(self) -> usize {
        self.muted_signals + self.muted_opaque_surfaces
    }

    pub(super) fn muted_signals(self) -> usize {
        self.muted_signals
    }

    pub(super) fn muted_opaque_surfaces(self) -> usize {
        self.muted_opaque_surfaces
    }

    pub(crate) fn evidence_tier_summary(
        self,
        semantic_actions: SemanticActionCounts,
    ) -> EvidenceTierSummary {
        EvidenceTierSummary::new(
            self.review(),
            semantic_actions.degraded_coverage_entries(),
            self.muted(),
            semantic_actions.coverage_unavailable_diagnostics(),
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct SemanticFindingCounts {
    verified: usize,
    rule_backed: usize,
    candidate: usize,
}

impl SemanticFindingCounts {
    pub(super) fn new(verified: usize, rule_backed: usize, candidate: usize) -> Self {
        Self {
            verified,
            rule_backed,
            candidate,
        }
    }

    pub(super) fn verified(self) -> usize {
        self.verified
    }

    pub(super) fn rule_backed(self) -> usize {
        self.rule_backed
    }

    pub(super) fn candidate(self) -> usize {
        self.candidate
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct SemanticActionCounts {
    safe_actions: usize,
    action_blockers: usize,
    review_findings: usize,
    degraded_findings: usize,
    degraded_coverage_entries: usize,
    coverage_unavailable_diagnostics: usize,
}

impl SemanticActionCounts {
    pub(super) fn new(
        safe_actions: usize,
        action_blockers: usize,
        review_findings: usize,
        degraded_findings: usize,
        degraded_coverage_entries: usize,
        coverage_unavailable_diagnostics: usize,
    ) -> Self {
        Self {
            safe_actions,
            action_blockers,
            review_findings,
            degraded_findings,
            degraded_coverage_entries,
            coverage_unavailable_diagnostics,
        }
    }

    pub(crate) fn safe_actions(self) -> usize {
        self.safe_actions
    }

    pub(crate) fn action_blockers(self) -> usize {
        self.action_blockers
    }

    pub(crate) fn review_findings(self) -> usize {
        self.review_findings
    }

    pub(crate) fn review_fix(self) -> usize {
        self.action_blockers + self.review_findings
    }

    pub(crate) fn review_visible_cleanup(self) -> usize {
        self.safe_actions + self.review_fix()
    }

    pub(crate) fn degraded_findings(self) -> usize {
        self.degraded_findings
    }

    pub(crate) fn degraded_coverage_entries(self) -> usize {
        self.degraded_coverage_entries
    }

    pub(crate) fn coverage_unavailable_diagnostics(self) -> usize {
        self.coverage_unavailable_diagnostics
    }

    pub(crate) fn action_tier_summary(self) -> ActionTierSummary {
        ActionTierSummary::new(
            self.safe_actions(),
            self.review_fix(),
            self.degraded_findings(),
            0,
            self.coverage_unavailable_diagnostics(),
        )
    }
}

impl ActionPolicy<'_> {
    pub(crate) fn semantic_action_counts(&self) -> SemanticActionCounts {
        self.semantic_actions
    }

    pub(crate) fn syntax_muted_evidence_count(&self) -> usize {
        self.syntax_evidence.muted()
    }

    pub(crate) fn semantic_cleanup_candidates(&self) -> &[CleanupCandidate<'_>] {
        &self.semantic_cleanup_candidates
    }
}
