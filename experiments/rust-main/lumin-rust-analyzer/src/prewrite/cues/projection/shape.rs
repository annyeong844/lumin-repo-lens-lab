use std::collections::BTreeMap;

use crate::prewrite::lookup::{ShapeLookup, ShapeLookupMatch};

mod hash;
mod signature;

use super::add_cue_for_candidate;
use crate::prewrite::cues::model::{
    Cue, CueCandidate, CueCardBuilder, CueTier, MutedReason, SuppressedCue,
};

use hash::shape_hash_cue;
use signature::function_signature_cue;

pub(super) fn add_shape_cues(
    shape_lookups: &[ShapeLookup],
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
) {
    for lookup in shape_lookups.iter().filter(|lookup| lookup.is_match()) {
        let Some(shape_hash) = lookup.shape_hash() else {
            continue;
        };
        for candidate in lookup.matches() {
            add_shape_cue(
                cards,
                suppressed,
                candidate,
                shape_lookup_cue(candidate, shape_hash),
            );
        }
    }
}

fn add_shape_cue(
    cards: &mut BTreeMap<String, CueCardBuilder>,
    suppressed: &mut Vec<SuppressedCue>,
    candidate: &ShapeLookupMatch,
    cue: Cue,
) {
    if candidate.policy_excluded() {
        suppressed.push(SuppressedCue {
            cue_tier: CueTier::Muted,
            original_cue_tier: Some(cue.cue_tier),
            evidence_lane: cue.evidence_lane,
            reason: MutedReason::PolicyExcluded,
            candidate: CueCandidate::from(candidate),
            path_classifications: candidate.path_classifications().to_vec(),
            tokens: Vec::new(),
            distance: None,
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

fn shape_lookup_cue(candidate: &ShapeLookupMatch, shape_hash: &str) -> Cue {
    match candidate {
        ShapeLookupMatch::Shape(_) => shape_hash_cue(candidate, shape_hash),
        ShapeLookupMatch::Signature(_) => function_signature_cue(candidate, shape_hash),
    }
}
