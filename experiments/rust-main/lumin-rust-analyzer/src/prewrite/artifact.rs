use std::path::Path;

use anyhow::Result;
use lumin_rust_common::posix_path_text;
use lumin_rust_source_health::protocol::HealthResponse;
use serde::Serialize;

mod contract;
mod coverage;
mod meta;

use coverage::IntentLaneCoverage;
use meta::PreWriteMeta;

use super::cues::{self, CueCard, CueProjection, SuppressedCue};
use super::index::CandidateIndex;
use super::intent::{IntentWarning, LoadedIntent, NormalizedIntent};
use super::lookup::{
    self, DependencyLookup, FileLookup, InlinePatternLookup, NameLookup, ShapeLookup,
    UnavailableEvidence,
};
use super::tokens::TOKEN_POLICY_VERSION;

const SCHEMA_VERSION: &str = "rust-pre-write.v1";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreWriteArtifact {
    schema_version: &'static str,
    policy_version: &'static str,
    meta: PreWriteMeta,
    intent: NormalizedIntent,
    intent_warnings: Vec<IntentWarning>,
    coverage: IntentLaneCoverage,
    lookups: Vec<NameLookup>,
    shape_lookups: Vec<ShapeLookup>,
    file_lookups: Vec<FileLookup>,
    dependency_lookups: Vec<DependencyLookup>,
    inline_pattern_lookups: Vec<InlinePatternLookup>,
    cue_cards: Vec<CueCard>,
    suppressed_cues: Vec<SuppressedCue>,
    unavailable_evidence: Vec<UnavailableEvidence>,
}

impl PreWriteArtifact {
    pub(crate) fn to_pretty_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

pub(super) fn build(
    loaded: LoadedIntent,
    syntax: &HealthResponse,
    root: &Path,
) -> Result<PreWriteArtifact> {
    let index = CandidateIndex::from_health(syntax);
    let lookups = lookup::lookup_names(&loaded.intent, &index, syntax);
    let shape_lookups = lookup::lookup_shapes(&loaded.intent, syntax);
    let inline_pattern_lookups = lookup::lookup_inline_patterns(&loaded.intent, syntax);
    let mut unavailable_evidence = lookup::unavailable_evidence_from_shape_lookups(&shape_lookups);
    unavailable_evidence.extend(lookup::unavailable_evidence_from_inline_pattern_lookups(
        &inline_pattern_lookups,
    ));
    let file_lookups = lookup::lookup_files(&loaded.intent, syntax, root);
    let dependency_lookups = lookup::lookup_dependencies(&loaded.intent, syntax, root)?;
    let CueProjection {
        cue_cards,
        suppressed_cues,
    } = cues::project(
        &lookups,
        &shape_lookups,
        &file_lookups,
        &dependency_lookups,
        &inline_pattern_lookups,
    );
    let coverage = IntentLaneCoverage::from_intent(&loaded.intent);
    let artifact = PreWriteArtifact {
        schema_version: SCHEMA_VERSION,
        policy_version: TOKEN_POLICY_VERSION,
        meta: PreWriteMeta::from_syntax(syntax),
        intent: loaded.intent,
        intent_warnings: loaded.warnings,
        coverage,
        lookups,
        shape_lookups,
        file_lookups,
        dependency_lookups,
        inline_pattern_lookups,
        cue_cards,
        suppressed_cues,
        unavailable_evidence,
    };
    artifact.validate_contract()?;
    Ok(artifact)
}

fn intent_has_file(intent: &NormalizedIntent, lookup_file: &str) -> bool {
    intent
        .files
        .iter()
        .any(|file| posix_path_text(file) == lookup_file)
}
