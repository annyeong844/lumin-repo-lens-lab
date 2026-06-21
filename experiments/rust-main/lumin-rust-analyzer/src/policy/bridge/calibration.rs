mod readiness;

use crate::calibration::CalibrationAdjudication;
use crate::policy::{ActionPolicy, CalibrationStatus};

use self::readiness::{readiness, readiness_policy};
use super::projection::{
    OracleBridgeCalibrationCandidateCounts, OracleBridgeCalibrationPrecedent,
    OracleBridgeCalibrationPrecedentRef, OracleBridgeCalibrationProjection,
    OracleBridgeCalibrationReadiness, OracleBridgeCalibrationRequiredEvidence,
    OracleBridgeCalibrationStatusReason,
};

const CALIBRATION_REQUIRED_EVIDENCE: [OracleBridgeCalibrationRequiredEvidence; 3] = [
    OracleBridgeCalibrationRequiredEvidence::NonEmptySafeFixPopulation,
    OracleBridgeCalibrationRequiredEvidence::KnownSafeFixFpDenominator,
    OracleBridgeCalibrationRequiredEvidence::ReadinessGateFromRealCorpus,
];

#[derive(Debug, Clone)]
pub(super) struct OracleBridgeCalibrationPolicy {
    status: CalibrationStatus,
    reason: OracleBridgeCalibrationStatusReason,
    candidate_counts: OracleBridgeCalibrationCandidateCounts,
    readiness: OracleBridgeCalibrationReadiness,
}

impl OracleBridgeCalibrationPolicy {
    pub(super) fn from_action_policy(
        action_policy: &ActionPolicy<'_>,
        calibration_adjudication: Option<&CalibrationAdjudication>,
    ) -> Self {
        let actions = action_policy.semantic_action_counts();
        let observed_candidate_counts = OracleBridgeCalibrationCandidateCounts {
            available: true,
            safe_fix: actions.safe_actions(),
            review_fix: actions.review_fix(),
            review_visible_cleanup: actions.review_visible_cleanup(),
            degraded: actions.degraded_findings(),
            muted: 0,
            syntax_muted_evidence: action_policy.syntax_muted_evidence_count(),
            unavailable: actions.coverage_unavailable_diagnostics(),
        };
        let candidate_counts =
            calibration_candidate_counts(observed_candidate_counts, calibration_adjudication);
        let status = if calibration_adjudication.is_some() {
            CalibrationStatus::Measured
        } else {
            CalibrationStatus::Pending
        };
        let reason = if calibration_adjudication.is_some() {
            OracleBridgeCalibrationStatusReason::MeasuredWithReadinessLimits
        } else {
            OracleBridgeCalibrationStatusReason::NotMeasured
        };
        Self {
            status,
            reason,
            candidate_counts,
            readiness: readiness(
                candidate_counts,
                action_policy.semantic_cleanup_candidates(),
                calibration_adjudication,
            ),
        }
    }

    pub(super) fn status(&self) -> CalibrationStatus {
        self.status
    }

    pub(super) fn into_projection(self) -> OracleBridgeCalibrationProjection {
        OracleBridgeCalibrationProjection {
            status: self.status,
            reason: self.reason,
            candidate_counts: self.candidate_counts,
            readiness: self.readiness,
            readiness_policy: readiness_policy(),
            required_evidence: CALIBRATION_REQUIRED_EVIDENCE,
            js_ts_precedent: OracleBridgeCalibrationPrecedent {
                measurement_artifact: OracleBridgeCalibrationPrecedentRef::MeasurementArtifact,
                measurement_owner: OracleBridgeCalibrationPrecedentRef::MeasurementOwner,
                readiness_gate_owner: OracleBridgeCalibrationPrecedentRef::ReadinessGateOwner,
                calibration_corpus_registry:
                    OracleBridgeCalibrationPrecedentRef::CalibrationCorpusRegistry,
                threshold_policy_metadata:
                    OracleBridgeCalibrationPrecedentRef::ThresholdPolicyMetadata,
            },
        }
    }
}

fn calibration_candidate_counts(
    observed: OracleBridgeCalibrationCandidateCounts,
    calibration_adjudication: Option<&CalibrationAdjudication>,
) -> OracleBridgeCalibrationCandidateCounts {
    let Some(adjudication) = calibration_adjudication else {
        return observed;
    };
    if !adjudication.has_readiness_evidence() {
        return observed;
    }
    let counts = adjudication.candidate_counts();
    let safe_fix = counts.safe_fix().unwrap_or(0);
    let review_fix = counts.review_fix().unwrap_or_else(|| {
        counts
            .review_visible_cleanup()
            .unwrap_or(0)
            .saturating_sub(safe_fix)
    });
    let review_visible_cleanup = counts
        .review_visible_cleanup()
        .unwrap_or(safe_fix + review_fix);
    OracleBridgeCalibrationCandidateCounts {
        available: counts.is_available(),
        safe_fix,
        review_fix,
        review_visible_cleanup,
        degraded: counts.degraded().unwrap_or(0),
        muted: counts.muted().unwrap_or(0),
        syntax_muted_evidence: observed.syntax_muted_evidence,
        unavailable: observed.unavailable,
    }
}
