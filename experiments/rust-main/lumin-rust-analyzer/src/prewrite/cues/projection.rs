use std::collections::BTreeMap;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{
    CandidateRecord, NameLookup, ServiceOperationPolicyEntry, SuppressedNearNameHint,
    SuppressedSemanticHint,
};
use crate::prewrite::tokens::TOKEN_POLICY_VERSION;

use super::model::{
    Cue, CueCandidate, CueCard, CueCardBuilder, CueClaim, CueConfidence, CueEvidence,
    CueMatchedField, CueProjection, CueTier, EvidenceLane, MutedReason, NotSafeFor, SafeMeaning,
    SuppressedCue,
};

pub(in crate::prewrite) fn project(lookups: &[NameLookup]) -> CueProjection {
    let mut cards = BTreeMap::<String, CueCardBuilder>::new();
    let mut suppressed = Vec::new();
    for lookup in lookups {
        for candidate in &lookup.identities {
            add_active_cue(&mut cards, &mut suppressed, candidate, safe_cue(candidate));
        }
        for hint in &lookup.near_names {
            add_active_cue(
                &mut cards,
                &mut suppressed,
                &hint.candidate,
                near_name_cue(&hint.candidate, hint.distance),
            );
        }
        for hint in &lookup.semantic_hints {
            add_active_cue(
                &mut cards,
                &mut suppressed,
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
        add_service_operation_sibling_policy(lookup, &mut cards, &mut suppressed);
    }

    let mut cue_cards = cards
        .into_values()
        .map(|builder| CueCard {
            candidate: builder.candidate,
            render_tier: builder.render_tier,
            cues: builder.cues,
        })
        .collect::<Vec<_>>();
    cue_cards.sort_by(|left, right| {
        tier_rank(left.render_tier)
            .cmp(&tier_rank(right.render_tier))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.identity.cmp(&right.candidate.identity))
    });
    suppressed.sort_by(|left, right| {
        left.reason
            .cmp(&right.reason)
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.identity.cmp(&right.candidate.identity))
    });
    CueProjection {
        cue_cards,
        suppressed_cues: suppressed,
    }
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
        });
        return;
    }

    add_cue_for_candidate(cards, CueCandidate::from(candidate), cue);
}

fn add_cue_for_candidate(
    cards: &mut BTreeMap<String, CueCardBuilder>,
    candidate: CueCandidate,
    cue: Cue,
) {
    let card = cards
        .entry(candidate.identity.clone())
        .or_insert_with(|| CueCardBuilder {
            candidate,
            render_tier: CueTier::Safe,
            cues: Vec::new(),
        });
    if cue.cue_tier == CueTier::AgentReview {
        card.render_tier = CueTier::AgentReview;
    }
    card.cues.push(cue);
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
        claim: CueClaim::ExactRustDefinitionExists,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: candidate.matched_field.into(),
            algorithm_version: Some("exact-symbol.v1"),
            candidate_identity: candidate.identity.clone(),
            distance: None,
            tokens: Vec::new(),
            policy_id: None,
            policy_version: None,
            operation_family: None,
            shared_domain_tokens: Vec::new(),
            locality: None,
            supporting_reasons: Vec::new(),
        }],
    }
}

fn near_name_cue(candidate: &CandidateRecord, distance: usize) -> Cue {
    let impl_method = candidate.matched_field == MatchedField::ImplMethodIndex;
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
            algorithm_version: Some("near-name.v1"),
            candidate_identity: candidate.identity.clone(),
            distance: Some(distance),
            tokens: Vec::new(),
            policy_id: None,
            policy_version: None,
            operation_family: None,
            shared_domain_tokens: Vec::new(),
            locality: None,
            supporting_reasons: Vec::new(),
        }],
    }
}

fn semantic_hint_cue(candidate: &CandidateRecord, tokens: &[String]) -> Cue {
    let impl_method = candidate.matched_field == MatchedField::ImplMethodIndex;
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
            algorithm_version: Some(TOKEN_POLICY_VERSION),
            candidate_identity: candidate.identity.clone(),
            distance: None,
            tokens: tokens.to_vec(),
            policy_id: None,
            policy_version: None,
            operation_family: None,
            shared_domain_tokens: Vec::new(),
            locality: None,
            supporting_reasons: Vec::new(),
        }],
    }
}

fn add_service_operation_sibling_policy(
    lookup: &NameLookup,
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    let policy = &lookup.service_operation_sibling_policy;
    for entry in &policy.promoted {
        if entry.matched_field == MatchedField::ImplMethodIndex {
            suppressed.push(service_operation_muted_cue(
                policy.policy_id,
                policy.policy_version,
                policy.evaluated_candidate_count,
                entry,
                MutedReason::ServiceSiblingClassMethodLane,
                Some(CueTier::AgentReview),
            ));
            continue;
        }
        add_cue_for_candidate(
            cards,
            CueCandidate::from(entry),
            service_operation_cue(policy.policy_id, policy.policy_version, entry),
        );
    }
    suppressed.extend(policy.muted.iter().map(|entry| {
        service_operation_muted_cue(
            policy.policy_id,
            policy.policy_version,
            policy.evaluated_candidate_count,
            entry,
            entry
                .reason
                .map(MutedReason::from)
                .unwrap_or(MutedReason::ServiceSiblingInsufficientMetadata),
            None,
        )
    }));
}

fn service_operation_cue(
    policy_id: &'static str,
    policy_version: &'static str,
    entry: &ServiceOperationPolicyEntry,
) -> Cue {
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: EvidenceLane::ServiceOperationSibling,
        claim: CueClaim::RelatedServiceOperationSibling,
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "pre-write-advisory.json",
            matched_field: CueMatchedField::ServiceOperationSiblingPolicyPromoted,
            algorithm_version: None,
            candidate_identity: entry.identity.clone(),
            distance: None,
            tokens: Vec::new(),
            policy_id: Some(policy_id),
            policy_version: Some(policy_version),
            operation_family: entry.operation_family,
            shared_domain_tokens: entry.shared_domain_tokens.clone(),
            locality: Some(entry.locality),
            supporting_reasons: entry.supporting_reasons.clone(),
        }],
    }
}

fn service_operation_muted_cue(
    policy_id: &'static str,
    policy_version: &'static str,
    candidate_count: usize,
    entry: &ServiceOperationPolicyEntry,
    reason: MutedReason,
    original_cue_tier: Option<CueTier>,
) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier,
        evidence_lane: EvidenceLane::ServiceOperationSibling,
        reason,
        candidate: CueCandidate::from(entry),
        path_classifications: Vec::new(),
        tokens: Vec::new(),
        distance: None,
        score: None,
        candidate_count,
        policy_id: Some(policy_id),
        policy_version: Some(policy_version),
        matched_field: Some(entry.matched_field),
        operation_family: entry.operation_family,
        shared_domain_tokens: entry.shared_domain_tokens.clone(),
        supporting_reasons: entry.supporting_reasons.clone(),
        locality: Some(entry.locality),
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
    }
}

fn tier_rank(tier: CueTier) -> usize {
    match tier {
        CueTier::Safe => 0,
        CueTier::AgentReview => 1,
        CueTier::Muted => 2,
    }
}
