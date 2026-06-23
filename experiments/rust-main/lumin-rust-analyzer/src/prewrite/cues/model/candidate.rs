use serde::Serialize;

use crate::prewrite::lookup::{
    CandidateRecord, LocalOperationPolicyEntry, ServiceOperationPolicyEntry, ShapeLookupMatch,
};

use super::payload::Cue;
use super::vocabulary::CueTier;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CueCandidate {
    pub(in crate::prewrite) identity: String,
    pub(in crate::prewrite::cues) owner_file: String,
    pub(in crate::prewrite::cues) name: String,
}

impl From<&CandidateRecord> for CueCandidate {
    fn from(candidate: &CandidateRecord) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

impl From<&ServiceOperationPolicyEntry> for CueCandidate {
    fn from(candidate: &ServiceOperationPolicyEntry) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

impl From<&LocalOperationPolicyEntry> for CueCandidate {
    fn from(candidate: &LocalOperationPolicyEntry) -> Self {
        Self {
            identity: candidate.identity.clone(),
            owner_file: candidate.owner_file.clone(),
            name: candidate.name.clone(),
        }
    }
}

impl From<&ShapeLookupMatch> for CueCandidate {
    fn from(candidate: &ShapeLookupMatch) -> Self {
        Self {
            identity: candidate.identity().to_string(),
            owner_file: candidate.owner_file().to_string(),
            name: candidate.name().to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct CueCard {
    pub(in crate::prewrite) candidate: CueCandidate,
    pub(in crate::prewrite) render_tier: CueTier,
    pub(in crate::prewrite) cues: Vec<Cue>,
}

pub(in crate::prewrite) struct CueProjection {
    pub(in crate::prewrite) cue_cards: Vec<CueCard>,
    pub(in crate::prewrite) suppressed_cues: Vec<super::muted::SuppressedCue>,
}

pub(in crate::prewrite::cues) struct CueCardBuilder {
    pub(in crate::prewrite::cues) candidate: CueCandidate,
    pub(in crate::prewrite::cues) render_tier: CueTier,
    pub(in crate::prewrite::cues) cues: Vec<Cue>,
}
