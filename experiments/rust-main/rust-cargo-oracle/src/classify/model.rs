use crate::cargo_json::{CargoJsonMessages, CargoJsonStream};
use crate::protocol::{
    ClaimKind, ClassificationRule, CodeKind, CodeNamespace, CodePresence, ConfidenceTier,
    CoverageEffect, DiagnosticCode, Disposition, PrimarySpan, RustcDiagnosticLevel,
    StreamParseStatus,
};
use crate::rustc_span::RustcSuggestionSpan;

#[derive(Debug, Clone)]
pub(crate) struct Classification {
    pub(crate) disposition: Disposition,
    pub(crate) confidence: Option<ConfidenceTier>,
    pub(crate) claim_kind: Option<ClaimKind>,
    pub(crate) coverage_effect: Option<CoverageEffect>,
    pub(crate) rule: ClassificationRule,
}

#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    pub(crate) level: Option<RustcDiagnosticLevel>,
    pub(crate) raw_code: DiagnosticCode,
    pub(crate) code_presence: CodePresence,
    pub(crate) code_value: Option<String>,
    pub(crate) code_namespace: CodeNamespace,
    pub(crate) code_kind: CodeKind,
    pub(crate) primary_spans: Vec<PrimarySpan>,
    pub(crate) suggestion_candidate_spans: Vec<RustcSuggestionSpan>,
    pub(crate) classification: Classification,
    pub(crate) message: Option<String>,
    pub(crate) rendered_first_line: Option<String>,
}

impl Diagnostic {
    pub(crate) fn is_warning_level(&self) -> bool {
        self.level
            .as_ref()
            .is_some_and(RustcDiagnosticLevel::is_warning)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedJsonl {
    messages: CargoJsonStream,
    invalid_json_line_count: usize,
    stream_parse_status: StreamParseStatus,
}

impl ParsedJsonl {
    pub(crate) fn new(
        messages: CargoJsonStream,
        invalid_json_line_count: usize,
        stream_parse_status: StreamParseStatus,
    ) -> Self {
        Self {
            messages,
            invalid_json_line_count,
            stream_parse_status,
        }
    }

    pub(crate) fn skipped() -> Self {
        Self::new(CargoJsonStream::empty(), 0, StreamParseStatus::NotRun)
    }

    pub(crate) fn messages(&self) -> CargoJsonMessages<'_> {
        self.messages.as_messages()
    }

    pub(crate) fn invalid_json_line_count(&self) -> usize {
        self.invalid_json_line_count
    }

    pub(crate) fn stream_parse_status(&self) -> StreamParseStatus {
        self.stream_parse_status
    }

    #[cfg(test)]
    pub(crate) fn message_count(&self) -> usize {
        self.messages.len()
    }

    #[cfg(test)]
    pub(crate) fn has_no_messages(&self) -> bool {
        self.messages.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn contains_reason(&self, expected: crate::cargo_json::CargoJsonReason) -> bool {
        self.messages().contains_reason(expected)
    }
}
