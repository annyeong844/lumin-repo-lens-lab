use crate::prewrite::index::model::{Candidate, LocalOperationCandidate};

pub(super) fn sort_candidates(candidates: &mut [Candidate<'_>]) {
    candidates.sort_by(|left, right| {
        left.file
            .cmp(right.file)
            .then(left.owner_name().cmp(&right.owner_name()))
            .then(left.name.cmp(right.name))
            .then(left.location.byte_start.cmp(&right.location.byte_start))
            .then(left.lane.cmp(&right.lane))
    });
}

pub(super) fn sort_local_operations(local_operations: &mut [LocalOperationCandidate<'_>]) {
    local_operations.sort_by(|left, right| {
        left.file
            .cmp(right.file)
            .then(left.container_name.cmp(right.container_name))
            .then(left.name.cmp(right.name))
            .then(left.location.byte_start.cmp(&right.location.byte_start))
    });
}
