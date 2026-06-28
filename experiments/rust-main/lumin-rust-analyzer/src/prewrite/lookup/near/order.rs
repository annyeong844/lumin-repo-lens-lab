use std::cmp::Ordering;

use super::SuppressedNearNameHint;
use crate::prewrite::index::MatchedField;

pub(super) fn lane_rank(field: MatchedField) -> usize {
    match field {
        MatchedField::ImplMethod => 0,
        MatchedField::Def => 1,
        MatchedField::UseTree => 2,
        MatchedField::PreWriteLocalOperation => 3,
    }
}

pub(super) fn suppressed_near_order(
    left: &SuppressedNearNameHint,
    right: &SuppressedNearNameHint,
) -> Ordering {
    right
        .locality
        .rank()
        .cmp(&left.locality.rank())
        .then(
            left.distance
                .unwrap_or(usize::MAX)
                .cmp(&right.distance.unwrap_or(usize::MAX)),
        )
        .then(
            left.length_delta
                .unwrap_or(usize::MAX)
                .cmp(&right.length_delta.unwrap_or(usize::MAX)),
        )
        .then(left.candidate.name.cmp(&right.candidate.name))
        .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
}
