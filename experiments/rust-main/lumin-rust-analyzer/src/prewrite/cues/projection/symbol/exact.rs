use crate::prewrite::cues::model::{
    Cue, CueClaim, CueConfidence, CueEvidence, CueTier, EvidenceLane, NotSafeFor, SafeMeaning,
};
use crate::prewrite::index::MatchedField;
use crate::prewrite::lookup::CandidateRecord;

pub(super) fn safe_cue(candidate: &CandidateRecord) -> Cue {
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
