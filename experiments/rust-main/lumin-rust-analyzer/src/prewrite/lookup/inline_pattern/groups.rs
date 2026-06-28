use std::collections::{BTreeMap, BTreeSet};

use lumin_rust_source_health::protocol::{AstInlinePattern, HealthResponse};

use crate::prewrite::intent::{NormalizedIntent, RefactorSource};

use super::model::{GroupBuilder, InlinePatternGroup, InlinePatternOccurrence};
use super::source::occurrence_matches_source;
use super::INLINE_PATTERN_MIN_OCCURRENCES;

pub(super) fn matching_groups(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
) -> Vec<InlinePatternGroup> {
    let mut by_hash = BTreeMap::<String, GroupBuilder<'_>>::new();
    for (file, health) in &syntax.files {
        for pattern in &health.ast.inline_patterns {
            by_hash
                .entry(pattern.pattern_hash.clone())
                .and_modify(|group| group.occurrences.push(occurrence(file, pattern)))
                .or_insert_with(|| GroupBuilder {
                    pattern,
                    occurrences: vec![occurrence(file, pattern)],
                });
        }
    }

    let mut groups = by_hash
        .into_values()
        .filter(|group| group.occurrences.len() >= INLINE_PATTERN_MIN_OCCURRENCES)
        .filter_map(|group| finalize_group(group, intent.refactor_sources()))
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .size
            .cmp(&left.size)
            .then(left.pattern_hash.cmp(&right.pattern_hash))
    });
    groups
}

fn finalize_group(
    mut group: GroupBuilder<'_>,
    refactor_sources: &[RefactorSource],
) -> Option<InlinePatternGroup> {
    group.occurrences.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.line.cmp(&right.line))
            .then(left.end_line.cmp(&right.end_line))
            .then(left.enclosing_function.cmp(&right.enclosing_function))
    });
    let matching_sources = refactor_sources
        .iter()
        .filter(|source| {
            group
                .occurrences
                .iter()
                .any(|occurrence| occurrence_matches_source(occurrence, source))
        })
        .cloned()
        .collect::<Vec<_>>();
    if matching_sources.is_empty() {
        return None;
    }
    let owner_files = group
        .occurrences
        .iter()
        .map(|occurrence| occurrence.file.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Some(InlinePatternGroup {
        pattern_hash: group.pattern.pattern_hash.clone(),
        kind: group.pattern.kind,
        size: group.occurrences.len(),
        owner_files,
        normalized_pattern: group.pattern.normalized_pattern.clone(),
        normalizer_version: group.pattern.normalized_version,
        occurrences: group.occurrences,
        review_reason:
            "same normalized Rust statement block; verify control-flow and ownership before extracting",
        refactor_sources: matching_sources,
    })
}

fn occurrence(file: &str, pattern: &AstInlinePattern) -> InlinePatternOccurrence {
    InlinePatternOccurrence {
        file: file.to_string(),
        line: pattern.location.line,
        end_line: pattern.location.end_line,
        enclosing_function: pattern.enclosing_function.clone(),
    }
}
