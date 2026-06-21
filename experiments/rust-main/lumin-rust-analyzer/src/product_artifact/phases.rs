use lumin_rust_cargo_oracle::protocol::{
    ArtifactMeta as SemanticMeta, ArtifactProfile, MissingInfluenceKind, RustcCommandSource,
    SemanticArtifactMode, SemanticArtifactProducer, SemanticHealthArtifact,
    Summary as SemanticSummary,
};
use lumin_rust_cargo_oracle::CargoCheckMode;
use lumin_rust_source_health::protocol::{
    HealthResponse, ParserEdition, ParserEditionPolicy, ParserEditionSource, ParserKind,
    PolicyMeta as SyntaxPolicyMeta, ResponseMeta as SyntaxMeta, RuntimeMeta as SyntaxRuntimeMeta,
    SidecarMeta as SyntaxSidecarMeta, SignalPolicyMeta as SyntaxSignalPolicyMeta, SkippedFile,
    SourceHealthLimit, SourceHealthMode, SourceHealthProducer, Summary as SyntaxSummary,
    Thresholds as SyntaxThresholds,
};
use serde::Serialize;

use super::meta::{ArtifactLane, EmbeddedLane};
use crate::policy::{RawLaneOmitted, SKIPPED_FILE_SAMPLE_LIMIT};
use crate::product_files::{
    ProductSemanticDiagnosticsProjection, ProductSemanticFindingsProjection,
};

#[derive(Debug, Serialize)]
pub(super) struct PhaseBriefs<'a> {
    pub(super) syntax: SyntaxPhaseBrief<'a>,
    pub(super) semantic: SemanticPhaseBrief<'a>,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticPhaseCounts {
    finding_count: usize,
    diagnostic_count: usize,
}

impl SemanticPhaseCounts {
    pub(super) fn from_projections(
        findings: &ProductSemanticFindingsProjection<'_>,
        diagnostics: &ProductSemanticDiagnosticsProjection<'_>,
    ) -> Self {
        Self {
            finding_count: findings.len(),
            diagnostic_count: diagnostics.len(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SyntaxPhaseBrief<'a> {
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

pub(super) fn syntax_phase_brief<'a>(
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
    thresholds: SyntaxThresholds,
    signal_policy: SyntaxPhaseSignalPolicyBrief<'a>,
}

impl<'a> SyntaxPhasePolicyBrief<'a> {
    fn from_meta(meta: &'a SyntaxPolicyMeta) -> Self {
        Self {
            version: meta.version.as_str(),
            thresholds: meta.thresholds,
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SemanticPhaseBrief<'a> {
    artifact: ArtifactLane,
    embedded: EmbeddedLane,
    raw_embedded: RawLaneOmitted,
    mode: CargoCheckMode,
    elapsed_ms: u128,
    schema_version: &'static str,
    meta: SemanticPhaseMetaBrief<'a>,
    summary: SemanticPhaseSummaryBrief,
    #[serde(flatten)]
    counts: SemanticPhaseCounts,
}

pub(super) fn semantic_phase_brief<'a>(
    semantic: &'a SemanticHealthArtifact,
    elapsed_ms: u128,
    mode: CargoCheckMode,
    counts: SemanticPhaseCounts,
) -> SemanticPhaseBrief<'a> {
    SemanticPhaseBrief {
        artifact: ArtifactLane::RustCargoOracle,
        embedded: EmbeddedLane::Brief,
        raw_embedded: RawLaneOmitted,
        mode,
        elapsed_ms,
        schema_version: semantic.schema_version,
        meta: SemanticPhaseMetaBrief::from_meta(&semantic.meta),
        summary: SemanticPhaseSummaryBrief::from_summary(&semantic.summary),
        counts,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPhaseSummaryBrief {
    findings: usize,
    diagnostics: usize,
    coverage: usize,
    verified_findings: usize,
    rule_backed_findings: usize,
    candidate_findings: usize,
    coverage_unavailable_diagnostics: usize,
}

impl SemanticPhaseSummaryBrief {
    fn from_summary(summary: &SemanticSummary) -> Self {
        Self {
            findings: summary.findings,
            diagnostics: summary.diagnostics,
            coverage: summary.coverage,
            verified_findings: summary.verified_findings,
            rule_backed_findings: summary.rule_backed_findings,
            candidate_findings: summary.candidate_findings,
            coverage_unavailable_diagnostics: summary.coverage_unavailable_diagnostics,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPhaseMetaBrief<'a> {
    producer: SemanticArtifactProducer,
    mode: SemanticArtifactMode,
    oracle_registry_version: &'static str,
    evidence_policy_version: &'static str,
    diagnostic_policy_version: &'static str,
    analysis_input_set_complete: bool,
    missing_influence_kind_count: usize,
    missing_influence_kinds: &'a [MissingInfluenceKind],
    toolchain: SemanticPhaseToolchainBrief<'a>,
}

impl<'a> SemanticPhaseMetaBrief<'a> {
    fn from_meta(meta: &'a SemanticMeta) -> Self {
        Self {
            producer: meta.producer,
            mode: meta.mode,
            oracle_registry_version: meta.oracle_registry_version,
            evidence_policy_version: meta.evidence_policy_version,
            diagnostic_policy_version: meta.diagnostic_policy_version,
            analysis_input_set_complete: meta.analysis_input_set_complete,
            missing_influence_kind_count: meta.missing_influence_kinds.len(),
            missing_influence_kinds: &meta.missing_influence_kinds,
            toolchain: SemanticPhaseToolchainBrief::from_meta(meta),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticPhaseToolchainBrief<'a> {
    cargo_version: Option<&'a str>,
    rustc_source: RustcCommandSource,
    host: Option<&'a str>,
    profile: ArtifactProfile,
}

impl<'a> SemanticPhaseToolchainBrief<'a> {
    fn from_meta(meta: &'a SemanticMeta) -> Self {
        Self {
            cargo_version: meta.toolchain.cargo_version.as_deref(),
            rustc_source: meta.toolchain.rustc_source,
            host: meta.toolchain.host.as_deref(),
            profile: meta.toolchain.profile,
        }
    }
}
