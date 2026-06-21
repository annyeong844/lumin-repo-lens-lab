use std::borrow::Cow;
use std::collections::BTreeMap;

use lumin_rust_source_health::protocol::AstOpaqueMuteReason;
use serde::Serialize;

use crate::policy::{
    evidence::CoverageBridgeEntry, CalibrationStatus, OracleBridgeStatus,
    ProductSemanticCleanSummaryProjection,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationProjection {
    pub(super) status: CalibrationStatus,
    pub(super) reason: OracleBridgeCalibrationStatusReason,
    pub(super) candidate_counts: OracleBridgeCalibrationCandidateCounts,
    pub(super) readiness: OracleBridgeCalibrationReadiness,
    pub(super) readiness_policy: OracleBridgeCalibrationReadinessPolicy,
    pub(super) required_evidence: [OracleBridgeCalibrationRequiredEvidence; 3],
    pub(super) js_ts_precedent: OracleBridgeCalibrationPrecedent,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum OracleBridgeCalibrationStatusReason {
    #[serde(rename = "rust-safe-fix-calibration-corpus-measured-with-readiness-limits")]
    MeasuredWithReadinessLimits,
    #[serde(rename = "rust-safe-fix-calibration-corpus-not-measured")]
    NotMeasured,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum OracleBridgeCalibrationRequiredEvidence {
    NonEmptySafeFixPopulation,
    KnownSafeFixFpDenominator,
    ReadinessGateFromRealCorpus,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationCandidateCounts {
    pub(super) available: bool,
    pub(super) safe_fix: usize,
    pub(super) review_fix: usize,
    pub(super) review_visible_cleanup: usize,
    pub(super) degraded: usize,
    pub(super) muted: usize,
    pub(super) syntax_muted_evidence: usize,
    pub(super) unavailable: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationReadiness {
    pub(super) gate: OracleBridgeCalibrationGate,
    pub(super) reasons: Vec<OracleBridgeCalibrationReason>,
    pub(super) safe_fix: OracleBridgeCalibrationAdjudicationStats,
    pub(super) review_visible_cleanup: OracleBridgeCalibrationAdjudicationStats,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationReadinessPolicy {
    pub(super) source: OracleBridgeCalibrationPrecedentRef,
    pub(super) safe_fix_fp_red_threshold: f64,
    pub(super) review_visible_fp_red_threshold: f64,
    pub(super) review_visible_fp_green_threshold: f64,
    pub(super) min_non_trivial_corpus: usize,
    pub(super) default_min_adjudicated_per_corpus: usize,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum OracleBridgeCalibrationGate {
    Red,
    Yellow,
    Green,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationReason {
    pub(super) code: OracleBridgeCalibrationReasonCode,
    pub(super) severity: OracleBridgeCalibrationSeverity,
    pub(super) detail: Cow<'static, str>,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum OracleBridgeCalibrationReasonCode {
    CandidateCountsUnavailable,
    FpRateUnknown,
    AdjudicationCandidateMismatch,
    SafeFixFpThreshold,
    ReviewVisibleFpThreshold,
    SchemaRoundtripNotAttempted,
    SchemaDriftKnown,
    CorpusIdentityMissing,
    DirtyWorktreeUnknown,
    DirtyWorktreeWithoutSnapshot,
    UnresolvedHighFinding,
    SafeFixPopulationEmpty,
    BenchmarkIncomplete,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum OracleBridgeCalibrationSeverity {
    Red,
    Yellow,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationAdjudicationStats {
    pub(super) false_positives: usize,
    pub(super) true_dead: usize,
    pub(super) inconclusive: usize,
    pub(super) not_applicable: usize,
    pub(super) fp_rate: Option<f64>,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OracleBridgeCalibrationPrecedent {
    pub(super) measurement_artifact: OracleBridgeCalibrationPrecedentRef,
    pub(super) measurement_owner: OracleBridgeCalibrationPrecedentRef,
    pub(super) readiness_gate_owner: OracleBridgeCalibrationPrecedentRef,
    pub(super) calibration_corpus_registry: OracleBridgeCalibrationPrecedentRef,
    pub(super) threshold_policy_metadata: OracleBridgeCalibrationPrecedentRef,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(super) enum OracleBridgeCalibrationPrecedentRef {
    #[serde(rename = "p6-measurement.json")]
    MeasurementArtifact,
    #[serde(rename = "_lib/p6-measurement.mjs")]
    MeasurementOwner,
    #[serde(rename = "_lib/p6-measurement.mjs::computeReadiness")]
    ReadinessGateOwner,
    #[serde(rename = "_lib/calibration-corpora.mjs")]
    CalibrationCorpusRegistry,
    #[serde(rename = "_lib/threshold-policies.mjs")]
    ThresholdPolicyMetadata,
}
