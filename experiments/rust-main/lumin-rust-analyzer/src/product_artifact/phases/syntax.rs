use lumin_rust_source_health::protocol::{
    HealthResponse, ParserEdition, ParserEditionPolicy, ParserEditionSource, ParserKind,
    PolicyMeta as SyntaxPolicyMeta, ResponseMeta as SyntaxMeta, RuntimeMeta as SyntaxRuntimeMeta,
    SidecarMeta as SyntaxSidecarMeta, SignalPolicyMeta as SyntaxSignalPolicyMeta, SkippedFile,
    SourceHealthLimit, SourceHealthMode, SourceHealthProducer, Summary as SyntaxSummary,
};
use serde::Serialize;

use crate::policy::{RawLaneOmitted, SKIPPED_FILE_SAMPLE_LIMIT};
use crate::product_artifact::meta::{ArtifactLane, EmbeddedLane};

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
        summary: SyntaxPhaseSummaryBrief::from_summary(&syntax.summary),
        skipped_file_count: syntax.skipped_files.len(),
        skipped_file_examples: &syntax.skipped_files
            [..syntax.skipped_files.len().min(SKIPPED_FILE_SAMPLE_LIMIT)],
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseSummaryBrief {
    files: usize,
    skipped_files: usize,
    parse_error_files: usize,
    parse_errors: usize,
    review_signals: usize,
    muted_signals: usize,
    definitions: usize,
    impl_blocks: usize,
    impl_methods: usize,
    use_trees: usize,
    path_refs: usize,
    method_call_sites: usize,
    method_calls: usize,
    macro_calls: usize,
    cfg_gates: usize,
    opaque_surfaces: usize,
    review_opaque_surfaces: usize,
    muted_opaque_surfaces: usize,
}

impl SyntaxPhaseSummaryBrief {
    fn from_summary(summary: &SyntaxSummary) -> Self {
        Self {
            files: summary.files,
            skipped_files: summary.skipped_files,
            parse_error_files: summary.parse_error_files,
            parse_errors: summary.parse_errors,
            review_signals: summary.review_signals,
            muted_signals: summary.muted_signals,
            definitions: summary.definitions,
            impl_blocks: summary.impl_blocks,
            impl_methods: summary.impl_methods,
            use_trees: summary.use_trees,
            path_refs: summary.path_refs,
            method_call_sites: summary.method_call_sites,
            method_calls: summary.method_calls,
            macro_calls: summary.macro_calls,
            cfg_gates: summary.cfg_gates,
            opaque_surfaces: summary.opaque_surfaces,
            review_opaque_surfaces: summary.review_opaque_surfaces,
            muted_opaque_surfaces: summary.muted_opaque_surfaces,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseMetaBrief<'a> {
    producer: SourceHealthProducer,
    mode: SourceHealthMode,
    parser: SyntaxPhaseParserBrief<'a>,
    policy: SyntaxPhasePolicyBrief<'a>,
    runtime: SyntaxPhaseRuntimeBrief,
    limits: [SourceHealthLimit; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    sidecar: Option<SyntaxPhaseSidecarBrief<'a>>,
}

impl<'a> SyntaxPhaseMetaBrief<'a> {
    fn from_meta(meta: &'a SyntaxMeta) -> Self {
        Self {
            producer: meta.producer,
            mode: meta.mode,
            parser: SyntaxPhaseParserBrief::from_meta(meta),
            policy: SyntaxPhasePolicyBrief::from_meta(&meta.policy),
            runtime: SyntaxPhaseRuntimeBrief::from_meta(&meta.runtime),
            limits: meta.limits,
            sidecar: meta
                .sidecar
                .as_ref()
                .map(SyntaxPhaseSidecarBrief::from_meta),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseParserBrief<'a> {
    kind: ParserKind,
    version: &'a str,
    edition_policy: ParserEditionPolicy,
    edition: ParserEdition,
    edition_source: ParserEditionSource,
}

impl<'a> SyntaxPhaseParserBrief<'a> {
    fn from_meta(meta: &'a SyntaxMeta) -> Self {
        Self {
            kind: meta.parser.kind,
            version: meta.parser.version.as_str(),
            edition_policy: meta.parser.edition_policy,
            edition: meta.parser.edition,
            edition_source: meta.parser.edition_source,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhasePolicyBrief<'a> {
    version: &'a str,
    signal_policy: SyntaxPhaseSignalPolicyBrief<'a>,
}

impl<'a> SyntaxPhasePolicyBrief<'a> {
    fn from_meta(meta: &'a SyntaxPolicyMeta) -> Self {
        Self {
            version: meta.version.as_str(),
            signal_policy: SyntaxPhaseSignalPolicyBrief::from_meta(&meta.signal_policy),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseSignalPolicyBrief<'a> {
    id: &'a str,
    version: &'a str,
}

impl<'a> SyntaxPhaseSignalPolicyBrief<'a> {
    fn from_meta(meta: &'a SyntaxSignalPolicyMeta) -> Self {
        Self {
            id: meta.id.as_str(),
            version: meta.version.as_str(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseRuntimeBrief {
    thread_count: usize,
    worker_stack_bytes: usize,
}

impl SyntaxPhaseRuntimeBrief {
    fn from_meta(meta: &SyntaxRuntimeMeta) -> Self {
        Self {
            thread_count: meta.thread_count,
            worker_stack_bytes: meta.worker_stack_bytes,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SyntaxPhaseSidecarBrief<'a> {
    source_commit: &'a str,
    binary_sha256: &'a str,
}

impl<'a> SyntaxPhaseSidecarBrief<'a> {
    fn from_meta(meta: &'a SyntaxSidecarMeta) -> Self {
        Self {
            source_commit: meta.source_commit.as_str(),
            binary_sha256: meta.binary_sha256.as_str(),
        }
    }
}
