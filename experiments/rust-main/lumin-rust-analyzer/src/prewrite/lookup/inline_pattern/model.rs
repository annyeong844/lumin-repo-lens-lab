use lumin_rust_source_health::protocol::{AstInlinePattern, AstInlinePatternKind};
use serde::Serialize;

use crate::prewrite::intent::RefactorSource;

use super::UnavailableEvidence;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct InlinePatternLookup {
    kind: InlinePatternLookupKind,
    result: InlinePatternLookupResult,
    groups: Vec<InlinePatternGroup>,
    citations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact: Option<&'static str>,
}

impl InlinePatternLookup {
    pub(super) fn matched(groups: Vec<InlinePatternGroup>, citations: Vec<String>) -> Self {
        Self {
            kind: InlinePatternLookupKind::InlinePattern,
            result: InlinePatternLookupResult::InlinePatternMatch,
            groups,
            citations,
            reason: None,
            artifact: None,
        }
    }

    pub(super) fn no_match(groups: Vec<InlinePatternGroup>, citations: Vec<String>) -> Self {
        Self {
            kind: InlinePatternLookupKind::InlinePattern,
            result: InlinePatternLookupResult::NoInlinePatternMatch,
            groups,
            citations,
            reason: None,
            artifact: None,
        }
    }

    pub(super) fn unavailable(
        reason: &'static str,
        artifact: &'static str,
        citations: Vec<String>,
    ) -> Self {
        Self {
            kind: InlinePatternLookupKind::InlinePattern,
            result: InlinePatternLookupResult::Unavailable,
            groups: Vec::new(),
            citations,
            reason: Some(reason),
            artifact: Some(artifact),
        }
    }

    pub(in crate::prewrite) fn unavailable_evidence(&self) -> UnavailableEvidence {
        UnavailableEvidence::inline_extraction(
            self.reason.unwrap_or("lookup-unavailable"),
            self.artifact.unwrap_or("rust-source-health"),
            self.citations.clone(),
        )
    }

    pub(in crate::prewrite) fn is_unavailable(&self) -> bool {
        self.result == InlinePatternLookupResult::Unavailable
    }

    pub(in crate::prewrite) fn is_match(&self) -> bool {
        self.result == InlinePatternLookupResult::InlinePatternMatch
    }

    pub(in crate::prewrite) fn is_no_match(&self) -> bool {
        self.result == InlinePatternLookupResult::NoInlinePatternMatch
    }

    pub(in crate::prewrite) fn groups(&self) -> &[InlinePatternGroup] {
        &self.groups
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum InlinePatternLookupKind {
    #[serde(rename = "inline-pattern")]
    InlinePattern,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
enum InlinePatternLookupResult {
    #[serde(rename = "INLINE_PATTERN_MATCH")]
    InlinePatternMatch,
    #[serde(rename = "NO_INLINE_PATTERN_MATCH")]
    NoInlinePatternMatch,
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct InlinePatternGroup {
    pub(in crate::prewrite) pattern_hash: String,
    pub(in crate::prewrite) kind: AstInlinePatternKind,
    pub(in crate::prewrite) size: usize,
    pub(in crate::prewrite) owner_files: Vec<String>,
    pub(in crate::prewrite) normalized_pattern: String,
    pub(in crate::prewrite) normalizer_version: &'static str,
    pub(in crate::prewrite) occurrences: Vec<InlinePatternOccurrence>,
    pub(in crate::prewrite) review_reason: &'static str,
    pub(in crate::prewrite) refactor_sources: Vec<RefactorSource>,
}

impl InlinePatternGroup {
    pub(in crate::prewrite) fn identity(&self) -> String {
        format!("inline-pattern:{}", self.pattern_hash)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct InlinePatternOccurrence {
    pub(in crate::prewrite) file: String,
    pub(in crate::prewrite) line: usize,
    pub(in crate::prewrite) end_line: usize,
    pub(in crate::prewrite) enclosing_function: String,
}

pub(super) struct GroupBuilder<'a> {
    pub(super) pattern: &'a AstInlinePattern,
    pub(super) occurrences: Vec<InlinePatternOccurrence>,
}
