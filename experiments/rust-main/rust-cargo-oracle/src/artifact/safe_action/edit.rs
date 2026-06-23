use crate::protocol::{
    ActionBlockerReason, PrimarySpan, RustcSuggestionApplicability, SafeActionEdit,
};
use crate::rustc_span::RustcSuggestionSpan;

pub(super) enum CandidateSpanDecision {
    NotMachineApplicable,
    Edit(SafeActionEdit),
    Blocked(Vec<ActionBlockerReason>),
}

struct EditLocation<'a> {
    file_name: &'a str,
    line_start: i64,
    line_end: i64,
    column_start: i64,
    column_end: i64,
}

pub(super) fn candidate_span_decision(
    span: &RustcSuggestionSpan,
    primary_spans: &[PrimarySpan],
) -> CandidateSpanDecision {
    if span.suggestion_applicability() != Some(RustcSuggestionApplicability::MachineApplicable) {
        return CandidateSpanDecision::NotMachineApplicable;
    }
    if span.has_expansion() {
        return CandidateSpanDecision::Blocked(vec![ActionBlockerReason::MacroExpansion]);
    }
    if !span_matches_user_primary(span, primary_spans) {
        return CandidateSpanDecision::Blocked(vec![ActionBlockerReason::NonUserCodePrimary]);
    }

    candidate_edit_decision(span)
}

pub(super) fn has_overlapping_edits(edits: &[SafeActionEdit]) -> bool {
    for (index, left) in edits.iter().enumerate() {
        for right in edits.iter().skip(index + 1) {
            if left.file_name == right.file_name && edits_overlap(left, right) {
                return true;
            }
        }
    }
    false
}

fn span_matches_user_primary(span: &RustcSuggestionSpan, primary_spans: &[PrimarySpan]) -> bool {
    primary_spans.iter().any(|primary| {
        primary.is_user_code_without_expansion()
            && (primary.same_position_as_suggestion_span(span)
                || primary.contains_suggestion_span(span)
                || primary.is_contained_by_suggestion_span(span))
    })
}

fn candidate_edit_decision(span: &RustcSuggestionSpan) -> CandidateSpanDecision {
    let mut blockers = Vec::new();
    let replacement = span.suggested_replacement();
    if replacement.is_none() {
        blockers.push(ActionBlockerReason::MissingSuggestedReplacement);
    }

    let location = edit_location(span);
    if location.is_none() {
        blockers.push(ActionBlockerReason::InvalidEditRange);
    }

    let (Some(location), Some(replacement)) = (location, replacement) else {
        return CandidateSpanDecision::Blocked(blockers);
    };

    CandidateSpanDecision::Edit(SafeActionEdit {
        file_name: location.file_name.to_string(),
        line_start: location.line_start,
        line_end: location.line_end,
        column_start: location.column_start,
        column_end: location.column_end,
        replacement: replacement.to_string(),
    })
}

fn edit_location(span: &RustcSuggestionSpan) -> Option<EditLocation<'_>> {
    let file_name = span.file_name()?;
    let line_start = span.line_start()?;
    let line_end = span.line_end()?;
    let column_start = span.column_start()?;
    let column_end = span.column_end()?;
    if line_start <= 0 || line_end <= 0 || column_start <= 0 || column_end <= 0 {
        return None;
    }
    if (line_start, column_start) > (line_end, column_end) {
        return None;
    }
    Some(EditLocation {
        file_name,
        line_start,
        line_end,
        column_start,
        column_end,
    })
}

fn edits_overlap(left: &SafeActionEdit, right: &SafeActionEdit) -> bool {
    let left_start = (left.line_start, left.column_start);
    let left_end = (left.line_end, left.column_end);
    let right_start = (right.line_start, right.column_start);
    let right_end = (right.line_end, right.column_end);
    left_start < right_end && right_start < left_end
}
