use lumin_rust_source_health::protocol::HealthResponse;

use crate::prewrite::intent::{NormalizedIntent, RefactorSource};

use super::model::InlinePatternOccurrence;

pub(super) fn refactor_sources_unavailable(
    intent: &NormalizedIntent,
    syntax: &HealthResponse,
) -> bool {
    intent.refactor_sources().iter().any(|source| {
        syntax
            .files
            .get(&source.file)
            .is_none_or(|file| !file.parse.ok)
    })
}

pub(super) fn occurrence_matches_source(
    occurrence: &InlinePatternOccurrence,
    source: &RefactorSource,
) -> bool {
    occurrence.file == source.file && source_line_intersects_occurrence(source, occurrence)
}

fn source_line_intersects_occurrence(
    source: &RefactorSource,
    occurrence: &InlinePatternOccurrence,
) -> bool {
    let Some(lines) = &source.lines else {
        return true;
    };
    lines.iter().any(|line| {
        let line = *line as usize;
        line >= occurrence.line && line <= occurrence.end_line
    })
}
