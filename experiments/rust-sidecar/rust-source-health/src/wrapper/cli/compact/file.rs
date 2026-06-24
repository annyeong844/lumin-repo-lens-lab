use serde::Serialize;

use super::ast_summary::CompactAstSummary;
use crate::protocol::{Facts, FileHealth, ParseStatus, PathMeta, Signal};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CompactFileHealth<'a> {
    sha256: &'a str,
    facts: &'a Facts,
    ast_summary: CompactAstSummary<'a>,
    signals: &'a [Signal],
    parse: &'a ParseStatus,
    path: &'a PathMeta,
}

impl<'a> CompactFileHealth<'a> {
    pub(super) fn from_file(file: &'a FileHealth) -> Self {
        Self {
            sha256: &file.sha256,
            facts: &file.facts,
            ast_summary: CompactAstSummary::from_ast(&file.ast),
            signals: &file.signals,
            parse: &file.parse,
            path: &file.path,
        }
    }
}
