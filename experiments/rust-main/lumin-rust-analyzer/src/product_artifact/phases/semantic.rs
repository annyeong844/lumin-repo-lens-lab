use lumin_rust_cargo_oracle::protocol::{
    ArtifactMeta as SemanticMeta, ArtifactProfile, MissingInfluenceKind, RustcCommandSource,
    SemanticArtifactMode, SemanticArtifactProducer, SemanticHealthArtifact,
    Summary as SemanticSummary,
};
use lumin_rust_cargo_oracle::CargoCheckMode;
use serde::Serialize;

use crate::policy::RawLaneOmitted;
use crate::product_artifact::meta::{ArtifactLane, EmbeddedLane};
use crate::product_files::{
    ProductSemanticDiagnosticsProjection, ProductSemanticFindingsProjection,
};

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::product_artifact) struct SemanticPhaseCounts {
    finding_count: usize,
    diagnostic_count: usize,
}

impl SemanticPhaseCounts {
    pub(in crate::product_artifact) fn from_projections(
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
pub(in crate::product_artifact) struct SemanticPhaseBrief<'a> {
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

pub(in crate::product_artifact) fn semantic_phase_brief<'a>(
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
