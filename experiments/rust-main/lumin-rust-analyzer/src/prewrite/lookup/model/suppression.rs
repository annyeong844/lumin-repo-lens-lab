use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum SuppressionReason {
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum PolicySupportingReason {
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
    LocalOperationSameFileDomainOverlap,
}

impl From<SuppressionReason> for PolicySupportingReason {
    fn from(reason: SuppressionReason) -> Self {
        match reason {
            SuppressionReason::DomainTokenOverlap => Self::DomainTokenOverlap,
            SuppressionReason::NearLengthDeltaExceeded => Self::NearLengthDeltaExceeded,
            SuppressionReason::NearPrefixMismatch => Self::NearPrefixMismatch,
            SuppressionReason::NearDistanceExceeded => Self::NearDistanceExceeded,
            SuppressionReason::SingleNonWeakTokenOnly => Self::SingleNonWeakTokenOnly,
            SuppressionReason::InsufficientNonWeakSupport => Self::InsufficientNonWeakSupport,
        }
    }
}
