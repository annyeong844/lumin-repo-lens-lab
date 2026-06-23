use std::collections::BTreeMap;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{
    CandidateRecord, NameLookup, SuppressedNearNameHint, SuppressedSemanticHint,
};
use crate::prewrite::tokens::TOKEN_POLICY_VERSION;

use super::add_cue_for_candidate;
use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueClaim, CueConfidence, CueEvidence, CueTier, EvidenceLane,
    MutedReason, NotSafeFor, SafeMeaning, SuppressedCue,
};

pub(super) fn add_symbol_cues(
    lookup: &NameLookup,
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    for candidate in &lookup.identities {
        add_active_cue(cards, suppressed, candidate, safe_cue(candidate));
    }
    for hint in &lookup.near_names {
        add_active_cue(
            cards,
            suppressed,
            &hint.candidate,
            near_name_cue(&hint.candidate, hint.distance),
        );
    }
    for hint in &lookup.semantic_hints {
        add_active_cue(
            cards,
            suppressed,
            &hint.candidate,
            semantic_hint_cue(&hint.candidate, &hint.matched_tokens),
        );
    }
    suppressed.extend(lookup.suppressed_near_names.iter().map(suppressed_near_cue));
    suppressed.extend(
        lookup
            .suppressed_semantic_hints
            .iter()
            .map(suppressed_semantic_cue),
    );
}

fn add_active_cue(
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
    candidate: &CandidateRecord,
    cue: Cue,
) {
    if candidate.policy_excluded {
        suppressed.push(SuppressedCue {
            cue_tier: CueTier::Muted,
            original_cue_tier: Some(cue.cue_tier),
            evidence_lane: cue.evidence_lane,
            reason: MutedReason::PolicyExcluded,
            candidate: CueCandidate::from(candidate),
            path_classifications: candidate.path_classifications.clone(),
            tokens: Vec::new(),
            distance: cue.evidence.first().and_then(|evidence| evidence.distance),
            score: None,
            candidate_count: 1,
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
        });
        return;
    }

    add_cue_for_candidate(cards, CueCandidate::from(candidate), cue);
}

fn safe_cue(candidate: &CandidateRecord) -> Cue {
    Cue {
        cue_tier: CueTier::Safe,
        safe_meaning: Some(SafeMeaning::ClaimOnly),
        not_safe_for: vec![
            NotSafeFor::SemanticEquivalence,
            NotSafeFor::AutoReuse,
            NotSafeFor::AutoFix,
        ],
        evidence_lane: EvidenceLane::ExactSymbol,
        claim: match candidate.matched_field {
            MatchedField::UseTree => CueClaim::ExactRustUseTreeNameExists,
            _ => CueClaim::ExactRustDefinitionExists,
        },
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field.into(),
            matched_field_source: None,
            algorithm_version: Some("exact-symbol.v1"),
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

fn near_name_cue(candidate: &CandidateRecord, distance: usize) -> Cue {
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

fn semantic_hint_cue(candidate: &CandidateRecord, tokens: &[String]) -> Cue {
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

fn suppressed_near_cue(hint: &SuppressedNearNameHint) -> SuppressedCue {
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

fn suppressed_semantic_cue(hint: &SuppressedSemanticHint) -> SuppressedCue {
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
