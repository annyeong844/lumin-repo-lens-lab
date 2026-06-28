mod groups;
mod model;
mod source;

use lumin_rust_source_health::protocol::HealthResponse;

use crate::prewrite::intent::NormalizedIntent;

use super::UnavailableEvidence;
use groups::matching_groups;
pub(in crate::prewrite) use model::{InlinePatternGroup, InlinePatternLookup};
use source::refactor_sources_unavailable;

pub(in crate::prewrite) const INLINE_PATTERN_POLICY_ID: &str = "inline-pattern-policy";
pub(in crate::prewrite) const INLINE_PATTERN_POLICY_VERSION: &str = "inline-pattern-policy-v1";
pub(in crate::prewrite) const INLINE_PATTERN_MIN_OCCURRENCES: usize = 3;
const SOURCE_UNAVAILABLE_CITATION: &str =
    "[확인 불가, rust-source-health lacks a parsed refactor source; inline extraction cues unavailable]";

pub(in crate::prewrite) fn lookup_inline_patterns(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
) -> Vec<InlinePatternLookup> {
    if !intent.has_refactor_sources() {
        return Vec::new();
    }
    if refactor_sources_unavailable(intent, syntax) {
        return vec![unavailable_lookup()];
    }

    let refactor_source_count = intent.refactor_sources().len();
    let groups = matching_groups(intent, syntax);
    if groups.is_empty() {
        return vec![InlinePatternLookup::no_match(
            groups,
            vec![
                "[grounded, rust-source-health inlinePatterns present; no pattern group intersects refactorSources]"
                    .to_string(),
            ],
        )];
    }

    vec![InlinePatternLookup::matched(
        groups,
        vec![format!(
            "[grounded, rust-source-health inlinePatterns intersect {} refactor source{}]",
            refactor_source_count,
            if refactor_source_count == 1 { "" } else { "s" }
        )],
    )]
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

fn unavailable_lookup() -> InlinePatternLookup {
    InlinePatternLookup::unavailable(
        "source-unavailable",
        "rust-source-health",
        vec![SOURCE_UNAVAILABLE_CITATION.to_string()],
    )
}
