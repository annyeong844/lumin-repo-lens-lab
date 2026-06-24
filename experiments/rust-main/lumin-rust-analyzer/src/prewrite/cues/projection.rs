use std::collections::BTreeMap;

use crate::prewrite::lookup::{
    DependencyLookup, FileLookup, InlinePatternLookup, NameLookup, ShapeLookup,
};

use super::model::{Cue, CueCandidate, CueCard, CueCardBuilder, CueProjection, CueTier};

mod dependency;
mod file;
mod inline_pattern;
mod operation;
mod shape;
mod symbol;

pub(in crate::prewrite) fn project(
    lookups: &[NameLookup],
    shape_lookups: &[ShapeLookup],
    file_lookups: &[FileLookup],
    dependency_lookups: &[DependencyLookup],
    inline_pattern_lookups: &[InlinePatternLookup],
) -> CueProjection {
    let mut cards = BTreeMap::<String, CueCardBuilder>::new();
    let mut suppressed = Vec::new();
    for lookup in lookups {
        symbol::add_symbol_cues(lookup, &mut cards, &mut suppressed);
        operation::add_operation_cues(lookup, &mut cards, &mut suppressed);
    }
    shape::add_shape_cues(shape_lookups, &mut cards, &mut suppressed);
    file::add_file_cues(file_lookups, &mut cards);
    dependency::add_dependency_cues(dependency_lookups, &mut cards);
    inline_pattern::add_inline_pattern_cues(inline_pattern_lookups, &mut cards);

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

fn tier_rank(tier: CueTier) -> usize {
    match tier {
        CueTier::Safe => 0,
        CueTier::AgentReview => 1,
        CueTier::Muted => 2,
    }
}
