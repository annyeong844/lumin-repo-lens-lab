use super::super::{ServiceOperationFamily, ServiceOperationPolicyEntry};

pub(super) fn sort_service_entries(entries: &mut [ServiceOperationPolicyEntry]) {
    entries.sort_by(|left, right| {
        right
            .locality
            .rank()
            .cmp(&left.locality.rank())
            .then(
                operation_family_name(left.operation_family)
                    .cmp(operation_family_name(right.operation_family)),
            )
            .then(left.name.cmp(&right.name))
            .then(left.owner_file.cmp(&right.owner_file))
            .then(left.identity.cmp(&right.identity))
    });
}

fn operation_family_name(family: Option<ServiceOperationFamily>) -> &'static str {
    family.map(ServiceOperationFamily::as_str).unwrap_or("")
}
