use crate::calibration::CalibrationAdjudication;
use crate::policy::ActionPolicy;

use super::calibration::OracleBridgeCalibrationPolicy;
use super::projection::OracleBridgePolicyProjection;

#[derive(Debug, Clone)]
pub(super) struct OracleBridgePolicy {
    opaque_surfaces_remain_evidence: bool,
    does_not_promote_safe_fix: bool,
    policy_exclusions_remain_auditable: bool,
    calibration: OracleBridgeCalibrationPolicy,
}

impl OracleBridgePolicy {
    pub(super) fn from_action_policy(
        action_policy: &ActionPolicy<'_>,
        calibration_adjudication: Option<&CalibrationAdjudication>,
    ) -> Self {
        Self {
            opaque_surfaces_remain_evidence: true,
            does_not_promote_safe_fix: true,
            policy_exclusions_remain_auditable: true,
            calibration: OracleBridgeCalibrationPolicy::from_action_policy(
                action_policy,
                calibration_adjudication,
            ),
        }
    }

    pub(super) fn bridge_projection(self) -> OracleBridgePolicyProjection {
        OracleBridgePolicyProjection {
            opaque_surfaces_remain_evidence: self.opaque_surfaces_remain_evidence,
            does_not_promote_safe_fix: self.does_not_promote_safe_fix,
            policy_exclusions_remain_auditable: self.policy_exclusions_remain_auditable,
            calibration_status: self.calibration.status(),
            calibration: self.calibration.into_projection(),
        }
    }
}
