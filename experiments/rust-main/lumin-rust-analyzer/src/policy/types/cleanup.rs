use std::borrow::Cow;

use lumin_rust_common::posix_path_text;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct CleanupCandidate<'a> {
    pub(crate) file: Cow<'a, str>,
    pub(crate) diagnostic_code: Option<&'a str>,
    pub(crate) line_start: Option<i64>,
}

impl<'a> CleanupCandidate<'a> {
    pub(crate) fn new(
        file: &'a str,
        diagnostic_code: Option<&'a str>,
        line_start: Option<i64>,
    ) -> Self {
        Self {
            file: normalize_candidate_file(file),
            diagnostic_code,
            line_start,
        }
    }
}

pub(crate) fn normalize_candidate_file(path: &str) -> Cow<'_, str> {
    posix_path_text(path)
}
