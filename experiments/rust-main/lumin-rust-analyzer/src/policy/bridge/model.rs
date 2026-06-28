use std::collections::BTreeMap;

use lumin_rust_cargo_oracle::protocol::{
    OraclePlan, SemanticCleanSummary, Summary as SemanticSummary,
};
use lumin_rust_source_health::protocol::{AstOpaqueMuteReason, Summary as SyntaxSummary};

use crate::calibration::CalibrationAdjudication;
use crate::policy::{
    evidence::{CoverageBridgeEntry, CoverageEvidence},
    ActionPolicy, OracleBridgeStatus, ProductSemanticCleanSummaryProjection,
};

use super::{
    policy::OracleBridgePolicy,
    projection::{
        OracleBridgeCoverageProjection, OracleBridgeProjection, OracleBridgeSemanticProjection,
        OracleBridgeSyntaxProjection,
    },
};

pub(crate) fn oracle_bridge<'a>(
    syntax_summary: &'a SyntaxSummary,
    semantic_summary: &'a SemanticSummary,
    action_policy: &ActionPolicy<'_>,
    coverage_evidence: &CoverageEvidence<'a>,
    oracle_plan: &OraclePlan,
    calibration_adjudication: Option<&CalibrationAdjudication>,
) -> OracleBridge<'a> {
    let status = oracle_bridge_status(coverage_evidence, oracle_plan);
    let (cargo_event_stream, absence_clean) = coverage_evidence.bridge_entries();
    OracleBridge {
        status,
        policy: OracleBridgePolicy::from_action_policy(action_policy, calibration_adjudication),
        syntax_review_signals: syntax_summary.review_signals,
        syntax_muted_signals: syntax_summary.muted_signals,
        syntax_review_opaque_surfaces: syntax_summary.review_opaque_surfaces,
        syntax_muted_opaque_surfaces: syntax_summary.muted_opaque_surfaces,
        syntax_muted_opaque_surfaces_by_reason: &syntax_summary.muted_opaque_surfaces_by_reason,
        semantic_verified_findings: semantic_summary.verified_findings,
        semantic_rule_backed_findings: semantic_summary.rule_backed_findings,
        semantic_candidate_findings: semantic_summary.candidate_findings,
        semantic_coverage_unavailable_diagnostics: semantic_summary
            .coverage_unavailable_diagnostics,
        semantic_clean: &semantic_summary.semantic_clean,
        cargo_event_stream,
        absence_clean,
    }
}

fn oracle_bridge_status(
    coverage_evidence: &CoverageEvidence<'_>,
    oracle_plan: &OraclePlan,
) -> OracleBridgeStatus {
    let status = coverage_evidence.oracle_bridge_status();
    if status == OracleBridgeStatus::Covered
        && (oracle_plan.omitted_package_count > 0 || oracle_plan.unmatched_target_path_count > 0)
    {
        OracleBridgeStatus::Partial
    } else {
        status
    }
}

pub(crate) struct OracleBridge<'a> {
    status: OracleBridgeStatus,
    policy: OracleBridgePolicy,
    syntax_review_signals: usize,
    syntax_muted_signals: usize,
    syntax_review_opaque_surfaces: usize,
    syntax_muted_opaque_surfaces: usize,
    syntax_muted_opaque_surfaces_by_reason: &'a BTreeMap<AstOpaqueMuteReason, usize>,
    semantic_verified_findings: usize,
    semantic_rule_backed_findings: usize,
    semantic_candidate_findings: usize,
    semantic_coverage_unavailable_diagnostics: usize,
    semantic_clean: &'a SemanticCleanSummary,
    cargo_event_stream: CoverageBridgeEntry,
    absence_clean: CoverageBridgeEntry,
}

impl<'a> OracleBridge<'a> {
    pub(crate) fn status(&self) -> OracleBridgeStatus {
        self.status
    }

    pub(crate) fn into_projection(self) -> OracleBridgeProjection<'a> {
        OracleBridgeProjection {
            schema_version: "rust-oracle-bridge.v1",
            status: self.status,
            purpose:
                "connect AST syntax opacity to Cargo/rustc oracle coverage before accuracy calibration",
            policy: self.policy.bridge_projection(),
            syntax: OracleBridgeSyntaxProjection {
                review_signals: self.syntax_review_signals,
                muted_signals: self.syntax_muted_signals,
                review_opaque_surfaces: self.syntax_review_opaque_surfaces,
                muted_opaque_surfaces: self.syntax_muted_opaque_surfaces,
                muted_opaque_surfaces_by_reason: self.syntax_muted_opaque_surfaces_by_reason,
            },
            semantic: OracleBridgeSemanticProjection {
                verified_findings: self.semantic_verified_findings,
                rule_backed_findings: self.semantic_rule_backed_findings,
                candidate_findings: self.semantic_candidate_findings,
                coverage_unavailable_diagnostics: self.semantic_coverage_unavailable_diagnostics,
                semantic_clean: ProductSemanticCleanSummaryProjection::from_summary(
                    self.semantic_clean,
                ),
            },
            coverage: OracleBridgeCoverageProjection {
                cargo_event_stream: self.cargo_event_stream,
                absence_clean: self.absence_clean,
            },
        }
    }
}
