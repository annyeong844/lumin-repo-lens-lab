use std::collections::BTreeMap;

use crate::prewrite::lookup::{DependencyLookup, DEPENDENCY_WATCH_FOR_THRESHOLD};

use super::add_cue_for_candidate;
use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueClaim, CueConfidence, CueEvidence, CueMatchedField,
    CueTier, EvidenceLane,
};

pub(super) fn add_dependency_cues(
    dependency_lookups: &[DependencyLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
) {
    for lookup in dependency_lookups
        .iter()
        .filter(|lookup| lookup.is_watch_for_eligible())
    {
        let identity = candidate_identity(&lookup.dep_name);
        add_cue_for_candidate(
            cards,
            candidate(&lookup.dep_name),
            hub_cue(identity, lookup),
        );
    }
}

fn hub_cue(identity: String, lookup: &DependencyLookup) -> Cue {
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

fn candidate_identity(dep_name: &str) -> String {
    format!("Cargo.toml::dependency::{dep_name}")
}

fn candidate(dep_name: &str) -> CueCandidate {
    CueCandidate {
        owner_file: "Cargo.toml".to_string(),
        name: dep_name.to_string(),
        identity: candidate_identity(dep_name),
    }
}
