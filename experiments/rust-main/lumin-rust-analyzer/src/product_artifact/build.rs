use std::path::Path;

use anyhow::Result;
use lumin_rust_cargo_oracle::protocol::SemanticHealthArtifact;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use super::meta::{
    ProductArtifactInput, ProductArtifactMeta, ProductArtifactMode, ProductArtifactProducer,
    ProductPhaseTimings,
};
use super::model::{PhaseTimings, ProductArtifact, UnifiedArtifact};
use super::phases::{semantic_phase_brief, syntax_phase_brief, PhaseBriefs, SemanticPhaseCounts};
use super::refs::ArtifactRefs;
use super::semantic::{coverage_projection, oracle_plan_projection};
use crate::calibration::CalibrationAdjudication;
use crate::cli::{Options, SourceHealthProfile};
use crate::policy::{
    action_policy, oracle_bridge, policy_metadata, CoverageEvidence, POLICY_VERSION,
};
use crate::product_files::{
    merged_files, semantic_diagnostics_with_paths, semantic_findings_with_oracle_provenance,
};
use crate::product_summary::product_summary;
use crate::syntax_phase::SyntaxPhase;

const SCHEMA_VERSION: &str = "rust-analyzer-health.v1";

pub(crate) fn unified_artifact<'a>(
    options: &Options,
    effective_source_health_profile: SourceHealthProfile,
    root: &Path,
    syntax_phase: SyntaxPhase<'a>,
    semantic_phase: &'a SemanticHealthArtifact,
    calibration_adjudication: Option<&CalibrationAdjudication>,
    timings: PhaseTimings,
) -> Result<ProductArtifact<'a>> {
    let coverage_evidence = CoverageEvidence::from_coverage_entries(&semantic_phase.coverage);
    let semantic_findings = semantic_findings_with_oracle_provenance(
        root,
        syntax_phase,
        &semantic_phase.findings,
        &coverage_evidence,
    );
    let semantic_diagnostics = semantic_diagnostics_with_paths(root, &semantic_phase.diagnostics);
    let action_policy = action_policy(
        syntax_phase.summary(),
        &semantic_phase.summary,
        &coverage_evidence,
        &semantic_phase.coverage,
        &semantic_phase.findings,
    );
    let oracle_bridge = oracle_bridge(
        syntax_phase.summary(),
        &semantic_phase.summary,
        &action_policy,
        &coverage_evidence,
        &semantic_phase.oracle_plan,
        calibration_adjudication,
    );
    let action_policy = action_policy.into_projection();
    let files = merged_files(
        syntax_phase,
        &semantic_diagnostics,
        &semantic_findings,
        &oracle_bridge,
        &coverage_evidence,
    );
    let semantic_findings = semantic_findings.into_projection();
    let semantic_diagnostics = semantic_diagnostics.into_projection();
    let semantic_phase_counts =
        SemanticPhaseCounts::from_projections(&semantic_findings, &semantic_diagnostics);
    let unlinked_semantic_refs = files.unlinked_semantic_refs();
    let files = files.into_projection();
    let oracle_bridge = oracle_bridge.into_projection();

    let artifact = UnifiedArtifact {
        schema_version: SCHEMA_VERSION,
        policy_version: POLICY_VERSION,
        policy: policy_metadata(),
        meta: ProductArtifactMeta {
            producer: ProductArtifactProducer::LuminRustAnalyzer,
            mode: ProductArtifactMode::RustMain,
            generated: OffsetDateTime::now_utc().format(&Rfc3339)?,
            input: ProductArtifactInput {
                root: root.display().to_string(),
                package_name: options.package_name.clone(),
                features: options.features.clone(),
                cargo_bin: options.cargo_bin.clone(),
                semantic_mode: options.semantic_mode,
                cargo_target_dir_mode: options.cargo_target_dir_mode,
                cargo_target_dir_policy: semantic_phase.meta.input.cargo_target_dir_policy.clone(),
                cargo_target_dir: semantic_phase.meta.input.cargo_target_dir.clone(),
                source_health_profile: options.source_health_profile,
                effective_source_health_profile,
            },
            output: options
                .output
                .as_ref()
                .map(|path| path.display().to_string()),
            phase_timings: ProductPhaseTimings {
                syntax_ms: timings.syntax_ms,
                semantic_ms: timings.semantic_ms,
                analyzer_ms: timings.analyzer_ms,
            },
        },
        summary: product_summary(
            syntax_phase,
            &files,
            &semantic_phase.summary,
            &action_policy,
            &oracle_bridge,
            unlinked_semantic_refs,
        ),
        action_policy,
        oracle_bridge,
        files,
        coverage: coverage_projection(&semantic_phase.coverage),
        oracle_plan: oracle_plan_projection(&semantic_phase.oracle_plan),
        semantic_findings,
        semantic_diagnostics,
        artifact_refs: ArtifactRefs::default(),
        phases: PhaseBriefs {
            syntax: syntax_phase_brief(syntax_phase, timings.syntax_ms),
            semantic: semantic_phase_brief(
                semantic_phase,
                timings.semantic_ms,
                options.semantic_mode,
                semantic_phase_counts,
            ),
        },
    };

    let artifact = ProductArtifact { artifact };
    artifact.validate_contract()?;
    Ok(artifact)
}
