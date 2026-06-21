#[derive(Debug, Clone, Copy)]
pub(crate) struct ProductFileSemanticSummary {
    findings: usize,
    diagnostics: usize,
    safe_actions: usize,
    action_blocked_findings: usize,
    review_findings: usize,
    candidate_findings: usize,
}

impl ProductFileSemanticSummary {
    pub(crate) fn empty() -> Self {
        Self {
            findings: 0,
            diagnostics: 0,
            safe_actions: 0,
            action_blocked_findings: 0,
            review_findings: 0,
            candidate_findings: 0,
        }
    }

    pub(crate) fn from_finding_action(
        has_safe_action: bool,
        has_action_blockers: bool,
        is_review: bool,
        is_candidate: bool,
    ) -> Self {
        let mut summary = Self::empty();
        summary.record_finding_action(
            has_safe_action,
            has_action_blockers,
            is_review,
            is_candidate,
        );
        summary
    }

    fn record_finding_action(
        &mut self,
        has_safe_action: bool,
        has_action_blockers: bool,
        is_review: bool,
        is_candidate: bool,
    ) {
        self.findings += 1;
        if has_safe_action {
            self.safe_actions += 1;
        }
        if has_action_blockers {
            self.action_blocked_findings += 1;
        }
        if is_review {
            self.review_findings += 1;
        }
        if is_candidate {
            self.candidate_findings += 1;
        }
    }

    pub(crate) fn record_diagnostic(&mut self) {
        self.diagnostics += 1;
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.findings += other.findings;
        self.diagnostics += other.diagnostics;
        self.safe_actions += other.safe_actions;
        self.action_blocked_findings += other.action_blocked_findings;
        self.review_findings += other.review_findings;
        self.candidate_findings += other.candidate_findings;
    }

    pub(super) fn findings(self) -> usize {
        self.findings
    }

    pub(super) fn diagnostics(self) -> usize {
        self.diagnostics
    }

    pub(super) fn safe_actions(self) -> usize {
        self.safe_actions
    }

    pub(super) fn action_blocked_findings(self) -> usize {
        self.action_blocked_findings
    }

    pub(super) fn review_findings(self) -> usize {
        self.review_findings
    }

    pub(super) fn candidate_findings(self) -> usize {
        self.candidate_findings
    }
}
