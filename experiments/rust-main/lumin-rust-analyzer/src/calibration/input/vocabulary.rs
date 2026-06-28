use serde::Deserializer;

use crate::policy::ActionPolicyTier;

use super::super::CalibrationVerdict;

pub(in crate::calibration) fn deserialize_action_policy_tier<'de, D>(
    deserializer: D,
) -> Result<Option<ActionPolicyTier>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(raw) = super::deserialize_optional_string(deserializer)? else {
        return Ok(None);
    };
    Ok(match raw.as_str() {
        "SAFE_FIX" => Some(ActionPolicyTier::SafeFix),
        "REVIEW_FIX" => Some(ActionPolicyTier::ReviewFix),
        "DEGRADED" => Some(ActionPolicyTier::Degraded),
        "MUTED" => Some(ActionPolicyTier::Muted),
        "UNAVAILABLE" => Some(ActionPolicyTier::Unavailable),
        _ => None,
    })
}

pub(in crate::calibration) fn deserialize_calibration_verdict<'de, D>(
    deserializer: D,
) -> Result<CalibrationVerdict, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(raw) = super::deserialize_optional_string(deserializer)? else {
        return Ok(CalibrationVerdict::Inconclusive);
    };
    Ok(match raw.as_str() {
        "true_dead" => CalibrationVerdict::TrueDead,
        "false_positive" => CalibrationVerdict::FalsePositive,
        "not_applicable" => CalibrationVerdict::NotApplicable,
        "inconclusive" => CalibrationVerdict::Inconclusive,
        _ => CalibrationVerdict::Inconclusive,
    })
}
