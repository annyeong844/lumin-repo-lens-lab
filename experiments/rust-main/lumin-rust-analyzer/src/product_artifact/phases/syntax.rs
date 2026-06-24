use lumin_rust_source_health::protocol::{HealthResponse, SkippedFile};
use serde::Serialize;

use crate::policy::{RawLaneOmitted, SKIPPED_FILE_SAMPLE_LIMIT};
use crate::product_artifact::meta::{ArtifactLane, EmbeddedLane};

use meta::SyntaxPhaseMetaBrief;
use summary::SyntaxPhaseSummaryBrief;

mod meta;
mod summary;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::product_artifact) struct SyntaxPhaseBrief<'a> {
    artifact: ArtifactLane,
    embedded: EmbeddedLane,
    raw_embedded: RawLaneOmitted,
    elapsed_ms: u128,
    schema_version: u32,
    meta: SyntaxPhaseMetaBrief<'a>,
    summary: SyntaxPhaseSummaryBrief,
    skipped_file_count: usize,
    skipped_file_examples: &'a [SkippedFile],
}

pub(in crate::product_artifact) fn syntax_phase_brief<'a>(
    syntax: &'a HealthResponse,
    elapsed_ms: u128,
) -> SyntaxPhaseBrief<'a> {
    SyntaxPhaseBrief {
        artifact: ArtifactLane::RustSourceHealth,
        embedded: EmbeddedLane::Brief,
        raw_embedded: RawLaneOmitted,
        elapsed_ms,
        schema_version: syntax.schema_version,
        meta: SyntaxPhaseMetaBrief::from_meta(&syntax.meta),
        summary: SyntaxPhaseSummaryBrief::from_syntax(syntax),
        skipped_file_count: syntax.skipped_files.len(),
        skipped_file_examples: &syntax.skipped_files
            [..syntax.skipped_files.len().min(SKIPPED_FILE_SAMPLE_LIMIT)],
    }
}
