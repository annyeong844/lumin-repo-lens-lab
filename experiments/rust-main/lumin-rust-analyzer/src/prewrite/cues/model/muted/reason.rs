use serde::Serialize;

use crate::prewrite::lookup::{
    LocalOperationMuteReason, ServiceOperationMuteReason, SuppressionReason,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::prewrite) enum MutedReason {
    PolicyExcluded,
    DomainTokenOverlap,
    NearLengthDeltaExceeded,
    NearPrefixMismatch,
    NearDistanceExceeded,
    SingleNonWeakTokenOnly,
    InsufficientNonWeakSupport,
    ServiceSiblingInsufficientMetadata,
    ServiceSiblingPolicyExcluded,
    ServiceSiblingSurfaceKindUnsupported,
    ServiceSiblingClassMethodLane,
    ServiceSiblingNonCallableDefinition,
    ServiceSiblingInsufficientSuppressedSupport,
    ServiceSiblingLocalityMismatch,
    ServiceSiblingUnknownOperation,
    ServiceSiblingDomainMismatch,
    ServiceSiblingOperationFamilyMismatch,
    ServiceSiblingFamilyNotPromotable,
    LocalOperationInsufficientMetadata,
    LocalOperationSurfaceKindUnsupported,
    LocalOperationPolicyExcluded,
    LocalOperationLocalityMismatch,
    LocalOperationUnknownOperation,
    LocalOperationDomainMismatch,
    LocalOperationFamilyMismatch,
    LocalOperationFamilyNotPromotable,
}

impl From<SuppressionReason> for MutedReason {
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

impl From<ServiceOperationMuteReason> for MutedReason {
    fn from(reason: ServiceOperationMuteReason) -> Self {
        match reason {
            ServiceOperationMuteReason::InsufficientMetadata => {
                Self::ServiceSiblingInsufficientMetadata
            }
            ServiceOperationMuteReason::PolicyExcluded => Self::ServiceSiblingPolicyExcluded,
            ServiceOperationMuteReason::SurfaceKindUnsupported => {
                Self::ServiceSiblingSurfaceKindUnsupported
            }
            ServiceOperationMuteReason::NonCallableDefinition => {
                Self::ServiceSiblingNonCallableDefinition
            }
            ServiceOperationMuteReason::InsufficientSuppressedSupport => {
                Self::ServiceSiblingInsufficientSuppressedSupport
            }
            ServiceOperationMuteReason::LocalityMismatch => Self::ServiceSiblingLocalityMismatch,
            ServiceOperationMuteReason::UnknownOperation => Self::ServiceSiblingUnknownOperation,
            ServiceOperationMuteReason::DomainMismatch => Self::ServiceSiblingDomainMismatch,
            ServiceOperationMuteReason::OperationFamilyMismatch => {
                Self::ServiceSiblingOperationFamilyMismatch
            }
            ServiceOperationMuteReason::FamilyNotPromotable => {
                Self::ServiceSiblingFamilyNotPromotable
            }
        }
    }
}

impl From<LocalOperationMuteReason> for MutedReason {
    fn from(reason: LocalOperationMuteReason) -> Self {
        match reason {
            LocalOperationMuteReason::InsufficientMetadata => {
                Self::LocalOperationInsufficientMetadata
            }
            LocalOperationMuteReason::SurfaceKindUnsupported => {
                Self::LocalOperationSurfaceKindUnsupported
            }
            LocalOperationMuteReason::PolicyExcluded => Self::LocalOperationPolicyExcluded,
            LocalOperationMuteReason::LocalityMismatch => Self::LocalOperationLocalityMismatch,
            LocalOperationMuteReason::UnknownOperation => Self::LocalOperationUnknownOperation,
            LocalOperationMuteReason::DomainMismatch => Self::LocalOperationDomainMismatch,
            LocalOperationMuteReason::FamilyMismatch => Self::LocalOperationFamilyMismatch,
            LocalOperationMuteReason::FamilyNotPromotable => {
                Self::LocalOperationFamilyNotPromotable
            }
        }
    }
}
