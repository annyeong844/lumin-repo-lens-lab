use crate::protocol::{PrimarySpan, PrimarySpanClass, PrimarySpanExpansion, PrimarySpanLocation};

use super::{RustcSpan, RustcSuggestionSpan};

impl PrimarySpan {
    pub(crate) fn from_rustc_span(span: &RustcSpan, primary_span_class: PrimarySpanClass) -> Self {
        let expansion = PrimarySpanExpansion::from_rustc_expansion(span);
        Self {
            file_name: span.file_name().map(str::to_string),
            line_start: span.line_start(),
            line_end: span.line_end(),
            column_start: span.column_start(),
            column_end: span.column_end(),
            has_expansion: expansion.is_some(),
            expansion,
            primary_span_class,
        }
    }

    pub(crate) fn is_user_code_without_expansion(&self) -> bool {
        self.is_user_code() && !self.has_expansion
    }

    pub(crate) fn same_position_as_suggestion_span(&self, span: &RustcSuggestionSpan) -> bool {
        self.file_name.as_deref() == span.file_name()
            && self.line_start == span.line_start()
            && self.line_end == span.line_end()
            && self.column_start == span.column_start()
            && self.column_end == span.column_end()
    }

    pub(crate) fn contains_suggestion_span(&self, span: &RustcSuggestionSpan) -> bool {
        if self.file_name.as_deref() != span.file_name() {
            return false;
        }
        let Some(line_start) = span.line_start() else {
            return false;
        };
        let Some(line_end) = span.line_end() else {
            return false;
        };
        let Some(column_start) = span.column_start() else {
            return false;
        };
        let Some(column_end) = span.column_end() else {
            return false;
        };
        let Some(primary_line_start) = self.line_start else {
            return false;
        };
        let Some(primary_line_end) = self.line_end else {
            return false;
        };
        let Some(primary_column_start) = self.column_start else {
            return false;
        };
        let Some(primary_column_end) = self.column_end else {
            return false;
        };

        (primary_line_start, primary_column_start) <= (line_start, column_start)
            && (line_end, column_end) <= (primary_line_end, primary_column_end)
    }

    pub(crate) fn is_contained_by_suggestion_span(&self, span: &RustcSuggestionSpan) -> bool {
        if self.file_name.as_deref() != span.file_name() {
            return false;
        }
        let Some(line_start) = span.line_start() else {
            return false;
        };
        let Some(line_end) = span.line_end() else {
            return false;
        };
        let Some(column_start) = span.column_start() else {
            return false;
        };
        let Some(column_end) = span.column_end() else {
            return false;
        };
        let Some(primary_line_start) = self.line_start else {
            return false;
        };
        let Some(primary_line_end) = self.line_end else {
            return false;
        };
        let Some(primary_column_start) = self.column_start else {
            return false;
        };
        let Some(primary_column_end) = self.column_end else {
            return false;
        };

        (line_start, column_start) <= (primary_line_start, primary_column_start)
            && (primary_line_end, primary_column_end) <= (line_end, column_end)
    }
}

impl PrimarySpanExpansion {
    fn from_rustc_expansion(span: &RustcSpan) -> Option<Self> {
        let expansion = span.expansion()?;

        Some(Self {
            macro_decl_name: expansion.macro_decl_name().map(str::to_string),
            span: expansion.span().map(PrimarySpanLocation::from_rustc_span),
            def_site_span: expansion
                .def_site_span()
                .map(PrimarySpanLocation::from_rustc_span),
        })
    }
}

impl PrimarySpanLocation {
    fn from_rustc_span(span: &RustcSpan) -> Self {
        Self {
            file_name: span.file_name().map(str::to_string),
            line_start: span.line_start(),
            line_end: span.line_end(),
            column_start: span.column_start(),
            column_end: span.column_end(),
        }
    }
}
