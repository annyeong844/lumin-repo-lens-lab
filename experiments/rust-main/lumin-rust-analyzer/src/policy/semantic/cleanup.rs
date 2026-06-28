use crate::policy::{semantic::FindingActionRecord, CleanupCandidate};

pub(in crate::policy) fn cleanup_candidates<'a>(
    records: &[FindingActionRecord<'a>],
) -> Vec<CleanupCandidate<'a>> {
    records
        .iter()
        .filter(|record| {
            record.action.has_safe_action
                || !record.action.action_blockers.is_empty()
                || record.action.is_review
        })
        .filter_map(cleanup_candidate)
        .collect()
}

fn cleanup_candidate<'a>(record: &FindingActionRecord<'a>) -> Option<CleanupCandidate<'a>> {
    let finding = record.finding;
    let action = finding.safe_action.as_ref();
    let edit = action.and_then(|action| action.edits.first());
    let file = edit
        .map(|edit| edit.file_name.as_str())
        .or_else(|| finding.span.as_ref()?.file_name.as_deref())?;
    let line_start = edit
        .map(|edit| edit.line_start)
        .or_else(|| finding.span.as_ref().and_then(|span| span.line_start));
    let diagnostic_code = action
        .map(|action| action.proof.diagnostic_code.as_str())
        .or(finding.diagnostic_code.as_deref());
    Some(CleanupCandidate::new(file, diagnostic_code, line_start))
}
