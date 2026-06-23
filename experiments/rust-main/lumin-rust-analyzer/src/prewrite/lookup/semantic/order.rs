use super::super::{SemanticHint, SuppressedSemanticHint};

pub(super) fn sort_semantic_hints(hints: &mut [SemanticHint]) {
    hints.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(right.score.cmp(&left.score))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
    });
}

pub(super) fn sort_suppressed_semantic_hints(hints: &mut [SuppressedSemanticHint]) {
    hints.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(right.score.cmp(&left.score))
            .then(left.candidate.name.cmp(&right.candidate.name))
            .then(left.candidate.owner_file.cmp(&right.candidate.owner_file))
    });
}
