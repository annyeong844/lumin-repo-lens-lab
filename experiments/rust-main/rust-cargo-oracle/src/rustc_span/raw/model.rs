use crate::protocol::RustcSuggestionApplicability;

#[derive(Debug, Clone)]
pub(crate) struct RustcSpan {
    pub(super) file_name: Option<String>,
    pub(super) line_start: Option<i64>,
    pub(super) line_end: Option<i64>,
    pub(super) column_start: Option<i64>,
    pub(super) column_end: Option<i64>,
    pub(super) is_primary: bool,
    pub(super) suggestion_applicability: Option<RustcSuggestionApplicability>,
    pub(super) suggested_replacement: Option<String>,
    pub(super) has_suggestion_applicability_field: bool,
    pub(super) has_suggested_replacement_field: bool,
    pub(super) has_expansion: bool,
    pub(super) expansion: Option<Box<RustcExpansion>>,
}

#[derive(Debug, Clone)]
pub(in crate::rustc_span) struct RustcExpansion {
    pub(super) macro_decl_name: Option<String>,
    pub(super) span: Option<RustcSpan>,
    pub(super) def_site_span: Option<RustcSpan>,
}

impl RustcSpan {
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

    pub(crate) fn is_primary(&self) -> bool {
        self.is_primary
    }

    pub(crate) fn suggestion_applicability(&self) -> Option<RustcSuggestionApplicability> {
        self.suggestion_applicability
    }

    pub(crate) fn suggested_replacement(&self) -> Option<&str> {
        self.suggested_replacement.as_deref()
    }

    pub(crate) fn has_suggestion_payload(&self) -> bool {
        self.has_suggestion_applicability_field || self.has_suggested_replacement_field
    }

    pub(in crate::rustc_span) fn expansion(&self) -> Option<&RustcExpansion> {
        self.expansion.as_deref()
    }

    pub(crate) fn has_expansion(&self) -> bool {
        self.has_expansion
    }

    pub(crate) fn expansion_callsite_file_names(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = self.expansion();
        while let Some(expansion) = current {
            let Some(callsite) = expansion.span() else {
                break;
            };
            if let Some(file_name) = callsite.file_name() {
                out.push(file_name.to_string());
            }
            current = callsite.expansion();
        }
        out
    }
}

impl RustcExpansion {
    pub(in crate::rustc_span) fn macro_decl_name(&self) -> Option<&str> {
        self.macro_decl_name.as_deref()
    }

    pub(in crate::rustc_span) fn span(&self) -> Option<&RustcSpan> {
        self.span.as_ref()
    }

    pub(in crate::rustc_span) fn def_site_span(&self) -> Option<&RustcSpan> {
        self.def_site_span.as_ref()
    }
}
