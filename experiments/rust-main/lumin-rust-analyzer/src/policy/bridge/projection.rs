use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::AstOpaqueMuteReason;
use serde::Serialize;

use crate::policy::{
    evidence::CoverageBridgeEntry, CalibrationStatus, OracleBridgeStatus,
    ProductSemanticCleanSummaryProjection,
};

mod calibration;

pub(super) use calibration::{
    OracleBridgeCalibrationAdjudicationStats, OracleBridgeCalibrationCandidateCounts,
    OracleBridgeCalibrationGate, OracleBridgeCalibrationPrecedent,
    OracleBridgeCalibrationPrecedentRef, OracleBridgeCalibrationProjection,
    OracleBridgeCalibrationReadiness, OracleBridgeCalibrationReadinessPolicy,
    OracleBridgeCalibrationReason, OracleBridgeCalibrationReasonCode,
    OracleBridgeCalibrationRequiredEvidence, OracleBridgeCalibrationSeverity,
    OracleBridgeCalibrationStatusReason,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OracleBridgeProjection<'a> {
    pub(super) schema_version: &'static str,
    pub(super) status: OracleBridgeStatus,
    pub(super) purpose: &'static str,
    pub(super) policy: OracleBridgePolicyProjection,
    pub(super) syntax: OracleBridgeSyntaxProjection<'a>,
    pub(super) semantic: OracleBridgeSemanticProjection,
    pub(super) coverage: OracleBridgeCoverageProjection,
}

impl OracleBridgeProjection<'_> {
    pub(crate) fn status(&self) -> OracleBridgeStatus {
        self.status
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeSyntaxProjection<'a> {
    pub(super) review_signals: usize,
    pub(super) muted_signals: usize,
    pub(super) review_opaque_surfaces: usize,
    pub(super) muted_opaque_surfaces: usize,
    pub(super) muted_opaque_surfaces_by_reason: &'a BTreeMap<AstOpaqueMuteReason, usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeSemanticProjection {
    pub(super) verified_findings: usize,
    pub(super) rule_backed_findings: usize,
    pub(super) candidate_findings: usize,
    pub(super) coverage_unavailable_diagnostics: usize,
    pub(super) semantic_clean: ProductSemanticCleanSummaryProjection,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCoverageProjection {
    pub(super) cargo_event_stream: CoverageBridgeEntry,
    pub(super) absence_clean: CoverageBridgeEntry,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgePolicyProjection {
    pub(super) opaque_surfaces_remain_evidence: bool,
    pub(super) does_not_promote_safe_fix: bool,
    pub(super) policy_exclusions_remain_auditable: bool,
    pub(super) calibration_status: CalibrationStatus,
    pub(super) calibration: OracleBridgeCalibrationProjection,
}
