use std::collections::BTreeMap;

use crate::prewrite::lookup::FileLookup;

use super::add_cue_for_candidate;
use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueClaim, CueConfidence, CueEvidence, CueMatchedField,
    CueTier, EvidenceLane, NotSafeFor, SafeMeaning,
};

pub(super) fn add_file_cues(
    file_lookups: &[FileLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    add_exact_cues(file_lookups, cards);
    add_domain_cluster_cues(file_lookups, cards);
}

fn add_exact_cues(file_lookups: &[FileLookup], cards: &mut BTreeMap<String, CueCardBuilder>) {
    for lookup in file_lookups.iter().filter(|lookup| lookup.exists()) {
        let identity = candidate_identity(&lookup.intent_file);
        add_cue_for_candidate(
            cards,
            candidate(&lookup.intent_file),
            exact_cue(identity, lookup),
        );
    }
}

fn add_domain_cluster_cues(
    file_lookups: &[FileLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    for lookup in file_lookups
        .iter()
        .filter(|lookup| lookup.has_domain_cluster())
    {
        let identity = candidate_identity(&lookup.intent_file);
        add_cue_for_candidate(
            cards,
            candidate(&lookup.intent_file),
            domain_cluster_cue(identity, lookup),
        );
    }
}

fn exact_cue(identity: String, lookup: &FileLookup) -> Cue {
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

fn domain_cluster_cue(identity: String, lookup: &FileLookup) -> Cue {
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

fn candidate_identity(intent_file: &str) -> String {
    format!("{intent_file}::__file__")
}

fn candidate(intent_file: &str) -> CueCandidate {
    CueCandidate {
        owner_file: intent_file.to_string(),
        name: "__file__".to_string(),
        identity: candidate_identity(intent_file),
    }
}
