use serde::Serialize;

use crate::prewrite::intent::NormalizedIntent;

use super::UnavailableEvidence;

const INLINE_PATTERN_UNAVAILABLE_CITATION: &str =
    "[확인 불가, inline-patterns.json absent; inline extraction cues unavailable]";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::prewrite) struct InlinePatternLookup {
    kind: InlinePatternLookupKind,
    result: InlinePatternLookupResult,
    reason: &'static str,
    artifact: &'static str,
    citations: Vec<&'static str>,
}

impl InlinePatternLookup {
    pub(in crate::prewrite) fn unavailable_evidence(&self) -> UnavailableEvidence {
        UnavailableEvidence::inline_extraction(self.reason, self.artifact, self.citations.clone())
    }

    pub(in crate::prewrite) fn is_unavailable(&self) -> bool {
        self.result == InlinePatternLookupResult::Unavailable
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
enum InlinePatternLookupKind {
    #[serde(rename = "inline-pattern")]
    InlinePattern,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
enum InlinePatternLookupResult {
    #[serde(rename = "UNAVAILABLE")]
    Unavailable,
}

pub(in crate::prewrite) fn lookup_inline_patterns(
    intent: &NormalizedIntent,
) -> Vec<InlinePatternLookup> {
    if !intent.has_refactor_sources() {
        return Vec::new();
    }
    vec![InlinePatternLookup {
        kind: InlinePatternLookupKind::InlinePattern,
        result: InlinePatternLookupResult::Unavailable,
        reason: "missing-artifact",
        artifact: "inline-patterns.json",
        citations: vec![INLINE_PATTERN_UNAVAILABLE_CITATION],
    }]
}

pub(in crate::prewrite) fn unavailable_evidence_from_inline_pattern_lookups(
    lookups: &[InlinePatternLookup],
) -> Vec<UnavailableEvidence> {
    lookups
        .iter()
        .filter(|lookup| lookup.is_unavailable())
        .map(InlinePatternLookup::unavailable_evidence)
        .collect()
}
