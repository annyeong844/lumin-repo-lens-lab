use std::collections::BTreeMap;

use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::{
    CandidateRecord, DependencyLookup, FileLookup, LocalOperationPolicyEntry, NameLookup,
    ServiceOperationPolicyEntry, ShapeLookup, ShapeLookupMatch, SuppressedNearNameHint,
    SuppressedSemanticHint, DEPENDENCY_WATCH_FOR_THRESHOLD,
};
use crate::prewrite::tokens::TOKEN_POLICY_VERSION;

use super::model::{
    Cue, CueCandidate, CueCard, CueCardBuilder, CueClaim, CueConfidence, CueEvidence,
    CueMatchedField, CueProjection, CueTier, EvidenceLane, MutedReason, NotSafeFor, SafeMeaning,
    SuppressedCue,
};

pub(in crate::prewrite) fn project(
    lookups: &[NameLookup],
    shape_lookups: &[ShapeLookup],
    file_lookups: &[FileLookup],
    dependency_lookups: &[DependencyLookup],
) -> CueProjection {
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
        add_local_operation_sibling_policy(lookup, &mut cards, &mut suppressed);
    }
    add_shape_lookup_cues(shape_lookups, &mut cards);
    add_file_exact_cues(file_lookups, &mut cards);
    add_file_domain_cluster_cues(file_lookups, &mut cards);
    add_dependency_hub_cues(dependency_lookups, &mut cards);

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
            surface_kind: None,
            container_name: None,
            container_kind: None,
        });
        return;
    }

    add_cue_for_candidate(cards, CueCandidate::from(candidate), cue);
}

fn add_file_exact_cues(file_lookups: &[FileLookup], cards: &mut BTreeMap<String, CueCardBuilder>) {
    for lookup in file_lookups.iter().filter(|lookup| lookup.exists()) {
        let identity = file_candidate_identity(&lookup.intent_file);
        add_cue_for_candidate(
            cards,
            file_candidate(&lookup.intent_file),
            file_exact_cue(identity, lookup),
        );
    }
}

fn add_file_domain_cluster_cues(
    file_lookups: &[FileLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    for lookup in file_lookups
        .iter()
        .filter(|lookup| lookup.has_domain_cluster())
    {
        let identity = file_candidate_identity(&lookup.intent_file);
        add_cue_for_candidate(
            cards,
            file_candidate(&lookup.intent_file),
            file_domain_cluster_cue(identity, lookup),
        );
    }
}

fn add_dependency_hub_cues(
    dependency_lookups: &[DependencyLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    for lookup in dependency_lookups
        .iter()
        .filter(|lookup| lookup.is_watch_for_eligible())
    {
        let identity = dependency_candidate_identity(&lookup.dep_name);
        add_cue_for_candidate(
            cards,
            dependency_candidate(&lookup.dep_name),
            dependency_hub_cue(identity, lookup),
        );
    }
}

fn add_shape_lookup_cues(
    shape_lookups: &[ShapeLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    for lookup in shape_lookups.iter().filter(|lookup| lookup.is_match()) {
        let Some(shape_hash) = lookup.shape_hash() else {
            continue;
        };
        for candidate in lookup.matches() {
            add_cue_for_candidate(
                cards,
                CueCandidate::from(candidate),
                shape_lookup_cue(candidate, shape_hash),
            );
        }
    }
}

fn shape_lookup_cue(candidate: &ShapeLookupMatch, shape_hash: &str) -> Cue {
    match candidate {
        ShapeLookupMatch::Shape(_) => shape_hash_cue(candidate, shape_hash),
        ShapeLookupMatch::Signature(_) => function_signature_cue(candidate, shape_hash),
    }
}

fn shape_hash_cue(candidate: &ShapeLookupMatch, shape_hash: &str) -> Cue {
    Cue {
        cue_tier: CueTier::Safe,
        safe_meaning: Some(SafeMeaning::ClaimOnly),
        not_safe_for: vec![
            NotSafeFor::SemanticEquivalence,
            NotSafeFor::AutoReuse,
            NotSafeFor::AutoFix,
        ],
        evidence_lane: EvidenceLane::ShapeHash,
        claim: CueClaim::SameNormalizedTypeShape,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: CueMatchedField::RustSourceHealthShapeHash,
            matched_field_source: None,
            algorithm_version: Some("shape-hash.normalized.v1"),
            hash: Some(shape_hash.to_string()),
            visibility: None,
            local_name: None,
            candidate_identity: candidate.identity().to_string(),
            file: Some(candidate.owner_file().to_string()),
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

fn function_signature_cue(candidate: &ShapeLookupMatch, shape_hash: &str) -> Cue {
    let safe = candidate.is_safe_signature_surface();
    Cue {
        cue_tier: if safe {
            CueTier::Safe
        } else {
            CueTier::AgentReview
        },
        safe_meaning: safe.then_some(SafeMeaning::ClaimOnly),
        not_safe_for: if safe {
            vec![
                NotSafeFor::SemanticEquivalence,
                NotSafeFor::AutoReuse,
                NotSafeFor::AutoFix,
            ]
        } else {
            Vec::new()
        },
        evidence_lane: EvidenceLane::FunctionSignature,
        claim: CueClaim::SameNormalizedFunctionSignature,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: CueMatchedField::RustSourceHealthFunctionSignatureHash,
            matched_field_source: None,
            algorithm_version: Some("function-signature.normalized.v1"),
            hash: Some(shape_hash.to_string()),
            visibility: candidate.signature_visibility(),
            local_name: Some(candidate.name().to_string()),
            candidate_identity: candidate.identity().to_string(),
            file: Some(candidate.owner_file().to_string()),
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

fn file_exact_cue(identity: String, lookup: &FileLookup) -> Cue {
    Cue {
        cue_tier: CueTier::Safe,
        safe_meaning: Some(SafeMeaning::ClaimOnly),
        not_safe_for: vec![
            NotSafeFor::SemanticEquivalence,
            NotSafeFor::AutoReuse,
            NotSafeFor::AutoFix,
        ],
        evidence_lane: EvidenceLane::ExactFile,
        claim: CueClaim::ExactFileExists,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "rust-source-health",
            matched_field: CueMatchedField::RustSourceHealthFiles,
            matched_field_source: None,
            algorithm_version: Some("exact-file.v1"),
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: identity,
            file: Some(lookup.intent_file.clone()),
            file_lookup_result: Some(lookup.result()),
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

fn file_domain_cluster_cue(identity: String, lookup: &FileLookup) -> Cue {
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: EvidenceLane::FileDomainCluster,
        claim: CueClaim::RelatedRustFileDomainCluster,
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "pre-write-advisory.json",
            matched_field: CueMatchedField::FileDomainCluster,
            matched_field_source: None,
            algorithm_version: None,
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: identity,
            file: Some(lookup.intent_file.clone()),
            file_lookup_result: Some(lookup.result()),
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

fn dependency_hub_cue(identity: String, lookup: &DependencyLookup) -> Cue {
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: EvidenceLane::DependencyHub,
        claim: CueClaim::RustDependencyHub,
        confidence: CueConfidence::Grounded,
        evidence: vec![CueEvidence {
            artifact: "pre-write-advisory.json",
            matched_field: CueMatchedField::DependencyExistingImports,
            matched_field_source: None,
            algorithm_version: None,
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: identity,
            file: None,
            file_lookup_result: None,
            dependency_lookup_result: Some(lookup.result()),
            observed_import_count: lookup.observed_import_count(),
            consumer_threshold: Some(DEPENDENCY_WATCH_FOR_THRESHOLD),
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

fn file_candidate_identity(intent_file: &str) -> String {
    format!("{intent_file}::__file__")
}

fn file_candidate(intent_file: &str) -> CueCandidate {
    CueCandidate {
        owner_file: intent_file.to_string(),
        name: "__file__".to_string(),
        identity: file_candidate_identity(intent_file),
    }
}

fn dependency_candidate_identity(dep_name: &str) -> String {
    format!("Cargo.toml::dependency::{dep_name}")
}

fn dependency_candidate(dep_name: &str) -> CueCandidate {
    CueCandidate {
        owner_file: "Cargo.toml".to_string(),
        name: dep_name.to_string(),
        identity: dependency_candidate_identity(dep_name),
    }
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

fn add_service_operation_sibling_policy(
    lookup: &NameLookup,
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    let policy = &lookup.service_operation_sibling_policy;
    for entry in &policy.promoted {
        if entry.matched_field == MatchedField::ImplMethod {
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
            matched_field_source: None,
            algorithm_version: None,
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: entry.identity.clone(),
            file: None,
            file_lookup_result: None,
            dependency_lookup_result: None,
            observed_import_count: None,
            consumer_threshold: None,
            distance: None,
            tokens: Vec::new(),
            policy_id: Some(policy_id),
            policy_version: Some(policy_version),
            operation_family: entry.operation_family,
            shared_domain_tokens: entry.shared_domain_tokens.clone(),
            locality: Some(entry.locality),
            supporting_reasons: entry.supporting_reasons.clone(),
            surface_kind: None,
            container_name: None,
            container_kind: None,
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
        surface_kind: None,
        container_name: None,
        container_kind: None,
    }
}

fn add_local_operation_sibling_policy(
    lookup: &NameLookup,
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    let policy = &lookup.local_operation_sibling_policy;
    for entry in &policy.promoted {
        add_cue_for_candidate(
            cards,
            CueCandidate::from(entry),
            local_operation_cue(policy.policy_id, policy.policy_version, entry),
        );
    }
    suppressed.extend(policy.muted.iter().map(|entry| {
        local_operation_muted_cue(
            policy.policy_id,
            policy.policy_version,
            policy.evaluated_candidate_count,
            entry,
            entry
                .reason
                .map(MutedReason::from)
                .unwrap_or(MutedReason::LocalOperationInsufficientMetadata),
        )
    }));
}

fn local_operation_cue(
    policy_id: &'static str,
    policy_version: &'static str,
    entry: &LocalOperationPolicyEntry,
) -> Cue {
    Cue {
        cue_tier: CueTier::AgentReview,
        safe_meaning: None,
        not_safe_for: Vec::new(),
        evidence_lane: EvidenceLane::LocalOperationSibling,
        claim: CueClaim::RelatedLocalServiceOperation,
        confidence: CueConfidence::HeuristicReview,
        evidence: vec![CueEvidence {
            artifact: "pre-write-advisory.json",
            matched_field: CueMatchedField::LocalOperationSiblingPolicyPromoted,
            matched_field_source: Some(entry.matched_field),
            algorithm_version: None,
            hash: None,
            visibility: None,
            local_name: None,
            candidate_identity: entry.identity.clone(),
            file: None,
            file_lookup_result: None,
            dependency_lookup_result: None,
            observed_import_count: None,
            consumer_threshold: None,
            distance: None,
            tokens: Vec::new(),
            policy_id: Some(policy_id),
            policy_version: Some(policy_version),
            operation_family: Some(entry.operation_family),
            shared_domain_tokens: entry.shared_domain_tokens.clone(),
            locality: Some(entry.locality),
            supporting_reasons: entry.supporting_reasons.clone(),
            surface_kind: Some(entry.surface_kind),
            container_name: Some(entry.container_name.clone()),
            container_kind: Some(entry.container_kind),
        }],
    }
}

fn local_operation_muted_cue(
    policy_id: &'static str,
    policy_version: &'static str,
    candidate_count: usize,
    entry: &LocalOperationPolicyEntry,
    reason: MutedReason,
) -> SuppressedCue {
    SuppressedCue {
        cue_tier: CueTier::Muted,
        original_cue_tier: None,
        evidence_lane: EvidenceLane::LocalOperationSibling,
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
        operation_family: Some(entry.operation_family),
        shared_domain_tokens: entry.shared_domain_tokens.clone(),
        supporting_reasons: entry.supporting_reasons.clone(),
        locality: Some(entry.locality),
        surface_kind: Some(entry.surface_kind),
        container_name: Some(entry.container_name.clone()),
        container_kind: Some(entry.container_kind),
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

fn tier_rank(tier: CueTier) -> usize {
    match tier {
        CueTier::Safe => 0,
        CueTier::AgentReview => 1,
        CueTier::Muted => 2,
    }
}
