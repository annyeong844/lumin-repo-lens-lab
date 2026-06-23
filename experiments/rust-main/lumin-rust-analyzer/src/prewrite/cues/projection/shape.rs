use std::collections::BTreeMap;

use crate::prewrite::lookup::{ShapeLookup, ShapeLookupMatch};

use super::add_cue_for_candidate;
use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueClaim, CueConfidence, CueEvidence, CueMatchedField,
    CueTier, EvidenceLane, NotSafeFor, SafeMeaning,
};

pub(super) fn add_shape_cues(
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
