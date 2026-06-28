use serde::Serialize;

use lumin_rust_source_health::protocol::PathClassification;

pub(in crate::prewrite) use record::ShapeMatch;
pub(in crate::prewrite) use signature::{SignatureMatch, SignatureVisibility};

mod record;
mod signature;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub(in crate::prewrite) enum ShapeLookupMatch {
    Shape(ShapeMatch),
    Signature(SignatureMatch),
}

impl ShapeLookupMatch {
    pub(in crate::prewrite) fn identity(&self) -> &str {
        match self {
            Self::Shape(candidate) => candidate.identity(),
            Self::Signature(candidate) => candidate.identity(),
        }
    }

    pub(in crate::prewrite) fn owner_file(&self) -> &str {
        match self {
            Self::Shape(candidate) => candidate.owner_file(),
            Self::Signature(candidate) => candidate.owner_file(),
        }
    }

    pub(in crate::prewrite) fn name(&self) -> &str {
        match self {
            Self::Shape(candidate) => candidate.name(),
            Self::Signature(candidate) => candidate.name(),
        }
    }

    pub(in crate::prewrite) fn is_safe_signature_surface(&self) -> bool {
        match self {
            Self::Shape(_) => false,
            Self::Signature(candidate) => candidate.is_safe_surface(),
        }
    }

    pub(in crate::prewrite) fn signature_visibility(&self) -> Option<SignatureVisibility> {
        match self {
            Self::Shape(_) => None,
            Self::Signature(candidate) => Some(candidate.visibility_label()),
        }
    }

    pub(in crate::prewrite) fn path_classifications(&self) -> &[PathClassification] {
        match self {
            Self::Shape(candidate) => candidate.path_classifications(),
            Self::Signature(candidate) => candidate.path_classifications(),
        }
    }

    pub(in crate::prewrite) fn policy_excluded(&self) -> bool {
        match self {
            Self::Shape(candidate) => candidate.policy_excluded(),
            Self::Signature(candidate) => candidate.policy_excluded(),
        }
    }
}

fn is_path_policy_excluded(
    path_classifications: &[PathClassification],
    path_suppressed: bool,
) -> bool {
    path_suppressed
        || path_classifications.iter().any(|classification| {
            matches!(
                classification,
                PathClassification::Test | PathClassification::Generated
            )
        })
}
