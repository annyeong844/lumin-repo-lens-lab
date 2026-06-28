use crate::classify::{Classification, Diagnostic};
use crate::protocol::{
    ClassificationEvidence, DiagnosticEvidence, NormalizedDiagnostic, PrimarySpan,
};

pub(super) fn diagnostics_to_json(diagnostics: Vec<Diagnostic>) -> Vec<DiagnosticEvidence> {
    diagnostics
        .into_iter()
        .map(|diagnostic| DiagnosticEvidence {
            level: diagnostic.level,
            raw_code: diagnostic.raw_code,
            normalized: NormalizedDiagnostic {
                code_presence: diagnostic.code_presence,
                code_value: diagnostic.code_value,
                code_namespace: diagnostic.code_namespace,
                code_kind: diagnostic.code_kind,
                primary_span: PrimarySpan::representative_class(&diagnostic.primary_spans),
            },
            classification: classification_json(diagnostic.classification),
            message: diagnostic.message,
            primary_spans: diagnostic.primary_spans,
            rendered_first_line: diagnostic.rendered_first_line,
        })
        .collect()
}

fn classification_json(classification: Classification) -> ClassificationEvidence {
    ClassificationEvidence {
        disposition: classification.disposition,
        confidence: classification.confidence,
        claim_kind: classification.claim_kind,
        coverage_effect: classification.coverage_effect,
        rule: classification.rule,
    }
}
