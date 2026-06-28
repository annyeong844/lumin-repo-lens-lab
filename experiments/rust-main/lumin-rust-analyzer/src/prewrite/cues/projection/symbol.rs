use std::collections::BTreeMap;

mod active;
mod exact;
mod muted;
mod review;

use crate::prewrite::cues::model::{CueCardBuilder, SuppressedCue};
use crate::prewrite::lookup::NameLookup;

use active::add_active_cue;
use exact::safe_cue;
use muted::{suppressed_near_cue, suppressed_semantic_cue};
use review::{near_name_cue, semantic_hint_cue};

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
