use crate::classify::Diagnostic;
use crate::protocol::{
    ActionBlockerReason, ConfidenceTier, OracleId, PrimarySpan, PrimarySpanClass,
    RustcSuggestionApplicability, SafeAction, SafeActionEdit, SafeActionKind, SafeActionProof,
};
use crate::rustc_span::RustcSuggestionSpan;

pub(super) struct SafeActionDecision {
    pub(super) safe_action: Option<SafeAction>,
    pub(super) action_blockers: Vec<ActionBlockerReason>,
}

enum SafeActionAnalysis<'a> {
    Ineligible,
    Blocked(Vec<ActionBlockerReason>),
    Eligible {
        diagnostic_code: &'a str,
        edits: Vec<SafeActionEdit>,
    },
}

enum CandidateSpanDecision {
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

pub(super) fn safe_action_decision(diagnostic: &Diagnostic) -> SafeActionDecision {
    match safe_action_analysis(diagnostic) {
        SafeActionAnalysis::Ineligible => SafeActionDecision {
            safe_action: None,
            action_blockers: Vec::new(),
        },
        SafeActionAnalysis::Blocked(mut action_blockers) => {
            action_blockers.sort();
            action_blockers.dedup();
            SafeActionDecision {
                safe_action: None,
                action_blockers,
            }
        }
        SafeActionAnalysis::Eligible {
            diagnostic_code,
            edits,
        } => SafeActionDecision {
            safe_action: Some(safe_action_from_edits(diagnostic_code, edits)),
            action_blockers: Vec::new(),
        },
    }
}

fn safe_action_analysis(diagnostic: &Diagnostic) -> SafeActionAnalysis<'_> {
    let mut blockers = Vec::new();
    let mut edits = Vec::new();
    let Some(diagnostic_code) = diagnostic.code_value.as_deref() else {
        return SafeActionAnalysis::Ineligible;
    };

    let candidate_spans = &diagnostic.suggestion_candidate_spans;
    if candidate_spans.is_empty() {
        if is_rule_backed_warning(diagnostic) {
            return SafeActionAnalysis::Blocked(vec![
                ActionBlockerReason::MissingMachineApplicableSuggestion,
            ]);
        }
        return SafeActionAnalysis::Ineligible;
    }

    if !diagnostic.is_warning_level() {
        blockers.push(ActionBlockerReason::DiagnosticLevelNotWarning);
    }
    if !matches!(
        diagnostic.classification.confidence,
        Some(ConfidenceTier::RuleBacked)
    ) {
        blockers.push(ActionBlockerReason::DiagnosticNotRuleBacked);
    }

    let mut saw_machine_applicable = false;
    for span in candidate_spans {
        match candidate_span_decision(span, &diagnostic.primary_spans) {
            CandidateSpanDecision::NotMachineApplicable => {}
            CandidateSpanDecision::Edit(edit) => {
                saw_machine_applicable = true;
                edits.push(edit);
            }
            CandidateSpanDecision::Blocked(reasons) => {
                saw_machine_applicable = true;
                blockers.extend(reasons);
            }
        }
    }

    if !saw_machine_applicable {
        blockers.push(ActionBlockerReason::MissingMachineApplicableSuggestion);
    }
    if edits.is_empty() && blockers.is_empty() {
        blockers.push(ActionBlockerReason::MissingSafeEdit);
    }
    if has_overlapping_edits(&edits) {
        blockers.push(ActionBlockerReason::OverlappingEdits);
    }

    if !blockers.is_empty() {
        return SafeActionAnalysis::Blocked(blockers);
    }

    edits.sort_by(|left, right| {
        left.file_name
            .cmp(&right.file_name)
            .then(left.line_start.cmp(&right.line_start))
            .then(left.column_start.cmp(&right.column_start))
    });

    SafeActionAnalysis::Eligible {
        diagnostic_code,
        edits,
    }
}

fn is_rule_backed_warning(diagnostic: &Diagnostic) -> bool {
    diagnostic.is_warning_level()
        && matches!(
            diagnostic.classification.confidence,
            Some(ConfidenceTier::RuleBacked)
        )
}

fn span_matches_user_primary(span: &RustcSuggestionSpan, primary_spans: &[PrimarySpan]) -> bool {
    primary_spans.iter().any(|primary| {
        primary.is_user_code_without_expansion()
            && (primary.same_position_as_suggestion_span(span)
                || primary.contains_suggestion_span(span)
                || primary.is_contained_by_suggestion_span(span))
    })
}

fn candidate_span_decision(
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

fn has_overlapping_edits(edits: &[SafeActionEdit]) -> bool {
    for (index, left) in edits.iter().enumerate() {
        for right in edits.iter().skip(index + 1) {
            if left.file_name == right.file_name && edits_overlap(left, right) {
                return true;
            }
        }
    }
    false
}

fn edits_overlap(left: &SafeActionEdit, right: &SafeActionEdit) -> bool {
    let left_start = (left.line_start, left.column_start);
    let left_end = (left.line_end, left.column_end);
    let right_start = (right.line_start, right.column_start);
    let right_end = (right.line_end, right.column_end);
    left_start < right_end && right_start < left_end
}

fn safe_action_from_edits(diagnostic_code: &str, edits: Vec<SafeActionEdit>) -> SafeAction {
    SafeAction {
        kind: SafeActionKind::ApplyRustcMachineApplicableSuggestion,
        proof_complete: true,
        action_blockers: Vec::new(),
        stronger_action_blockers: Vec::new(),
        edits,
        proof: SafeActionProof {
            oracle_id: OracleId::RustCargoCheck,
            diagnostic_code: diagnostic_code.to_string(),
            applicability: RustcSuggestionApplicability::MachineApplicable,
            primary_span_class: PrimarySpanClass::UserCode,
            no_macro_expansion: true,
        },
    }
}
