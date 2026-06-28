use lumin_rust_source_health::protocol::RUST_SHAPE_HASH_NORMALIZED_VERSION;

use crate::prewrite::cues::model::{
    Cue, CueClaim, CueConfidence, CueEvidence, CueMatchedField, CueTier, EvidenceLane, NotSafeFor,
    SafeMeaning,
};
use crate::prewrite::lookup::ShapeLookupMatch;

pub(super) fn shape_hash_cue(candidate: &ShapeLookupMatch, shape_hash: &str) -> Cue {
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
            algorithm_version: Some(RUST_SHAPE_HASH_NORMALIZED_VERSION),
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
