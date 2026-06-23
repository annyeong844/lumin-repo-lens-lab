use crate::prewrite::cues::model::{
    Cue, CueClaim, CueConfidence, CueEvidence, CueTier, EvidenceLane,
};
use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::CandidateRecord;
use crate::prewrite::tokens::TOKEN_POLICY_VERSION;

pub(super) fn near_name_cue(candidate: &CandidateRecord, distance: usize) -> Cue {
    let impl_method = candidate.matched_field == MatchedField::ImplMethod;
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: if impl_method {
            EvidenceLane::ImplMethodName
        } else {
            EvidenceLane::NearName
        },
        claim: if impl_method {
            CueClaim::NearRustImplMethodName
        } else {
            CueClaim::NearRustDefinitionName
        },
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field.into(),
            matched_field_source: None,
            algorithm_version: Some("near-name.v1"),
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: candidate.identity.clone(),
            file: None,
            file_lookup_result: None,
            dependency_lookup_result: None,
            observed_import_count: None,
            consumer_threshold: None,
            distance: Some(distance),
            tokens: Vec::new(),
            policy_id: None,
            policy_version: None,
            operation_family: None,
            shared_domain_tokens: Vec::new(),
            locality: None,
            supporting_reasons: Vec::new(),
            surface_kind: None,
            container_name: None,
            container_kind: None,
        }],
    }
}

pub(super) fn semantic_hint_cue(candidate: &CandidateRecord, tokens: &[String]) -> Cue {
    let impl_method = candidate.matched_field == MatchedField::ImplMethod;
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: if impl_method {
            EvidenceLane::ImplMethodName
        } else {
            EvidenceLane::IntentToken
        },
        claim: if impl_method {
            CueClaim::RustImplMethodIntentTokenOverlap
        } else {
            CueClaim::SupportedIntentTokenOverlap
        },
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field.into(),
            matched_field_source: None,
            algorithm_version: Some(TOKEN_POLICY_VERSION),
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: candidate.identity.clone(),
            file: None,
            file_lookup_result: None,
            dependency_lookup_result: None,
            observed_import_count: None,
            consumer_threshold: None,
            distance: None,
            tokens: tokens.to_vec(),
            policy_id: None,
            policy_version: None,
            operation_family: None,
            shared_domain_tokens: Vec::new(),
            locality: None,
            supporting_reasons: Vec::new(),
            surface_kind: None,
            container_name: None,
            container_kind: None,
        }],
    }
}
