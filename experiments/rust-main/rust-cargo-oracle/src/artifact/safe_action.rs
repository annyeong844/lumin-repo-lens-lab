use crate::classify::Diagnostic;
use crate::protocol::{
    ActionBlockerReason, ConfidenceTier, OracleId, PrimarySpanClass, RustcSuggestionApplicability,
    SafeAction, SafeActionEdit, SafeActionKind, SafeActionProof,
};

mod edit;

use edit::{candidate_span_decision, has_overlapping_edits, CandidateSpanDecision};

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
