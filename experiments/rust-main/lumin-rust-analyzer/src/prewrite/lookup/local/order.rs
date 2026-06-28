use super::super::model::LocalOperationPolicyEntry;

pub(super) fn sort_local_entries(entries: &mut [LocalOperationPolicyEntry]) {
    entries.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(
                left.operation_family
                    .as_str()
                    .cmp(right.operation_family.as_str()),
            )
            .then(left.name.cmp(&right.name))
            .then(left.owner_file.cmp(&right.owner_file))
            .then(left.identity.cmp(&right.identity))
    });
}
