use super::RustcSpan;
use crate::protocol::RustcSuggestionApplicability;

#[derive(Debug, Clone)]
pub(crate) struct RustcSuggestionSpan {
    file_name: Option<String>,
    line_start: Option<i64>,
    line_end: Option<i64>,
    column_start: Option<i64>,
    column_end: Option<i64>,
    suggestion_applicability: Option<RustcSuggestionApplicability>,
    suggested_replacement: Option<String>,
    has_expansion: bool,
}

impl RustcSuggestionSpan {
    pub(crate) fn from_rustc_span(span: &RustcSpan) -> Self {
        Self {
            file_name: span.file_name().map(str::to_string),
            line_start: span.line_start(),
            line_end: span.line_end(),
            column_start: span.column_start(),
            column_end: span.column_end(),
            suggestion_applicability: span.suggestion_applicability(),
            suggested_replacement: span.suggested_replacement().map(str::to_string),
            has_expansion: span.has_expansion(),
        }
    }

    pub(crate) fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    pub(crate) fn line_start(&self) -> Option<i64> {
        self.line_start
    }

    pub(crate) fn line_end(&self) -> Option<i64> {
        self.line_end
    }

    pub(crate) fn column_start(&self) -> Option<i64> {
        self.column_start
    }

    pub(crate) fn column_end(&self) -> Option<i64> {
        self.column_end
    }

    pub(crate) fn suggestion_applicability(&self) -> Option<RustcSuggestionApplicability> {
        self.suggestion_applicability
    }

    pub(crate) fn suggested_replacement(&self) -> Option<&str> {
        self.suggested_replacement.as_deref()
    }

    pub(crate) fn has_expansion(&self) -> bool {
        self.has_expansion
    }
}
