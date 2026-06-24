use serde::Serialize;

use super::super::Location;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstInlinePattern {
    pub kind: AstInlinePatternKind,
    pub pattern_hash: String,
    pub normalized_pattern: String,
    pub normalized_version: &'static str,
    pub statement_count: usize,
    pub enclosing_function: String,
    pub location: Location,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AstInlinePatternKind {
    StatementSequence,
}
