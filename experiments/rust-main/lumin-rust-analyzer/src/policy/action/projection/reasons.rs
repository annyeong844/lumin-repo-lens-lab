use serde::Serialize;

use super::super::model::{SemanticActionCounts, SemanticFindingCounts, SyntaxEvidenceCounts};

#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) struct ActionPolicyReasons {
    safe_fix: SafeFixReasons,
    review_fix: ReviewFixReasons,
    muted: EmptyReasons,
    degraded: DegradedReasons,
    unavailable: UnavailableReasons,
}

impl ActionPolicyReasons {
    pub(super) fn from_counts(actions: SemanticActionCounts) -> Self {
        Self {
            safe_fix: SafeFixReasons {
                semantic_safe_action: actions.safe_actions(),
            },
            review_fix: ReviewFixReasons {
                semantic_action_blocker: actions.action_blockers(),
                semantic_review_finding: actions.review_findings(),
            },
            muted: EmptyReasons {},
            degraded: DegradedReasons {
                semantic_candidate_finding: actions.degraded_findings(),
            },
            unavailable: UnavailableReasons::from_actions(actions),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct SafeFixReasons {
    semantic_safe_action: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct ReviewFixReasons {
    semantic_action_blocker: usize,
    semantic_review_finding: usize,
}

#[derive(Debug, Serialize)]
struct EmptyReasons {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct DegradedReasons {
    semantic_candidate_finding: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct UnavailableReasons {
    coverage_unavailable_diagnostic: usize,
}

impl UnavailableReasons {
    fn from_actions(actions: SemanticActionCounts) -> Self {
        Self {
            coverage_unavailable_diagnostic: actions.coverage_unavailable_diagnostics(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct EvidenceReasons {
    review: EvidenceReviewReasons,
    degraded: EvidenceDegradedReasons,
    muted: EvidenceMutedReasons,
    unavailable: UnavailableReasons,
}

impl EvidenceReasons {
    pub(super) fn from_counts(syntax: SyntaxEvidenceCounts, actions: SemanticActionCounts) -> Self {
        Self {
            review: EvidenceReviewReasons {
                syntax_review_signal: syntax.review_signals(),
                syntax_review_opaque_surface: syntax.review_opaque_surfaces(),
            },
            degraded: EvidenceDegradedReasons {
                coverage_unavailable_entry: actions.degraded_coverage_entries(),
            },
            muted: EvidenceMutedReasons {
                syntax_muted_signal: syntax.muted_signals(),
                syntax_muted_opaque_surface: syntax.muted_opaque_surfaces(),
            },
            unavailable: UnavailableReasons::from_actions(actions),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct EvidenceReviewReasons {
    syntax_review_signal: usize,
    syntax_review_opaque_surface: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct EvidenceDegradedReasons {
    coverage_unavailable_entry: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
struct EvidenceMutedReasons {
    syntax_muted_signal: usize,
    syntax_muted_opaque_surface: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyntaxEvidence {
    review: usize,
    muted: usize,
    review_signals: usize,
    review_opaque_surfaces: usize,
    muted_signals: usize,
    muted_opaque_surfaces: usize,
}

impl SyntaxEvidence {
    pub(super) fn from_counts(syntax: SyntaxEvidenceCounts) -> Self {
        Self {
            review: syntax.review(),
            muted: syntax.muted(),
            review_signals: syntax.review_signals(),
            review_opaque_surfaces: syntax.review_opaque_surfaces(),
            muted_signals: syntax.muted_signals(),
            muted_opaque_surfaces: syntax.muted_opaque_surfaces(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticFindingConfidence {
    verified: usize,
    rule_backed: usize,
    candidate: usize,
    safe_action: usize,
    review: usize,
}

impl SemanticFindingConfidence {
    pub(super) fn from_counts(
        confidence: SemanticFindingCounts,
        actions: SemanticActionCounts,
    ) -> Self {
        Self {
            verified: confidence.verified(),
            rule_backed: confidence.rule_backed(),
            candidate: confidence.candidate(),
            safe_action: actions.safe_actions(),
            review: actions.review_findings(),
        }
    }
}
