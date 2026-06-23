use crate::prewrite::cues::model::{CueCandidate, CueTier, EvidenceLane, SuppressedCue};
use crate::prewrite::lookup::{SuppressedNearNameHint, SuppressedSemanticHint};

pub(super) fn suppressed_near_cue(hint: &SuppressedNearNameHint) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier: None,
        evidence_lane: EvidenceLane::NearName,
        reason: hint.reason.into(),
        candidate: CueCandidate::from(&hint.candidate),
        path_classifications: hint.candidate.path_classifications.clone(),
        tokens: hint.matched_tokens.clone(),
        distance: hint.distance,
        score: None,
        candidate_count: hint.candidate_count,
        policy_id: None,
        policy_version: None,
        matched_field: None,
        operation_family: None,
        shared_domain_tokens: Vec::new(),
        supporting_reasons: Vec::new(),
        locality: None,
        surface_kind: None,
        container_name: None,
        container_kind: None,
    }
}

pub(super) fn suppressed_semantic_cue(hint: &SuppressedSemanticHint) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier: None,
        evidence_lane: EvidenceLane::IntentToken,
        reason: hint.reason.into(),
        candidate: CueCandidate::from(&hint.candidate),
        path_classifications: hint.candidate.path_classifications.clone(),
        tokens: hint.matched_tokens.clone(),
        distance: None,
        score: Some(hint.score),
        candidate_count: hint.candidate_count,
        policy_id: None,
        policy_version: None,
        matched_field: None,
        operation_family: None,
        shared_domain_tokens: Vec::new(),
        supporting_reasons: Vec::new(),
        locality: None,
        surface_kind: None,
        container_name: None,
        container_kind: None,
    }
}
