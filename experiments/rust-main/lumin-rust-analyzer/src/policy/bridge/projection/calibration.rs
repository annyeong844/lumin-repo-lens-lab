use std::borrow::Cow;

use serde::Serialize;

use crate::policy::CalibrationStatus;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationProjection {
    pub(in crate::policy::bridge) status: CalibrationStatus,
    pub(in crate::policy::bridge) reason: OracleBridgeCalibrationStatusReason,
    pub(in crate::policy::bridge) candidate_counts: OracleBridgeCalibrationCandidateCounts,
    pub(in crate::policy::bridge) readiness: OracleBridgeCalibrationReadiness,
    pub(in crate::policy::bridge) readiness_policy: OracleBridgeCalibrationReadinessPolicy,
    pub(in crate::policy::bridge) required_evidence: [OracleBridgeCalibrationRequiredEvidence; 3],
    pub(in crate::policy::bridge) js_ts_precedent: OracleBridgeCalibrationPrecedent,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(in crate::policy::bridge) enum OracleBridgeCalibrationStatusReason {
    #[serde(rename = "rust-safe-fix-calibration-corpus-measured-with-readiness-limits")]
    MeasuredWithReadinessLimits,
    #[serde(rename = "rust-safe-fix-calibration-corpus-not-measured")]
    NotMeasured,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::policy::bridge) enum OracleBridgeCalibrationRequiredEvidence {
    NonEmptySafeFixPopulation,
    KnownSafeFixFpDenominator,
    ReadinessGateFromRealCorpus,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationCandidateCounts {
    pub(in crate::policy::bridge) available: bool,
    pub(in crate::policy::bridge) safe_fix: usize,
    pub(in crate::policy::bridge) review_fix: usize,
    pub(in crate::policy::bridge) review_visible_cleanup: usize,
    pub(in crate::policy::bridge) degraded: usize,
    pub(in crate::policy::bridge) muted: usize,
    pub(in crate::policy::bridge) syntax_muted_evidence: usize,
    pub(in crate::policy::bridge) unavailable: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationReadiness {
    pub(in crate::policy::bridge) gate: OracleBridgeCalibrationGate,
    pub(in crate::policy::bridge) reasons: Vec<OracleBridgeCalibrationReason>,
    pub(in crate::policy::bridge) safe_fix: OracleBridgeCalibrationAdjudicationStats,
    pub(in crate::policy::bridge) review_visible_cleanup: OracleBridgeCalibrationAdjudicationStats,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationReadinessPolicy {
    pub(in crate::policy::bridge) source: OracleBridgeCalibrationPrecedentRef,
    pub(in crate::policy::bridge) safe_fix_fp_red_threshold: f64,
    pub(in crate::policy::bridge) review_visible_fp_red_threshold: f64,
    pub(in crate::policy::bridge) review_visible_fp_green_threshold: f64,
    pub(in crate::policy::bridge) min_non_trivial_corpus: usize,
    pub(in crate::policy::bridge) default_min_adjudicated_per_corpus: usize,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub(in crate::policy::bridge) enum OracleBridgeCalibrationGate {
    Red,
    Yellow,
    Green,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationReason {
    pub(in crate::policy::bridge) code: OracleBridgeCalibrationReasonCode,
    pub(in crate::policy::bridge) severity: OracleBridgeCalibrationSeverity,
    pub(in crate::policy::bridge) detail: Cow<'static, str>,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::policy::bridge) enum OracleBridgeCalibrationReasonCode {
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
pub(in crate::policy::bridge) enum OracleBridgeCalibrationSeverity {
    Red,
    Yellow,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationAdjudicationStats {
    pub(in crate::policy::bridge) false_positives: usize,
    pub(in crate::policy::bridge) true_dead: usize,
    pub(in crate::policy::bridge) inconclusive: usize,
    pub(in crate::policy::bridge) not_applicable: usize,
    pub(in crate::policy::bridge) fp_rate: Option<f64>,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::policy::bridge) struct OracleBridgeCalibrationPrecedent {
    pub(in crate::policy::bridge) measurement_artifact: OracleBridgeCalibrationPrecedentRef,
    pub(in crate::policy::bridge) measurement_owner: OracleBridgeCalibrationPrecedentRef,
    pub(in crate::policy::bridge) readiness_gate_owner: OracleBridgeCalibrationPrecedentRef,
    pub(in crate::policy::bridge) calibration_corpus_registry: OracleBridgeCalibrationPrecedentRef,
    pub(in crate::policy::bridge) threshold_policy_metadata: OracleBridgeCalibrationPrecedentRef,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
pub(in crate::policy::bridge) enum OracleBridgeCalibrationPrecedentRef {
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
