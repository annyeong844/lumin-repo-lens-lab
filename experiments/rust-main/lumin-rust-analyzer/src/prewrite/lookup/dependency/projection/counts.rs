use super::super::model::{DependencyLookupResult, ImportCountConfidence};

pub(super) fn existing_import_count_fields(
    result: DependencyLookupResult,
    count: usize,
    partial_reason: Option<&str>,
) -> (Option<usize>, ImportCountConfidence, Option<&str>) {
    match result {
        DependencyLookupResult::Available => {
            if partial_reason.is_some() {
                (
                    Some(count),
                    ImportCountConfidence::SampleOnly,
                    partial_reason,
                )
            } else {
                (Some(count), ImportCountConfidence::Grounded, None)
            }
        }
        DependencyLookupResult::AvailableNoObservedImports => {
            (Some(0), ImportCountConfidence::Grounded, None)
        }
        DependencyLookupResult::AvailableImportGraphUnavailable => {
            (None, ImportCountConfidence::Unavailable, partial_reason)
        }
        DependencyLookupResult::ScopeUnavailable => {
            (None, ImportCountConfidence::Unavailable, partial_reason)
        }
        DependencyLookupResult::NewPackage => {
            if partial_reason.is_some() {
                (
                    Some(count),
                    ImportCountConfidence::SampleOnly,
                    partial_reason,
                )
            } else {
                (Some(count), ImportCountConfidence::Grounded, None)
            }
        }
    }
}
