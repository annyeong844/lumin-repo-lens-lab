use lumin_rust_cargo_oracle::protocol::{CoverageEntry, Finding, Summary as SemanticSummary};
use lumin_rust_source_health::protocol::Summary as SyntaxSummary;

use super::model::{SemanticActionCounts, SemanticFindingCounts, SyntaxEvidenceCounts};
use super::ActionPolicy;
use crate::policy::{semantic, semantic_examples, CoverageEvidence, ACTION_SAMPLE_LIMIT};

pub(crate) fn action_policy<'a>(
    syntax_summary: &SyntaxSummary,
    semantic_summary: &SemanticSummary,
    coverage_evidence: &CoverageEvidence<'_>,
    coverage: &'a [CoverageEntry],
    semantic_findings: &'a [Finding],
) -> ActionPolicy<'a> {
    let syntax_evidence = SyntaxEvidenceCounts::new(
        syntax_summary.review_signals,
        syntax_summary.review_opaque_surfaces,
        syntax_summary.muted_signals,
        syntax_summary.muted_opaque_surfaces,
    );
    let semantic_confidence = SemanticFindingCounts::new(
        semantic_summary.verified_findings,
        semantic_summary.rule_backed_findings,
        semantic_summary.candidate_findings,
    );
    let finding_actions = semantic::finding_actions(semantic_findings);
    let finding_action_summary = semantic::finding_action_summary(&finding_actions);
    let coverage_unavailable_diagnostics = semantic_summary.coverage_unavailable_diagnostics;
    let degraded_coverage_entries = coverage_evidence.unavailable_entries();
    let semantic_degraded_findings = semantic_confidence.candidate();
    let semantic_actions = SemanticActionCounts::new(
        finding_action_summary.safe_actions,
        finding_action_summary.action_blockers,
        finding_action_summary.review_findings,
        semantic_degraded_findings,
        degraded_coverage_entries,
        coverage_unavailable_diagnostics,
    );
    let semantic_degraded_by_reason =
        semantic::degraded_by_reason(semantic_degraded_findings, degraded_coverage_entries);

    ActionPolicy {
        syntax_evidence,
        semantic_confidence,
        semantic_actions,
        semantic_action_blockers_by_reason: finding_action_summary.action_blockers_by_reason,
        semantic_action_blocker_examples: semantic_examples::finding_action_blocker_examples(
            &finding_actions,
            ACTION_SAMPLE_LIMIT,
        ),
        semantic_review_by_reason: finding_action_summary.review_by_reason,
        semantic_review_examples: semantic_examples::finding_review_examples(
            &finding_actions,
            ACTION_SAMPLE_LIMIT,
        ),
        semantic_degraded_by_reason,
        semantic_degraded_examples: semantic_examples::finding_degraded_examples(
            &finding_actions,
            coverage,
            ACTION_SAMPLE_LIMIT,
        ),
        semantic_safe_action_examples: semantic_examples::finding_safe_action_examples(
            &finding_actions,
            ACTION_SAMPLE_LIMIT,
        ),
        semantic_cleanup_candidates: semantic::cleanup_candidates(&finding_actions),
    }
}
